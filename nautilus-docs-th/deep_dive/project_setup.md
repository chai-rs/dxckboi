---
title: "ตั้งค่าโปรเจกต์"
description: "toolchain, การ pin dependency ของ NautilusTrader, feature flag และการวางโมดูลให้ test ง่าย"
---

## Toolchain [#toolchain]

NautilusTrader ประกาศ `rust-version` ใน `Cargo.toml` ของ workspace และ pin toolchain ไว้ใน `rust-toolchain.toml`
ที่ commit `04f766b4` ใช้ Rust **1.99.1** และ edition 2024 โปรเจกต์ของเราจึงล็อกเวอร์ชันเดียวกัน:

```toml
[toolchain]
channel = "1.99.1"
```

`rustup` จะติดตั้ง toolchain นี้ให้อัตโนมัติเมื่อสั่ง `cargo` ครั้งแรกในโฟลเดอร์นี้ ถ้าใช้ toolchain เก่ากว่า
`rust-version` ที่ crate ต้องการ cargo จะหยุดพร้อม error ตั้งแต่ขั้นตอน resolve

## Pin dependency ไว้กับ commit [#pin-dependencies]

API ฝั่ง Rust ของ Nautilus ยังเปลี่ยนเร็ว signature ของ method เปลี่ยนได้ระหว่างเวอร์ชันย่อย
และเวอร์ชันบน crates.io อาจไม่ตรงกับ `develop` ที่เอกสารอ้างถึง ทางที่ปลอดภัยที่สุดคือดึง crate จาก git
แล้ว pin ด้วย `rev` ให้ตรงกับ commit ที่โค้ดถูกเขียนและทดสอบ:

```toml
[dependencies]
nautilus-backtest = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be" }
nautilus-common = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be" }
nautilus-core = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be" }
nautilus-execution = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be" }
nautilus-indicators = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be" }
nautilus-model = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be", features = ["test-support"] }
nautilus-risk = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be" }
nautilus-trading = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be" }
ahash = "0.8"
anyhow = "1"
log = "0.4"
rust_decimal = "1"
```

ทุก crate ต้องชี้ไปที่ `rev` เดียวกัน ถ้าผสมเวอร์ชัน cargo จะ build crate ซ้ำสองชุด แล้ว type อย่าง `InstrumentId`
จากสองชุดจะเข้ากันไม่ได้ (error ประมาณ "expected `InstrumentId`, found `InstrumentId`")

เมื่อต้องการอัปเกรด ให้เปลี่ยน `rev` ทีเดียวทุกบรรทัด แล้วรัน `cargo test` ทั้งชุด
compiler จะชี้จุดที่ API เปลี่ยนให้เอง

### แต่ละ crate ใช้ทำอะไร [#crates]

| Crate | ใช้ในโปรเจกต์นี้สำหรับ |
| --- | --- |
| `nautilus-trading` | trait `Strategy`, `StrategyCore`, macro `nautilus_strategy!` |
| `nautilus-common` | trait `DataActor`, `TimeEvent`, `LoggerConfig`, `RateLimit` |
| `nautilus-model` | `Bar`, `BarType`, `Price`, `Quantity`, event, instrument |
| `nautilus-indicators` | `BollingerBands`, `AverageTrueRange` |
| `nautilus-backtest` | `BacktestEngine`, `SimulatedVenueConfig`, `BacktestResult` |
| `nautilus-execution` | fee model และ fill model ของ venue จำลอง |
| `nautilus-risk` | `RiskEngineConfig` |
| `nautilus-core` | `UnixNanos`, `DurationNanos` |

## Feature flag ที่ต้องรู้ [#features]

- **`nautilus-model/test-support`** เปิด `instruments::stubs` เช่น `audusd_sim()` ซึ่งเป็น instrument สำเร็จรูปสำหรับ backtest
  ถ้าใช้ instrument จริงจาก adapter หรือ catalog ก็ไม่ต้องเปิด
- **`high-precision`** เปลี่ยนชนิดข้อมูลภายในของ `Price` และ `Quantity` ให้แม่นยำขึ้น flag นี้มีผลกับ **ทุก crate ใน dependency graph**
  เพราะ cargo รวม feature เข้าด้วยกัน adapter บางตัว เช่น `nautilus-bybit` เปิด flag นี้เป็นค่า default
  โปรเจกต์นี้จึงประกาศ adapter ด้วย `default-features = false` (ดู [ไปสู่ live](live.md#dependencies))
- **`nautilus-live/node`** จำเป็นสำหรับ `LiveNode` ส่วน `nautilus-backtest/streaming` จำเป็นเฉพาะเมื่อใช้
  `BacktestNode` กับ Parquet catalog (บทนี้ใช้ `BacktestEngine` จึงไม่ต้องเปิด)

ส่วนของ live trading แยกไว้หลัง feature ของโปรเจกต์เอง เพื่อให้คนที่ทำแค่ backtest ไม่ต้อง compile adapter และ tokio:

```toml
nautilus-bybit = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be", default-features = false, optional = true }
nautilus-live = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be", default-features = false, features = ["node"], optional = true }
tokio = { version = "1", features = ["macros", "rt-multi-thread"], optional = true }

[features]
live = ["dep:nautilus-bybit", "dep:nautilus-live", "dep:tokio"]

[[bin]]
name = "live"
required-features = ["live"]
```

## วางโมดูลให้ test ได้ [#module-design]

กฎข้อเดียวที่ทำให้โปรเจกต์นี้ test ง่าย คือ **แยกการตัดสินใจออกจากการเชื่อมต่อ**

| ชั้น | ไฟล์ | รู้จัก Nautilus ไหม | test อย่างไร |
| --- | --- | --- | --- |
| กฎการเทรด | `signal.rs`, `sizing.rs` | ไม่ (รับ/คืน `f64` และ enum ของเราเอง) | unit test ธรรมดา รันเสร็จในเสี้ยววินาที |
| เชื่อมกับ runtime | `strategy.rs` | ใช่ | ผ่าน integration test |
| ประกอบระบบ | `backtest.rs`, `bin/live.rs` | ใช่ | integration test และการรันจริง |

strategy ที่เขียน logic ทั้งหมดไว้ใน `on_bar` test ได้ทางเดียวคือรัน engine ทั้งตัว ซึ่งช้าและหาสาเหตุยากเมื่อ test พัง
เมื่อแยก logic ออกเป็น pure function ก็ test กรณีขอบได้ทีละกรณีด้วยตัวเลขที่คำนวณด้วยมือได้

## Build และรัน [#build-and-run]

```bash
cargo test --release          # unit test + integration test
cargo run --release -- run    # backtest หนึ่งรอบพร้อม log
cargo run --release -- sweep  # parameter sweep แบบไม่มี log
```

ใช้ `--release` เสมอเมื่อรัน backtest เพราะ debug build ของ engine ช้ากว่ามาก build ครั้งแรกใช้เวลาหลายนาที
หลังจากนั้นจะ compile ใหม่เฉพาะโค้ดของเรา
