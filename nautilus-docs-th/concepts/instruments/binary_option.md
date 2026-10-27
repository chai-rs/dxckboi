---
title: "Binary Option"
---

`BinaryOption` แทน instrument ที่มีผลลัพธ์แบบสองทาง (binary outcome) ซึ่ง settle เป็น payoff คงที่
ตามว่าเงื่อนไขหนึ่งเป็นจริงหรือไม่ ใช้จำลอง prediction market, binary option
หรือสัญญาแบบ yes/no เฉพาะของแต่ละ venue ได้

ตัวอย่างเช่น ผลลัพธ์ของ prediction market และสัญญา binary event

## Fields [#fields]

| Field             | Rust type          | Required/default | หมายเหตุ                                    |
| ----------------- | ------------------ | ---------------- | ------------------------------------------- |
| `instrument_id`   | `InstrumentId`     | Required         | เก็บเป็น `id` ใน Rust                        |
| `raw_symbol`      | `Symbol`           | Required         | symbol ดั้งเดิมของ venue                     |
| `asset_class`     | `AssetClass`       | Required         | asset class ของตลาดผลลัพธ์                   |
| `currency`        | `Currency`         | Required         | สกุลเงินสำหรับ quote และ settlement          |
| `activation_ns`   | `UnixNanos`        | Required         | timestamp ที่สัญญาเริ่ม active               |
| `expiration_ns`   | `UnixNanos`        | Required         | timestamp ที่สัญญาหมดอายุ                    |
| `price_precision` | `u8`               | Required         | จำนวนทศนิยมที่อนุญาตสำหรับราคา               |
| `size_precision`  | `u8`               | Required         | จำนวนทศนิยมที่อนุญาตสำหรับขนาด order          |
| `price_increment` | `Price`            | Required         | step ของราคาที่เล็กที่สุดที่ใช้ได้             |
| `size_increment`  | `Quantity`         | Required         | step ของขนาดที่เล็กที่สุดที่ใช้ได้             |
| `event_id`        | `Option<Ustr>`     | `None`           | identifier ของ event แม่ภายในขอบเขตของ venue  |
| `outcome`         | `Option<Ustr>`     | `None`           | label ของผลลัพธ์ หาก venue ให้มา             |
| `description`     | `Option<Ustr>`     | `None`           | คำอธิบายตลาดที่มนุษย์อ่านได้                  |
| `max_quantity`    | `Option<Quantity>` | `None`           | จำนวน order สูงสุด                           |
| `min_quantity`    | `Option<Quantity>` | `None`           | จำนวน order ต่ำสุด                           |
| `max_notional`    | `Option<Money>`    | `None`           | มูลค่า notional ของ order สูงสุด              |
| `min_notional`    | `Option<Money>`    | `None`           | มูลค่า notional ของ order ต่ำสุด              |
| `max_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้          |
| `min_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้          |
| `margin_init`     | `Option<Decimal>`  | `0`              | อัตรา initial margin                         |
| `margin_maint`    | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                     |
| `tick_scheme`     | `Option<Ustr>`     | `None`           | ชื่อ variable tick scheme ที่ลงทะเบียนไว้      |
| `info`            | `Option<Params>`   | `None`           | metadata จาก adapter                        |
| `ts_event`        | `UnixNanos`        | Required         | timestamp ของ event หน่วยนาโนวินาที           |
| `ts_init`         | `UnixNanos`        | Required         | timestamp ตอน initialize หน่วยนาโนวินาที       |

**หมายเหตุ**: builder ใช้ชื่อ `instrument_id` ส่วน Rust struct เก็บค่าเดียวกันไว้ในฟิลด์ `id`

## พฤติกรรม [#behavior]

- `BinaryOption` มี instrument class เป็น `BinaryOption`
- ไม่เป็น inverse และใช้ multiplier กับ lot size เท่ากับหนึ่งเสมอ
- หลาย venue quote ผลลัพธ์แบบ binary ในช่วงศูนย์ถึงหนึ่ง แต่ venue เป็นผู้กำหนด
  ช่วงราคาที่อนุญาตและ tick size
- `outcome` และ `description` ให้บริบทที่มนุษย์อ่านได้สำหรับสัญญา
- `event_id` ระบุ event ที่มีตลาดของ instrument นี้อยู่ โดยมีขอบเขตภายใน venue นั้น
- instrument ที่มี `event_id` เดียวกันอยู่ใน event เดียวกัน แต่ไม่ได้หมายความว่าใช้ collateral ร่วมกัน
  มีผลลัพธ์ที่ mutually exclusive หรือมีกฎการ settle เหมือนกัน

## ตัวอย่าง [#example]

```rust tab="Rust"
use jiff::Timestamp;
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::AssetClass,
    identifiers::{InstrumentId, Symbol, Venue},
    instruments::BinaryOption,
    types::{Currency, Price, Quantity},
};
use ustr::Ustr;

let raw_symbol = Symbol::from(
    "0x12a0cb60174abc437bf1178367c72d11f069e1a3add20b148fb0ab4279b772b2-92544998123698303655208967887569360731013655782348975589292031774495159624905",
);
let expiration: Timestamp = "2024-01-01T00:00:00Z".parse().unwrap();

let yes_outcome = BinaryOption::builder()
    .instrument_id(InstrumentId::new(raw_symbol, Venue::from("POLYMARKET")))
    .raw_symbol(raw_symbol)
    .asset_class(AssetClass::Alternative)
    .currency(Currency::from("USDC"))
    .activation_ns(UnixNanos::default())
    .expiration_ns(UnixNanos::from(expiration))
    .price_precision(3)
    .size_precision(2)
    .price_increment(Price::from("0.001"))
    .size_increment(Quantity::from("0.01"))
    .outcome(Ustr::from("Yes"))
    .description(Ustr::from("Will the outcome of this market be 'Yes'?"))
    .min_quantity(Quantity::from("5"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapters [#adapters]

**Adapter ที่สร้างหรือใช้ instrument แบบ `BinaryOption` ได้แก่**:

- [Hyperliquid](../../integrations/hyperliquid.md) สำหรับตลาดแบบ binary และแบบ prediction
- [OKX](../../integrations/okx.md) สำหรับผลิตภัณฑ์ผลลัพธ์แบบ binary ที่ venue กำหนด
- [Polymarket](../../integrations/polymarket.md) สำหรับผลลัพธ์ของ prediction market

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Order Book](../order_book.md) อธิบายพฤติกรรมของ order book ในตลาด binary
- [Data](../data/) อธิบาย market data ที่อ้างอิงถึง instrument
