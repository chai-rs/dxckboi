---
title: "Crypto Futures Spread"
---

`CryptoFuturesSpread` แทน spread strategy ที่ exchange กำหนดขึ้นบน crypto futures
venue จะเผยแพร่ strategy นี้เป็น instrument เดียวที่มี symbol, ประเภท strategy, precision,
increment และวันหมดอายุเป็นของตัวเอง

ตัวอย่างเช่น crypto futures calendar spread ที่จดทะเบียนซื้อขายใน exchange

## Field [#fields]

| Field                 | Rust type          | จำเป็น/ค่าเริ่มต้น | หมายเหตุ                                       |
| --------------------- | ------------------ | ------------------ | ---------------------------------------------- |
| `instrument_id`       | `InstrumentId`     | จำเป็น             | ใน Rust เก็บเป็น `id`                          |
| `raw_symbol`          | `Symbol`           | จำเป็น             | symbol ดั้งเดิมของ venue                       |
| `underlying`          | `Currency`         | จำเป็น             | crypto asset ที่ strategy อ้างอิง              |
| `quote_currency`      | `Currency`         | จำเป็น             | สกุลเงินที่ใช้ quote ราคา                      |
| `settlement_currency` | `Currency`         | จำเป็น             | สกุลเงินที่ใช้ settle PnL และค่าธรรมเนียม       |
| `is_inverse`          | `bool`             | จำเป็น             | เป็น true เมื่อการคิดขนาด/ต้นทุนเป็นแบบ inverse |
| `strategy_type`       | `Ustr`             | จำเป็น             | ประเภท strategy ของ venue เช่น calendar        |
| `activation_ns`       | `UnixNanos`        | จำเป็น             | timestamp ที่ strategy เริ่มใช้งาน             |
| `expiration_ns`       | `UnixNanos`        | จำเป็น             | timestamp ที่ strategy หมดอายุ                 |
| `price_precision`     | `u8`               | จำเป็น             | จำนวนทศนิยมที่อนุญาตสำหรับราคา                 |
| `size_precision`      | `u8`               | จำเป็น             | จำนวนทศนิยมที่อนุญาตสำหรับขนาด order          |
| `price_increment`     | `Price`            | จำเป็น             | ขั้นราคาที่เล็กที่สุดที่ใช้ได้                  |
| `size_increment`      | `Quantity`         | จำเป็น             | ขั้นขนาดที่เล็กที่สุดที่ใช้ได้                  |
| `multiplier`          | `Quantity`         | `1`                | ตัวคูณของ strategy                             |
| `lot_size`            | `Quantity`         | `1`                | ขนาด lot หรือ board แบบปัดเศษ                  |
| `max_quantity`        | `Option<Quantity>` | `None`             | quantity สูงสุดของ order                       |
| `min_quantity`        | `Option<Quantity>` | `None`             | quantity ต่ำสุดของ order                       |
| `max_notional`        | `Option<Money>`    | `None`             | มูลค่า notional สูงสุดของ order                |
| `min_notional`        | `Option<Money>`    | `None`             | มูลค่า notional ต่ำสุดของ order                |
| `max_price`           | `Option<Price>`    | `None`             | ราคา quote หรือ order สูงสุดที่ใช้ได้           |
| `min_price`           | `Option<Price>`    | `None`             | ราคา quote หรือ order ต่ำสุดที่ใช้ได้           |
| `margin_init`         | `Option<Decimal>`  | `0`                | อัตรา initial margin                           |
| `margin_maint`        | `Option<Decimal>`  | `0`                | อัตรา maintenance margin                       |
| `tick_scheme`         | `Option<Ustr>`     | `None`             | ชื่อ variable tick scheme ที่ลงทะเบียนไว้        |
| `info`                | `Option<Params>`   | `None`             | metadata ของ adapter                           |
| `ts_event`            | `UnixNanos`        | จำเป็น             | timestamp ของ event หน่วยนาโนวินาที            |
| `ts_init`             | `UnixNanos`        | จำเป็น             | timestamp ตอน initialize หน่วยนาโนวินาที       |

## พฤติกรรม [#behavior]

- `CryptoFuturesSpread` มี asset class เป็น `Cryptocurrency` และ instrument class เป็น
  `FuturesSpread`
- venue เผยแพร่ spread เป็น instrument เดียวที่ซื้อขายได้
- strategy อาจเป็นแบบ linear, inverse หรือ quanto ขึ้นอยู่กับชุดสกุลเงิน
- ราคา spread อาจเป็นศูนย์หรือติดลบได้ และ `RiskEngine` ยอมรับราคาที่ไม่เป็นบวกสำหรับ
  instrument class นี้ ยกเว้น spread แบบ inverse
- หาก adapter มีรายละเอียด leg เฉพาะของ venue ให้ ให้เก็บไว้ใน `info`

## ตัวอย่าง [#example]

```rust tab="Rust"
use jiff::Timestamp;
use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Symbol},
    instruments::CryptoFuturesSpread,
    types::{Currency, Price, Quantity},
};
use ustr::Ustr;

let activation: Timestamp = "2026-05-12T00:00:00Z".parse().unwrap();
let expiration: Timestamp = "2026-05-19T08:00:00Z".parse().unwrap();

let btc_spread = CryptoFuturesSpread::builder()
    .instrument_id(InstrumentId::from("BTC-FS-19MAY26_PERP.DERIBIT"))
    .raw_symbol(Symbol::from("BTC-FS-19MAY26_PERP"))
    .underlying(Currency::from("BTC"))
    .quote_currency(Currency::from("USD"))
    .settlement_currency(Currency::from("BTC"))
    .is_inverse(false)
    .strategy_type(Ustr::from("FS"))
    .activation_ns(UnixNanos::from(activation))
    .expiration_ns(UnixNanos::from(expiration))
    .price_precision(1)
    .size_precision(0)
    .price_increment(Price::from("0.5"))
    .size_increment(Quantity::from("1"))
    .multiplier(Quantity::from("10"))
    .lot_size(Quantity::from("1"))
    .min_quantity(Quantity::from("1"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapter [#adapters]

**Adapter ที่สร้างหรือใช้ instrument `CryptoFuturesSpread` ได้แก่**:

- [Deribit](../../integrations/deribit.md) สำหรับ crypto futures combo
- [OKX](../../integrations/okx.md) สำหรับตลาด crypto futures spread

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Crypto Future](crypto_future.md) อธิบาย crypto futures แบบมีวันหมดอายุที่มี leg เดียว
- [Futures Spread](futures_spread.md) อธิบาย futures spread ที่ไม่ใช่ crypto
