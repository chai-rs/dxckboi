---
title: "Futures Contract"
---

`FuturesContract` แทนสัญญา futures แบบมีวันหมดอายุที่ซื้อขายในตลาด exchange โดยมี
underlying, เวลาเริ่ม active, เวลาหมดอายุ, สกุลเงิน, multiplier และ lot size ที่กำหนดไว้

ตัวอย่างเช่น equity index futures, commodity futures, interest-rate futures และ
currency futures

## Fields [#fields]

| Field             | Rust type          | Required/default | หมายเหตุ                                   |
| ----------------- | ------------------ | ---------------- | ------------------------------------------ |
| `instrument_id`   | `InstrumentId`     | Required         | เก็บเป็น `id` ใน Rust                       |
| `raw_symbol`      | `Symbol`           | Required         | symbol ดั้งเดิมของ venue                    |
| `asset_class`     | `AssetClass`       | Required         | asset class ของ underlying                 |
| `exchange`        | `Option<Ustr>`     | `None`           | MIC ของ exchange หรือรหัส venue หากทราบ      |
| `underlying`      | `Ustr`             | Required         | สินทรัพย์ ดัชนี หรือผลิตภัณฑ์อ้างอิง            |
| `activation_ns`   | `UnixNanos`        | Required         | timestamp ที่สัญญาเริ่ม active              |
| `expiration_ns`   | `UnixNanos`        | Required         | timestamp ที่สัญญาหมดอายุ                   |
| `currency`        | `Currency`         | Required         | สกุลเงินสำหรับ quote และ settlement         |
| `price_precision` | `u8`               | Required         | จำนวนทศนิยมที่อนุญาตสำหรับราคา              |
| `price_increment` | `Price`            | Required         | step ของราคาที่เล็กที่สุดที่ใช้ได้            |
| `size_precision`  | `u8`               | Fixed `0`        | futures ซื้อขายเป็นจำนวนสัญญาเต็ม            |
| `size_increment`  | `Quantity`         | Fixed `1`        | step ของขนาดสัญญาที่เล็กที่สุด                |
| `multiplier`      | `Quantity`         | Required         | multiplier ของสัญญา                        |
| `lot_size`        | `Quantity`         | Required         | ขนาด lot ที่ปัดแล้วหรือขนาด lot ของสัญญา       |
| `margin_init`     | `Option<Decimal>`  | `0`              | อัตรา initial margin                        |
| `margin_maint`    | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                    |
| `max_quantity`    | `Option<Quantity>` | `None`           | จำนวน order สูงสุด                          |
| `min_quantity`    | `Option<Quantity>` | `1`              | จำนวน order ต่ำสุด                          |
| `max_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้         |
| `min_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้         |
| `tick_scheme`     | `Option<Ustr>`     | `None`           | ชื่อ variable tick scheme ที่ลงทะเบียนไว้     |
| `info`            | `Option<Params>`   | `None`           | metadata จาก adapter                       |
| `ts_event`        | `UnixNanos`        | Required         | timestamp ของ event หน่วยนาโนวินาที          |
| `ts_init`         | `UnixNanos`        | Required         | timestamp ตอน initialize หน่วยนาโนวินาที      |

**หมายเหตุ**: builder ใช้ชื่อ `instrument_id` ส่วน Rust struct เก็บค่าเดียวกันไว้ในฟิลด์ `id`

## พฤติกรรม [#behavior]

- `FuturesContract` มี instrument class เป็น `Future`
- ไม่เป็น inverse ต้นทุน settlement และ quote currency ใช้ `currency`
- ราคาเป็นศูนย์หรือติดลบได้
- ซื้อขายเป็นจำนวนสัญญาเต็ม โดยมี size precision เป็น `0` และ size increment เป็น `1`
- ใช้ `CryptoFuture` สำหรับ crypto futures แบบมีวันหมดอายุที่สกุลเงินของ underlying และ settlement
  อาจต่างกัน

## ตัวอย่าง [#example]

```rust tab="Rust"
use jiff::Timestamp;
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::AssetClass,
    identifiers::{InstrumentId, Symbol},
    instruments::FuturesContract,
    types::{Currency, Price, Quantity},
};
use ustr::Ustr;

let activation: Timestamp = "2021-09-10T00:00:00Z".parse().unwrap();
let expiration: Timestamp = "2021-12-17T00:00:00Z".parse().unwrap();

let esz21 = FuturesContract::builder()
    .instrument_id(InstrumentId::from("ESZ21.GLBX"))
    .raw_symbol(Symbol::from("ESZ21"))
    .asset_class(AssetClass::Index)
    .exchange(Ustr::from("XCME"))
    .underlying(Ustr::from("ES"))
    .activation_ns(UnixNanos::from(activation))
    .expiration_ns(UnixNanos::from(expiration))
    .currency(Currency::from("USD"))
    .price_precision(2)
    .price_increment(Price::from("0.25"))
    .multiplier(Quantity::from("1"))
    .lot_size(Quantity::from("1"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapters [#adapters]

**Adapter ที่สร้างหรือใช้ instrument แบบ `FuturesContract` ได้แก่**:

- [Databento](../../integrations/databento.md) สำหรับ reference data และ market data ของ futures
- [Interactive Brokers](../../integrations/interactive_brokers.md) สำหรับสัญญา futures ที่จดทะเบียนในตลาด

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Continuous Futures](../continuous_futures.md) อธิบายชุดข้อมูล futures ที่ปรับตามการ roll
- [Crypto Future](crypto_future.md) อธิบายสัญญา crypto futures แบบมีวันหมดอายุ
