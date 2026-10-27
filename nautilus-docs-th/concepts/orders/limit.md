---
title: "Limit"
---

`FIX OrdType <40>=2`

*Limit* order จะวางพักอยู่ใน limit order book ที่ราคาที่กำหนด และจะ execute ที่ราคานั้นหรือดีกว่าเท่านั้น

## กรณีการใช้งาน [#use-cases]

ใช้ *Limit* order เมื่อต้องการควบคุมราคา execution และเมื่อเหมาะสมก็ใช้เพื่อให้สภาพคล่อง (provide liquidity)
การใช้งานทั่วไปได้แก่ market making, การทยอยเข้าหรือออกจาก position ที่ระดับราคาที่เลือกไว้ และการมุ่งรับค่าธรรมเนียมแบบ
maker ด้วย `post_only` order นี้จะไม่ fill ที่ราคาแย่กว่า limit price แต่อาจไม่ได้ fill เลยหรือ fill เพียงบางส่วน

## ตัวอย่าง [#example]

**ตัวอย่าง**: สร้าง *Limit* order บน exchange Binance Futures Crypto เพื่อ SELL สัญญา Perpetual Futures
ETHUSDT-PERP จำนวน 20 สัญญา โดยตั้ง limit price ไว้ที่ 5000 USDT เพื่อทำหน้าที่เป็น market maker

```rust tab="Rust"
use nautilus_model::{
    enums::{OrderSide, TimeInForce},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};

let order = self.order().limit(
    InstrumentId::from("ETHUSDT-PERP.BINANCE"),
    OrderSide::Sell,
    Quantity::from(20),
    Price::from("5000.00"),
    Some(TimeInForce::Gtc), // optional (default GTC)
    None,                   // expire_time
    Some(true),             // post_only (default false)
    Some(false),            // reduce_only (default false)
    None,                   // quote_quantity (default false)
    None,                   // display_qty (default full display)
    None,                   // emulation_trigger
    None,                   // trigger_instrument_id
    None,                   // exec_algorithm_id
    None,                   // exec_algorithm_params
    None,                   // tags
    None,                   // client_order_id
);
```

ดูรายละเอียดเพิ่มเติมได้ที่ [Rust API docs](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Orders](index.md) - แนวคิดของ order, execution instruction และ order factory
- [Emulated orders](emulated.md) - การจำลอง *Limit* order ซึ่งจะถูกปล่อยออกไปเป็น *Market* order เมื่อถึง trigger
- [Execution](../execution/) - order ถูกส่งไปยัง venue อย่างไร และจัดการ fill อย่างไร
