---
title: "เขียน Strategy (Rust)"
---

strategy คือ actor ที่เพิ่มความสามารถในการจัดการ order เข้ามา คู่มือนี้จะพาสร้าง strategy ขนาดเล็กที่ subscribe quote
และส่ง market order ควรอ่าน [เขียน Actor (Rust)](write_rust_actor.md) ก่อน

สำหรับพื้นฐานเรื่อง concept ของ strategy และการจัดการ order ดูคู่มือ concept
[Strategies](../concepts/strategies.md) และ [Rust](../concepts/rust.md)

## กำหนด struct [#define-the-struct]

strategy เก็บฟิลด์ `StrategyCore` ไว้สำหรับการเชื่อมต่อกับ runtime logic ของ strategy ตามปกติจะไม่ใช้ฟิลด์นี้โดยตรง
แต่จะใช้ facade method บน `self`

```rust
use nautilus_common::actor::DataActor;
use nautilus_model::{
    data::QuoteTick,
    enums::OrderSide,
    identifiers::{InstrumentId, StrategyId},
    types::Quantity,
};
use nautilus_trading::{nautilus_strategy, strategy::{Strategy, StrategyConfig, StrategyCore}};

pub struct MyStrategy {
    core: StrategyCore,
    instrument_id: InstrumentId,
    trade_size: Quantity,
}
```

## Implement constructor [#implement-the-constructor]

`StrategyConfig` รับ `strategy_id` และ `order_id_tag` tag นี้จะถูกต่อท้าย client order ID ทุกตัวที่มาจาก strategy นี้
เพื่อป้องกันการชนกันเมื่อมีหลาย strategy เทรด instrument เดียวกัน

tag ต้องไม่มีเครื่องหมายขีด (hyphen) เพราะ runtime จะอ่าน tag กลับมาจากส่วนสุดท้ายของ strategy ID ที่คั่นด้วยขีด
`StrategyCore::new` จะ panic เมื่อ tag ไม่ถูกต้อง หากต้องการจัดการเป็น error แทนให้ใช้ `StrategyCore::new_checked`

```rust
impl MyStrategy {
    pub fn new(instrument_id: InstrumentId) -> Self {
        let config = StrategyConfig {
            strategy_id: Some(StrategyId::from("MY_STRAT-001")),
            order_id_tag: Some("001".to_string()),
            ..Default::default()
        };
        Self {
            core: StrategyCore::new(config),
            instrument_id,
            trade_size: Quantity::from("1.0"),
        }
    }
}
```

## เชื่อม core และ implement Debug [#wire-up-the-core-and-implement-debug]

macro `nautilus_strategy!` generate การเชื่อมต่อกับ native runtime ที่ใช้ในการ register และ impl ของ trait `Strategy`
โดยค่าเริ่มต้นจะ delegate ไปยังฟิลด์ชื่อ `core` หากใช้ชื่อฟิลด์อื่นให้ส่ง argument ตัวที่สองเข้าไป macro ไม่ได้ทำให้
strategy หรือ `StrategyCore` ของมัน deref ไปยัง runtime internals และยังเพิ่ม `config()` ซึ่งคืนค่า `StrategyConfig`
ที่ส่งให้ `StrategyCore::new`

การ register กับ runtime ใช้ blanket implementation ของ `Actor` และ `Component` ซึ่งต้องการ native wiring และ `Debug`
macro จัดเตรียม native wiring ให้ ส่วน `Debug` ให้ implement เองหรือ derive เอา

```rust
nautilus_strategy!(MyStrategy);

impl std::fmt::Debug for MyStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyStrategy").finish()
    }
}
```

## Implement trait DataActor [#implement-the-dataactor-trait]

การจัดการข้อมูลทำงานเหมือนกับใน actor คือ subscribe ใน `on_start` แล้วตอบสนองใน handler

