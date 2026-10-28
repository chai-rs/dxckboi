//! Bollinger Band mean reversion with ATR stops and risk-based sizing.

use nautilus_common::{actor::DataActor, timer::TimeEvent};
use nautilus_core::DurationNanos;
use nautilus_indicators::{
    indicator::Indicator, momentum::bb::BollingerBands, volatility::atr::AverageTrueRange,
};
use nautilus_model::{
    data::{Bar, BarType},
    enums::OrderSide,
    events::{OrderDenied, OrderRejected, PositionClosed},
    identifiers::{InstrumentId, StrategyId},
};
use nautilus_trading::{
    nautilus_strategy,
    strategy::{Strategy, StrategyConfig, StrategyCore},
};

use crate::{
    execution::{RiskSettings, cooldown_end, enter_bracket, is_busy},
    signal::{Bands, Signal, evaluate},
};

#[derive(Clone, Debug)]
pub struct MeanReversionConfig {
    pub strategy_id: StrategyId,
    pub order_id_tag: String,
    pub instrument_id: InstrumentId,
    pub bar_type: BarType,
    pub bb_period: usize,
    pub bb_k: f64,
    pub atr_period: usize,
    /// Stop distance in ATRs.
    pub atr_stop: f64,
    /// Minimum distance to the middle band, in ATRs, to take a trade.
    pub min_reward_atr: f64,
    /// Fraction of equity lost if the stop is hit.
    pub risk_fraction: f64,
    /// Upper bound on entry notional, in quote currency.
    pub max_notional: f64,
    /// Minutes to wait after an order is denied or rejected before entering again.
    pub cooldown_mins: u64,
    /// How often to log a status line.
    pub status_interval_mins: u64,
}

pub struct MeanReversion {
    core: StrategyCore,
    config: MeanReversionConfig,
    bb: BollingerBands,
    atr: AverageTrueRange,
    bars_seen: u64,
    trades_closed: u32,
    rejections: u32,
    /// No new entries before this time (UNIX nanoseconds).
    cooldown_until_ns: u64,
}

impl MeanReversion {
    pub fn new(config: MeanReversionConfig) -> Self {
        let core = StrategyCore::new(StrategyConfig {
            strategy_id: Some(config.strategy_id),
            order_id_tag: Some(config.order_id_tag.clone()),
            // After a live restart the cache is empty, so orders still open at the
            // venue come back as external orders. Claim them for this strategy.
            external_order_instrument_ids: Some(vec![config.instrument_id]),
            ..Default::default()
        });
        Self {
            core,
            bb: BollingerBands::new(config.bb_period, config.bb_k, None),
            atr: AverageTrueRange::new(config.atr_period, None, None, None),
            config,
            bars_seen: 0,
            trades_closed: 0,
            rejections: 0,
            cooldown_until_ns: 0,
        }
    }

    fn risk(&self) -> RiskSettings {
        RiskSettings {
            risk_fraction: self.config.risk_fraction,
            max_notional: self.config.max_notional,
        }
    }
}

nautilus_strategy!(MeanReversion, {
    // Denied = stopped by our own risk engine before reaching the venue.
    // Back off instead of resubmitting the same order on every bar.
    fn on_order_denied(&mut self, event: OrderDenied) {
        self.cooldown_until_ns = cooldown_end(self, self.config.cooldown_mins);
        log::warn!("Order {} denied: {}", event.client_order_id, event.reason);
    }

    fn on_order_rejected(&mut self, event: OrderRejected) {
        self.rejections += 1;
        self.cooldown_until_ns = cooldown_end(self, self.config.cooldown_mins);
        log::warn!("Order {} rejected: {}", event.client_order_id, event.reason);
    }

    fn on_position_closed(&mut self, event: PositionClosed) {
        self.trades_closed += 1;
        if let Some(pnl) = event.realized_pnl {
            log::info!("Position closed, realized PnL {pnl}");
        }
    }
});

impl std::fmt::Debug for MeanReversion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeanReversion")
            .field("config", &self.config)
            .finish()
    }
}

impl DataActor for MeanReversion {
    fn on_start(&mut self) -> anyhow::Result<()> {
        self.subscribe_bars(self.config.bar_type, None, None);
        self.clock().set_timer_ns(
            "status",
            DurationNanos::from_mins(self.config.status_interval_mins),
            None,
            None,
            None,
            None,
            None,
        )?;
        Ok(())
    }

    fn on_bar(&mut self, bar: &Bar) -> anyhow::Result<()> {
        self.bars_seen += 1;
        self.bb.handle_bar(bar);
        self.atr.handle_bar(bar);
        if !self.bb.initialized()
            || !self.atr.initialized()
            || bar.ts_event.as_u64() < self.cooldown_until_ns
            || is_busy(self, self.config.instrument_id, self.config.strategy_id)
        {
            return Ok(());
        }

        let close = bar.close.as_f64();
        let bands = Bands {
            upper: self.bb.upper,
            middle: self.bb.middle,
            lower: self.bb.lower,
            atr: self.atr.value,
        };
        match evaluate(
            close,
            bands,
            self.config.atr_stop,
            self.config.min_reward_atr,
        ) {
            Signal::Long {
                take_profit,
                stop_loss,
            } => enter_bracket(
                self,
                self.config.instrument_id,
                OrderSide::Buy,
                close,
                take_profit,
                stop_loss,
                self.risk(),
            ),
            Signal::Short {
                take_profit,
                stop_loss,
            } => enter_bracket(
                self,
                self.config.instrument_id,
                OrderSide::Sell,
                close,
                take_profit,
                stop_loss,
                self.risk(),
            ),
            Signal::None => Ok(()),
        }
    }

    fn on_time_event(&mut self, event: &TimeEvent) -> anyhow::Result<()> {
        if event.name.as_str() == "status" {
            log::info!(
                "Status: bars={} trades_closed={} rejections={} flat={}",
                self.bars_seen,
                self.trades_closed,
                self.rejections,
                self.portfolio().is_net_flat(&self.config.instrument_id),
            );
        }
        Ok(())
    }

    fn on_stop(&mut self) -> anyhow::Result<()> {
        let id = self.config.instrument_id;
        self.cancel_all_orders(id, None, None, true, None)?;
        self.close_all_positions(id, None, None, None, None, Some(true), None, None)?;
        self.unsubscribe_bars(self.config.bar_type, None, None);
        Ok(())
    }
}
