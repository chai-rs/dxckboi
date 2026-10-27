---
title: "FundingRateUpdate"
---

`FundingRateUpdate` แทน funding rate ของ instrument ประเภท perpetual swap และอาจรวมถึง
ช่วงเวลา funding (funding interval) กับ timestamp ของรอบ funding ถัดไปด้วย หาก venue เผยแพร่ข้อมูลเหล่านี้

## ฟิลด์ [#fields]

| Field             | Rust type           | Required/default | หมายเหตุ                                  |
| ----------------- | ------------------- | ---------------- | ---------------------------------------- |
| `instrument_id`   | `InstrumentId`      | Required         | perpetual instrument ของ rate นี้          |
| `rate`            | `Decimal`           | Required         | funding rate ปัจจุบัน                      |
| `interval`        | `Option<u16>`       | `None`           | ช่วงเวลา funding หน่วยนาที                  |
| `next_funding_ns` | `Option<UnixNanos>` | `None`           | timestamp ของรอบ funding ถัดไป หน่วย nanosecond |
| `ts_event`        | `UnixNanos`         | Required         | timestamp ของ event หน่วย nanosecond        |
| `ts_init`         | `UnixNanos`         | Required         | timestamp ตอน initialize หน่วย nanosecond   |

## พฤติกรรม [#behavior]

- funding rate จะถูกเก็บใน cache แยกตาม instrument เมื่อได้รับเข้ามา
- การเปรียบเทียบความเท่ากันและการ hash ใช้ instrument ID, rate, interval และเวลา funding ถัดไป
- funding rate เป็นข้อมูลอ้างอิง (reference data) ไม่ได้หมายความว่ามีการจ่าย funding เกิดขึ้นแล้ว
- ใช้ `interval` และ `next_funding_ns` เฉพาะเมื่อ venue เผยแพร่ค่าเหล่านี้เท่านั้น

## ตัวอย่าง [#example]

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{data::FundingRateUpdate, identifiers::InstrumentId};
use rust_decimal::Decimal;

let funding = FundingRateUpdate::new(
    InstrumentId::from("BTCUSDT-PERP.BINANCE"),
    Decimal::new(1, 4),
    Some(480),
    Some(UnixNanos::from(1_000_008_000)),
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);
```

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [MarkPriceUpdate](mark_price_update.md) อธิบาย mark price ของอนุพันธ์
- [IndexPriceUpdate](index_price_update.md) อธิบายราคาอ้างอิงจาก index
- [Rust API reference](https://nautilustrader.io/docs/core-latest/) รวมรายละเอียดของ type ฝั่ง Rust
