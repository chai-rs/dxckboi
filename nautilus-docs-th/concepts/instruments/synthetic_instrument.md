# Synthetic Instrument

`SyntheticInstrument` แทน instrument ภายในระบบที่ราคามาจากสูตรคำนวณบน instrument อื่น เหมาะสำหรับ spread,
basket, ratio และราคาที่ derive มาแบบอื่น ๆ ที่ต้องการให้ปรากฏในระบบเป็น instrument

ตัวอย่างเช่น `(BTC.BINANCE + LTC.BINANCE) / 2.0` และคู่แบบ ratio ที่สร้างจากราคาของ instrument ที่เป็นองค์ประกอบ

## Fields {#fields}

| Field             | Rust type           | จำเป็น/ค่าเริ่มต้น  | หมายเหตุ                                        |
| ----------------- | ------------------- | ---------------- | ---------------------------------------------- |
| `symbol`          | `Symbol`            | จำเป็น            | Synthetic symbol ที่ใช้คู่กับ venue `SYNTH`        |
| `id`              | `InstrumentId`      | Derive ให้        | Instrument ID ที่สร้างจาก `symbol.SYNTH`         |
| `price_precision` | `u8`                | จำเป็น            | จำนวนทศนิยมที่อนุญาตสำหรับราคา synthetic           |
| `price_increment` | `Price`             | Derive ให้        | ขั้นราคาที่เล็กที่สุดซึ่งคำนวณจาก precision           |
| `components`      | `Vec<InstrumentId>` | จำเป็น            | Instrument องค์ประกอบที่สูตรใช้                    |
| `formula`         | `String`            | จำเป็น            | Expression เชิงตัวเลขบน ID ของ component          |
| `ts_event`        | `UnixNanos`         | จำเป็น            | Timestamp ของ event หน่วย nanosecond             |
| `ts_init`         | `UnixNanos`         | จำเป็น            | Timestamp ตอน initialize หน่วย nanosecond        |

**หมายเหตุ**: instrument ID สร้างจาก `symbol` และ venue `SYNTH` โดย Rust เก็บค่านี้ไว้ใน field `id`

## พฤติกรรม {#behavior}

- `SyntheticInstrument` เป็นของภายใน Nautilus และไม่ได้แทนตลาดที่ส่ง order ไปยัง venue ได้
- ใช้ synthetic venue `SYNTH` เสมอ
- สูตรต้อง compile ได้กับ identifier ของ component ที่ให้มา object จึงจะถือว่าถูกต้อง
- ไม่มี limit ของ venue, margin, fee, order book หรือ metadata เฉพาะของ adapter

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Symbol},
    instruments::SyntheticInstrument,
};

let synthetic = SyntheticInstrument::builder()
    .symbol(Symbol::from("BTC-LTC"))
    .price_precision(2)
    .components(vec![
        InstrumentId::from("BTC.BINANCE"),
        InstrumentId::from("LTC.BINANCE"),
    ])
    .formula("(BTC.BINANCE + LTC.BINANCE) / 2.0")
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapters {#adapters}

`SyntheticInstrument` ใช้ภายในระบบเท่านั้น โดย derive ราคาจาก instrument องค์ประกอบ ซึ่งอาจมาจาก adapter
ใดก็ได้ที่โหลดเข้าระบบแล้ว

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Synthetics](../synthetics.md) ครอบคลุม instrument ที่ derive จากสูตรและ synthetic bar
- [Data](../data/) อธิบายข้อมูลตลาดที่อ้างอิงถึง instrument
