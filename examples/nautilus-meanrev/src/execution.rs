//! Order placement shared by both designs: the single `MeanReversion` strategy
//! and the `BandSignalActor` + `SignalExecutor` pair.

use nautilus_common::actor::{DataActor, DataActorNative};
use nautilus_model::{
    accounts::Account,
    enums::OrderSide,
    identifiers::{InstrumentId, StrategyId},
    instruments::Instrument,
};
use nautilus_trading::strategy::{Strategy, StrategyNative};

use crate::sizing::{SizeLimits, units_for_risk};

/// Risk settings for sizing an entry.
#[derive(Clone, Copy, Debug)]
pub struct RiskSettings {
    /// Fraction of equity lost if the stop is hit.
    pub risk_fraction: f64,
    /// Upper bound on entry notional, in quote currency.
    pub max_notional: f64,
}

/// True while `strategy_id` holds a position in, or has working orders for, the instrument.
pub fn is_busy<S>(strategy: &S, instrument_id: InstrumentId, strategy_id: StrategyId) -> bool
where
    S: Strategy + StrategyNative + DataActorNative,
{
    !strategy.portfolio().is_net_flat(&instrument_id)
        || strategy.cache().has_orders_open(
            None,
            Some(&instrument_id),
            Some(&strategy_id),
            None,
            None,
        )
}

/// Sizes an entry from account equity and submits it as a bracket order.
pub fn enter_bracket<S>(
    strategy: &mut S,
    instrument_id: InstrumentId,
    side: OrderSide,
    entry: f64,
    take_profit: f64,
    stop_loss: f64,
    risk: RiskSettings,
) -> anyhow::Result<()>
where
    S: Strategy + StrategyNative + DataActorNative,
{
    let Some(instrument) = strategy.cache().instrument(&instrument_id) else {
        anyhow::bail!("instrument {instrument_id} not in cache");
    };
    let quote = instrument.quote_currency();
    let equity = strategy
        .cache()
        .account_for_venue(&instrument_id.venue)
        .and_then(|account| account.balance_total(Some(quote)))
        .map_or(0.0, |money| money.as_f64());

    let limits = SizeLimits {
        increment: instrument.size_increment().as_f64(),
        min_units: instrument.min_quantity().map_or(0.0, |q| q.as_f64()),
        max_units: instrument
            .max_quantity()
            .map_or(f64::INFINITY, |q| q.as_f64()),
        max_notional: risk.max_notional,
    };
    let units = units_for_risk(equity, risk.risk_fraction, entry, stop_loss, limits);
    if units <= 0.0 {
        log::warn!("Skipping {side:?}: position size rounds to zero (equity {equity})");
        return Ok(());
    }

    let orders = strategy
        .order()
        .bracket()
        .instrument_id(instrument_id)
        .order_side(side)
        .quantity(instrument.make_qty(units, Some(true)))
        .tp_price(instrument.make_price(take_profit))
        .tp_post_only(false)
        .sl_trigger_price(instrument.make_price(stop_loss))
        .call();
    log::info!("Entering {side:?} {units} @ ~{entry:.5} tp={take_profit:.5} sl={stop_loss:.5}");
    strategy.submit_order_list(orders, None, None, None)
}

/// Clock time plus `mins`, as nanoseconds: the end of a cooldown that starts now.
pub fn cooldown_end<S: DataActor + DataActorNative>(strategy: &S, mins: u64) -> u64 {
    strategy.clock().timestamp_ns().as_u64() + mins * 60 * 1_000_000_000
}
