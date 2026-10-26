# การรัน Live Trading (Rust)

`LiveNode` เชื่อมต่อกับ venue และแหล่งข้อมูลจริงผ่าน adapter client คู่มือนี้จะพาตั้งค่า live trading
แบบครบวงจร โดยใช้ OKX เป็นตัวอย่าง

สำหรับ lifecycle ของ node ดูที่ [Live trading](../concepts/live.md) สำหรับผลลัพธ์ของ command ดูที่
[Execution policies](../concepts/execution/policies.md#command-outcomes) สำหรับการกู้คืน state ดูที่
[Execution reconciliation](../concepts/execution/reconciliation.md) สำหรับการตั้งค่าโปรเจกต์และ feature flag
ดูคู่มือแนวคิด [Rust](../concepts/rust.md#project-setup)

## Dependencies {#dependencies}

เพิ่ม crate live, adapter ของ venue ที่ใช้ และ crate สนับสนุนอื่น ๆ ลงใน `Cargo.toml`:

```toml
[dependencies]
nautilus-common = "0.66"
nautilus-live = "0.66"
nautilus-model = "0.66"
nautilus-okx = "0.66"
nautilus-trading = { version = "0.66", features = ["examples"] }

anyhow = "1"
dotenvy = "0.15"
log = "0.4"
tokio = { version = "1", features = ["full"] }
```

## สร้าง node {#build-the-node}

`LiveNode` ใช้ builder pattern ให้เพิ่ม factory ของ data client และ execution client สำหรับ venue ของคุณ
ตั้งค่า logging แล้ว build

```rust
use log::LevelFilter;
use nautilus_common::{enums::Environment, logging::logger::LoggerConfig};
use nautilus_live::node::LiveNode;
use nautilus_model::identifiers::{AccountId, TraderId};
use nautilus_okx::{
    common::enums::OKXInstrumentType,
    config::{OKXDataClientConfig, OKXExecutionClientConfig},
    factories::{OKXDataClientFactory, OKXExecutionClientFactory},
};

let trader_id = TraderId::from("TESTER-001");
let account_id = AccountId::from("OKX-001");

let data_config = OKXDataClientConfig::builder()
    .instrument_types(vec![OKXInstrumentType::Swap])
    .build();

let exec_config = OKXExecutionClientConfig::builder()
    .account_id(account_id)
    .instrument_types(vec![OKXInstrumentType::Swap])
    .build();

let log_config = LoggerConfig {
    stdout_level: LevelFilter::Info,
    ..Default::default()
};

let mut node = LiveNode::builder(trader_id, Environment::Live)?
    .with_name("MY-NODE-001".to_string())
    .with_logging(log_config)
    .add_data_client(
        None,
        Box::new(OKXDataClientFactory::new()),
        Box::new(data_config),
    )?
    .add_exec_client(
        None,
        Box::new(OKXExecutionClientFactory::new()),
        Box::new(exec_config),
    )?
    .with_reconciliation(false) // Simplified; enable in production
    .with_delay_post_stop_secs(5)
    .build()?;
```

:::warning
ตัวอย่างนี้ปิด reconciliation เพื่อความเรียบง่าย ใน production ให้ลบ `.with_reconciliation(false)` ออก
เพื่อให้ engine ปรับ state ใน cache ให้ตรงกับ venue ตอนเริ่มทำงาน ดู
[Execution reconciliation](../concepts/execution/reconciliation.md)
:::

## เพิ่ม strategy และรัน {#add-strategies-and-run}

```rust
use nautilus_model::{identifiers::InstrumentId, types::Quantity};
use nautilus_trading::examples::strategies::{
    GridMarketMaker, GridMarketMakerConfig,
};

let mut config = GridMarketMakerConfig::builder()
    .instrument_id(InstrumentId::from("ETH-USDT-SWAP.OKX"))
    .max_position(Quantity::from("0.10"))
    .num_levels(3)
    .grid_step_bps(100)
    .skew_factor(0.5)
    .requote_threshold_bps(10)
    .expire_time_secs(8)
    .on_cancel_resubmit(true)
    .build();

// OKX rejects hyphens in client order IDs
config.base.use_hyphens_in_client_order_ids = false;

let strategy = GridMarketMaker::new(config);

node.add_strategy(strategy)?;
node.run().await?;
```

node จะทำงานไปจนกว่าจะถูก interrupt (Ctrl+C) หรือถูกสั่ง shut down จากโค้ด

## Environment variable {#environment-variables}

OKX อ่าน API credential จาก environment variable ใช้ไฟล์ `.env` ร่วมกับ `dotenvy` หรือตั้งค่าใน shell:

```bash
export OKX_API_KEY="your_api_key"
export OKX_API_SECRET="your_api_secret"
export OKX_API_PASSPHRASE="your_passphrase"
```

สำหรับ demo trading ให้ตั้ง `.environment(OKXEnvironment::Demo)` บน config builder ทั้งสองตัว และใช้
API credential สำหรับ demo จาก OKX

adapter แต่ละตัวระบุ environment variable ที่ต้องใช้ไว้ใน[คู่มือ integration](../integrations/) ของ venue นั้น

## Async runtime {#async-runtime}

`LiveNode::run()` เป็น async และต้องใช้ Tokio runtime ใส่ `#[tokio::main]` ที่ฟังก์ชัน main:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // ... node setup ...

    node.run().await?;
    Ok(())
}
```

## ตัวอย่างของ adapter {#adapter-examples}

adapter ส่วนใหญ่มีตัวอย่างที่รันได้ ทั้ง data tester และ execution tester:

| Adapter             | ไดเรกทอรีตัวอย่าง                                |
| ------------------- | ----------------------------------------------- |
| Architect AX        | `crates/adapters/architect_ax/examples/`        |
| Betfair             | `crates/adapters/betfair/examples/`             |
| Binance             | `crates/adapters/binance/examples/`             |
| Blockchain          | `crates/adapters/blockchain/examples/`          |
| Bybit               | `crates/adapters/bybit/examples/`               |
| Coinbase            | `crates/adapters/coinbase/examples/`            |
| Databento           | `crates/adapters/databento/examples/`           |
| Deribit             | `crates/adapters/deribit/examples/`             |
| Derive              | `crates/adapters/derive/examples/`              |
| dYdX                | `crates/adapters/dydx/examples/`                |
| Hyperliquid         | `crates/adapters/hyperliquid/examples/`         |
| Interactive Brokers | `crates/adapters/interactive_brokers/examples/` |
| Kraken              | `crates/adapters/kraken/examples/`              |
| Lighter             | `crates/adapters/lighter/examples/`             |
| OKX                 | `crates/adapters/okx/examples/`                 |
| Polymarket          | `crates/adapters/polymarket/examples/`          |
| Sandbox             | `crates/adapters/sandbox/examples/`             |
| Tardis              | `crates/adapters/tardis/examples/`              |
