# IndexPriceUpdate

`IndexPriceUpdate` แทนราคา index ภายนอกที่ตลาด derivatives ใช้ venue มักใช้ราคา index
ในการคำนวณ mark price, funding หรือมูลค่า settlement

## Field {#fields}

| Field           | Rust type      | จำเป็น/ค่าเริ่มต้น | หมายเหตุ                                 |
| --------------- | -------------- | ------------------ | ---------------------------------------- |
| `instrument_id` | `InstrumentId` | จำเป็น             | instrument ของราคา index                 |
| `value`         | `Price`        | จำเป็น             | ราคา index ปัจจุบัน                      |
| `ts_event`      | `UnixNanos`    | จำเป็น             | timestamp ของ event หน่วยนาโนวินาที      |
| `ts_init`       | `UnixNanos`    | จำเป็น             | timestamp ตอน initialize หน่วยนาโนวินาที |

## พฤติกรรม {#behavior}

- ราคา index จะถูก cache แยกตาม instrument เมื่อได้รับ
- ราคา index เป็นข้อมูลอ้างอิง ไม่ได้หมายความว่ามี trade เกิดขึ้น
- venue ที่เป็น perpetual และ futures อาจเผยแพร่ทั้ง mark price และ index price
- catalog เก็บราคา index พร้อม metadata ของ instrument ID และ price precision

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::IndexPriceUpdate,
    identifiers::InstrumentId,
    types::Price,
};

let index = IndexPriceUpdate::new(
    InstrumentId::from("BTCUSDT-PERP.BINANCE"),
    Price::from("64995.50"),
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);
```

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [MarkPriceUpdate](mark_price_update.md) อธิบายเรื่อง mark price
- [FundingRateUpdate](funding_rate_update.md) อธิบาย metadata ของ perpetual funding
- [Rust API reference](https://nautilustrader.io/docs/core-latest/) แสดงรายละเอียดของ type ฝั่ง Rust
