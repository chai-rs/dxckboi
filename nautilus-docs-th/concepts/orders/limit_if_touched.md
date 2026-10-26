# Limit-If-Touched

`FIX OrdType <40>` ไม่มีค่าเฉพาะ (มักใช้ `4` Stop Limit ที่มี trigger ฝั่งที่ได้เปรียบ)

order แบบ *Limit-If-Touched* จะปล่อย order แบบ *Limit* ที่ราคาที่กำหนด เมื่อราคาแตะ trigger price

## กรณีการใช้งาน {#use-cases}

ใช้ order แบบ *Limit-If-Touched* เพื่อเปิดใช้งาน order ที่มีการป้องกันราคา (price-protected) หลังจากราคาแตะ
trigger แล้วเท่านั้น เช่น วาง order แบบ *Limit* สำหรับ take-profit เมื่อราคาเข้าใกล้เป้าหมาย แทนที่จะวาง order
ทิ้งไว้ตั้งแต่เนิ่นๆ เช่นเดียวกับ *Stop-Limit* order อาจไม่ได้ fill ถ้าตลาดวิ่งผ่านราคา limit ไปหลังจาก trigger

## ตัวอย่าง {#example}

**ตัวอย่าง**: สร้าง order แบบ *Limit-If-Touched* เพื่อ BUY สัญญา perpetual futures BTCUSDT-PERP จำนวน 5 สัญญา
บน Binance Futures กำหนดราคา limit ที่ 30,100 USDT และ trigger price ที่ 30,150 USDT โดย order จะหมดอายุ
หนึ่งชั่วโมงหลังจากสร้าง

```rust tab="Rust"
use nautilus_core::DurationNanos;
use nautilus_model::{
    enums::{OrderSide, TimeInForce, TriggerType},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};
use ustr::Ustr;

let expire_time = self.clock().timestamp_ns() + DurationNanos::from_mins(60);
let order = self.order().limit_if_touched(
    InstrumentId::from("BTCUSDT-PERP.BINANCE"),
    OrderSide::Buy,
    Quantity::from(5),
    Price::from("30100"),
    Price::from("30150"),
    Some(TriggerType::LastPrice), // optional (default DEFAULT)
    Some(TimeInForce::Gtd),       // optional (default GTC)
    Some(expire_time),            // one hour from now
    Some(true),                   // post_only (default false)
    Some(false),                  // reduce_only (default false)
    None,                         // quote_quantity (default false)
    None,                         // display_qty
    None,                         // emulation_trigger
    None,                         // trigger_instrument_id
    None,                         // exec_algorithm_id
    None,                         // exec_algorithm_params
    Some(vec![Ustr::from("TAKE_PROFIT")]), // tags
    None,                         // client_order_id
);
```

ดูรายละเอียดเพิ่มเติมได้ใน [Rust API Reference](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Orders](index.md#trigger-type) - ประเภทของ trigger และคำสั่งการ execute อื่นๆ
- [Emulated orders](emulated.md) - การจำลอง conditional order บน venue ที่ไม่รองรับแบบ native
- [Execution](../execution/) - order ไปถึง venue อย่างไรและจัดการ fill อย่างไร
