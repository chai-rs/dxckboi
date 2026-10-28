//! Bollinger Band mean reversion with ATR stops and risk-based sizing.

use nautilus_common::{actor::DataActor, timer::TimeEvent};
use nautilus_core::DurationNanos;
use nautilus_indicators::{
    indicator::Indicator, momentum::bb::BollingerBands, volatility::atr::AverageTrueRange,
};
use nautilus_model::{
    accounts::Account,
    data::{Bar, BarType},
    enums::OrderSide,
    events::{OrderDenied, OrderRejected, PositionClosed},
    identifiers::{InstrumentId, StrategyId},
    instruments::Instrument,
};
use nautilus_trading::{
    nautilus_strategy,
    strategy::{Strategy, StrategyConfig, StrategyCore},
};

use crate::{
    signal::{Bands, Signal, evaluate},
    sizing::{SizeLimits, units_for_risk},
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
    /// Bars to wait after an order is denied before trying again.
    pub cooldown_bars: u64,
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
    /// No new entries until `bars_seen` reaches this.
    cooldown_until: u64,
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
            cooldown_until: 0,
        }
    }

    /// True while we hold a position or still have working orders.
    fn is_busy(&self) -> bool {
        let id = self.config.instrument_id;
        !self.portfolio().is_net_flat(&id)
            || self.cache().has_orders_open(
                None,
                Some(&id),
                Some(&self.config.strategy_id),
                None,
                None,
            )
    }

    fn enter(
        &mut self,
        side: OrderSide,
        entry: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> anyhow::Result<()> {
        let id = self.config.instrument_id;
        let Some(instrument) = self.cache().instrument(&id) else {
            anyhow::bail!("instrument {id} not in cache");
        };
        let quote = instrument.quote_currency();
        let equity = self
            .cache()
            .account_for_venue(&id.venue)
            .and_then(|account| account.balance_total(Some(quote)))
            .map_or(0.0, |money| money.as_f64());

        let limits = SizeLimits {
            increment: instrument.size_increment().as_f64(),
            min_units: instrument.min_quantity().map_or(0.0, |q| q.as_f64()),
            max_units: instrument
                .max_quantity()
                .map_or(f64::INFINITY, |q| q.as_f64()),
            max_notional: self.config.max_notional,
        };
        let units = units_for_risk(equity, self.config.risk_fraction, entry, stop_loss, limits);
        if units <= 0.0 {
            log::warn!("Skipping {side:?}: position size rounds to zero (equity {equity})");
            return Ok(());
        }

        let orders = self
            .order()
            .bracket()
            .instrument_id(id)
            .order_side(side)
            .quantity(instrument.make_qty(units, Some(true)))
            .tp_price(instrument.make_price(take_profit))
            .tp_post_only(false)
            .sl_trigger_price(instrument.make_price(stop_loss))
            .call();
        log::info!("Entering {side:?} {units} @ ~{entry:.5} tp={take_profit:.5} sl={stop_loss:.5}");
        self.submit_order_list(orders, None, None, None)
    }
}

nautilus_strategy!(MeanReversion, {
    // Denied = stopped by our own risk engine before reaching the venue.
    // Back off instead of resubmitting the same order on every bar.
    fn on_order_denied(&mut self, event: OrderDenied) {
        self.cooldown_until = self.bars_seen + self.config.cooldown_bars;
        log::warn!("Order {} denied: {}", event.client_order_id, event.reason);
    }

    fn on_order_rejected(&mut self, event: OrderRejected) {
        self.rejections += 1;
        self.cooldown_until = self.bars_seen + self.config.cooldown_bars;
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
            || self.bars_seen < self.cooldown_until
            || self.is_busy()
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
            } => self.enter(OrderSide::Buy, close, take_profit, stop_loss),
            Signal::Short {
                take_profit,
                stop_loss,
            } => self.enter(OrderSide::Sell, close, take_profit, stop_loss),
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
