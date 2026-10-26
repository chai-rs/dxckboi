# เขียน Actor (Rust)

actor รับ market data, custom data/signal และ system event แต่ไม่ได้จัดการ order
คู่มือนี้จะพาสร้าง `SpreadMonitor` ที่ subscribe quote และ log ค่า bid-ask spread

สำหรับพื้นฐานเรื่อง actor, trait และการ dispatch handler ดูคู่มือ concept
[Actors](../concepts/actors.md) และ [Rust](../concepts/rust.md)

## กำหนด struct {#define-the-struct}

actor เป็นเจ้าของ `DataActorCore` และ state ที่ตัวเองต้องใช้ core เก็บ runtime state ของ actor ไว้
โดยปกติ user code จะเข้าถึง state นั้นผ่าน facade method ของ `DataActor` เช่น:

- `clock()`
- `cache()`
- `config()`
- `actor_id()`
- `trader_id()`
- method สำหรับ subscription

```rust
use nautilus_common::{nautilus_actor, actor::{DataActor, DataActorConfig, DataActorCore}};
use nautilus_model::{data::QuoteTick, identifiers::{ActorId, InstrumentId}};

pub struct SpreadMonitor {
    core: DataActorCore,
    instrument_id: InstrumentId,
}
```

## Implement constructor {#implement-the-constructor}

สร้าง `DataActorConfig` พร้อม actor ID แล้วส่งให้ `DataActorCore::new`
ฟิลด์ของ config ใช้ `Option` ที่มีค่าเริ่มต้น ดังนั้น `..Default::default()` จะครอบคลุมทุกอย่างยกเว้น actor ID

```rust
impl SpreadMonitor {
    pub fn new(instrument_id: InstrumentId) -> Self {
        let config = DataActorConfig {
            actor_id: Some(ActorId::from("SPREAD_MON-001")),
            ..Default::default()
        };
        Self {
            core: DataActorCore::new(config),
            instrument_id,
        }
    }
}
```

## เชื่อม core และ implement Debug {#wire-up-the-core-and-implement-debug}

macro `nautilus_actor!` เชื่อมฟิลด์ `DataActorCore` ของ actor เข้ากับ runtime contract โดยค่าเริ่มต้นจะ delegate
ไปยังฟิลด์ชื่อ `core` หากใช้ชื่อฟิลด์อื่นให้ส่ง argument ตัวที่สองเข้าไป callback ทั่วไปไม่ต้องเรียก native accessor ที่
macro generate ขึ้น ให้ใช้ facade method ของ `DataActor` บน `self` แทน

การ register กับ runtime ใช้ blanket implementation ของ `Actor` และ `Component`
macro จัดเตรียมการเชื่อมต่อกับ native runtime ให้ ส่วน `Debug` ให้ implement เองหรือ derive เอา

```rust
nautilus_actor!(SpreadMonitor);

impl std::fmt::Debug for SpreadMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpreadMonitor").finish()
    }
}
```

## Implement trait DataActor {#implement-the-dataactor-trait}

override handler method เพื่อรับข้อมูล handler ทุกตัวมี implementation เริ่มต้นแบบ no-op ดังนั้น override เฉพาะตัวที่
ต้องการก็พอ handler แต่ละตัวคืนค่า `anyhow::Result<()>`

```rust
impl DataActor for SpreadMonitor {
    fn on_start(&mut self) -> anyhow::Result<()> {
        self.subscribe_quotes(self.instrument_id, None, None);
        Ok(())
    }

    fn on_quote(&mut self, quote: &QuoteTick) -> anyhow::Result<()> {
        let spread = quote.ask_price.as_f64() - quote.bid_price.as_f64();
        log::info!("Spread: {spread:.5}");
        Ok(())
    }
}
```

`subscribe_quotes` เรียกใช้ได้โดยตรงบน `self` ผ่าน trait `DataActor` ดู handler ทั้งหมดที่มีได้ใน
[ตาราง handler](../concepts/rust.md#handler-methods)

## การเข้าถึง native runtime {#native-runtime-access}

โดยค่าเริ่มต้นให้ใช้ facade สาธารณะของ `DataActor` เพิ่ม `DataActorNative` เฉพาะเมื่อต้องการ access path แบบ
native-only อย่างชัดเจนที่ facade method ให้ไม่ได้ property แบบอ่านอย่างเดียวมีให้ใช้บน facade:

- `config()`
- `actor_id()`
- `trader_id()`
- `is_registered()`

ส่วน [Rust native traits](../concepts/rust.md#native-traits) ครอบคลุม applicability matrix ของ native trait และ
ตาราง method นี้:

- [method ของ `DataActorNative`](../concepts/rust.md#dataactornative-methods)

actor ที่ต้องการความ portable จึงควรใช้ facade method เช่น:

- `clock()`
- `cache()`

## Register actor {#register-the-actor}

กับ `BacktestEngine`:

```rust
let actor = SpreadMonitor::new(instrument_id);
engine.add_actor(actor)?;
```

กับ `LiveNode`:

```rust
let actor = SpreadMonitor::new(instrument_id);
node.add_actor(actor)?;
```

## ความปลอดภัยของ guard {#guard-safety}

เมื่อระบบ dispatch message ไปยัง actor ของคุณ ระบบจะขอ guard `ActorRef` อายุสั้นจาก registry คุณไม่ต้องจัดการ guard
เหล่านี้เอง แต่หากเขียน code ที่เข้าถึง actor ตัวอื่นภายใน callback ให้ทำตามกฎเหล่านี้:

- ค้นหา actor ด้วย ID ทุกครั้ง อย่า cache `ActorRef` ไว้
- drop guard ก่อนที่ scope จะจบ ห้ามเก็บไว้ในฟิลด์
- ห้ามถือ guard ข้ามจุด `.await`

method สำหรับ subscription บน `DataActorCore` จัดการเรื่องนี้ได้ถูกต้องอยู่แล้ว โดยจับ actor ID ไว้และทำการค้นหาภายใน
callback closure ดูโมเดลด้าน threading และ registry ฉบับเต็มได้ที่
[Runtime invariants](../developer_guide/rust.md#runtime-invariants)

## ตัวอย่างฉบับเต็ม {#full-example}

ดู
[`BookImbalanceActor`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/trading/src/examples/actors/imbalance)
สำหรับ actor ที่สมบูรณ์กว่า ซึ่งติดตาม state แยกตาม instrument และพิมพ์สรุปเมื่อหยุดทำงาน
