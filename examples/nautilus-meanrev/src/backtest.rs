//! Assembles a BacktestEngine for the strategy; shared by the binary and the tests.

use ahash::AHashMap;
use log::LevelFilter;
use nautilus_backtest::{
    config::{BacktestEngineConfig, SimulatedVenueConfig},
    engine::BacktestEngine,
    result::BacktestResult,
};
use nautilus_common::{logging::logger::LoggerConfig, throttler::RateLimit};
use nautilus_core::DurationNanos;
use nautilus_execution::models::{
    fee::{FeeModelAny, MakerTakerFeeModel},
    fill::{DefaultFillModel, FillModelAny},
};
use nautilus_model::{
    data::BarType,
    enums::{AccountType, BookType, OmsType},
    identifiers::{StrategyId, Venue},
    instruments::{Instrument, InstrumentAny, stubs::audusd_sim},
    types::Money,
};
use nautilus_risk::engine::config::RiskEngineConfig;
use rust_decimal::Decimal;

use crate::{
    data::SyntheticBars,
    strategy::{MeanReversion, MeanReversionConfig},
};

/// Which synthetic market to generate.
#[derive(Clone, Copy, Debug)]
pub enum Market {
    /// Prices pull back toward a slowly drifting anchor: the strategy's edge exists.
    MeanReverting,
    /// Pure random walk: no edge exists, so the strategy should lose its costs.
    RandomWalk,
}

/// Parameters that a sweep varies; everything else is fixed below.
#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub bb_period: usize,
    pub bb_k: f64,
    pub atr_stop: f64,
}

/// Runs one backtest over `bars` one-minute bars.
///
/// The logger is global to the process: the first engine created sets it up,
/// so `log_level` only takes effect on the first run in a process.
pub fn run(
    params: Params,
    market: Market,
    bars: usize,
    log_level: LevelFilter,
) -> anyhow::Result<BacktestResult> {
    let instrument = InstrumentAny::CurrencyPair(audusd_sim());
    let instrument_id = instrument.id();
    let bar_type = BarType::from("AUD/USD.SIM-1-MINUTE-LAST-EXTERNAL");

    let risk = RiskEngineConfig::builder()
        // At most 10 order submissions per second, and no single order above 2M USD notional.
        .max_order_submit(RateLimit::new(10, DurationNanos::from_secs(1)))
        .max_notional_per_order(AHashMap::from([(instrument_id, Decimal::from(2_000_000))]))
        .build()?;
    let config = BacktestEngineConfig::builder()
        .logging(LoggerConfig::builder().stdout_level(log_level).build()?)
        .risk_engine(risk)
        .build();
    let mut engine = BacktestEngine::new(config)?;

    engine.add_venue(
        SimulatedVenueConfig::builder()
            .venue(Venue::from("SIM"))
            .oms_type(OmsType::Netting)
            .account_type(AccountType::Margin)
            .book_type(BookType::L1_MBP)
            .starting_balances(vec![Money::from("100_000 USD")])
            // 0.2 bps maker / 0.5 bps taker.
            .fee_model(
                FeeModelAny::MakerTaker(MakerTakerFeeModel::new(
                    Decimal::new(2, 5),
                    Decimal::new(5, 5),
                ))
                .into(),
            )
            // Limit orders fill when touched; every fill has a 30% chance of slipping one tick.
            .fill_model(FillModelAny::Default(DefaultFillModel::new(1.0, 0.3, Some(42))?).into())
            .build()?,
    )?;
    engine.add_instrument(&instrument)?;

    let data = SyntheticBars {
        bar_type,
        start_ns: 1_704_067_200_000_000_000, // 2024-01-01T00:00:00Z
        interval_ns: 60_000_000_000,
        count: bars,
        anchor: 0.6800,
        reversion: match market {
            Market::MeanReverting => 0.02,
            Market::RandomWalk => 0.0,
        },
        volatility: 0.00012,
        anchor_drift: 0.00015,
        price_precision: instrument.price_precision(),
        seed: 7,
    }
    .generate();
    engine.add_data(data, None, true, true)?;

    engine.add_strategy(MeanReversion::new(MeanReversionConfig {
        strategy_id: StrategyId::from("MEANREV-001"),
        order_id_tag: "001".to_string(),
        instrument_id,
        bar_type,
        bb_period: params.bb_period,
        bb_k: params.bb_k,
        atr_period: 14,
        atr_stop: params.atr_stop,
        min_reward_atr: 1.0,
        risk_fraction: 0.005,
        max_notional: 1_500_000.0,
        cooldown_bars: 30,
        status_interval_mins: 24 * 60,
    }))?;

    engine.run(None, None, None, false)?;
    Ok(engine.get_result())
}
