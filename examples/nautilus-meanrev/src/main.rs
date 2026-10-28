//! `meanrev run [random]` one backtest with full logging (mean-reverting or random-walk market)
//! `meanrev sweep` a parameter grid with logging off
use std::time::Instant;

use log::LevelFilter;
use meanrev::backtest::{Market, Params, run};

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
    match std::env::args().nth(1).as_deref() {
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
