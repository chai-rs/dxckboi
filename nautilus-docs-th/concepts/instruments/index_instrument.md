# Index Instrument

`IndexInstrument` แทน index อ้างอิง เช่น ดัชนีหุ้น, ดัชนีความผันผวน หรือชุดราคา benchmark
มันมี metadata ของ precision และ increment เพื่อให้ Nautilus จัดเก็บและ route ราคาได้อย่างสม่ำเสมอ
แต่ไม่ใช่สัญญาที่ซื้อขายได้โดยตรง

ตัวอย่างเช่น `SPX.XCBO`, `VIX.XCBO` และ index อ้างอิงเฉพาะของแต่ละ venue

## Field {#fields}

| Field             | Rust type        | จำเป็น/ค่าเริ่มต้น | หมายเหตุ                                 |
| ----------------- | ---------------- | ------------------ | ---------------------------------------- |
| `instrument_id`   | `InstrumentId`   | จำเป็น             | ใน Rust เก็บเป็น `id`                    |
| `raw_symbol`      | `Symbol`         | จำเป็น             | symbol ดั้งเดิมของ venue                 |
| `currency`        | `Currency`       | จำเป็น             | สกุลเงินอ้างอิงของค่าที่ quote            |
| `price_precision` | `u8`             | จำเป็น             | จำนวนทศนิยมที่อนุญาตสำหรับราคา           |
| `size_precision`  | `u8`             | จำเป็น             | จำนวนทศนิยมที่อนุญาตสำหรับ quantity      |
| `price_increment` | `Price`          | จำเป็น             | ขั้นราคาที่เล็กที่สุดที่ใช้ได้            |
| `size_increment`  | `Quantity`       | จำเป็น             | ขั้นขนาดที่เล็กที่สุดที่ใช้ได้            |
| `ts_event`        | `UnixNanos`      | จำเป็น             | timestamp ของ event หน่วยนาโนวินาที      |
| `ts_init`         | `UnixNanos`      | จำเป็น             | timestamp ตอน initialize หน่วยนาโนวินาที |
| `tick_scheme`     | `Option<Ustr>`   | `None`             | ชื่อ variable tick scheme ที่ลงทะเบียนไว้  |
| `info`            | `Option<Params>` | `None`             | metadata ของ adapter                     |

## พฤติกรรม {#behavior}

- `IndexInstrument` มี asset class เป็น `Index` และ instrument class เป็น `Spot`
- ไม่มี limit, margin, ค่าธรรมเนียม, contract multiplier, วันหมดอายุ หรือ settlement currency
- สำหรับ derivatives ที่ซื้อขายได้และมี underlying เป็น index ให้ใช้ type ของ option หรือ futures

:::warning
`IndexInstrument` เป็น instrument อ้างอิง ไม่ใช่สัญญาที่ซื้อขายได้ อย่าส่ง order ไปที่มัน ให้เทรด
instrument futures หรือ option ที่สอดคล้องกันแทน
:::

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Symbol},
    instruments::IndexInstrument,
    types::{Currency, Price, Quantity},
};

let spx = IndexInstrument::builder()
    .instrument_id(InstrumentId::from("SPX.XCBO"))
    .raw_symbol(Symbol::from("SPX"))
    .currency(Currency::from("USD"))
    .price_precision(2)
    .size_precision(0)
    .price_increment(Price::from("0.01"))
    .size_increment(Quantity::from("1"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapter {#adapters}

Adapter [Interactive Brokers](../../integrations/interactive_brokers.md) สร้าง definition ของ
`IndexInstrument` สำหรับ index อ้างอิง

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Option Contract](option_contract.md) อธิบาย option ที่จดทะเบียนซื้อขายซึ่งมี underlying เป็น index
- [Futures Contract](futures_contract.md) อธิบาย index futures
