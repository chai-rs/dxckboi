use log::LevelFilter;
use meanrev::backtest::{Market, Params, run};
use nautilus_backtest::result::BacktestResult;

const PARAMS: Params = Params {
    bb_period: 20,
    bb_k: 2.0,
    atr_stop: 1.5,
};
const DAYS: usize = 24 * 60;

fn pnl(result: &BacktestResult) -> f64 {
    result.stats_pnls["USD"]["PnL (total)"]
}

#[test]
fn every_entry_is_a_full_bracket() {
    let result = run(PARAMS, Market::MeanReverting, 5 * DAYS, LevelFilter::Off).unwrap();
    assert!(result.total_positions > 0, "expected the strategy to trade");
    // Entry + take profit + stop loss, and nothing submitted twice.
    assert_eq!(result.total_orders, 3 * result.total_positions);
}

#[test]
fn same_inputs_give_same_results() {
    let a = run(PARAMS, Market::MeanReverting, 5 * DAYS, LevelFilter::Off).unwrap();
    let b = run(PARAMS, Market::MeanReverting, 5 * DAYS, LevelFilter::Off).unwrap();
    assert_eq!(a.total_orders, b.total_orders);
    assert_eq!(pnl(&a), pnl(&b));
}

#[test]
fn no_profit_without_an_edge() {
    // A pure random walk has no mean reversion to exploit, so costs must win.
    let result = run(PARAMS, Market::RandomWalk, 5 * DAYS, LevelFilter::Off).unwrap();
    assert!(result.total_positions > 0);
    assert!(
        pnl(&result) < 0.0,
        "profit on a random walk: {}",
        pnl(&result)
    );
}
