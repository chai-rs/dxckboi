# Option Spread

`OptionSpread` แทน options strategy ที่ตลาดกำหนดขึ้นและมีมากกว่าหนึ่ง leg
venue จะ publish strategy นั้นเป็น instrument เดียวที่มี symbol, tick size,
วันหมดอายุ และกฎการ execute เป็นของตัวเอง

ตัวอย่างเช่น vertical spread, calendar spread ที่จดทะเบียนในตลาด และ option strategy อื่น ๆ

## ฟิลด์ {#fields}

| Field             | Rust type          | Required/default | หมายเหตุ                                    |
| ----------------- | ------------------ | ---------------- | ------------------------------------------ |
| `instrument_id`   | `InstrumentId`     | Required         | ใน Rust เก็บไว้ในชื่อ `id`                    |
| `raw_symbol`      | `Symbol`           | Required         | symbol ดั้งเดิมของ venue                     |
| `asset_class`     | `AssetClass`       | Required         | asset class ของ strategy อ้างอิง             |
| `exchange`        | `Option<Ustr>`     | `None`           | รหัส MIC ของตลาดหรือรหัส venue (ถ้าทราบ)      |
| `underlying`      | `Ustr`             | Required         | สินทรัพย์อ้างอิง, future หรือ index           |
| `strategy_type`   | `Ustr`             | Required         | ประเภท strategy ของ venue เช่น vertical      |
| `activation_ns`   | `UnixNanos`        | Required         | timestamp ที่ strategy เริ่มใช้งาน              |
| `expiration_ns`   | `UnixNanos`        | Required         | timestamp ที่ strategy หมดอายุ                 |
| `currency`        | `Currency`         | Required         | สกุลเงินของ premium quote และการชำระราคา     |
| `price_precision` | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับราคา          |
| `price_increment` | `Price`            | Required         | step ราคาที่เล็กที่สุดที่ใช้ได้                   |
| `size_precision`  | `u8`               | Fixed `0`        | option spread ซื้อขายเป็นจำนวนสัญญาเต็มเท่านั้น |
| `size_increment`  | `Quantity`         | Fixed `1`        | step ขนาดสัญญาขั้นต่ำ                         |
| `multiplier`      | `Quantity`         | Required         | ตัวคูณของ strategy                           |
| `lot_size`        | `Quantity`         | Required         | ขนาด lot หรือ lot ของสัญญาที่ปัดแล้ว             |
| `margin_init`     | `Option<Decimal>`  | `0`              | อัตรา initial margin                        |
| `margin_maint`    | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                    |
| `max_quantity`    | `Option<Quantity>` | `None`           | ปริมาณ order สูงสุด                           |
| `min_quantity`    | `Option<Quantity>` | `1`              | ปริมาณ order ต่ำสุด                           |
| `max_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้             |
| `min_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้              |
| `tick_scheme`     | `Option<Ustr>`     | `None`           | ชื่อ tick scheme แบบแปรผันที่ลงทะเบียนไว้          |
| `info`            | `Option<Params>`   | `None`           | metadata จาก adapter                        |
| `ts_event`        | `UnixNanos`        | Required         | timestamp ของ event หน่วยนาโนวินาที           |
| `ts_init`         | `UnixNanos`        | Required         | timestamp ตอน initialize หน่วยนาโนวินาที       |

**หมายเหตุ**: builder ใช้ชื่อ `instrument_id` แต่ Rust เก็บค่าเดียวกันไว้ในชื่อ `id`

## พฤติกรรม {#behavior}

- `OptionSpread` มี instrument class เป็น `OptionSpread`
- venue publish spread เป็น instrument เดียวที่ซื้อขายได้
- ซื้อขายเป็นจำนวนสัญญาเต็ม โดยมี size precision `0` และ size increment `1`
- ราคาของ spread เป็นศูนย์หรือติดลบได้ และ `RiskEngine` ยอมรับราคาที่ไม่เป็นบวกสำหรับ
  instrument class นี้
- เก็บรายละเอียดของ leg ที่เฉพาะกับ venue ไว้ใน `info` เมื่อ adapter มีข้อมูลนี้ให้

## ตัวอย่าง {#example}

```rust tab="Rust"
use jiff::Timestamp;
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::AssetClass,
    identifiers::{InstrumentId, Symbol},
    instruments::OptionSpread,
    types::{Currency, Price, Quantity},
};
use ustr::Ustr;

let activation: Timestamp = "2023-11-06T20:54:07Z".parse().unwrap();
let expiration: Timestamp = "2024-02-23T22:59:00Z".parse().unwrap();

let sr3_spread = OptionSpread::builder()
    .instrument_id(InstrumentId::from("UD:U$: GN 2534559.GLBX"))
    .raw_symbol(Symbol::from("UD:U$: GN 2534559"))
    .asset_class(AssetClass::FX)
    .exchange(Ustr::from("XCME"))
    .underlying(Ustr::from("SR3"))
    .strategy_type(Ustr::from("GN"))
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

**adapter ที่สร้างหรือใช้ instrument แบบ `OptionSpread` ได้แก่**:

- [Databento](../../integrations/databento.md) สำหรับตลาด option spread ที่จดทะเบียน
- [Interactive Brokers](../../integrations/interactive_brokers.md) สำหรับ option strategy ที่ตลาดกำหนด

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Option Contract](option_contract.md) อธิบาย option contract แบบ leg เดียว
- [Options](../options.md) อธิบายข้อมูล option, Greeks และการ subscribe option chain
