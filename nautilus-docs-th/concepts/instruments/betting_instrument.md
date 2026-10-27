---
title: "Betting Instrument"
---

`BettingInstrument` แทนตัวเลือก (selection) หนึ่งตัวในตลาดกีฬาหรือเกม โดยเก็บ metadata ของ event,
competition, market และ selection ไว้ เพื่อให้ Nautilus ปฏิบัติต่อ selection นั้นเหมือน instrument ที่มีราคา,
ขนาด, ข้อจำกัด, margin และค่าธรรมเนียม

ตัวอย่างเช่น selection ในตลาด match odds และตลาด handicap ของ Betfair

## ฟิลด์ [#fields]

| ฟิลด์                 | Rust type          | จำเป็น/ค่าเริ่มต้น  | หมายเหตุ                                    |
| -------------------- | ------------------ | ---------------- | ---------------------------------------- |
| `instrument_id`      | `InstrumentId`     | Required         | ใน Rust เก็บไว้ในชื่อ `id`                  |
| `raw_symbol`         | `Symbol`           | Required         | symbol ดั้งเดิมของ venue หรือที่สร้างขึ้น       |
| `event_type_id`      | `u64`              | Required         | ตัวระบุชนิดของ event                        |
| `event_type_name`    | `Ustr`             | Required         | ชื่อชนิดของ event เช่น ประเภทกีฬา            |
| `competition_id`     | `u64`              | Required         | ตัวระบุ competition                        |
| `competition_name`   | `Ustr`             | Required         | ชื่อ competition                           |
| `event_id`           | `u64`              | Required         | ตัวระบุ event                              |
| `event_name`         | `Ustr`             | Required         | ชื่อ event                                 |
| `event_country_code` | `Ustr`             | Required         | รหัสประเทศของ event                         |
| `event_open_date`    | `UnixNanos`        | Required         | เวลาเปิดของ event                           |
| `betting_type`       | `Ustr`             | Required         | ประเภทการเดิมพันที่ venue เผยแพร่              |
| `market_id`          | `Ustr`             | Required         | ตัวระบุ market                             |
| `market_name`        | `Ustr`             | Required         | ชื่อ market                                |
| `market_type`        | `Ustr`             | Required         | ชนิดของ market เช่น match odds             |
| `market_start_time`  | `UnixNanos`        | Required         | เวลาเริ่มของ market                          |
| `selection_id`       | `u64`              | Required         | ตัวระบุ selection หรือ runner                |
| `selection_name`     | `Ustr`             | Required         | ชื่อ selection หรือ runner                   |
| `selection_handicap` | `f64`              | Required         | ค่า handicap สำหรับตลาด handicap            |
| `currency`           | `Currency`         | Required         | สกุลเงินของ quote และการชำระ                 |
| `price_precision`    | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับราคา           |
| `size_precision`     | `u8`               | Required         | จำนวนตำแหน่งทศนิยมที่อนุญาตสำหรับขนาด order      |
| `price_increment`    | `Price`            | Required         | ขั้นราคา มักกำหนดโดย tick scheme             |
| `size_increment`     | `Quantity`         | Required         | ขั้นขนาดขั้นต่ำ                               |
| `max_quantity`       | `Option<Quantity>` | `None`           | ปริมาณ order สูงสุด                          |
| `min_quantity`       | `Option<Quantity>` | `None`           | ปริมาณ order ต่ำสุด                          |
| `max_notional`       | `Option<Money>`    | `None`           | มูลค่า notional ของ order สูงสุด              |
| `min_notional`       | `Option<Money>`    | `None`           | มูลค่า notional ของ order ต่ำสุด              |
| `max_price`          | `Option<Price>`    | `None`           | ราคา quote หรือ order สูงสุดที่ใช้ได้            |
| `min_price`          | `Option<Price>`    | `None`           | ราคา quote หรือ order ต่ำสุดที่ใช้ได้            |
| `margin_init`        | `Option<Decimal>`  | `1`              | อัตรา initial margin                       |
| `margin_maint`       | `Option<Decimal>`  | `1`              | อัตรา maintenance margin                   |
| `tick_scheme`        | `Option<Ustr>`     | `None`           | ชื่อ tick scheme แบบแปรผันที่ลงทะเบียนไว้        |
| `info`               | `Option<Params>`   | `None`           | metadata ของ adapter                       |
| `ts_event`           | `UnixNanos`        | Required         | timestamp ของ event หน่วยนาโนวินาที         |
| `ts_init`            | `UnixNanos`        | Required         | timestamp ตอนสร้าง หน่วยนาโนวินาที           |

## พฤติกรรม [#behavior]

- `BettingInstrument` มี asset class เป็น `Alternative` และ instrument class เป็น `SportsBetting`
- แต่ละ selection หรือ runner ถูกจำลองเป็น instrument ของตัวเอง
- betting instrument มักใช้ tick scheme ที่ลงทะเบียนไว้เพื่อกำหนดขั้นของ odds ที่ใช้ได้
- margin มีค่าเริ่มต้นเป็นหนึ่ง เพราะการวางเดิมพันมักกันเงินเดิมพันไว้เต็มจำนวน

## ตัวอย่าง [#example]

```rust tab="Rust"
use jiff::Timestamp;
use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Symbol},
    instruments::BettingInstrument,
    types::{Currency, Money, Price, Quantity},
};
use rust_decimal_macros::dec;
use ustr::Ustr;

let event_open: Timestamp = "2022-02-07T23:30:00Z".parse().unwrap();
let market_start: Timestamp = "2022-02-07T23:30:00Z".parse().unwrap();

let selection = BettingInstrument::builder()
    .instrument_id(InstrumentId::from("1-123456789-50214.BETFAIR"))
    .raw_symbol(Symbol::from("1-123456789-50214"))
    .event_type_id(6423)
    .event_type_name(Ustr::from("American Football"))
    .competition_id(12_282_733)
    .competition_name(Ustr::from("NFL"))
    .event_id(29_678_534)
    .event_name(Ustr::from("NFL"))
    .event_country_code(Ustr::from("GB"))
    .event_open_date(UnixNanos::from(event_open))
    .betting_type(Ustr::from("ODDS"))
    .market_id(Ustr::from("1-123456789"))
    .market_name(Ustr::from("AFC Conference Winner"))
    .market_type(Ustr::from("SPECIAL"))
    .market_start_time(UnixNanos::from(market_start))
    .selection_id(50214)
    .selection_name(Ustr::from("Kansas City Chiefs"))
    .selection_handicap(0.0)
    .currency(Currency::from("GBP"))
    .price_precision(2)
    .size_precision(2)
    .price_increment(Price::from("0.01"))
    .size_increment(Quantity::from("0.01"))
    .max_quantity(Quantity::from("1000"))
    .min_quantity(Quantity::from("1"))
    .max_notional(Money::from("10000 GBP"))
    .min_notional(Money::from("10 GBP"))
    .max_price(Price::from("100.00"))
    .min_price(Price::from("1.00"))
    .margin_init(dec!(1))
    .margin_maint(dec!(1))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapter [#adapters]

adapter [Betfair](../../integrations/betfair.md) สร้างหรือใช้ instrument แบบ `BettingInstrument`
สำหรับตลาดเดิมพันกีฬา

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Accounting](../accounting.md) อธิบายพฤติกรรมของ betting account
- [Data](../data/) อธิบายข้อมูลตลาดที่อ้างอิงถึง instrument
