---
title: "รัน backtest (Rust)"
---

Nautilus มี Rust API สำหรับ backtest สองแบบ คือ `BacktestEngine` (low-level) และ `BacktestNode`
(high-level พร้อม streaming ข้อมูลจาก catalog) คู่มือนี้ครอบคลุมทั้งสองแบบ

พื้นฐานเรื่องแนวคิดของ backtest, fill model และพฤติกรรมของ matching engine ดูได้ที่คู่มือแนวคิด
[Backtesting](../concepts/backtesting/) ส่วนการตั้งค่าโปรเจกต์และ feature flag ดูได้ที่คู่มือแนวคิด
[Rust](../concepts/rust.md#project-setup)

## Dependency [#dependencies]

เพิ่มรายการต่อไปนี้ลงใน `Cargo.toml` โดย `streaming` และ `nautilus-persistence` จำเป็นเฉพาะเมื่อใช้
`BacktestNode` API แบบ high-level

```toml
[dependencies]
nautilus-backtest = { version = "0.66", features = ["streaming"] }
nautilus-execution = "0.66"
nautilus-model = { version = "0.66", features = ["test-support"] }
nautilus-persistence = "0.66"
nautilus-trading = { version = "0.66", features = ["examples"] }

ahash = "0.8"
anyhow = "1"
tempfile = "3"
ustr = "1"
```

ถ้าต้องการใช้แค่ `BacktestEngine` แบบ low-level ให้ตัด `streaming`, `nautilus-persistence`,
`tempfile` และ `ustr` ออก

## BacktestEngine (low-level API) [#backtestengine-low-level-api]

API แบบ low-level ให้ควบคุมได้โดยตรง: สร้าง engine เอง เพิ่ม venue และ instrument โหลดข้อมูลไว้ใน
memory ลงทะเบียน strategy แล้วสั่งรัน

### 1. สร้าง engine [#1-create-the-engine]

```rust
use nautilus_backtest::{config::BacktestEngineConfig, engine::BacktestEngine};

let mut engine = BacktestEngine::new(BacktestEngineConfig::default())?;
```

### 2. เพิ่ม venue [#2-add-a-venue]

`SimulatedVenueConfig` ใช้ `bon::Builder`: ต้องตั้งค่าเฉพาะฟิลด์ที่จำเป็น ส่วนค่าอื่นจะใช้ค่า default
ที่ระบุไว้ในเอกสาร `build()` จะตรวจสอบความถูกต้องของ config และคืนค่าเป็น `ConfigResult` จึงต้อง
propagate หรือ unwrap ผลลัพธ์

```rust
use nautilus_backtest::config::SimulatedVenueConfig;
use nautilus_execution::models::fee::{FeeModelAny, MakerTakerFeeModel};
use nautilus_model::{
    enums::{AccountType, BookType, OmsType},
    identifiers::Venue,
    types::Money,
};

engine.add_venue(
    SimulatedVenueConfig::builder()
        .venue(Venue::from("SIM"))
        .oms_type(OmsType::Hedging)
        .account_type(AccountType::Margin)
        .book_type(BookType::L1_MBP)
        .starting_balances(vec![Money::from("1_000_000 USD")])
        .fee_model(FeeModelAny::MakerTaker(MakerTakerFeeModel::zero()).into())
        .build()?,
)?;
```

เปลี่ยนค่า default ใดก็ได้ด้วยการต่อ setter เช่น `.reject_stop_orders(false)` หรือ
`.allow_cash_borrowing(true)`

### 3. เพิ่ม instrument และข้อมูล [#3-add-instruments-and-data]

```rust
use nautilus_model::instruments::{
    Instrument, InstrumentAny, stubs::audusd_sim,
};

let instrument = InstrumentAny::CurrencyPair(audusd_sim());
let instrument_id = instrument.id();
engine.add_instrument(&instrument)?;

let quotes = generate_quotes(instrument_id); // Your data loading function
engine.add_data(quotes, None, true, true)?;
```

### 4. ลงทะเบียน strategy แล้วรัน [#4-register-a-strategy-and-run]

```rust
use nautilus_model::types::Quantity;
use nautilus_trading::examples::strategies::EmaCross;

let strategy = EmaCross::new(
    instrument_id,
    Quantity::from("100000"),
    10, // fast EMA period
    20, // slow EMA period
);

engine.add_strategy(strategy)?;
engine.run(None, None, None, false)?;
```

### รันตัวอย่างฉบับเต็ม [#run-the-full-example]

```bash
cargo run -p nautilus-backtest --features examples --example engine-ema-cross
```

ซอร์สโค้ด:
[`crates/backtest/examples/engine_ema_cross.rs`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/backtest/examples/engine_ema_cross.rs)

## BacktestNode (high-level API) [#backtestnode-high-level-api]

API แบบ high-level โหลดข้อมูลจาก `ParquetDataCatalog` และ stream เข้ามาเป็น chunk ตามขนาดที่กำหนดได้
ต้องเปิด feature `streaming` ของ `nautilus-backtest`

### 1. เขียนข้อมูลลง catalog [#1-write-data-to-a-catalog]

```rust
use nautilus_model::instruments::{
    Instrument, InstrumentAny, stubs::audusd_sim,
};
use nautilus_persistence::backend::parquet::catalog::ParquetDataCatalog;
use tempfile::TempDir;

let instrument = InstrumentAny::CurrencyPair(audusd_sim());
let instrument_id = instrument.id();
let quotes = generate_quotes(instrument_id);

let temp_dir = TempDir::new()?;
let catalog_path = temp_dir.path().to_str()
    .context("temp dir path is not valid UTF-8")?
    .to_string();
let catalog = ParquetDataCatalog::new(
    temp_dir.path(), None, None, None, None,
);

catalog.write_instruments(vec![instrument])?;
catalog.write_to_parquet(&quotes, None, None, None)?;
```

### 2. ตั้งค่าการรัน [#2-configure-the-run]

```rust
use nautilus_backtest::config::{
    BacktestDataConfig, BacktestRunConfig, BacktestVenueConfig, NautilusDataType,
};
use nautilus_model::enums::{AccountType, BookType, OmsType};

let venue_config = BacktestVenueConfig::builder()
    .name("SIM")
    .oms_type(OmsType::Hedging)
    .account_type(AccountType::Margin)
    .book_type(BookType::L1_MBP)
    .starting_balances(vec!["1_000_000 USD".to_string()])
    .build()?;

let data_config = BacktestDataConfig::builder()
    .data_type(NautilusDataType::QuoteTick)
    .catalog_path(catalog_path)
    .instrument_id(instrument_id)
    .build()?;

let run_config = BacktestRunConfig::builder()
    .id("ema-cross-run".to_string())
    .venues(vec![venue_config])
    .data(vec![data_config])
    .chunk_size(100)
    .build()?;
```

### 3. build เพิ่ม strategy แล้วรัน [#3-build-add-strategies-and-run]

```rust
use nautilus_backtest::node::BacktestNode;
use nautilus_model::types::Quantity;
use nautilus_trading::examples::strategies::EmaCross;

let mut node = BacktestNode::new(vec![run_config])?;
node.build()?;

let engine = node.get_engine_mut("ema-cross-run")
    .context("engine not found for run config ID")?;
let strategy = EmaCross::new(
    instrument_id,
    Quantity::from("100000"),
    10,
    20,
);
engine.add_strategy(strategy)?;

node.run()?;
```

### รันตัวอย่างฉบับเต็ม [#run-the-full-example-1]

```bash
cargo run -p nautilus-backtest --features examples,streaming --example node-ema-cross
```

ซอร์สโค้ด:
[`crates/backtest/examples/node_ema_cross.rs`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/backtest/examples/node_ema_cross.rs)
