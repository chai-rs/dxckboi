# CFD

`Cfd` แทน contract for difference (สัญญาส่วนต่าง) ที่อ้างอิงราคาสินทรัพย์อ้างอิงโดยไม่มีการโอนกรรมสิทธิ์
ในสินทรัพย์อ้างอิงนั้น venue เป็นผู้กำหนด quote currency, precision, increment, limit, margin และ fee

ตัวอย่างเช่น สัญญา CFD บน FX, หุ้น, ดัชนี และสินค้าโภคภัณฑ์

## Fields {#fields}

| Field             | Rust type          | จำเป็น/ค่าเริ่มต้น  | หมายเหตุ                                    |
| ----------------- | ------------------ | ---------------- | ------------------------------------------ |
| `instrument_id`   | `InstrumentId`     | จำเป็น            | ใน Rust เก็บเป็น `id`                        |
| `raw_symbol`      | `Symbol`           | จำเป็น            | Symbol ดั้งเดิมของ venue                     |
| `asset_class`     | `AssetClass`       | จำเป็น            | Asset class ของสินทรัพย์อ้างอิง               |
| `base_currency`   | `Option<Currency>` | `None`           | Base currency ในกรณีที่ CFD อ้างอิงสกุลเงิน     |
| `quote_currency`  | `Currency`         | จำเป็น            | สกุลเงินที่ใช้ quote และประเมินมูลค่าราคา        |
| `price_precision` | `u8`               | จำเป็น            | จำนวนทศนิยมที่อนุญาตสำหรับราคา                 |
| `size_precision`  | `u8`               | จำเป็น            | จำนวนทศนิยมที่อนุญาตสำหรับขนาด order          |
| `price_increment` | `Price`            | จำเป็น            | ขั้นราคาที่เล็กที่สุดที่ใช้ได้                     |
| `size_increment`  | `Quantity`         | จำเป็น            | ขั้นขนาดที่เล็กที่สุดที่ใช้ได้                      |
| `lot_size`        | `Option<Quantity>` | `None`           | ขนาด lot หรือ board ที่ปัดแล้ว                 |
| `max_quantity`    | `Option<Quantity>` | `None`           | ปริมาณ order สูงสุด                          |
| `min_quantity`    | `Option<Quantity>` | `None`           | ปริมาณ order ต่ำสุด                          |
| `max_notional`    | `Option<Money>`    | `None`           | มูลค่า notional ของ order สูงสุด               |
| `min_notional`    | `Option<Money>`    | `None`           | มูลค่า notional ของ order ต่ำสุด               |
| `max_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้            |
| `min_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้            |
| `margin_init`     | `Option<Decimal>`  | `0`              | อัตรา initial margin                         |
| `margin_maint`    | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                     |
| `tick_scheme`     | `Option<Ustr>`     | `None`           | ชื่อ variable tick scheme ที่ลงทะเบียนไว้         |
| `info`            | `Option<Params>`   | `None`           | Metadata ของ adapter                        |
| `ts_event`        | `UnixNanos`        | จำเป็น            | Timestamp ของ event หน่วย nanosecond         |
| `ts_init`         | `UnixNanos`        | จำเป็น            | Timestamp ตอน initialize หน่วย nanosecond    |

**หมายเหตุ**: builder ใช้ชื่อ `instrument_id` แต่ Rust เก็บค่าเดียวกันไว้ใน field `id`

## พฤติกรรม {#behavior}

- `Cfd` มี instrument class เป็น `Cfd`
- ไม่เป็น inverse และใช้ multiplier เท่ากับหนึ่งเสมอ
- ไม่มี activation timestamp, expiration timestamp, strike หรือ option kind
- หาก venue มีทั้ง instrument แบบ cash และแบบ CFD บนสินทรัพย์อ้างอิงเดียวกัน ให้ใช้ `Cfd` เฉพาะกับ CFD
  และใช้ cash type ที่ตรงกันสำหรับตลาดของสินทรัพย์อ้างอิง

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::AssetClass,
    identifiers::{InstrumentId, Symbol},
    instruments::Cfd,
    types::{Currency, Price, Quantity},
};
use rust_decimal_macros::dec;

let audusd = Cfd::builder()
    .instrument_id(InstrumentId::from("AUDUSD.OANDA"))
    .raw_symbol(Symbol::from("AUD/USD"))
    .asset_class(AssetClass::FX)
    .base_currency(Currency::from("AUD"))
    .quote_currency(Currency::from("USD"))
    .price_precision(5)
    .size_precision(0)
    .price_increment(Price::from("0.00001"))
    .size_increment(Quantity::from("1"))
    .lot_size(Quantity::from("1000"))
    .margin_init(dec!(0.03))
    .margin_maint(dec!(0.03))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapters {#adapters}

Adapter ของ [Interactive Brokers](../../integrations/interactive_brokers.md) สร้างหรือใช้ instrument แบบ
`Cfd` สำหรับสัญญา CFD

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Currency Pair](currency_pair.md) ครอบคลุมคู่เงิน FX แบบ cash และคู่ crypto แบบ spot
- [Commodity](commodity.md) ครอบคลุม instrument สินค้าโภคภัณฑ์แบบ spot
