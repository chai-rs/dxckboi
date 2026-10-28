use std::{sync::mpsc::sync_channel, time::Duration};

use log::LevelFilter;
use meanrev::{
    backtest::{Design, Market, Params, RunOptions, run_with},
    exporter::spawn_jsonl_writer,
};
use nautilus_backtest::result::BacktestResult;

const PARAMS: Params = Params {
    bb_period: 20,
    bb_k: 2.0,
    atr_stop: 1.5,
};
const DAYS: usize = 24 * 60;

fn options(design: Design) -> RunOptions {
    RunOptions {
        params: PARAMS,
        market: Market::MeanReverting,
        bars: 5 * DAYS,
        log_level: LevelFilter::Off,
        design,
        fill_sink: None,
    }
}

fn pnl(result: &BacktestResult) -> f64 {
    result.stats_pnls["USD"]["PnL (total)"]
}

#[test]
fn split_design_trades_exactly_like_the_single_strategy() {
    let single = run_with(options(Design::SingleStrategy)).unwrap();
    let split = run_with(options(Design::ActorAndExecutor)).unwrap();
    assert!(single.total_positions > 0);
    assert_eq!(single.total_orders, split.total_orders);
    assert_eq!(single.total_positions, split.total_positions);
    assert_eq!(pnl(&single), pnl(&split));
}

#[test]
fn exporter_receives_every_fill() {
    // Large enough that nothing is dropped in this test.
    let (sink, receiver) = sync_channel(100_000);
    let result = run_with(RunOptions {
        fill_sink: Some(sink),
        ..options(Design::ActorAndExecutor)
    })
    .unwrap();
    let fills: Vec<_> = receiver.try_iter().collect();

    // Orders often fill in pieces (bar volume is spread over open/high/low/close),
    // so count quantity, not fills: the strategy ends flat, so every unit bought
    // was sold again.
    assert!(fills.len() >= 2 * result.total_positions);
    let net: f64 = fills
        .iter()
        .map(|f| {
            if f.side == "Buy" {
                f.quantity
            } else {
                -f.quantity
            }
        })
        .sum();
    assert_eq!(net, 0.0);
    assert!(fills.iter().all(|f| f.instrument_id == "AUD/USD.SIM"));
    assert!(fills.windows(2).all(|w| w[0].ts_event <= w[1].ts_event));
}

#[test]
fn writer_thread_finishes_after_the_run() {
    let path = std::env::temp_dir().join("meanrev-writer-test.jsonl");
    let (sink, writer) = spawn_jsonl_writer(&path, 100_000).unwrap();
    let result = run_with(RunOptions {
        fill_sink: Some(sink),
        ..options(Design::ActorAndExecutor)
    })
    .unwrap();

    // join() would hang forever if any sender were still alive, so wait with a deadline.
    let (done, finished) = std::sync::mpsc::channel();
    std::thread::spawn(move || done.send(writer.join()));
    let lines = finished
        .recv_timeout(Duration::from_secs(30))
        .expect("writer thread did not finish: a sender is still alive")
        .unwrap()
        .unwrap();
    assert!(lines >= 2 * result.total_positions);
    std::fs::remove_file(path).ok();
}
