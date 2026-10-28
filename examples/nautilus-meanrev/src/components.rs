//! The same strategy split into two components that talk over the message bus:
//! `BandSignalActor` turns bars into `BandSignal`s, `SignalExecutor` turns
//! signals into orders. Neither knows about the other, only about the message.

use std::sync::Arc;

use nautilus_common::{
    actor::{DataActor, DataActorConfig, DataActorCore},
    nautilus_actor,
};
use nautilus_core::UnixNanos;
use nautilus_indicators::{
    indicator::Indicator, momentum::bb::BollingerBands, volatility::atr::AverageTrueRange,
};
use nautilus_model::{
    custom_data,
    data::{Bar, BarType, CustomData, DataType},
    enums::OrderSide,
    events::{OrderDenied, OrderRejected},
    identifiers::{ActorId, InstrumentId, StrategyId},
};
use nautilus_trading::{
    nautilus_strategy,
    strategy::{Strategy, StrategyConfig, StrategyCore},
};

use crate::{
    execution::{RiskSettings, cooldown_end, enter_bracket, is_busy},
    signal::{Bands, Signal, evaluate},
};

/// A trade idea: enter at about `close`, exit at `take_profit` or `stop_loss`.
#[custom_data]
pub struct BandSignal {
    pub instrument_id: InstrumentId,
    pub long: bool,
    pub close: f64,
    pub take_profit: f64,
    pub stop_loss: f64,
    pub ts_event: UnixNanos,
    pub ts_init: UnixNanos,
}

/// The message-bus data type both sides agree on.
pub fn band_signal_type() -> DataType {
    DataType::new("BandSignal", None, None)
}

/// Computes the bands on each bar and publishes a `BandSignal` when price closes outside them.
pub struct BandSignalActor {
    core: DataActorCore,
    instrument_id: InstrumentId,
    bar_type: BarType,
    bb: BollingerBands,
    atr: AverageTrueRange,
    atr_stop: f64,
    min_reward_atr: f64,
}

impl BandSignalActor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        actor_id: &str,
        instrument_id: InstrumentId,
        bar_type: BarType,
        bb_period: usize,
        bb_k: f64,
        atr_period: usize,
        atr_stop: f64,
        min_reward_atr: f64,
    ) -> Self {
        Self {
            core: DataActorCore::new(DataActorConfig {
                actor_id: Some(ActorId::from(actor_id)),
                ..Default::default()
            }),
            instrument_id,
            bar_type,
            bb: BollingerBands::new(bb_period, bb_k, None),
            atr: AverageTrueRange::new(atr_period, None, None, None),
            atr_stop,
            min_reward_atr,
        }
    }
}

nautilus_actor!(BandSignalActor);

impl std::fmt::Debug for BandSignalActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BandSignalActor")
            .field("bar_type", &self.bar_type)
            .finish()
    }
}

impl DataActor for BandSignalActor {
    fn on_start(&mut self) -> anyhow::Result<()> {
        self.subscribe_bars(self.bar_type, None, None);
        Ok(())
    }

    fn on_bar(&mut self, bar: &Bar) -> anyhow::Result<()> {
        self.bb.handle_bar(bar);
        self.atr.handle_bar(bar);
        if !self.bb.initialized() || !self.atr.initialized() {
            return Ok(());
        }

        let close = bar.close.as_f64();
        let bands = Bands {
            upper: self.bb.upper,
            middle: self.bb.middle,
            lower: self.bb.lower,
            atr: self.atr.value,
        };
        let (long, take_profit, stop_loss) =
            match evaluate(close, bands, self.atr_stop, self.min_reward_atr) {
                Signal::Long {
                    take_profit,
                    stop_loss,
                } => (true, take_profit, stop_loss),
                Signal::Short {
                    take_profit,
                    stop_loss,
                } => (false, take_profit, stop_loss),
                Signal::None => return Ok(()),
            };

        let signal = BandSignal::new(
            self.instrument_id,
            long,
            close,
            take_profit,
            stop_loss,
            bar.ts_event,
            bar.ts_init,
        );
        self.publish_data(
            &band_signal_type(),
            &CustomData::new(Arc::new(signal), band_signal_type()),
        );
        Ok(())
    }

    fn on_stop(&mut self) -> anyhow::Result<()> {
        self.unsubscribe_bars(self.bar_type, None, None);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SignalExecutorConfig {
    pub strategy_id: StrategyId,
    pub order_id_tag: String,
    pub instrument_id: InstrumentId,
    pub risk: RiskSettings,
    pub cooldown_mins: u64,
}

/// Executes `BandSignal`s: checks state, sizes the trade and submits a bracket.
pub struct SignalExecutor {
    core: StrategyCore,
    config: SignalExecutorConfig,
    cooldown_until_ns: u64,
}

impl SignalExecutor {
    pub fn new(config: SignalExecutorConfig) -> Self {
        let core = StrategyCore::new(StrategyConfig {
            strategy_id: Some(config.strategy_id),
            order_id_tag: Some(config.order_id_tag.clone()),
            external_order_instrument_ids: Some(vec![config.instrument_id]),
            ..Default::default()
        });
        Self {
            core,
            config,
            cooldown_until_ns: 0,
        }
    }
}

nautilus_strategy!(SignalExecutor, {
    fn on_order_denied(&mut self, event: OrderDenied) {
        self.cooldown_until_ns = cooldown_end(self, self.config.cooldown_mins);
        log::warn!("Order {} denied: {}", event.client_order_id, event.reason);
    }

    fn on_order_rejected(&mut self, event: OrderRejected) {
        self.cooldown_until_ns = cooldown_end(self, self.config.cooldown_mins);
        log::warn!("Order {} rejected: {}", event.client_order_id, event.reason);
    }
});

impl std::fmt::Debug for SignalExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalExecutor")
            .field("config", &self.config)
            .finish()
    }
}

impl DataActor for SignalExecutor {
    fn on_start(&mut self) -> anyhow::Result<()> {
        self.subscribe_data(band_signal_type(), None, None);
        Ok(())
    }

    fn on_data(&mut self, data: &CustomData) -> anyhow::Result<()> {
        let Some(signal) = data.data.as_any().downcast_ref::<BandSignal>() else {
            return Ok(());
        };
        if signal.instrument_id != self.config.instrument_id
            || signal.ts_event.as_u64() < self.cooldown_until_ns
            || is_busy(self, self.config.instrument_id, self.config.strategy_id)
        {
            return Ok(());
        }
        let side = if signal.long {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };
        enter_bracket(
            self,
            self.config.instrument_id,
            side,
            signal.close,
            signal.take_profit,
            signal.stop_loss,
            self.config.risk,
        )
    }

    fn on_stop(&mut self) -> anyhow::Result<()> {
        let id = self.config.instrument_id;
        self.cancel_all_orders(id, None, None, true, None)?;
        self.close_all_positions(id, None, None, None, None, Some(true), None, None)?;
        self.unsubscribe_data(band_signal_type(), None, None);
        Ok(())
    }
}
