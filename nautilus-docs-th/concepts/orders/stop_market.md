# Stop-Market

`FIX OrdType <40>=3` (Stop)

*Stop-Market* order จะปล่อย *Market* order ออกไปเมื่อราคาถึง trigger price มักใช้เป็น stop-loss
คือ SELL order เพื่อป้องกัน LONG position หรือ BUY order เพื่อป้องกัน SHORT position

## กรณีการใช้งาน {#use-cases}

ใช้ *Stop-Market* order เมื่อต้องการให้ความสำคัญกับการ execute หลังจากราคาทะลุระดับที่กำหนด เช่น
stop-loss เพื่อป้องกันความเสี่ยง หรือการเข้า position แบบ breakout

trigger price **ไม่ใช่ราคา fill ที่รับประกัน** ตลาดที่เคลื่อนไหวเร็วหรือเกิด gap อาจทำให้เกิด slippage
มาก order ที่ถูกปล่อยออกไปยังอาจถูก reject หรือไม่ fill ได้หากไม่มีตลาดรองรับ ส่วน *Stop-Limit*
ให้การป้องกันราคาแทน แต่อาจไม่ได้ fill

## ตัวอย่าง {#example}

**ตัวอย่าง**: สร้าง *Stop-Market* order บน Binance Spot/Margin exchange เพื่อ SELL 1 BTC โดยตั้ง trigger
price ไว้ที่ 100,000 USDT และให้ order active ไปจนกว่าจะมีการเปลี่ยนแปลง

```rust tab="Rust"
use nautilus_model::{
    enums::{OrderSide, TimeInForce, TriggerType},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};

let order = self.order().stop_market(
    InstrumentId::from("BTCUSDT.BINANCE"),
    OrderSide::Sell,
    Quantity::from(1),
    Price::from("100000"),
    Some(TriggerType::LastPrice), // optional (default DEFAULT)
    Some(TimeInForce::Gtc),       // optional (default GTC)
    None,                         // expire_time
    Some(false),                  // reduce_only (default false)
    None,                         // quote_quantity (default false)
    None,                         // display_qty
    None,                         // emulation_trigger
    None,                         // trigger_instrument_id
    None,                         // exec_algorithm_id
    None,                         // exec_algorithm_params
    None,                         // tags
    None,                         // client_order_id
);
```

ดูรายละเอียดเพิ่มเติมได้ที่ [Rust API docs](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Orders](index.md#trigger-type) - trigger type และ execution instruction อื่น ๆ
- [Emulated orders](emulated.md) - การจำลอง conditional order บน venue ที่ไม่รองรับโดยตรง
- [Execution](../execution/) - วิธีที่ order ไปถึง venue และการจัดการ fill
