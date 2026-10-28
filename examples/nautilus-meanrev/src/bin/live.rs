//! The same strategy on Bybit testnet (USDT linear perpetual).
//!
//! Credentials come from BYBIT_TESTNET_API_KEY / BYBIT_TESTNET_API_SECRET.
//! Run with: cargo run --release --features live --bin live
use meanrev::strategy::{MeanReversion, MeanReversionConfig};
use nautilus_bybit::{
    common::enums::{BybitEnvironment, BybitProductType},
    config::{BybitDataClientConfig, BybitExecutionClientConfig},
    factories::{BybitDataClientFactory, BybitExecutionClientFactory},
};
use nautilus_common::enums::Environment;
use nautilus_live::{config::LiveExecutionEngineConfig, node::LiveNode};
use nautilus_model::{
    data::BarType,
    identifiers::{AccountId, InstrumentId, StrategyId, TraderId},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let environment = BybitEnvironment::Testnet;
    let instrument_id = InstrumentId::from("BTCUSDT-LINEAR.BYBIT");

    let data_config = BybitDataClientConfig {
        environment,
        api_key: None,    // read from BYBIT_TESTNET_API_KEY
        api_secret: None, // read from BYBIT_TESTNET_API_SECRET
        product_types: vec![BybitProductType::Linear],
        ..Default::default()
    };
    let exec_config = BybitExecutionClientConfig {
        environment,
        api_key: None,
        api_secret: None,
        product_types: vec![BybitProductType::Linear],
        account_id: Some(AccountId::from("BYBIT-001")),
        ..Default::default()
    };
    // Re-check open orders and positions against the venue while running.
    let exec_engine_config = LiveExecutionEngineConfig {
        open_check_interval_secs: Some(10.0),
        position_check_interval_secs: Some(30.0),
        ..Default::default()
    };

    let mut node = LiveNode::builder(TraderId::from("TRADER-001"), Environment::Live)?
        .with_name("MEANREV-NODE".to_string())
        .with_exec_engine_config(exec_engine_config)
        .add_data_client(
            None,
            Box::new(BybitDataClientFactory::new()),
            Box::new(data_config),
        )?
        .add_exec_client(
            None,
            Box::new(BybitExecutionClientFactory::new()),
            Box::new(exec_config),
        )?
        .with_reconciliation(true)
        .with_delay_post_stop_secs(5)
        .build()?;

    node.add_strategy(MeanReversion::new(MeanReversionConfig {
        strategy_id: StrategyId::from("MEANREV-001"),
        order_id_tag: "001".to_string(),
        instrument_id,
        bar_type: BarType::from("BTCUSDT-LINEAR.BYBIT-1-MINUTE-LAST-EXTERNAL"),
        bb_period: 20,
        bb_k: 2.0,
        atr_period: 14,
        atr_stop: 1.5,
        min_reward_atr: 1.0,
        risk_fraction: 0.001,
        max_notional: 200.0, // USDT; keep testnet orders small
        cooldown_bars: 30,
        status_interval_mins: 60,
    }))?;

    node.run().await?;
    Ok(())
}
