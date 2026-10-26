# Futures Spread

`FuturesSpread` แทน futures strategy ที่ exchange กำหนดขึ้นและมีมากกว่าหนึ่ง leg เช่น calendar spread
หรือ inter-commodity spread โดย venue เป็นผู้กำหนด strategy, symbol, tick size และวันหมดอายุ

ตัวอย่างได้แก่ futures calendar spread ที่จดทะเบียนซื้อขาย และตลาด spread ที่ exchange รองรับ

## ฟิลด์ {#fields}

| Field             | Rust type          | Required/default | หมายเหตุ                                    |
| ----------------- | ------------------ | ---------------- | ----------------------------------------- |
| `instrument_id`   | `InstrumentId`     | Required         | ใน Rust เก็บในชื่อ `id`                       |
| `raw_symbol`      | `Symbol`           | Required         | symbol ดั้งเดิมของ venue                     |
| `asset_class`     | `AssetClass`       | Required         | asset class ของ strategy อ้างอิง             |
| `exchange`        | `Option<Ustr>`     | `None`           | MIC ของ exchange หรือรหัส venue (ถ้าทราบ)      |
| `underlying`      | `Ustr`             | Required         | ผลิตภัณฑ์อ้างอิงหรือกลุ่มผลิตภัณฑ์               |
| `strategy_type`   | `Ustr`             | Required         | ประเภท strategy ของ venue เช่น calendar       |
| `activation_ns`   | `UnixNanos`        | Required         | timestamp ที่ strategy เริ่มใช้งาน              |
| `expiration_ns`   | `UnixNanos`        | Required         | timestamp ที่ strategy หมดอายุ                 |
| `currency`        | `Currency`         | Required         | สกุลเงินของ quote และ settlement               |
| `price_precision` | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับราคา             |
| `price_increment` | `Price`            | Required         | ขั้นราคาที่เล็กที่สุดที่ใช้ได้                      |
| `size_precision`  | `u8`               | Fixed `0`        | futures spread ซื้อขายเป็นจำนวนสัญญาเต็มเท่านั้น   |
| `size_increment`  | `Quantity`         | Fixed `1`        | ขั้นขนาดสัญญาที่เล็กที่สุด                        |
| `multiplier`      | `Quantity`         | Required         | ตัวคูณของ strategy                           |
| `lot_size`        | `Quantity`         | Required         | ขนาด lot หรือ lot ของสัญญาที่ปัดแล้ว              |
| `margin_init`     | `Option<Decimal>`  | `0`              | อัตรา initial margin                         |
| `margin_maint`    | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                     |
| `max_quantity`    | `Option<Quantity>` | `None`           | จำนวนสูงสุดต่อ order                          |
| `min_quantity`    | `Option<Quantity>` | `1`              | จำนวนต่ำสุดต่อ order                          |
| `max_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้             |
| `min_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้             |
| `tick_scheme`     | `Option<Ustr>`     | `None`           | ชื่อ tick scheme แบบแปรผันที่ลงทะเบียนไว้          |
| `info`            | `Option<Params>`   | `None`           | metadata จาก adapter                        |
| `ts_event`        | `UnixNanos`        | Required         | timestamp ของ event หน่วย nanosecond         |
| `ts_init`         | `UnixNanos`        | Required         | timestamp ตอน initialize หน่วย nanosecond    |

**หมายเหตุ**: builder ฝั่ง Rust ใช้ชื่อ `instrument_id` แต่ struct เก็บค่าเดียวกันไว้ในฟิลด์ `id`

## พฤติกรรม {#behavior}

- `FuturesSpread` มี instrument class เป็น `FuturesSpread`
- venue เผยแพร่ spread นี้เป็น instrument เดียวที่ซื้อขายได้
- ซื้อขายเป็นจำนวนสัญญาเต็ม โดยมี size precision เป็น `0` และ size increment เป็น `1`
- ราคา spread อาจเป็นศูนย์หรือติดลบได้ และ `RiskEngine` ยอมรับราคาที่ไม่เป็นบวกสำหรับ
  instrument class นี้
- หาก strategy ต้องการรายละเอียดของแต่ละ leg เฉพาะของ venue ให้ใช้ข้อมูล leg จาก metadata ของ adapter

## ตัวอย่าง {#example}

```rust tab="Rust"
use jiff::Timestamp;
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::AssetClass,
    identifiers::{InstrumentId, Symbol},
    instruments::FuturesSpread,
    types::{Currency, Price, Quantity},
};
use ustr::Ustr;

let activation: Timestamp = "2022-06-21T13:30:00Z".parse().unwrap();
let expiration: Timestamp = "2024-06-21T13:30:00Z".parse().unwrap();

let es_spread = FuturesSpread::builder()
    .instrument_id(InstrumentId::from("ESM4-ESU4.GLBX"))
    .raw_symbol(Symbol::from("ESM4-ESU4"))
    .asset_class(AssetClass::Index)
    .exchange(Ustr::from("XCME"))
    .underlying(Ustr::from("ES"))
    .strategy_type(Ustr::from("EQ"))
    .activation_ns(UnixNanos::from(activation))
    .expiration_ns(UnixNanos::from(expiration))
    .currency(Currency::from("USD"))
    .price_precision(2)
    .price_increment(Price::from("0.01"))
    .multiplier(Quantity::from("1"))
    .lot_size(Quantity::from("1"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapter {#adapters}

**Adapter ที่สร้างหรือใช้งาน instrument ประเภท `FuturesSpread` ได้แก่**:

- [Databento](../../integrations/databento.md) สำหรับตลาด futures spread ที่จดทะเบียนซื้อขาย
- [Interactive Brokers](../../integrations/interactive_brokers.md) สำหรับ futures strategy ที่ exchange กำหนด

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Futures Contract](futures_contract.md) อธิบาย futures แบบ leg เดียว
- [Continuous Futures](../continuous_futures.md) อธิบายชุด futures ที่ปรับตามการ roll
