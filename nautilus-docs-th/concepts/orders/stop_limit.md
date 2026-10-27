---
title: "Stop-Limit"
---

`FIX OrdType <40>=4` (Stop Limit)

order แบบ *Stop-Limit* จะปล่อย order แบบ *Limit* ที่ราคาที่กำหนดไว้ เมื่อราคาแตะ trigger price

## กรณีการใช้งาน [#use-cases]

ใช้ order แบบ *Stop-Limit* เมื่อต้องการให้ stop trigger บังคับราคา fill ที่แย่ที่สุดที่ยอมรับได้ด้วย เช่น การออกจาก
position แบบมีการป้องกันราคา หรือการเข้า position แบบ breakout หากตลาด gap ทะลุทั้ง trigger และ limit ไป order
อาจไม่ถูก fill และอาจทำให้ position ไม่มีการป้องกัน

## ตัวอย่าง [#example]

**ตัวอย่าง**: สร้าง order แบบ *Stop-Limit* บน Currenex FX ECN เพื่อ BUY 50,000 GBP กำหนด limit price ที่ 1.3000
USD และ trigger price ที่ 1.30010 USD โดย order จะหมดอายุหลังจากสร้างไปหนึ่งชั่วโมง

```rust tab="Rust"
use nautilus_core::DurationNanos;
use nautilus_model::{
    enums::{OrderSide, TimeInForce, TriggerType},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};

let expire_time = self.clock().timestamp_ns() + DurationNanos::from_mins(60);
let order = self.order().stop_limit(
    InstrumentId::from("GBP/USD.CURRENEX"),
    OrderSide::Buy,
    Quantity::from(50_000),
    Price::from("1.30000"),
    Price::from("1.30010"),
    Some(TriggerType::BidAsk), // optional (default DEFAULT)
    Some(TimeInForce::Gtd),    // optional (default GTC)
    Some(expire_time),         // one hour from now
    Some(true),                // post_only (default false)
    Some(false),               // reduce_only (default false)
    None,                      // quote_quantity (default false)
    None,                      // display_qty
    None,                      // emulation_trigger
    None,                      // trigger_instrument_id
    None,                      // exec_algorithm_id
    None,                      // exec_algorithm_params
    None,                      // tags
    None,                      // client_order_id
);
```

ดูรายละเอียดเพิ่มเติมได้ใน [Rust API docs](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Orders](index.md#trigger-type) - trigger type และ execution instruction อื่น ๆ
- [Emulated orders](emulated.md) - การจำลอง conditional order บน venue ที่ไม่รองรับโดยตรง
- [Execution](../execution/) - วิธีที่ order ไปถึง venue และการจัดการ fill
