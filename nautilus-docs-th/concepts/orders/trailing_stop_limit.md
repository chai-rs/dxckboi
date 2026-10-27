---
title: "Trailing-Stop-Limit"
---

`FIX OrdType <40>=4` (Stop Limit) + trailing peg

order แบบ *Trailing-Stop-Limit* จะรักษาจุด trigger ของ stop ให้ห่างจากราคาตลาดที่กำหนดเป็นระยะคงที่
ขณะที่ตลาดเคลื่อนไปในทิศที่เป็นประโยชน์ เมื่อถูก trigger จะปล่อย order แบบ *Limit* ออกไป และก่อนหน้านั้น
ราคา limit ก็จะปรับตามตลาดไปด้วย

## กรณีการใช้งาน [#use-cases]

ใช้ order แบบ *Trailing-Stop-Limit* เมื่อต้องการการป้องกันแบบ trailing ที่ปรับตามตลาด พร้อมกำหนดราคา fill
ที่แย่ที่สุดที่ยอมรับได้ เช่นเดียวกับ *Stop-Limit* order แบบ *Limit* ที่ถูกปล่อยออกไปอาจไม่ได้ fill
หากตลาดกลับตัวเร็ว และอาจทำให้ position ยังเปิดค้างอยู่

## ตัวอย่าง [#example]

**ตัวอย่าง**: สร้าง order แบบ *Trailing-Stop-Limit* บน Currenex FX ECN เพื่อ BUY 1,250,000 AUD โดยใช้ USD
เริ่มที่ราคา limit 0.71000 USD และ activate ที่ 0.72000 USD จากนั้น trail ด้วย stop offset 0.00100 USD
จากราคา ask ปัจจุบัน order จะคงสถานะ active ไปจนกว่าจะมีการเปลี่ยนแปลง

```rust tab="Rust"
use nautilus_model::{
    enums::{OrderSide, TimeInForce, TrailingOffsetType, TriggerType},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};
use rust_decimal_macros::dec;
use ustr::Ustr;

let order = self.order().trailing_stop_limit(
    InstrumentId::from("AUD/USD.CURRENEX"),
    OrderSide::Buy,
    Quantity::from(1_250_000),
    Price::from("0.71000"),          // limit price
    dec!(0.00050),                   // limit_offset
    dec!(0.00100),                   // trailing_offset
    Some(TrailingOffsetType::Price), // optional (default PRICE)
    Some(Price::from("0.72000")),    // activation_price
    None,                            // trigger_price (materializes from the offset on the first trail)
    Some(TriggerType::BidAsk),       // optional (default DEFAULT)
    Some(TimeInForce::Gtc),          // optional (default GTC)
    None,                            // expire_time
    Some(false),                     // post_only (default false)
    Some(true),                      // reduce_only (default false)
    None,                            // quote_quantity (default false)
    None,                            // display_qty
    None,                            // emulation_trigger
    None,                            // trigger_instrument_id
    None,                            // exec_algorithm_id
    None,                            // exec_algorithm_params
    Some(vec![Ustr::from("TRAILING_STOP")]), // tags
    None,                            // client_order_id
);
```

:::info
หากไม่ระบุทั้ง `activation_price` และ `trigger_price` order จะ activate ทันทีที่ราคาตลาดปัจจุบัน
และในการอัปเดตครั้งแรก ราคา trigger จะถูกคำนวณจาก `trailing_offset`
:::

ดูรายละเอียดเพิ่มเติมได้ที่ [Rust API reference](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Orders](index.md#trailing-offset-type) - ประเภทของ trigger และ trailing offset
- [Emulated orders](emulated.md) - การ emulate trailing stop บน venue ที่ไม่รองรับโดยตรง
- [Execution](../execution/) - วิธีที่ order ถูกส่งไปยัง venue และการจัดการ fill
