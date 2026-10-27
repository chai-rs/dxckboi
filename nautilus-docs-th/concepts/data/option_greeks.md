---
title: "OptionGreeks"
---

`OptionGreeks` แทนค่า sensitivity ของ option และ implied volatility ที่ venue ให้มาสำหรับ option
instrument หนึ่งตัว เนื่องจากเป็น variant ของ enum `Data` แบบ native จึงบันทึก, replay และ query
ผ่าน catalog ได้

## ฟิลด์ [#fields]

| ฟิลด์               | Rust type           | จำเป็น/ค่าเริ่มต้น  | หมายเหตุ                                      |
| ------------------ | ------------------- | ---------------- | ------------------------------------------ |
| `instrument_id`    | `InstrumentId`      | Required         | option instrument ของค่า greeks นี้          |
| `convention`       | `GreeksConvention`  | Default          | convention ของ numeraire สำหรับค่าเหล่านี้       |
| `greeks`           | `OptionGreekValues` | Required         | Delta, gamma, vega, theta และ rho          |
| `mark_iv`          | `Option<f64>`       | `None`           | Mark implied volatility                    |
| `bid_iv`           | `Option<f64>`       | `None`           | Bid implied volatility                     |
| `ask_iv`           | `Option<f64>`       | `None`           | Ask implied volatility                     |
| `underlying_price` | `Option<f64>`       | `None`           | ราคาของสินทรัพย์อ้างอิงที่ใช้ในการคำนวณ            |
| `open_interest`    | `Option<f64>`       | `None`           | Open interest เมื่อมีการเผยแพร่                |
| `ts_event`         | `UnixNanos`         | Required         | timestamp ของ event หน่วยนาโนวินาที           |
| `ts_init`          | `UnixNanos`         | Required         | timestamp ตอนสร้าง หน่วยนาโนวินาที             |

## พฤติกรรม [#behavior]

- ใน Rust, `OptionGreeks` dereference ไปเป็น `OptionGreekValues` ซึ่งเป็นแกนหลักของมัน
- การ subscribe option chain ใช้ `underlying_price` และค่า delta เพื่อหาช่วง strike แบบ ATM และแบบอิง
  delta

## ตัวอย่าง [#example]

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::{OptionGreekValues, OptionGreeks},
    enums::GreeksConvention,
    identifiers::InstrumentId,
};

let greeks = OptionGreeks {
    instrument_id: InstrumentId::from("BTC-20240628-65000-C.DERIBIT"),
    convention: GreeksConvention::PriceAdjusted,
    greeks: OptionGreekValues {
        delta: 0.51,
        gamma: 0.0002,
        vega: 12.5,
        theta: -3.2,
        rho: 0.1,
    },
    mark_iv: Some(0.55),
    bid_iv: Some(0.54),
    ask_iv: Some(0.56),
    underlying_price: Some(65_000.0),
    open_interest: Some(120.0),
    ts_event: UnixNanos::from(1_000_000_000),
    ts_init: UnixNanos::from(1_000_000_100),
};
```

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Greeks](../greeks.md) อธิบาย greeks ที่ venue ให้มาและที่คำนวณเองในเครื่อง
- [Options](../options.md#optiongreeks-data-type) อธิบายการ subscribe option chain
