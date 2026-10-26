# TradeTick

`TradeTick` แทน trade หรือ match event หนึ่งครั้งจาก venue โดยเก็บราคาที่ซื้อขาย
size, ฝั่ง aggressor และ trade identifier ของ venue

## ฟิลด์ {#fields}

| Field            | Rust type       | Required/default | Notes                                    |
| ---------------- | --------------- | ---------------- | ---------------------------------------- |
| `instrument_id`  | `InstrumentId`  | Required         | instrument ของ trade                     |
| `price`          | `Price`         | Required         | ราคาที่ execute                          |
| `size`           | `Quantity`      | Required         | ปริมาณที่ execute                        |
| `aggressor_side` | `AggressorSide` | Required         | `BUY`, `SELL` หรือ `NO_AGGRESSOR`        |
| `trade_id`       | `TradeId`       | Required         | match ID ที่ venue กำหนด                 |
| `ts_event`       | `UnixNanos`     | Required         | timestamp ของ event หน่วยนาโนวินาที      |
| `ts_init`        | `UnixNanos`     | Required         | timestamp ตอน initialize หน่วยนาโนวินาที |

## พฤติกรรม {#behavior}

- `size` ต้องเป็นค่าบวก
- information-driven bar ต้องใช้ข้อมูล `TradeTick` เพราะใช้ `aggressor_side`
- trade bar ใช้ price type แบบ `LAST`
- `trade_id` ควรคงที่สำหรับ event เดียวกันของ venue เมื่อ venue ให้ค่านี้มา
- การ parse และ deserialize ยอมรับค่า `BUYER`/`SELLER` ที่ deprecated แล้ว แต่ string ที่ output ออกมาจะเป็นค่า canonical
  `BUY`/`SELL` เสมอ

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::TradeTick,
    enums::AggressorSide,
    identifiers::{InstrumentId, TradeId},
    types::{Price, Quantity},
};

let trade = TradeTick::new(
    InstrumentId::from("BTCUSDT.BINANCE"),
    Price::from("65000.10"),
    Quantity::from("0.25"),
    AggressorSide::Buy,
    TradeId::from("123456789"),
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);
```

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Bar](bar.md) อธิบายการ aggregate trade เป็น bar
- [Information-driven bars](index.md#information-driven-bars) อธิบายการใช้ฝั่ง aggressor
- [Rust API reference](https://nautilustrader.io/docs/core-latest/) แสดงรายละเอียด API ฝั่ง Rust
