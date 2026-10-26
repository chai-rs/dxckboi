# Rust

Nautilus มี implementation ที่สมบูรณ์ด้วย Rust อยู่ใต้ไดเรกทอรี `crates/`
คุณเขียน actor, strategy, รัน backtest และทำ live trading ได้โดยไม่ต้องใช้ Python เลย

:::warning
Rust API ยังอยู่ระหว่างการพัฒนาอย่างต่อเนื่อง signature ของ method และข้อกำหนดของ trait
อาจเปลี่ยนแปลงได้ระหว่าง release
:::

## ส่วนประกอบของระบบ {#system-implementations}

implementation แบบ Rust ล้วนอยู่ใต้ `crates/` และรันได้โดยไม่ต้องมี Python runtime ให้ performance แบบ native
และมีฟังก์ชันการเทรดหลักครบทั้งหมด เหมาะกับ deployment ที่อ่อนไหวต่อ latency หรือทีมที่ชอบภาษาแบบ compile

### ตารางความสามารถ {#capability-matrix}

| Component       | Rust |
| --------------- | ---- |
| Strategy        | ✓    |
| Actor           | ✓    |
| DataEngine      | ✓    |
| ExecutionEngine | ✓    |
| RiskEngine      | ✓    |
| BacktestEngine  | ✓    |
| BacktestNode    | ✓    |
| LiveNode        | ✓    |
| OrderEmulator   | ✓    |
| Matching engine | ✓    |
| Portfolio       | ✓    |
| Accounts        | ✓    |
| Cache           | ✓    |
| MessageBus      | ✓    |
| Data catalog    | ✓    |
| Indicators      | ✓    |
| Exec algorithms | TWAP |
| Controller      | -    |
| Tearsheets      | -    |

:::note
runtime ของ Controller เขียนด้วย Rust แต่ตารางระบุว่าไม่มีใน Rust เพราะเส้นทางการลงทะเบียนที่รองรับ
(controller config ที่ import ได้) ยังใช้จาก Rust ไม่ได้
:::

### Adapters {#adapters}

| Adapter             | Rust |
| ------------------- | ---- |
| Architect AX        | ✓    |
| Betfair             | ✓    |
| Binance             | ✓    |
| Blockchain          | ✓    |
| Bybit               | ✓    |
| Coinbase            | ✓    |
| Databento           | ✓    |
| Deribit             | ✓    |
| Derive              | ✓    |
| dYdX                | ✓    |
| Hyperliquid         | ✓    |
| Interactive Brokers | ✓    |
| Kraken              | ✓    |
| Lighter             | ✓    |
| OKX                 | ✓    |
| Polymarket          | ✓    |
| Sandbox             | ✓    |
| Tardis              | ✓    |

## การตั้งค่าโปรเจกต์ {#project-setup}

