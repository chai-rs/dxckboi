//! `meanrev run [random]` one backtest with full logging (mean-reverting or random-walk market)
//! `meanrev sweep` a parameter grid with logging off
//! `meanrev compare` the single-strategy and actor + executor designs side by side
//! `meanrev export <file>` one run with every fill written to a JSON-lines file
use std::time::Instant;

use log::LevelFilter;
use meanrev::{
    backtest::{Design, Market, Params, RunOptions, run, run_with},
    exporter::spawn_jsonl_writer,
};

const BARS: usize = 30 * 24 * 60; // 30 days of 1-minute bars

fn pnl_total(result: &nautilus_backtest::result::BacktestResult) -> f64 {
    result
        .stats_pnls
        .get("USD")
        .and_then(|stats| stats.get("PnL (total)"))
        .copied()
        .unwrap_or(f64::NAN)
}

fn main() -> anyhow::Result<()> {
    let base = Params {
        bb_period: 20,
        bb_k: 2.0,
        atr_stop: 1.5,
    };
    match std::env::args().nth(1).as_deref() {
        Some("compare") => {
            println!("design            orders  positions   pnl_total");
            for design in [Design::SingleStrategy, Design::ActorAndExecutor] {
                let result = run_with(RunOptions {
                    params: base,
                    market: Market::MeanReverting,
                    bars: BARS,
                    log_level: LevelFilter::Off,
                    design,
                    fill_sink: None,
                })?;
                println!(
                    "{:<16}  {:>6}  {:>9}  {:>10.2}",
                    format!("{design:?}"),
                    result.total_orders,
                    result.total_positions,
                    pnl_total(&result),
                );
            }
        }
        Some("export") => {
            let path = std::env::args()
                .nth(2)
                .unwrap_or_else(|| "fills.jsonl".to_string());
            let (sink, writer) = spawn_jsonl_writer(path.as_ref(), 10_000)?;
            let result = run_with(RunOptions {
                params: base,
                market: Market::MeanReverting,
                bars: BARS,
                log_level: LevelFilter::Off,
                design: Design::ActorAndExecutor,
                fill_sink: Some(sink),
            })?;
            // The engine (and with it the exporter's sender) is gone, so the writer finishes.
            let lines = writer.join().expect("writer thread panicked")?;
            println!(
                "positions={} fills_written={lines} file={path}",
                result.total_positions
            );
        }
        Some("sweep") => {
            println!("market         bb_period  bb_k  atr_stop  positions   pnl_total");
            for market in [Market::MeanReverting, Market::RandomWalk] {
                for bb_period in [20, 40] {
                    for bb_k in [2.0, 2.5] {
                        for atr_stop in [1.0, 2.0] {
                            let params = Params {
                                bb_period,
                                bb_k,
                                atr_stop,
                            };
                            let result = run(params, market, BARS, LevelFilter::Off)?;
                            println!(
                                "{:<13}  {bb_period:>9}  {bb_k:>4}  {atr_stop:>8}  {:>9}  {:>10.2}",
                                format!("{market:?}"),
                                result.total_positions,
                                pnl_total(&result),
                            );
                        }
                    }
                }
            }
        }
        _ => {
            let started = Instant::now();
            let market = match std::env::args().nth(2).as_deref() {
                Some("random") => Market::RandomWalk,
                _ => Market::MeanReverting,
            };
            let params = Params {
                bb_period: 20,
                bb_k: 2.0,
                atr_stop: 1.5,
            };
            let result = run(params, market, BARS, LevelFilter::Info)?;
            println!(
                "orders={} positions={} pnl_total={:.2} simulated_days={:.1} wall_time={:.2?}",
                result.total_orders,
                result.total_positions,
                pnl_total(&result),
                result.elapsed_time_secs / 86_400.0,
                started.elapsed(),
            );
        }
    }
    Ok(())
}
