---
title: "Option Contract"
---

`OptionContract` แทน option แบบ put หรือ call ที่จดทะเบียนซื้อขาย โดยมีสินทรัพย์อ้างอิงที่ไม่ใช่ crypto
กำหนดชนิดของ option, strike price, เวลาเริ่มใช้งาน, เวลาหมดอายุ, สกุลเงิน, multiplier และ lot size

ตัวอย่างเช่น equity option, index option และ futures option

## ฟิลด์ [#fields]

| ฟิลด์              | Rust type          | จำเป็น/ค่าเริ่มต้น  | หมายเหตุ                                    |
| ----------------- | ------------------ | ---------------- | ---------------------------------------- |
| `instrument_id`   | `InstrumentId`     | Required         | ใน Rust เก็บไว้ในชื่อ `id`                  |
| `raw_symbol`      | `Symbol`           | Required         | symbol ดั้งเดิมของ venue                    |
| `asset_class`     | `AssetClass`       | Required         | asset class ของสินทรัพย์อ้างอิง               |
| `exchange`        | `Option<Ustr>`     | `None`           | MIC ของ exchange หรือรหัส venue หากทราบ     |
| `underlying`      | `Ustr`             | Required         | สินทรัพย์อ้างอิง, future หรือ index           |
| `option_kind`     | `OptionKind`       | Required         | Put หรือ call                             |
| `strike_price`    | `Price`            | Required         | strike price ของ option                   |
| `activation_ns`   | `UnixNanos`        | Required         | timestamp ที่สัญญาเริ่มใช้งาน                  |
| `expiration_ns`   | `UnixNanos`        | Required         | timestamp ที่สัญญาหมดอายุ                    |
| `currency`        | `Currency`         | Required         | สกุลเงินของ premium quote และการชำระ         |
| `price_precision` | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับราคา           |
| `price_increment` | `Price`            | Required         | ขั้นราคาเล็กที่สุดที่ใช้ได้                      |
| `size_precision`  | `u8`               | Fixed `0`        | option ซื้อขายเป็นจำนวนสัญญาเต็ม              |
| `size_increment`  | `Quantity`         | Fixed `1`        | ขั้นขนาดสัญญาขั้นต่ำ                          |
| `multiplier`      | `Quantity`         | Required         | multiplier ของสัญญา                       |
| `lot_size`        | `Quantity`         | Required         | ขนาด lot แบบปัดหรือขนาด lot ของสัญญา          |
| `margin_init`     | `Option<Decimal>`  | `0`              | อัตรา initial margin                       |
| `margin_maint`    | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                   |
| `max_quantity`    | `Option<Quantity>` | `None`           | ปริมาณ order สูงสุด                          |
| `min_quantity`    | `Option<Quantity>` | `1`              | ปริมาณ order ต่ำสุด                          |
| `max_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้            |
| `min_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้            |
| `tick_scheme`     | `Option<Ustr>`     | `None`           | ชื่อ tick scheme แบบแปรผันที่ลงทะเบียนไว้        |
| `info`            | `Option<Params>`   | `None`           | metadata ของ adapter                       |
| `ts_event`        | `UnixNanos`        | Required         | timestamp ของ event หน่วยนาโนวินาที         |
| `ts_init`         | `UnixNanos`        | Required         | timestamp ตอนสร้าง หน่วยนาโนวินาที           |

## พฤติกรรม [#behavior]

- `OptionContract` มี instrument class เป็น `Option`
- ซื้อขายเป็นจำนวนสัญญาเต็ม โดยมี size precision เป็น `0` และ size increment เป็น `1`
- ชนิดของ option และ strike price กำหนดรูปแบบของ payoff
- ใช้ `CryptoOption` สำหรับ option ที่สินทรัพย์อ้างอิงและการชำระเป็นสกุลเงิน crypto

## ตัวอย่าง [#example]

```rust tab="Rust"
use jiff::Timestamp;
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::{AssetClass, OptionKind},
    identifiers::{InstrumentId, Symbol},
    instruments::OptionContract,
    types::{Currency, Price, Quantity},
};
use ustr::Ustr;

let activation: Timestamp = "2021-09-17T00:00:00Z".parse().unwrap();
let expiration: Timestamp = "2021-12-17T00:00:00Z".parse().unwrap();

let aapl_call = OptionContract::builder()
    .instrument_id(InstrumentId::from("AAPL211217C00150000.OPRA"))
    .raw_symbol(Symbol::from("AAPL211217C00150000"))
    .asset_class(AssetClass::Equity)
    .exchange(Ustr::from("GMNI"))
    .underlying(Ustr::from("AAPL"))
    .option_kind(OptionKind::Call)
    .strike_price(Price::from("150.00"))
    .currency(Currency::from("USD"))
    .activation_ns(UnixNanos::from(activation))
    .expiration_ns(UnixNanos::from(expiration))
    .price_precision(2)
    .price_increment(Price::from("0.01"))
    .multiplier(Quantity::from("100"))
    .lot_size(Quantity::from("1"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapter [#adapters]

**adapter ที่สร้างหรือใช้ instrument แบบ `OptionContract` ได้แก่**:

- [Databento](../../integrations/databento.md) สำหรับข้อมูล option ที่จดทะเบียนซื้อขาย
- [Interactive Brokers](../../integrations/interactive_brokers.md) สำหรับสัญญา option ที่จดทะเบียนซื้อขาย

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Options](../options.md) อธิบายข้อมูล option, greeks และการ subscribe option chain
- [Crypto Option](crypto_option.md) อธิบายสัญญา crypto option
