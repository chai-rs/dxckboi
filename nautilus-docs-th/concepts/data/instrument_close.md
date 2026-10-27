---
title: "InstrumentClose"
---

`InstrumentClose` แทน event ราคาปิดของ instrument ที่ venue หนึ่ง ครอบคลุมทั้งการปิดเมื่อจบ session
และ event ราคาปิดเมื่อสัญญาหมดอายุ

## Fields [#fields]

| Field           | Rust type             | Required/default | หมายเหตุ                                    |
| --------------- | --------------------- | ---------------- | ------------------------------------------- |
| `instrument_id` | `InstrumentId`        | Required         | instrument ที่ถูกปิด                          |
| `close_price`   | `Price`               | Required         | ราคาปิดหรือราคา settlement                   |
| `close_type`    | `InstrumentCloseType` | Required         | `END_OF_SESSION` หรือ `CONTRACT_EXPIRED`     |
| `ts_event`      | `UnixNanos`           | Required         | timestamp ของ event หน่วยนาโนวินาที             |
| `ts_init`       | `UnixNanos`           | Required         | timestamp ตอน initialize หน่วยนาโนวินาที        |

## พฤติกรรม [#behavior]

- การปิดเมื่อจบ session ให้ราคาปิดระดับ session
- การปิดเมื่อสัญญาหมดอายุใช้ระบุ event การหมดอายุของสัญญาที่มีวันครบกำหนด
- ราคาปิดเป็นข้อมูลอ้างอิง ไม่ได้หมายความว่ามี trade เกิดขึ้นที่ราคานั้นจริง

## ตัวอย่าง [#example]

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::InstrumentClose,
    enums::InstrumentCloseType,
    identifiers::InstrumentId,
    types::Price,
};

let close = InstrumentClose::new(
    InstrumentId::from("ESM4.XCME"),
    Price::from("5325.25"),
    InstrumentCloseType::EndOfSession,
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);
```

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [InstrumentStatus](instrument_status.md) อธิบาย event สถานะของ instrument
- [Instruments](../instruments/) อธิบายการนิยาม instrument
- [Rust API reference](https://nautilustrader.io/docs/core-latest/) รวมรายละเอียด API ฝั่ง Rust
