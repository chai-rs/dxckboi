# QuoteTick

`QuoteTick` แทน bid และ ask ระดับ top-of-book ของ instrument หนึ่งตัว คือราคาและขนาดที่ดีที่สุด
ที่มีอยู่ ณ เวลาของ event นั้น

## Fields {#fields}

| Field           | Rust type      | Required/default | หมายเหตุ                                  |
| --------------- | -------------- | ---------------- | ----------------------------------------- |
| `instrument_id` | `InstrumentId` | Required         | instrument ของ quote                       |
| `bid_price`     | `Price`        | Required         | ราคา bid ที่ดีที่สุด                          |
| `ask_price`     | `Price`        | Required         | ราคา ask ที่ดีที่สุด                          |
| `bid_size`      | `Quantity`     | Required         | ปริมาณที่มีอยู่ที่ bid ที่ดีที่สุด               |
| `ask_size`      | `Quantity`     | Required         | ปริมาณที่มีอยู่ที่ ask ที่ดีที่สุด               |
| `ts_event`      | `UnixNanos`    | Required         | timestamp ของ event หน่วยนาโนวินาที          |
| `ts_init`       | `UnixNanos`    | Required         | timestamp ตอน initialize หน่วยนาโนวินาที     |

## พฤติกรรม {#behavior}

- ราคา bid และ ask ต้องใช้ precision เดียวกัน
- ขนาด bid และ ask ต้องใช้ precision เดียวกัน
- `extract_price(...)` คืนราคาแบบ `BID`, `ASK` หรือ `MID` ส่วน `extract_size(...)` คืนขนาดที่สอดคล้องกัน
  price type อื่นนอกจากนี้ถือเป็น error
- ผลลัพธ์แบบ `MID` มี precision เพิ่มขึ้นหนึ่งหลัก โดยไม่เกิน `FIXED_PRECISION`
- quote bar ใช้ price type แบบ `BID`, `ASK` หรือ `MID` ได้

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::QuoteTick,
    identifiers::InstrumentId,
    types::{Price, Quantity},
};

let quote = QuoteTick::new(
    InstrumentId::from("AUD/USD.SIM"),
    Price::from("0.65000"),
    Price::from("0.65002"),
    Quantity::from("1000000"),
    Quantity::from("1200000"),
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);
```

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [OrderBookDepth](order_book_depth.md) อธิบาย snapshot แบบความลึกแปรผันพร้อมระดับราคาบนสุด
- [Bars และการ aggregate](index.md#bars-and-aggregation) อธิบายการ aggregate quote เป็น bar
- [Rust API reference](https://nautilustrader.io/docs/core-latest/) รวมรายละเอียด API ฝั่ง Rust