crate ของ Nautilus publish อยู่บน
[crates.io](https://crates.io/crates/nautilus-backtest) เพิ่มลงใน
`Cargo.toml` ของคุณ:

```toml
[dependencies]
nautilus-backtest = "0.66"
nautilus-common = "0.66"
nautilus-execution = "0.66"
nautilus-model = { version = "0.66", features = ["test-support"] }
nautilus-trading = { version = "0.66", features = ["examples"] }

anyhow = "1"
log = "0.4"
```

สำหรับ live trading ให้เพิ่ม crate live และ adapter ของ venue ที่ใช้:

```toml
[dependencies]
nautilus-live = "0.66"
nautilus-okx = "0.66"
```

ถ้าต้องการตาม branch พัฒนาล่าสุด ให้ชี้ dependency ของ Nautilus ทั้งหมดไปที่ git source เดียวกัน เพื่อเลี่ยง
type ไม่ตรงกันระหว่างเวอร์ชันจาก crates.io กับ git:

```toml
[dependencies]
nautilus-backtest = { git = "https://github.com/nautechsystems/nautilus_trader.git", branch = "develop" }
nautilus-common = { git = "https://github.com/nautechsystems/nautilus_trader.git", branch = "develop" }
nautilus-execution = { git = "https://github.com/nautechsystems/nautilus_trader.git", branch = "develop" }
nautilus-model = { git = "https://github.com/nautechsystems/nautilus_trader.git", branch = "develop", features = ["test-support"] }
nautilus-trading = { git = "https://github.com/nautechsystems/nautilus_trader.git", branch = "develop", features = ["examples"] }
```

Rust เวอร์ชันต่ำสุดที่รองรับ (MSRV) คือ **1.99.1**

### Feature flags {#feature-flags}

| Flag             | Crate               | ผล                                                    |
| ---------------- | ------------------- | ---------------------------------------------------- |
| `high-precision` | `nautilus-model`    | fixed precision 16 หลัก (ค่า default คือ 9)            |
| `test-support`   | `nautilus-model`    | test fixture, builder, spec และค่า default            |
| `examples`       | `nautilus-trading`  | strategy ตัวอย่าง (`EmaCross`, `GridMarketMaker`)     |
| `streaming`      | `nautilus-backtest` | การ stream ข้อมูลจาก catalog ผ่าน `BacktestNode`       |
| `defi`           | `nautilus-model`    | data type สำหรับ DeFi เปิด `high-precision` ให้ด้วย     |

:::tip
precision มาตรฐาน 9 หลักเพียงพอสำหรับ instrument การเงินแบบดั้งเดิมส่วนใหญ่
เปิด `high-precision` สำหรับ venue crypto ที่ราคาหรือ quantity ต้องการทศนิยมมากกว่าเก้าตำแหน่ง
:::

### Memory allocator {#memory-allocator}

CLI `nautilus` ใช้ [mimalloc](https://crates.io/crates/mimalloc) สำหรับการจองหน่วยความจำของ Rust
binary ของ Rust จะเลือก allocator ของตัวเอง ดังนั้นให้เพิ่ม mimalloc ลงใน binary ของคุณเพื่อให้ตรงกัน:

```toml
[dependencies]
mimalloc = "0.1"
```

```rust
use mimalloc::MiMalloc;
use nautilus_common::logging::headers::register_allocator_mimalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    register_allocator_mimalloc();
}
```

การประกาศ `GLOBAL` คือการเลือกใช้ mimalloc ให้เรียก `register_allocator_mimalloc` ตอนต้นของ `main` ก่อน
สร้าง Nautilus node เพื่อให้ version header รายงาน `allocator: mimalloc <version>` การลงทะเบียนนี้
**อัปเดตเฉพาะ metadata ใน header** ไม่ได้เป็นการเลือก allocator

system allocator แบบ default ก็ใช้ได้เช่นกัน เมื่อเปรียบเทียบ allocator ให้วัด throughput และ resident memory บน
workload และ platform ของคุณเอง ดูพื้นหลังเพิ่มเติมได้ที่ [architecture guide](architecture.md#memory-allocation)

## Actors {#actors}

actor รับ market data, custom data/signal และ system event แต่ไม่จัดการ order ให้ implement trait `DataActor`
และใช้ `nautilus_actor!` เพื่อเชื่อมฟิลด์ `DataActorCore` ของคุณเข้ากับ runtime contract type ของคุณต้อง implement
หรือ derive `Debug` ส่วน macro จะจัดการการเชื่อมต่อกับ native runtime ให้

โดยปกติโค้ดของผู้ใช้จะใช้ facade method ของ `DataActor` สำหรับการ subscribe, การเข้าถึง cache และ clock

### Handler methods {#handler-methods}

override handler ใดก็ได้บน trait `DataActor` เพื่อรับข้อมูลหรือ event ที่ตรงกัน handler ทุกตัวมี implementation
default ที่ไม่ทำอะไร (no-op) คุณจึง override เฉพาะที่ต้องใช้

| Handler                | รับอะไร                    |
| ---------------------- | ------------------------- |
| `on_start`             | actor เริ่มทำงาน            |
| `on_stop`              | actor หยุดทำงาน            |
| `on_quote`             | `QuoteTick`               |
| `on_trade`             | `TradeTick`               |
| `on_bar`               | `Bar`                     |
| `on_book_deltas`       | `OrderBookDeltas`         |
| `on_book`              | `OrderBook` (ตาม interval) |
| `on_instrument`        | `InstrumentAny`           |
| `on_mark_price`        | `MarkPriceUpdate`         |
| `on_index_price`       | `IndexPriceUpdate`        |
| `on_funding_rate`      | `FundingRateUpdate`       |
| `on_option_greeks`     | `OptionGreeks`            |
| `on_option_chain`      | `OptionChainSlice`        |
| `on_instrument_status` | `InstrumentStatus`        |
| `on_time_event`        | `TimeEvent`               |

ดูคำอธิบายทีละขั้นได้ที่คู่มือ [เขียน Actor (Rust)](../how_to/write_rust_actor.md)
และดูตัวอย่างที่สมบูรณ์ได้ที่
[`BookImbalanceActor`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/trading/src/examples/actors/imbalance)

## Strategies {#strategies}

strategy ขยาย actor ด้วยความสามารถในการจัดการ order ให้ implement `DataActor` สำหรับการจัดการข้อมูล และใช้
`nautilus_strategy!` เพื่อเชื่อมฟิลด์ `StrategyCore` ของคุณเข้ากับ strategy runtime contract `StrategyCore`
เก็บ state ของ strategy ตอน runtime และการลงทะเบียนกับ runtime ต้องใช้ native wiring ที่ macro สร้างให้

logic ของ strategy ตามปกติใช้ method ของ `Strategy` และ facade method บน `self` เพื่อเข้าถึง state นั้น
strategy ยัง override order event handler บน trait `Strategy` ได้ด้วย เช่น `on_order_filled` (`OrderFilled`) และ
`on_order_canceled` (`OrderCanceled`)

### การจัดการ order {#order-management}

trait `Strategy` มี method สำหรับ order ผ่าน facade:

| Method                | การทำงาน                                    |
| --------------------- | ----------------------------------------- |
| `submit_order`        | submit order ใหม่ไปยัง venue               |
| `submit_order_list`   | submit รายการ contingent order            |
| `modify_order`        | แก้ไข price, quantity หรือ trigger price    |
| `cancel_order`        | cancel order ที่ระบุ                        |
| `cancel_orders`       | cancel ชุด order ที่กรองไว้                  |
| `cancel_all_orders`   | cancel order ทั้งหมดของ instrument หนึ่ง     |
| `close_position`      | ปิด position ด้วย market order             |
| `close_all_positions` | ปิด position ที่เปิดอยู่ทั้งหมด                |

`OrderApi` (เข้าถึงผ่าน `self.order()`) ใช้สร้าง order และ order list:

- `generate_client_order_id`
- `generate_order_list_id`
- `market`
- `limit`
- `stop_market`
- `stop_limit`
- `market_to_limit`
- `market_if_touched`
- `limit_if_touched`
- `trailing_stop_market`
- `trailing_stop_limit`
- `bracket`
- `create_list`

### การเข้าถึง cache และ clock {#cache-and-clock-access}

การเข้าถึง cache และ clock รายงาน borrow conflict ผ่าน interface ที่ต่างกัน:

- lookup แบบ `try_*` ของ cache facade คืน variant `Access` ของ lookup error เมื่อเกิด borrow conflict ส่วน
  `NotFound` ยังคงหมายถึงไม่มีข้อมูลที่ร้องขอ
- method `get()` ของ cache facade และ method ตั้งเวลาของ clock ที่ใช้ native backend คืน access error ผ่าน
  `anyhow::Result`
- accessor แบบ infallible ที่มีอยู่เดิมยังคง panic เมื่อเกิด conflict

ให้ปล่อย borrow ที่ขัดแย้งกันก่อน retry error เหล่านี้ไม่ได้เลื่อน callback หรือเปลี่ยนลำดับการส่ง callback

### Macro สำหรับเชื่อม core {#core-wiring-macros}

actor, strategy และ execution algorithm ของ Rust เก็บ runtime core ไว้เป็นฟิลด์ของ struct macro จะบอก trait
ว่าฟิลด์นั้นอยู่ที่ไหน

| Macro                                          | Core field               | สร้างอะไร                       |
| ---------------------------------------------- | ------------------------ | ------------------------------ |
| `nautilus_actor!(Type)`                        | `DataActorCore`          | Runtime wiring                 |
| `nautilus_strategy!(Type)`                     | `StrategyCore`           | Runtime wiring และ `Strategy`  |
| `nautilus_execution_algorithm!(Type, { ... })` | `ExecutionAlgorithmCore` | Runtime wiring และ algorithm   |

macro คาดหวังฟิลด์ชื่อ `core` ถ้าจำเป็นให้ส่งชื่อฟิลด์เป็น argument ตัวที่สอง macro ไม่ได้ทำให้ actor, strategy
หรือ `StrategyCore` deref ไปยังส่วนภายในของ runtime macro ของ execution algorithm รับ implementation block
ของ `on_order()` เพราะ method นั้นนิยามการจัดการ order ที่ algorithm ต้องมี

### Native trait {#native-traits}

โดย default ให้ใช้ facade method:

- `actor_id()`
- `trader_id()`
- `is_registered()`
- `config()`
- `strategy_id()`
- `clock()`
- `cache()`
- `order()`
- `portfolio()`

`DataActorNative`, `StrategyNative` และ `ExecutionAlgorithmNative` มีไว้สำหรับการเข้าถึงแบบ native-only ที่อยู่
ต่ำกว่า facade หัวข้อนี้อธิบายโค้ด Rust แบบ native สำหรับ engine, runtime และ path ที่อ่อนไหวต่อ latency อย่างชัดเจน
ไม่ใช่วิธีเขียน component แบบ portable ตามปกติ

| วิธีเขียน                    | ใช้ native trait? | API ปกติ                              |
| ------------------------- | ---------------- | ----------------------------------- |
| Native Rust binary        | เฉพาะเมื่อจำเป็น     | facade ของ `Strategy` และ `DataActor` |

native trait เปิดให้เข้าถึง core state แบบ borrow, `Rc<RefCell<_>>` และ reference ของ runtime ใช้เมื่อโค้ด Rust
แบบ native ตั้งใจยอมรับกฎการ borrow เหล่านั้นสำหรับ path ที่อ่อนไหวต่อ latency อย่างชัดเจน

โค้ดของ engine, runtime, registration และ testkit สามารถ import `DataActorNative`, `StrategyNative` หรือ
`ExecutionAlgorithmNative` เมื่อต้องเข้าถึง actor core, strategy core หรือ execution algorithm core อย่าใช้ใน
logic ทั่วไปของ actor, strategy หรือ execution algorithm แบบ portable

`ExecutionAlgorithmCore` เป็นเจ้าของ `DataActorCore` แต่ไม่ได้ deref ไปยังมัน
logic ปกติของ execution algorithm ควรใช้ `id()`, `actor_id()`, `trader_id()`, `clock()` และ `cache()`
ใช้ `ExecutionAlgorithmNative` เฉพาะเมื่อโค้ดต้องการ state ของ execution algorithm แบบ native

ให้เลือก native handle ที่เล็กที่สุดและจำกัดขอบเขตของแต่ละ borrow ใช้ `order()` สำหรับการสร้าง order ตามปกติของ
strategy และใช้ `order_factory()` เฉพาะเมื่อโค้ด native ต้องการ borrow แบบ mutable ของ factory โดยตรง

#### Method ของ `DataActorNative` {#dataactornative-methods}

| Native method | Return shape             | ใช้เมื่อ                                 |
| ------------- | ------------------------ | --------------------------------------- |
| `core()`      | `&DataActorCore`         | อ่านส่วนภายในของ actor                   |
| `core_mut()`  | `&mut DataActorCore`     | แก้ไขส่วนภายในของ actor                  |
| `clock_mut()` | `RefMut<'_, dyn Clock>`  | ต้องการ borrow clock แบบ mutable         |
| `clock_rc()`  | `Rc<RefCell<dyn Clock>>` | เก็บหรือส่งต่อ clock ที่ใช้ร่วมกัน             |
| `cache_ref()` | `Ref<'_, Cache>`         | ต้องการอ่าน live cache แบบสั้น ๆ           |
| `cache_rc()`  | `Rc<RefCell<Cache>>`     | แก้ไข เก็บ หรือส่งต่อ cache                |

ใช้ `try_cache_ref()` และ `try_clock_mut()` เพื่อจัดการ native borrow ที่ล้มเหลวโดยไม่ panic ให้ import
`ComponentAccessError` จาก `nautilus_common::component`:

- `try_cache_ref()` คืน `ComponentAccessError::ReadConflict` เมื่อ cache ถูก borrow แบบ mutable อยู่
- `try_clock_mut()` คืน `ComponentAccessError::WriteConflict` เมื่อ clock ถูก borrow อยู่แล้ว
- ทั้งคู่คืน `ComponentAccessError::NotRegistered` เมื่อการลงทะเบียนยังไม่ได้จัดหา resource นั้นให้

error จะระบุ resource และการทำงานที่พยายามทำ การ reentry ของ callback อาจทำให้เกิด conflict ได้ แต่ conflict
เพียงอย่างเดียวไม่ได้บอกสาเหตุ ดูความหมายของข้อความและวิธีแก้ได้ที่ [reentrancy
diagnostics](../developer_guide/callback_dispatch.md#reentrancy-and-dispatch-diagnostics)

#### Method ของ `StrategyNative` {#strategynative-methods}

| Native method         | Return shape                | ใช้เมื่อ                                 |
| --------------------- | --------------------------- | --------------------------------------- |
| `strategy_core()`     | `&StrategyCore`             | อ่านส่วนภายในของ strategy                |
| `strategy_core_mut()` | `&mut StrategyCore`         | แก้ไขส่วนภายในของ strategy               |
| `order_factory()`     | `RefMut<'_, OrderFactory>`  | ต้องการ borrow factory แบบ mutable โดยตรง |
| `order_factory_rc()`  | `Rc<RefCell<OrderFactory>>` | เก็บหรือส่งต่อ factory                     |
| `portfolio_rc()`      | `Rc<RefCell<Portfolio>>`    | เก็บหรือส่งต่อ portfolio                   |

#### Method ของ `ExecutionAlgorithmNative` {#executionalgorithmnative-methods}

| Native method               | Return shape                  | ใช้เมื่อ                                     |
| --------------------------- | ----------------------------- | ------------------------------------------- |
| `exec_algorithm_core()`     | `&ExecutionAlgorithmCore`     | อ่านส่วนภายในของ execution algorithm         |
| `exec_algorithm_core_mut()` | `&mut ExecutionAlgorithmCore` | แก้ไขส่วนภายในของ execution algorithm        |

ดูคำอธิบายทีละขั้นได้ที่คู่มือ [เขียน Strategy (Rust)](../how_to/write_rust_strategy.md)
และดูตัวอย่างที่สมบูรณ์ได้ที่
[`EmaCross`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/trading/src/examples/strategies/ema_cross)
และ
[`GridMarketMaker`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/trading/src/examples/strategies/grid_mm)

### การรัน Rust component {#running-rust-components}

ตัวอย่างด้านล่างใช้ strategy แต่ใช้รูปแบบเดียวกันกับ actor ผ่าน `add_actor`

#### Rust ล้วน {#pure-rust}

เขียน strategy และฟังก์ชัน `main` ด้วย Rust แล้ว build เป็น binary แบบ standalone ด้วย `cargo build`
วิธีนี้ไม่ต้องมี Python runtime

```rust
let strategy = GridMarketMaker::new(config);
node.add_strategy(strategy)?;
node.run().await?;
```

ดูคำอธิบายฉบับเต็มได้ที่ [Run Live Trading (Rust)](../how_to/run_rust_live_trading.md)

## Backtesting {#backtesting}

ดูคำอธิบายพร้อม annotation ของทั้งสอง API ได้ที่คู่มือ
[Run a Backtest (Rust)](../how_to/run_rust_backtest.md)

### `BacktestEngine` (API ระดับล่าง) {#backtestengine-low-level-api}

สร้าง engine เพิ่ม venue และ instrument โหลดข้อมูล ลงทะเบียน strategy แล้วรัน ดูตัวอย่างที่ใช้งานได้ฉบับเต็ม:

```bash
cargo run -p nautilus-backtest --features examples --example engine-ema-cross
```

Source:
[`crates/backtest/examples/engine_ema_cross.rs`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/backtest/examples/engine_ema_cross.rs)

### `BacktestNode` (API ระดับสูง) {#backtestnode-high-level-api}

โหลดข้อมูลจาก `ParquetDataCatalog` และรองรับการ stream เป็น chunk ที่ตั้งขนาดได้ ต้องเปิด feature `streaming`
บน `nautilus-backtest` ดูตัวอย่างที่ใช้งานได้ฉบับเต็ม:

```bash
cargo run -p nautilus-backtest --features examples,streaming --example node-ema-cross
```

Source:
[`crates/backtest/examples/node_ema_cross.rs`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/backtest/examples/node_ema_cross.rs)

## Live trading {#live-trading}

ดูคำอธิบายพร้อม annotation ได้ที่คู่มือ
[Run Live Trading (Rust)](../how_to/run_rust_live_trading.md)

`LiveNode` เชื่อมต่อกับ venue และแหล่งข้อมูลจริงผ่าน adapter client โดยใช้ builder pattern ตั้งค่า data client และ
execution client จากนั้น `run()` จะเริ่ม async event loop adapter แต่ละตัวมี factory และ config type ของตัวเอง

| Adapter             | Example                                         |
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

adapter ส่วนใหญ่มีตัวอย่าง `node_data_tester.rs` และ `node_exec_tester.rs` ซึ่งทดสอบการ request ข้อมูล
การ stream และการ execute order กับ venue จริง

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [เขียน Actor (Rust)](../how_to/write_rust_actor.md) - คำอธิบาย actor ทีละขั้น
- [เขียน Strategy (Rust)](../how_to/write_rust_strategy.md) - คำอธิบาย strategy ทีละขั้น
- [Run a Backtest (Rust)](../how_to/run_rust_backtest.md) - การใช้ BacktestEngine และ BacktestNode
- [Run Live Trading (Rust)](../how_to/run_rust_live_trading.md) - การตั้งค่า LiveNode และการเชื่อมต่อ venue
- [Architecture](architecture.md) - การออกแบบระบบและ flow ของ data/execution
- [Actors](actors.md) - แนวคิดของ actor
- [Strategies](strategies.md) - แนวคิดของ strategy และ reference ของ handler
- [Events](events/) - ประเภทของ event และการ dispatch handler
- [Backtesting](backtesting/) - แนวคิดของ backtest และพฤติกรรมของ matching engine
