---
title: "Currency Pair"
---

`CurrencyPair` แทนตลาด spot หรือตลาดเงินสด (cash market) ที่ quote ในรูป `BASE/QUOTE` โดย base
currency คือสินทรัพย์ที่ซื้อหรือขาย และ quote currency คือสกุลเงินที่ใช้ตั้งราคาของ base หนึ่งหน่วย
Nautilus ใช้ type นี้กับทั้งคู่เงิน FX แบบ fiat และคู่ crypto spot

ตัวอย่างเช่น `EUR/USD.SIM`, `BTCUSDT.BINANCE` และ `ETH/USD.KRAKEN`

## ฟิลด์ [#fields]

| Field             | Rust type          | Required/default | Notes                                       |
| ----------------- | ------------------ | ---------------- | ------------------------------------------- |
| `instrument_id`   | `InstrumentId`     | Required         | ใน Rust เก็บไว้ในชื่อ `id`                   |
| `raw_symbol`      | `Symbol`           | Required         | symbol ดั้งเดิมของ venue                     |
| `base_currency`   | `Currency`         | Required         | สินทรัพย์ที่ซื้อหรือขาย                      |
| `quote_currency`  | `Currency`         | Required         | สกุลเงินที่ใช้ตั้งราคาสินทรัพย์ base          |
| `price_precision` | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับราคา        |
| `size_precision`  | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับขนาด order  |
| `price_increment` | `Price`            | Required         | ขั้นราคาที่เล็กที่สุดที่ใช้ได้               |
| `size_increment`  | `Quantity`         | Required         | ขั้นขนาดที่เล็กที่สุดที่ใช้ได้               |
| `ts_event`        | `UnixNanos`        | Required         | timestamp ของ event หน่วยนาโนวินาที          |
| `ts_init`         | `UnixNanos`        | Required         | timestamp ตอน initialize หน่วยนาโนวินาที     |
| `multiplier`      | `Quantity`         | `1`              | ตัวคูณของสัญญา (contract multiplier)        |
| `lot_size`        | `Option<Quantity>` | `None`           | ขนาด lot หรือ board ที่ปัดแล้ว               |
| `max_quantity`    | `Option<Quantity>` | `None`           | ปริมาณ order สูงสุด                          |
| `min_quantity`    | `Option<Quantity>` | `None`           | ปริมาณ order ต่ำสุด                          |
| `max_notional`    | `Option<Money>`    | `None`           | มูลค่า notional ของ order สูงสุด             |
| `min_notional`    | `Option<Money>`    | `None`           | มูลค่า notional ของ order ต่ำสุด             |
| `max_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้        |
| `min_price`       | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้        |
| `margin_init`     | `Option<Decimal>`  | `0`              | อัตรา initial margin                        |
| `margin_maint`    | `Option<Decimal>`  | `0`              | อัตรา maintenance margin                    |
| `tick_scheme`     | `Option<Ustr>`     | `None`           | ชื่อ variable tick scheme ที่ลงทะเบียนไว้     |
| `info`            | `Option<Params>`   | `None`           | metadata จาก adapter                        |

**หมายเหตุ**: ใน Rust ค่า `instrument_id` ถูกเก็บไว้ในชื่อ `id`

## พฤติกรรม [#behavior]

- `CurrencyPair` มี instrument class เป็น `Spot`
- asset class เป็น `Cryptocurrency` เมื่อสกุลเงินใดสกุลหนึ่งเป็นประเภท crypto หากไม่มีสกุลใดเป็นประเภท crypto
  จะเป็น `FX`
- ไม่มีฟิลด์วันหมดอายุ, strike price, ชนิด option หรือ underlying ของ derivative
- ไม่เป็น inverse เลย settlement currency และ cost currency คือ quote currency
- ใช้ type นี้ได้ทั้งกับคู่เงิน FX แบบ fiat และคู่ crypto spot

:::note
ให้ใช้ type ของ derivative ที่ตรงกับสินค้าสำหรับ futures ที่มีวันหมดอายุ, swap และ option เพื่อให้ cost
currency, settlement currency, วันหมดอายุ และการคำนวณ notional สอดคล้องกับ venue
แม้ symbol จะดูเหมือนคู่เงินก็ตาม
:::

## ตัวอย่าง [#example]

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Symbol},
    instruments::CurrencyPair,
    types::{Currency, Money, Price, Quantity},
};
use rust_decimal_macros::dec;

let btcusdt = CurrencyPair::builder()
    .instrument_id(InstrumentId::from("BTCUSDT.BINANCE"))
    .raw_symbol(Symbol::from("BTCUSDT"))
    .base_currency(Currency::from("BTC"))
    .quote_currency(Currency::from("USDT"))
    .price_precision(2)
    .size_precision(6)
    .price_increment(Price::from("0.01"))
    .size_increment(Quantity::from("0.000001"))
    .min_quantity(Quantity::from("0.000001"))
    .min_notional(Money::from("10.00 USDT"))
    .max_price(Price::from("1000000.00"))
    .min_price(Price::from("0.01"))
    .margin_init(dec!(0.001))
    .margin_maint(dec!(0.001))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapters [#adapters]

**Adapter ที่สร้างหรือใช้ instrument แบบ `CurrencyPair` ได้แก่**:

- [Binance](../../integrations/binance.md) สำหรับตลาด spot
- [Kraken](../../integrations/kraken.md) สำหรับตลาด spot
- [OKX](../../integrations/okx.md) สำหรับตลาด spot
- [Tardis](../../integrations/tardis.md) สำหรับ metadata ของ spot
- [Interactive Brokers](../../integrations/interactive_brokers.md) สำหรับสัญญา FX แบบ cash
- [Hyperliquid](../../integrations/hyperliquid.md) สำหรับสินทรัพย์ spot

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Data](../data/) อธิบาย market data ที่อ้างอิงถึง instrument
- [Execution](../execution/) อธิบายการตรวจสอบ order ที่ใช้ precision ของ instrument
- [Value types](../value_types.md) อธิบาย `Price`, `Quantity` และ `Money`
