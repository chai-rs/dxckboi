---
title: "Market"
---

`FIX OrdType <40>=1`

*Market* order สั่งให้ venue ซื้อขายตามจำนวนที่กำหนดทันทีที่ราคาดีที่สุดที่มีอยู่
และสามารถกำหนด time in force และคำสั่ง reduce-only ได้ด้วย

## กรณีการใช้งาน [#use-cases]

ใช้ *Market* order เมื่อความรวดเร็วในการ execute สำคัญกว่าราคาที่แน่นอน เช่น การลดความเสี่ยงอย่างเร่งด่วน
หรือการเข้าตลาดที่มีสภาพคล่องสูงและเคลื่อนไหวเร็ว

*Market* order **ไม่มีการป้องกันราคา** อาจเกิดต้นทุนจาก spread และ slippage ได้ และ venue ยังอาจ
reject หรือไม่ fill order ได้หากไม่มีตลาดรองรับ

## ตัวอย่าง [#example]

**ตัวอย่าง**: สร้าง *Market* order บน Forex ECN [IdealPro](https://ibkr.info/node/1708) ของ Interactive Brokers
เพื่อ BUY 100,000 AUD โดยใช้ USD

```rust tab="Rust"
use nautilus_model::{
    enums::{OrderSide, TimeInForce},
    identifiers::InstrumentId,
    types::Quantity,
};
use ustr::Ustr;

let order = self.order().market(
    InstrumentId::from("AUD/USD.IDEALPRO"),
    OrderSide::Buy,
    Quantity::from(100_000),
    Some(TimeInForce::Ioc),          // optional (default GTC)
    Some(false),                     // reduce_only (default false)
    None,                            // quote_quantity (default false)
    None,                            // exec_algorithm_id
    None,                            // exec_algorithm_params
    Some(vec![Ustr::from("ENTRY")]), // tags
    None,                            // client_order_id (auto-generated if None)
);
```

ดูรายละเอียดเพิ่มเติมได้ที่ [Rust API docs](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Orders](index.md) - แนวคิดของ order, execution instruction และ order factory
- [Execution](../execution/) - วิธีที่ order ไปถึง venue และการจัดการ fill
