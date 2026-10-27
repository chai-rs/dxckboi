---
title: "Tokenized Asset"
---

`TokenizedAsset` แทน token ลักษณะคล้าย spot ที่อ้างอิงราคาของ asset อื่นบน crypto venue ใช้กับ
หุ้นแบบ tokenized, กองทุนแบบ tokenized หรือ instrument ที่คล้ายกัน ใน instrument ประเภทนี้ venue จะเปิดให้ซื้อขาย
token ที่มีการอ้างอิงทางเศรษฐกิจเป็น asset ภายนอก

ตัวอย่างเช่น symbol ของหุ้นหรือ ETF แบบ tokenized บน crypto venue

## ฟิลด์ [#fields]

| ฟิลด์              | Type ใน Rust        | จำเป็น/ค่าเริ่มต้น | หมายเหตุ                                    |
| ----------------- | ------------------ | ---------------- | ----------------------------------------- |
| `instrument_id`   | `InstrumentId`     | จำเป็น            | ใน Rust เก็บเป็น `id`                       |
| `raw_symbol`      | `Symbol`           | จำเป็น            | symbol ดั้งเดิมของ venue                     |
| `asset_class`     | `AssetClass`       | จำเป็น            | การจัดประเภท asset ทางเศรษฐกิจ               |
| `base_currency`   | `Currency`         | จำเป็น            | asset แบบ tokenized หรือ base token         |
| `quote_currency`  | `Currency`         | จำเป็น            | สกุลเงินที่ใช้กำหนดราคาของ token               |
| `price_precision` | `u8`               | จำเป็น            | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับราคา          |
| `size_precision`  | `u8`               | จำเป็น            | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับขนาด order   |
| `price_increment` | `Price`            | จำเป็น            | ขั้นราคาที่เล็กที่สุดที่ใช้ได้                     |
| `size_increment`  | `Quantity`         | จำเป็น            | ขั้นขนาดที่เล็กที่สุดที่ใช้ได้                     |
| `ts_event`        | `UnixNanos`        | จำเป็น            | timestamp ของ event หน่วยนาโนวินาที            |
| `ts_init`         | `UnixNanos`        | จำเป็น            | timestamp ตอน initialize หน่วยนาโนวินาที       |
| `isin`            | `Option<Ustr>`     | `None`           | รหัสหลักทรัพย์สากล (ISIN) หากทราบ             |
| `multiplier`      | `Quantity`         | `1`              | ตัวคูณของสัญญา (contract multiplier)          |
| `lot_size`        | `Option<Quantity>` | `None`           | ขนาด lot หรือ board lot ที่ปัดแล้ว              |
| `max_quantity`    | `Option<Quantity>` | `None`           | quantity สูงสุดของ order                     |
| `min_quantity`    | `Option<Quantity>` | `None`           | quantity ต่ำสุดของ order                     |
| `max_notional`    | `Option<Money>`    | `None`           | มูลค่า notional สูงสุดของ order               |
| `min_notional`    | `Option<Money>`    | `None`           | มูลค่า notional ต่ำสุดของ order               |
| `max_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้            |
| `min_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้            |
| `margin_init`     | `Option<Decimal>`  | `0`              | อัตรา initial margin                        |
| `margin_maint`    | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                    |
| `tick_scheme`     | `Option<Ustr>`     | `None`           | ชื่อ tick scheme แบบแปรผันที่ลงทะเบียนไว้        |
| `info`            | `Option<Params>`   | `None`           | metadata จาก adapter                        |

## พฤติกรรม [#behavior]

- `TokenizedAsset` มี instrument class เป็น `Spot`
- ไม่เป็นแบบ inverse เลย และ cost currency คือ quote currency
- สามารถมี `isin` ได้เมื่อ token อ้างอิงหลักทรัพย์ที่จดทะเบียนในตลาด
- ไม่มี activation timestamp, วันหมดอายุ, strike หรือ option kind

## ตัวอย่าง [#example]

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::AssetClass,
    identifiers::{InstrumentId, Symbol},
    instruments::TokenizedAsset,
    types::{Currency, Price, Quantity},
};

let aaplx = TokenizedAsset::builder()
    .instrument_id(InstrumentId::from("AAPLx/USD.KRAKEN"))
    .raw_symbol(Symbol::from("AAPLxUSD"))
    .asset_class(AssetClass::Equity)
    .base_currency(Currency::get_or_create_crypto("AAPLx"))
    .quote_currency(Currency::from("USD"))
    .price_precision(2)
    .size_precision(4)
    .price_increment(Price::from("0.01"))
    .size_increment(Quantity::from("0.0001"))
    .min_quantity(Quantity::from("0.0001"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapter [#adapters]

adapter ของ [Kraken](../../integrations/kraken.md) สร้างหรือใช้ instrument แบบ
`TokenizedAsset` สำหรับ asset แบบ tokenized ในกรณีที่ venue เปิดให้ใช้

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Currency Pair](currency_pair.md) ครอบคลุมคู่ spot ของ crypto ทั่วไป
- [Equity](equity.md) ครอบคลุมหุ้นสามัญที่จดทะเบียนในตลาด
