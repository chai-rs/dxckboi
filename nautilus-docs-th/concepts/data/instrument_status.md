---
title: "InstrumentStatus"
---

`InstrumentStatus` แทนการเปลี่ยนสถานะการซื้อขายของ instrument โดยเก็บ status event จาก venue
เช่น pre-open, trading, halt, pause, close และการเปลี่ยนแปลงข้อจำกัด short-selling

## ฟิลด์ [#fields]

| ฟิลด์                       | Rust type            | Required/default | หมายเหตุ                                      |
| -------------------------- | -------------------- | ---------------- | -------------------------------------------- |
| `instrument_id`            | `InstrumentId`       | Required         | instrument ที่สถานะเปลี่ยน                       |
| `action`                   | `MarketStatusAction` | Required         | status action ของ venue                       |
| `ts_event`                 | `UnixNanos`          | Required         | timestamp ของ event หน่วยนาโนวินาที             |
| `ts_init`                  | `UnixNanos`          | Required         | timestamp ตอน initialize หน่วยนาโนวินาที         |
| `reason`                   | `Option<Ustr>`       | `None`           | สาเหตุของการเปลี่ยนสถานะ (ถ้ามี)                  |
| `trading_event`            | `Option<Ustr>`       | `None`           | label ของ event จาก venue (ถ้ามี)                |
| `is_trading`               | `Option<bool>`       | `None`           | เปิดให้ซื้อขายอยู่หรือไม่ (ถ้าทราบ)                  |
| `is_quoting`               | `Option<bool>`       | `None`           | เปิดให้ quote อยู่หรือไม่ (ถ้าทราบ)                 |
| `is_short_sell_restricted` | `Option<bool>`       | `None`           | สถานะข้อจำกัด short-sell (ถ้าทราบ)                |

## พฤติกรรม [#behavior]

- boolean แบบ optional ช่วยให้ adapter เก็บสถานะตามที่ venue ส่งมาได้โดยไม่ต้องเดา
- `action` ให้สถานะระดับสูงที่ normalize แล้ว แม้รายละเอียดเฉพาะของ venue จะถูกเก็บไว้ใน
  `reason` หรือ `trading_event` ด้วยก็ตาม
- strategy สามารถรับ status update ได้ผ่าน `on_instrument_status(...)`

## ตัวอย่าง [#example]

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::InstrumentStatus,
    enums::MarketStatusAction,
    identifiers::InstrumentId,
};
use ustr::Ustr;

let status = InstrumentStatus::new(
    InstrumentId::from("AAPL.XNAS"),
    MarketStatusAction::Trading,
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
    Some(Ustr::from("Normal trading")),
    Some(Ustr::from("MARKET_OPEN")),
    Some(true),
    Some(true),
    Some(false),
);
```

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [InstrumentClose](instrument_close.md) อธิบาย event ราคาปิดของ instrument
- [Instruments](../instruments/) อธิบายการนิยาม instrument
- [Rust API reference](https://nautilustrader.io/docs/core-latest/) รวมรายละเอียด API ฝั่ง Rust
