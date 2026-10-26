# Market-To-Limit

`FIX OrdType <40>=K` (Market With Left Over as Limit)

order แบบ *Market-To-Limit* จะถูกส่งเป็น order แบบ *Market* หลังจาก fill ครั้งแรก ปริมาณที่ยังไม่ถูก fill
จะค้างอยู่ในตลาดเป็น order แบบ *Limit* ที่ราคาของ fill ครั้งแรกนั้น

## กรณีการใช้งาน {#use-cases}

ใช้ order แบบ *Market-To-Limit* เพื่อรับ liquidity ที่ราคาดีที่สุดที่มีอยู่ โดยไม่กวาดลึกลงไปยังระดับราคาถัดไป
เหมาะกับ order book ที่บาง หรือ order ขนาดใหญ่ที่ต้องการจำกัด market impact เพิ่มเติม ส่วนที่เหลือ
อาจไม่ถูก fill เลยหากตลาดเคลื่อนห่างออกไปจากราคาของ fill ครั้งแรก

## ตัวอย่าง {#example}

**ตัวอย่าง**: สร้าง order แบบ *Market-To-Limit* บน Forex ECN
[IdealPro](https://ibkr.info/node/1708) ของ Interactive Brokers เพื่อ BUY 200,000 USD โดยใช้ JPY

```rust tab="Rust"
use nautilus_model::{
    enums::{OrderSide, TimeInForce},
    identifiers::InstrumentId,
    types::Quantity,
};

let order = self.order().market_to_limit(
    InstrumentId::from("USD/JPY.IDEALPRO"),
    OrderSide::Buy,
    Quantity::from(200_000),
    Some(TimeInForce::Gtc), // optional (default GTC)
    None,                   // expire_time
    Some(false),            // reduce_only (default false)
    None,                   // quote_quantity (default false)
    None,                   // display_qty (default full display)
    None,                   // exec_algorithm_id
    None,                   // exec_algorithm_params
    None,                   // tags
    None,                   // client_order_id
);
```

ดูรายละเอียดเพิ่มเติมได้ที่ [Rust API docs](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Orders](index.md) - แนวคิดของ order, execution instruction และ order factory
- [Execution](../execution/) - order ไปถึง venue อย่างไร และการจัดการ fill
