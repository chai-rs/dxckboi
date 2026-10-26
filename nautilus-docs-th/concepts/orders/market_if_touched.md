# Market-If-Touched

`FIX OrdType <40>=J` (Market If Touched)

Order แบบ *Market-If-Touched* จะปล่อย *Market* order ออกไปเมื่อราคาถึง trigger price เทรดเดอร์มักใช้เพื่อเข้า
position ตอนราคาย่อตัว (pullback) หรือเพื่อทำกำไร order แบบ take-profit จะขายสวน position ฝั่ง LONG หรือซื้อสวน
position ฝั่ง SHORT

## กรณีการใช้งาน {#use-cases}

ใช้ order แบบ *Market-If-Touched* เมื่อต้องการให้ความสำคัญกับการได้ execute เมื่อราคาแตะเป้าหมาย order นี้ trigger
ในทิศทางตรงข้ามกับ stop order เช่น ซื้อต่ำกว่าราคาตลาดปัจจุบันหรือขายสูงกว่าราคาตลาดปัจจุบัน ราคาที่แตะไม่ได้
รับประกันว่าจะเป็นราคาที่ได้ fill และ *Market* order ที่ถูกปล่อยออกไปอาจเกิด slippage ถูก reject หรือไม่ได้ fill เลย

## ตัวอย่าง {#example}

**ตัวอย่าง**: สร้าง order แบบ *Market-If-Touched* บน Binance Futures เพื่อ SELL สัญญา ETHUSDT-PERP
Perpetual Futures จำนวน 10 สัญญา ตั้ง trigger price ที่ 10,000 USDT และให้ order active ไปจนกว่าจะยกเลิก

```rust tab="Rust"
use nautilus_model::{
    enums::{OrderSide, TimeInForce, TriggerType},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};
use ustr::Ustr;

let order = self.order().market_if_touched(
    InstrumentId::from("ETHUSDT-PERP.BINANCE"),
    OrderSide::Sell,
    Quantity::from(10),
    Price::from("10000.00"),
    Some(TriggerType::LastPrice),    // optional (default DEFAULT)
    Some(TimeInForce::Gtc),          // optional (default GTC)
    None,                            // expire_time
    Some(false),                     // reduce_only (default false)
    None,                            // quote_quantity (default false)
    None,                            // emulation_trigger
    None,                            // trigger_instrument_id
    None,                            // exec_algorithm_id
    None,                            // exec_algorithm_params
    Some(vec![Ustr::from("ENTRY")]), // tags
    None,                            // client_order_id
);
```

ดูรายละเอียดเพิ่มเติมได้ที่ [Rust API docs](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Orders](index.md#trigger-type) - Trigger type และ execution instruction อื่น ๆ
- [Emulated orders](emulated.md) - การ emulate conditional order บน venue ที่ไม่รองรับแบบ native
- [Execution](../execution/) - order ไปถึง venue อย่างไรและการจัดการ fill