```rust
impl DataActor for MyStrategy {
    fn on_start(&mut self) -> anyhow::Result<()> {
        self.subscribe_quotes(self.instrument_id, None, None);
        Ok(())
    }

    fn on_quote(&mut self, quote: &QuoteTick) -> anyhow::Result<()> {
        let order = self.order().market(
            self.instrument_id,
            OrderSide::Buy,
            self.trade_size,
            None, None, None, None, None, None, None,
        );
        self.submit_order(order, None, None, None)?;
        Ok(())
    }
}
```

`self.order()` ใช้สร้าง order และ order list โดยมี method ดังนี้:

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
- `generate_client_order_id`
- `generate_order_list_id`

`submit_order` เรียกใช้ได้บน `self` ผ่าน impl ของ trait `Strategy` ที่ macro generate ขึ้น

## การเข้าถึง native runtime [#native-runtime-access]

ใน logic ของ strategy ให้ใช้ facade สาธารณะ:

- `clock()`
- `cache()`
- `order()`
- `portfolio()`
- `strategy_id()`
- method สำหรับจัดการ order บน `Strategy`

code ของ strategy ตามปกติจะไม่ import `DataActorNative` หรือ `StrategyNative` และไม่เรียก native handle เช่น:

- `core()`
- `core_mut()`
- `strategy_core()`
- `strategy_core_mut()`
- `order_factory()`
- `order_factory_rc()`
- `portfolio_rc()`

native handle เหล่านั้นเปิดให้เข้าถึง runtime state แบบ borrow และใช้กันเฉพาะใน engine, runtime, การ register, testkit
หรือ native code ที่ไวต่อ latency อย่างชัดเจน ส่วน [Rust native traits](../concepts/rust.md#native-traits)
ครอบคลุม applicability matrix ของ native trait และตาราง method เหล่านี้:

- [method ของ `DataActorNative`](../concepts/rust.md#dataactornative-methods)
- [method ของ `StrategyNative`](../concepts/rust.md#strategynative-methods)

## Override hook ของ Strategy [#override-strategy-hooks]

หากต้องการ override method ของ trait `Strategy` เช่น handler ของ order event หรือ position event ให้ส่ง method
เหล่านั้นใน block macro จะ generate ส่วนเชื่อมต่อภายในให้อัตโนมัติ ส่วน handler ของ `DataActor` ให้เก็บไว้ใน block
`impl DataActor` แยกต่างหาก

```rust
nautilus_strategy!(MyStrategy, {
    fn on_order_rejected(&mut self, event: OrderRejected) {
        log::warn!("Order rejected: {}", event.reason);
    }
});
```

## Method สำหรับจัดการ order [#order-management-methods]

trait `Strategy` มี facade method ดังนี้:

| Method                | การทำงาน                                         |
| --------------------- | ----------------------------------------------- |
| `submit_order`        | ส่ง order ใหม่ไปยัง venue                          |
| `submit_order_list`   | ส่งรายการ contingent order                       |
| `modify_order`        | แก้ไขราคา, quantity หรือ trigger price              |
| `modify_orders`       | แก้ไขหลาย order ของ instrument เดียวกัน             |
| `cancel_order`        | ยกเลิก order ที่ระบุ                               |
| `cancel_orders`       | ยกเลิกชุด order ที่ผ่านการกรอง                       |
| `cancel_all_orders`   | ยกเลิกทุก order ของ instrument หนึ่ง                |
| `close_position`      | ปิด position ด้วย market order                    |
| `close_all_positions` | ปิดทุก position ที่เปิดอยู่                           |

## ตัวอย่างฉบับเต็ม [#full-examples]

- [`EmaCross`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/trading/src/examples/strategies/ema_cross):
  Dual-EMA crossover ที่ใช้งานร่วมกับ indicator
- [`GridMarketMaker`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/trading/src/examples/strategies/grid_mm):
  Grid market making ที่ตั้งค่าจำนวน level ได้และมีการ requote
