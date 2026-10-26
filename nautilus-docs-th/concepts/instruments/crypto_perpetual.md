# Crypto Perpetual

`CryptoPerpetual` แทนสัญญา crypto perpetual futures หรือที่เรียกว่า perpetual swap สัญญาประเภทนี้
ไม่มีวันหมดอายุ อ้างอิง crypto base asset และ settle ด้วย crypto, stablecoin หรือ settlement currency
อื่นที่ venue กำหนด

ตัวอย่างเช่น `ETHUSDT-PERP.BINANCE`, `BTCUSD.BYBIT` และ `BTC-USD-SWAP.OKX`

## ฟิลด์ {#fields}

| Field                 | Rust type          | Required/default | หมายเหตุ |
| --------------------- | ------------------ | ---------------- | ---------------------------------------- |
| `instrument_id`       | `InstrumentId`     | Required         | Rust เก็บไว้ในชื่อ `id`                  |
| `raw_symbol`          | `Symbol`           | Required         | symbol ดั้งเดิมของ venue                     |
| `base_currency`       | `Currency`         | Required         | crypto asset หลัก (base)                       |
| `quote_currency`      | `Currency`         | Required         | สกุลเงินที่ใช้ quote ราคา                    |
| `settlement_currency` | `Currency`         | Required         | สกุลเงินที่ใช้ settle PnL และค่าธรรมเนียม    |
| `is_inverse`          | `bool`             | Required         | เป็น true เมื่อการคิดขนาด/ต้นทุนเป็นแบบ inverse     |
| `price_precision`     | `u8`               | Required         | จำนวนทศนิยมที่อนุญาตสำหรับราคา       |
| `size_precision`      | `u8`               | Required         | จำนวนทศนิยมที่อนุญาตสำหรับขนาด order  |
| `price_increment`     | `Price`            | Required         | ขั้นราคาที่เล็กที่สุดที่ใช้ได้               |
| `size_increment`      | `Quantity`         | Required         | ขั้นขนาดที่เล็กที่สุดที่ใช้ได้                |
| `ts_event`            | `UnixNanos`        | Required         | timestamp ของ event หน่วย nanoseconds          |
| `ts_init`             | `UnixNanos`        | Required         | timestamp ตอนสร้าง object หน่วย nanoseconds |
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

**หมายเหตุ**: Rust เก็บค่า `instrument_id` ไว้ในชื่อ `id`

## พฤติกรรม {#behavior}

- `CryptoPerpetual` มี asset class เป็น `Cryptocurrency` และ instrument class เป็น `Swap`
- ไม่มี activation timestamp หรือ expiration timestamp

**รูปแบบการ settle**: ชุดสกุลเงินที่กำหนดเป็นตัวบอกรูปแบบการ settle

- **Linear**: โดยทั่วไปตั้ง `is_inverse=False` และ settle ด้วย quote currency
- **Inverse**: ตั้ง `is_inverse=True` และโดยทั่วไป settle ด้วย base currency
- **Quanto**: settle ด้วยสกุลเงินที่สามซึ่งไม่ใช่ทั้ง base และ quote

**Cost currency**: ใช้ base สำหรับสัญญา inverse, ใช้ settlement สำหรับสัญญา quanto และใช้ quote ในกรณีอื่น

:::note
funding payment ไม่ได้เป็นฟิลด์ของ instrument แต่จะเข้ามาในรูปของ data เช่น `FundingRateUpdate`
ซึ่งอ้างอิงถึง instrument ID
:::

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Symbol},
    instruments::{CryptoPerpetual, InstrumentAny},
    types::{Currency, Money, Price, Quantity},
};
use rust_decimal_macros::dec;

let ethusdt_perp = CryptoPerpetual::builder()
    .instrument_id(InstrumentId::from("ETHUSDT-PERP.BINANCE"))
    .raw_symbol(Symbol::from("ETHUSDT"))
    .base_currency(Currency::from("ETH"))
    .quote_currency(Currency::from("USDT"))
    .settlement_currency(Currency::from("USDT"))
    .is_inverse(false)
    .price_precision(2)
    .size_precision(3)
    .price_increment(Price::from("0.01"))
    .size_increment(Quantity::from("0.001"))
    .max_quantity(Quantity::from("10000.000"))
    .min_quantity(Quantity::from("0.001"))
    .min_notional(Money::from("10.00 USDT"))
    .max_price(Price::from("15000.00"))
    .min_price(Price::from("1.00"))
    .margin_init(dec!(1.0))
    .margin_maint(dec!(0.35))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();

let instrument = InstrumentAny::CryptoPerpetual(ethusdt_perp);
```

## Adapter {#adapters}

**Adapter ที่สร้างหรือใช้ instrument แบบ `CryptoPerpetual` ได้แก่**:

- [Binance](../../integrations/binance.md) สำหรับ perpetual futures แบบ USD-M และ COIN-M
- [Bybit](../../integrations/bybit.md) สำหรับผลิตภัณฑ์ perpetual แบบ linear และ inverse
- [dYdX](../../integrations/dydx.md) สำหรับตลาด perpetual
- [Hyperliquid](../../integrations/hyperliquid.md) สำหรับตลาด perpetual
- [Kraken](../../integrations/kraken.md) สำหรับตลาด perpetual ของ futures venue
- [OKX](../../integrations/okx.md) สำหรับตลาด swap
- [Tardis](../../integrations/tardis.md) สำหรับ metadata ของ crypto perpetual

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Data](../data/) อธิบาย mark price, index price และ funding rate update
- [Options](../options.md) อธิบายประเภท instrument เฉพาะของ option
- [Execution](../execution/) อธิบายการตรวจสอบ precision และ notional ก่อนที่ order จะไปถึง venue
