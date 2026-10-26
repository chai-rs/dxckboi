# Trailing-Stop-Market

`FIX OrdType <40>=3` (Stop) + trailing peg

order แบบ *Trailing-Stop-Market* จะรักษาระยะห่างคงที่ระหว่าง stop trigger กับราคาตลาดที่ระบุ
เมื่อตลาดเคลื่อนไปในทิศทางที่เป็นประโยชน์ และจะปล่อย order แบบ *Market* ออกไปเมื่อถูก trigger

## กรณีการใช้งาน {#use-cases}

ใช้ order แบบ *Trailing-Stop-Market* เพื่อปกป้องกำไรขณะที่ยังปล่อยให้ position วิ่งต่อไปตามทิศทางที่เป็นประโยชน์ได้
offset ที่แคบอาจถูก trigger จากความผันผวนปกติ ส่วน offset ที่กว้างอาจทำให้คืนกำไรมากขึ้น
order แบบ *Market* ที่ถูกปล่อยออกไปก็อาจเกิด slippage, ถูกปฏิเสธ หรือไม่ได้ fill เมื่อราคากลับตัวรุนแรง

## ตัวอย่าง {#example}

**ตัวอย่าง**: สร้าง order แบบ *Trailing-Stop-Market* บน Binance Futures exchange
เพื่อ SELL สัญญา ETHUSD-PERP COIN_M margined Perpetual Futures จำนวน 10 สัญญา โดย activate ที่ราคา
5,000 USD แล้ว trail ที่ offset 1% (หน่วยเป็น basis point) จากราคา last traded ปัจจุบัน

```rust tab="Rust"
use nautilus_model::{
    enums::{OrderSide, TimeInForce, TrailingOffsetType, TriggerType},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};
use rust_decimal::Decimal;
use ustr::Ustr;

let order = self.order().trailing_stop_market(
    InstrumentId::from("ETHUSD-PERP.BINANCE"),
    OrderSide::Sell,
    Quantity::from(10),
    Decimal::from(100),                    // trailing_offset
    Some(TrailingOffsetType::BasisPoints), // optional (default PRICE)
    Some(Price::from("5000")),             // activation_price
    None,                                  // trigger_price (materializes from the offset on the first trail)
    Some(TriggerType::LastPrice),          // optional (default DEFAULT)
    Some(TimeInForce::Gtc),                // optional (default GTC)
    None,                                  // expire_time
    Some(true),                            // reduce_only (default false)
    None,                                  // quote_quantity (default false)
    None,                                  // display_qty
    None,                                  // emulation_trigger
    None,                                  // trigger_instrument_id
    None,                                  // exec_algorithm_id
    None,                                  // exec_algorithm_params
    Some(vec![Ustr::from("TRAILING_STOP-1")]), // tags
    None,                                  // client_order_id
);
```

:::info
หากไม่ระบุทั้ง `activation_price` และ `trigger_price` order จะ activate ทันทีที่ราคาตลาด
ปัจจุบัน และใน update แรก trigger price จะถูกคำนวณขึ้นจาก `trailing_offset`
:::

ดูรายละเอียดเพิ่มเติมได้ที่ [Rust API reference](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Orders](index.md#trailing-offset-type) - ประเภทของ trigger และ trailing offset
- [Emulated orders](emulated.md) - การ emulate trailing stop บน venue ที่ไม่รองรับในตัว
- [Execution](../execution/) - order ไปถึง venue อย่างไร และจัดการ fill อย่างไร
