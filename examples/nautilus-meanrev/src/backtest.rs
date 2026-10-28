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

use std::sync::mpsc::SyncSender;

use crate::{
    components::{BandSignalActor, SignalExecutor, SignalExecutorConfig},
    data::SyntheticBars,
    execution::RiskSettings,
    exporter::{FillExporter, FillRecord},
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

/// How the trading logic is assembled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Design {
    /// One `MeanReversion` strategy does everything.
    SingleStrategy,
    /// `BandSignalActor` publishes signals, `SignalExecutor` trades them.
    ActorAndExecutor,
}

/// Everything that varies between runs.
pub struct RunOptions {
    pub params: Params,
    pub market: Market,
    pub bars: usize,
    pub log_level: LevelFilter,
    pub design: Design,
    /// When set, a `FillExporter` sends every fill here.
    pub fill_sink: Option<SyncSender<FillRecord>>,
}

/// Runs one backtest over `bars` one-minute bars with the single-strategy design.
///
/// The logger is global to the process: the first engine created sets it up,
/// so `log_level` only takes effect on the first run in a process.
pub fn run(
    params: Params,
    market: Market,
    bars: usize,
    log_level: LevelFilter,
) -> anyhow::Result<BacktestResult> {
    run_with(RunOptions {
        params,
        market,
        bars,
        log_level,
        design: Design::SingleStrategy,
        fill_sink: None,
    })
}

/// Runs one backtest as described by `options`.
pub fn run_with(options: RunOptions) -> anyhow::Result<BacktestResult> {
    let RunOptions {
        params,
        market,
        bars,
        log_level,
        design,
        fill_sink,
    } = options;
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

    const ATR_PERIOD: usize = 14;
    const MIN_REWARD_ATR: f64 = 1.0;
    const COOLDOWN_MINS: u64 = 30;
    let risk = RiskSettings {
        risk_fraction: 0.005,
        max_notional: 1_500_000.0,
    };

    if let Some(sink) = fill_sink {
        engine.add_actor(FillExporter::new("FILL-EXPORTER-001", sink))?;
    }
    match design {
        Design::SingleStrategy => engine.add_strategy(MeanReversion::new(MeanReversionConfig {
            strategy_id: StrategyId::from("MEANREV-001"),
            order_id_tag: "001".to_string(),
            instrument_id,
            bar_type,
            bb_period: params.bb_period,
            bb_k: params.bb_k,
            atr_period: ATR_PERIOD,
            atr_stop: params.atr_stop,
            min_reward_atr: MIN_REWARD_ATR,
            risk_fraction: risk.risk_fraction,
            max_notional: risk.max_notional,
            cooldown_mins: COOLDOWN_MINS,
            status_interval_mins: 24 * 60,
        }))?,
        Design::ActorAndExecutor => {
            engine.add_actor(BandSignalActor::new(
                "BAND-SIGNAL-001",
                instrument_id,
                bar_type,
                params.bb_period,
                params.bb_k,
                ATR_PERIOD,
                params.atr_stop,
                MIN_REWARD_ATR,
            ))?;
            engine.add_strategy(SignalExecutor::new(SignalExecutorConfig {
                strategy_id: StrategyId::from("EXECUTOR-001"),
                order_id_tag: "001".to_string(),
                instrument_id,
                risk,
                cooldown_mins: COOLDOWN_MINS,
            }))?;
        }
    }

    engine.run(None, None, None, false)?;
    let result = engine.get_result();
    // Disposing retires every actor and strategy, dropping what they own (such as the
    // exporter's channel sender). Dropping the engine alone does not: the actor
    // registry is thread-local and outlives it.
    engine.dispose();
    Ok(result)
}
