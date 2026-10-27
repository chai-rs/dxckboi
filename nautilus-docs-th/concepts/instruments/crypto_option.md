---
title: "Crypto Option"
---

`CryptoOption` แทน option แบบ put หรือ call บน underlying ที่เป็น crypto โดยกำหนดชนิดของ option,
strike price, เวลา activation, เวลาหมดอายุ, quote currency, settlement currency และขนาดของสัญญา

ตัวอย่างเช่น option ของ BTC และ ETH บน venue ประเภท crypto derivatives

## Field [#fields]

| Field                 | Rust type          | Required/default | หมายเหตุ                                       |
| --------------------- | ------------------ | ---------------- | --------------------------------------------- |
| `instrument_id`       | `InstrumentId`     | Required         | ใน Rust จัดเก็บเป็น `id`                        |
| `raw_symbol`          | `Symbol`           | Required         | Symbol ดั้งเดิมของ venue                         |
| `underlying`          | `Currency`         | Required         | สินทรัพย์ crypto ที่ option อ้างอิง               |
| `quote_currency`      | `Currency`         | Required         | สกุลเงินที่ใช้ quote ค่า premium                  |
| `settlement_currency` | `Currency`         | Required         | สกุลเงินที่ใช้ settle PnL และค่าธรรมเนียม          |
| `is_inverse`          | `bool`             | Required         | เป็น true เมื่อการคำนวณขนาด/ต้นทุนเป็นแบบ inverse |
| `option_kind`         | `OptionKind`       | Required         | Put หรือ call                                  |
| `strike_price`        | `Price`            | Required         | Strike price ของ option                        |
| `activation_ns`       | `UnixNanos`        | Required         | Timestamp ที่สัญญาเริ่มมีผล                       |
| `expiration_ns`       | `UnixNanos`        | Required         | Timestamp ที่สัญญาหมดอายุ                        |
| `price_precision`     | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับราคา              |
| `size_precision`      | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับขนาด order         |
| `price_increment`     | `Price`            | Required         | ช่วงราคาที่เล็กที่สุดที่ใช้ได้                        |
| `size_increment`      | `Quantity`         | Required         | ช่วงขนาดที่เล็กที่สุดที่ใช้ได้                        |
| `multiplier`          | `Quantity`         | `1`              | Contract multiplier                           |
| `lot_size`            | `Quantity`         | `1`              | ขนาด lot หรือ board ที่ปัดแล้ว                    |
| `max_quantity`        | `Option<Quantity>` | `None`           | ปริมาณ order สูงสุด                             |
| `min_quantity`        | `Option<Quantity>` | `None`           | ปริมาณ order ต่ำสุด                             |
| `max_notional`        | `Option<Money>`    | `None`           | มูลค่า notional ของ order สูงสุด                  |
| `min_notional`        | `Option<Money>`    | `None`           | มูลค่า notional ของ order ต่ำสุด                  |
| `max_price`           | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้                |
| `min_price`           | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้                |
| `margin_init`         | `Option<Decimal>`  | `0`              | อัตรา initial margin                           |
| `margin_maint`        | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                       |
| `tick_scheme`         | `Option<Ustr>`     | `None`           | ชื่อ variable tick scheme ที่ลงทะเบียนไว้            |
| `info`                | `Option<Params>`   | `None`           | Metadata ของ adapter                           |
| `ts_event`            | `UnixNanos`        | Required         | Event timestamp หน่วย nanosecond               |
| `ts_init`             | `UnixNanos`        | Required         | Initialization timestamp หน่วย nanosecond      |

## พฤติกรรม [#behavior]

- `CryptoOption` มี asset class เป็น `Cryptocurrency` และ instrument class เป็น `Option`
- ชนิดของ option และ strike price เป็นตัวกำหนดรูปแบบ payoff
- สัญญาอาจเป็นแบบ linear, inverse หรือ quanto ขึ้นอยู่กับชุดของสกุลเงิน
- สำหรับ listed option ที่ไม่ใช่ crypto ให้ใช้ `OptionContract`

## ตัวอย่าง [#example]

```rust tab="Rust"
use jiff::Timestamp;
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::OptionKind,
    identifiers::{InstrumentId, Symbol},
    instruments::CryptoOption,
    types::{Currency, Money, Price, Quantity},
};
use rust_decimal_macros::dec;

let activation: Timestamp = "2022-12-22T00:00:00Z".parse().unwrap();
let expiration: Timestamp = "2023-01-13T08:00:00Z".parse().unwrap();

let btc_option = CryptoOption::builder()
    .instrument_id(InstrumentId::from("BTC-13JAN23-16000-P.DERIBIT"))
    .raw_symbol(Symbol::from("BTC-13JAN23-16000-P"))
    .underlying(Currency::from("BTC"))
    .quote_currency(Currency::from("USD"))
    .settlement_currency(Currency::from("BTC"))
    .is_inverse(false)
    .option_kind(OptionKind::Put)
    .strike_price(Price::from("16000.00"))
    .activation_ns(UnixNanos::from(activation))
    .expiration_ns(UnixNanos::from(expiration))
    .price_precision(2)
    .size_precision(1)
    .price_increment(Price::from("0.01"))
    .size_increment(Quantity::from("0.1"))
    .multiplier(Quantity::from("1"))
    .lot_size(Quantity::from("1"))
    .max_quantity(Quantity::from("9000"))
    .min_quantity(Quantity::from("0.1"))
    .min_notional(Money::from("10.00 USD"))
    .margin_init(dec!(0))
    .margin_maint(dec!(0))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapter [#adapters]

**Adapter ที่สร้างหรือใช้งาน instrument ประเภท `CryptoOption` ได้แก่**:

- [Bybit](../../integrations/bybit.md) สำหรับ crypto option
- [Deribit](../../integrations/deribit.md) สำหรับ crypto option
- [OKX](../../integrations/okx.md) สำหรับ crypto option
- [Tardis](../../integrations/tardis.md) สำหรับ metadata ของ crypto option

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Options](../options.md) อธิบายข้อมูล option, Greeks และการ subscribe option chain
- [Crypto Option Spread](crypto_option_spread.md) อธิบาย crypto option spread ที่ exchange กำหนดขึ้น
