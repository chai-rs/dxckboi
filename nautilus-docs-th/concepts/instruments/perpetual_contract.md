# Perpetual Contract

`PerpetualContract` แทนสัญญา perpetual futures แบบทั่วไปที่ใช้ได้กับทุก asset class ใช้เมื่อ venue
มี perpetual swap ที่ไม่ได้ถูก model ไว้เป็น `CryptoPerpetual` โดยเฉพาะ

ตัวอย่างเช่น สัญญา perpetual ที่ไม่ใช่ crypto และ synthetic swap เฉพาะของแต่ละ venue

## ฟิลด์ {#fields}

| Field                 | Rust type          | Required/default | หมายเหตุ |
| --------------------- | ------------------ | ---------------- | ---------------------------------------- |
| `instrument_id`       | `InstrumentId`     | Required         | Rust เก็บไว้ในชื่อ `id`                  |
| `raw_symbol`          | `Symbol`           | Required         | symbol ดั้งเดิมของ venue                     |
| `underlying`          | `Ustr`             | Required         | สินทรัพย์อ้างอิงหรือตลาดอ้างอิง    |
| `asset_class`         | `AssetClass`       | Required         | asset class ของสินทรัพย์อ้างอิง           |
| `base_currency`       | `Option<Currency>` | `None`           | สกุลเงินหลัก จำเป็นสำหรับแบบ inverse     |
| `quote_currency`      | `Currency`         | Required         | สกุลเงินที่ใช้ quote ราคา        |
| `settlement_currency` | `Currency`         | Required         | สกุลเงินที่ใช้ settle PnL และค่าธรรมเนียม    |
| `is_inverse`          | `bool`             | Required         | เป็น true เมื่อการคิดขนาด/ต้นทุนเป็นแบบ inverse     |
| `price_precision`     | `u8`               | Required         | จำนวนทศนิยมที่อนุญาตสำหรับราคา       |
| `size_precision`      | `u8`               | Required         | จำนวนทศนิยมที่อนุญาตสำหรับขนาด order  |
| `price_increment`     | `Price`            | Required         | ขั้นราคาที่เล็กที่สุดที่ใช้ได้               |
| `size_increment`      | `Quantity`         | Required         | ขั้นขนาดที่เล็กที่สุดที่ใช้ได้                |
| `multiplier`          | `Quantity`         | `1`              | ตัวคูณของสัญญา                     |
| `lot_size`            | `Quantity`         | `1`              | ขนาด lot หรือ board ที่ปัดแล้ว               |
| `max_quantity`        | `Option<Quantity>` | `None`           | ปริมาณ order สูงสุด                  |
| `min_quantity`        | `Option<Quantity>` | `None`           | ปริมาณ order ต่ำสุด                  |
| `max_notional`        | `Option<Money>`    | `None`           | มูลค่า notional ของ order สูงสุด            |
| `min_notional`        | `Option<Money>`    | `None`           | มูลค่า notional ของ order ต่ำสุด            |
| `max_price`           | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้      |
| `min_price`           | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้      |
| `margin_init`         | `Option<Decimal>`  | `0`              | อัตรา initial margin                     |
| `margin_maint`        | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                 |
| `tick_scheme`         | `Option<Ustr>`     | `None`           | ชื่อ variable tick scheme ที่ลงทะเบียนไว้    |
| `info`                | `Option<Params>`   | `None`           | metadata ของ adapter                        |
| `ts_event`            | `UnixNanos`        | Required         | timestamp ของ event หน่วย nanoseconds          |
| `ts_init`             | `UnixNanos`        | Required         | timestamp ตอนสร้าง object หน่วย nanoseconds |

**หมายเหตุ**: Rust เก็บค่า `instrument_id` ไว้ในชื่อ `id`

## พฤติกรรม {#behavior}

- `PerpetualContract` มี instrument class เป็น `Swap`
- ไม่มี activation timestamp หรือ expiration timestamp
- สัญญาแบบ inverse ต้องมี base currency
- สัญญาแบบ linear โดยทั่วไป settle ด้วย quote currency
- ใช้ `CryptoPerpetual` สำหรับ crypto perpetual ที่ base asset เป็นสกุลเงิน

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::AssetClass,
    identifiers::{InstrumentId, Symbol},
    instruments::PerpetualContract,
    types::{Currency, Price, Quantity},
};
use rust_decimal_macros::dec;
use ustr::Ustr;

let eurusd_perp = PerpetualContract::builder()
    .instrument_id(InstrumentId::from("EURUSD-PERP.AX"))
    .raw_symbol(Symbol::from("EURUSD-PERP"))
    .underlying(Ustr::from("EURUSD"))
    .asset_class(AssetClass::FX)
    .base_currency(Currency::from("EUR"))
    .quote_currency(Currency::from("USD"))
    .settlement_currency(Currency::from("USD"))
    .is_inverse(false)
    .price_precision(5)
    .size_precision(0)
    .price_increment(Price::from("0.00001"))
    .size_increment(Quantity::from("1"))
    .margin_init(dec!(0.03))
    .margin_maint(dec!(0.03))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapter {#adapters}

**Adapter ที่สร้างหรือใช้ instrument แบบ `PerpetualContract` ได้แก่**:

- [Architect AX](../../integrations/architect_ax.md) สำหรับสัญญา perpetual ที่ venue กำหนด
- [Binance](../../integrations/binance.md) สำหรับสัญญา perpetual แบบ USD-M TradFi

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Crypto Perpetual](crypto_perpetual.md) อธิบาย crypto perpetual futures
- [Data](../data/) อธิบาย mark price, index price และ funding rate update
