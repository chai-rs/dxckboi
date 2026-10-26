# Crypto Option Spread

`CryptoOptionSpread` แทน strategy ที่ exchange นิยามขึ้นบน crypto option โดย venue จะ publish
strategy นี้เป็น instrument เดียวที่มี symbol, strategy type, precision, increment และวันหมดอายุเป็นของตัวเอง

ตัวอย่างเช่น option combo ของ BTC หรือ ETH ที่ list อยู่บน venue ซื้อขาย crypto derivatives

## ฟิลด์ {#fields}

| ฟิลด์                  | Rust type          | Required/default | หมายเหตุ                                      |
| --------------------- | ------------------ | ---------------- | -------------------------------------------- |
| `instrument_id`       | `InstrumentId`     | Required         | ใน Rust เก็บเป็นฟิลด์ `id`                      |
| `raw_symbol`          | `Symbol`           | Required         | symbol ดั้งเดิมของ venue                        |
| `underlying`          | `Currency`         | Required         | สินทรัพย์ crypto ที่ strategy อ้างอิง              |
| `quote_currency`      | `Currency`         | Required         | สกุลเงินที่ใช้ quote premium                     |
| `settlement_currency` | `Currency`         | Required         | สกุลเงินที่ใช้ settle PnL และค่าธรรมเนียม           |
| `is_inverse`          | `bool`             | Required         | เป็น true เมื่อการคิดขนาด/ต้นทุนเป็นแบบ inverse    |
| `strategy_type`       | `Ustr`             | Required         | strategy type ของ venue เช่น vertical         |
| `activation_ns`       | `UnixNanos`        | Required         | timestamp ที่ strategy เริ่มใช้งาน                |
| `expiration_ns`       | `UnixNanos`        | Required         | timestamp ที่ strategy หมดอายุ                  |
| `price_precision`     | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับราคา             |
| `size_precision`      | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับขนาด order       |
| `price_increment`     | `Price`            | Required         | ขั้นราคาที่เล็กที่สุดที่ใช้ได้                         |
| `size_increment`      | `Quantity`         | Required         | ขั้นขนาดที่เล็กที่สุดที่ใช้ได้                        |
| `multiplier`          | `Quantity`         | `1`              | ตัวคูณของ strategy                             |
| `lot_size`            | `Quantity`         | `1`              | ขนาด lot หรือ board ที่ปัดแล้ว                   |
| `max_quantity`        | `Option<Quantity>` | `None`           | quantity สูงสุดของ order                        |
| `min_quantity`        | `Option<Quantity>` | `None`           | quantity ต่ำสุดของ order                        |
| `max_notional`        | `Option<Money>`    | `None`           | มูลค่า notional สูงสุดของ order                  |
| `min_notional`        | `Option<Money>`    | `None`           | มูลค่า notional ต่ำสุดของ order                  |
| `max_price`           | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้               |
| `min_price`           | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้               |
| `margin_init`         | `Option<Decimal>`  | `0`              | อัตรา initial margin                           |
| `margin_maint`        | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                       |
| `tick_scheme`         | `Option<Ustr>`     | `None`           | ชื่อ variable tick scheme ที่ลงทะเบียนไว้          |
| `info`                | `Option<Params>`   | `None`           | metadata จาก adapter                          |
| `ts_event`            | `UnixNanos`        | Required         | timestamp ของ event หน่วยนาโนวินาที             |
| `ts_init`             | `UnixNanos`        | Required         | timestamp ตอน initialize หน่วยนาโนวินาที         |

**หมายเหตุ**: Rust เก็บค่าของ `instrument_id` ไว้ในฟิลด์ `id` (builder ใช้ `.instrument_id(...)`)

## พฤติกรรม {#behavior}

- `CryptoOptionSpread` มี asset class เป็น `Cryptocurrency` และ instrument class เป็น
  `OptionSpread`
- venue publish spread เป็น instrument เดียวที่ซื้อขายได้
- strategy อาจเป็นแบบ linear, inverse หรือ quanto ขึ้นกับชุดสกุลเงิน
- ราคา spread อาจเป็นศูนย์หรือติดลบได้ และ `RiskEngine` ยอมรับราคาที่ไม่เป็นบวกสำหรับ instrument class นี้
- ให้เก็บรายละเอียดของแต่ละ leg ที่เฉพาะกับ venue ไว้ใน `info` เมื่อ adapter ส่งมาให้

## ตัวอย่าง {#example}

```rust tab="Rust"
use jiff::Timestamp;
use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Symbol},
    instruments::CryptoOptionSpread,
    types::{Currency, Price, Quantity},
};
use ustr::Ustr;

let activation: Timestamp = "2026-05-12T00:00:00Z".parse().unwrap();
let expiration: Timestamp = "2026-05-19T08:00:00Z".parse().unwrap();

let btc_spread = CryptoOptionSpread::builder()
    .instrument_id(InstrumentId::from("BTC-CS-19MAY26-70000_75000.DERIBIT"))
    .raw_symbol(Symbol::from("BTC-CS-19MAY26-70000_75000"))
    .underlying(Currency::from("BTC"))
    .quote_currency(Currency::from("USD"))
    .settlement_currency(Currency::from("BTC"))
    .is_inverse(false)
    .strategy_type(Ustr::from("CS"))
    .activation_ns(UnixNanos::from(activation))
    .expiration_ns(UnixNanos::from(expiration))
    .price_precision(4)
    .size_precision(1)
    .price_increment(Price::from("0.0001"))
    .size_increment(Quantity::from("0.1"))
    .multiplier(Quantity::from("1"))
    .min_quantity(Quantity::from("0.1"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapters {#adapters}

**adapter ที่สร้างหรือใช้ instrument แบบ `CryptoOptionSpread` ได้แก่**:

- [Deribit](../../integrations/deribit.md) สำหรับ crypto option combo
- [OKX](../../integrations/okx.md) สำหรับตลาด crypto option spread

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Crypto Option](crypto_option.md) อธิบาย crypto option แบบ leg เดียว
- [Option Spread](option_spread.md) อธิบาย option spread ที่ไม่ใช่ crypto
