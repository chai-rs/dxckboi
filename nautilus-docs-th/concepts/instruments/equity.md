# Equity

`Equity` แทนหุ้นจดทะเบียน, ETF หรือหลักทรัพย์ในตลาดเงินสดที่คล้ายกัน Nautilus ใช้
type นี้กับ instrument ที่ซื้อขายเป็นหน่วยเต็ม, quote ในสกุลเงินเดียว และไม่มี
วันหมดอายุของสัญญา

ตัวอย่างเช่น `AAPL.XNAS`, `MSFT.XNAS` และ symbol ของ ETF ที่เฉพาะเจาะจงตาม venue

## ฟิลด์ {#fields}

| Field             | Rust type          | Required/default | Notes                                        |
| ----------------- | ------------------ | ---------------- | -------------------------------------------- |
| `instrument_id`   | `InstrumentId`     | Required         | ใน Rust เก็บไว้ในชื่อ `id`                    |
| `raw_symbol`      | `Symbol`           | Required         | symbol ดั้งเดิมของ venue                      |
| `currency`        | `Currency`         | Required         | สกุลเงินสำหรับ quote และ settlement           |
| `price_precision` | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับราคา         |
| `price_increment` | `Price`            | Required         | ขั้นราคาที่เล็กที่สุดที่ใช้ได้                |
| `lot_size`        | `Option<Quantity>` | `None`           | ขนาด board lot หรือ lot แบบหุ้นเต็มจำนวน       |
| `ts_event`        | `UnixNanos`        | Required         | timestamp ของ event หน่วยนาโนวินาที           |
| `ts_init`         | `UnixNanos`        | Required         | timestamp ตอน initialize หน่วยนาโนวินาที      |
| `isin`            | `Option<Ustr>`     | `None`           | International Securities ID หากทราบ          |
| `max_quantity`    | `Option<Quantity>` | `None`           | ปริมาณ order สูงสุด                           |
| `min_quantity`    | `Option<Quantity>` | `None`           | ปริมาณ order ต่ำสุด                           |
| `max_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้         |
| `min_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้         |
| `margin_init`     | `Option<Decimal>`  | `0`              | อัตรา initial margin                         |
| `margin_maint`    | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                     |
| `tick_scheme`     | `Option<Ustr>`     | `None`           | ชื่อ variable tick scheme ที่ลงทะเบียนไว้      |
| `info`            | `Option<Params>`   | `None`           | metadata จาก adapter                         |

**หมายเหตุ**: ใน Rust ค่า `instrument_id` ถูกเก็บไว้ในชื่อ `id`

## พฤติกรรม {#behavior}

- `Equity` มี asset class เป็น `Equity` และ instrument class เป็น `Spot`
- quantity precision เป็นศูนย์เสมอ ดังนั้น order จึงใช้ปริมาณเป็นจำนวนหุ้นเต็ม
- multiplier และ size increment เท่ากับหนึ่ง
- ไม่มี base currency, วันหมดอายุ, strike, ชนิด option หรือ flag การคิดต้นทุนแบบ inverse
- ใช้ขีดจำกัดราคาเฉพาะเมื่อ venue เผยแพร่ข้อมูลนั้นเท่านั้น

:::warning
`Equity` กำหนด size precision ไว้ที่ศูนย์ `RiskEngine` จะปฏิเสธ order ใด ๆ ที่ quantity
precision เกิน size precision ของ instrument ดังนั้นปริมาณที่เป็นเศษส่วนของหุ้นจะถูกปฏิเสธ
:::

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Symbol},
    instruments::Equity,
    types::{Currency, Price, Quantity},
};
use ustr::Ustr;

let aapl = Equity::builder()
    .instrument_id(InstrumentId::from("AAPL.XNAS"))
    .raw_symbol(Symbol::from("AAPL"))
    .isin(Ustr::from("US0378331005"))
    .currency(Currency::from("USD"))
    .price_precision(2)
    .price_increment(Price::from("0.01"))
    .lot_size(Quantity::from("100"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapters {#adapters}

**Adapter ที่สร้างหรือใช้ instrument แบบ `Equity` ได้แก่**:

- [Databento](../../integrations/databento.md) สำหรับหุ้นและ ETF ที่จดทะเบียนในสหรัฐฯ
- [Interactive Brokers](../../integrations/interactive_brokers.md) สำหรับสัญญาหุ้นจดทะเบียน

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Data](../data/) อธิบาย market data ที่อ้างอิงถึง instrument
- [Value types](../value_types.md) อธิบาย `Price`, `Quantity` และ `Money`
