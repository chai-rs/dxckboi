---
title: "OrderBookDepth"
---

`OrderBookDepth` แทน snapshot ที่มีจำนวนระดับราคา bid และ ask แปรผันได้
ใช้เมื่อ venue ส่ง depth snapshot ที่สมบูรณ์ในตัวเอง แทนที่จะเป็น delta แบบ incremental

## Fields [#fields]

| Field           | Rust type                   | Required/default | หมายเหตุ                                      |
| --------------- | --------------------------- | ---------------- | --------------------------------------------- |
| `instrument_id` | `InstrumentId`              | Required         | instrument ที่ book นี้แทน                     |
| `bids`          | `SmallVec<[BookOrder; 10]>` | Required         | ระดับ bid ตามลำดับใน book                      |
| `asks`          | `SmallVec<[BookOrder; 10]>` | Required         | ระดับ ask ตามลำดับใน book                      |
| `bid_counts`    | `SmallVec<[u32; 10]>`       | Required         | จำนวน bid order ในแต่ละระดับ                   |
| `ask_counts`    | `SmallVec<[u32; 10]>`       | Required         | จำนวน ask order ในแต่ละระดับ                   |
| `flags`         | `u8`                        | Required         | bit field ของ `RecordFlag` สำหรับ metadata ของ event |
| `sequence`      | `u64`                       | Required         | sequence number ของ venue หรือศูนย์หากไม่มี     |
| `ts_event`      | `UnixNanos`                 | Required         | timestamp ของ event หน่วยนาโนวินาที            |
| `ts_init`       | `UnixNanos`                 | Required         | timestamp ตอน initialize หน่วยนาโนวินาที        |

## พฤติกรรม [#behavior]

- constructor ใน Rust รับ side ที่มีความยาวแปรผันได้ แต่ละ side ต้องมี count หนึ่งค่าต่อหนึ่ง order
  และฝั่ง bid กับ ask มีความยาวต่างกันได้
- side ที่ว่างใช้ sequence ว่าง inline capacity คือสิบ snapshot ที่ใหญ่กว่านั้นจะ allocate เพิ่มตามต้องการ
- type นี้ใช้แทนกันกับ stream ของ `OrderBookDelta` แบบ incremental ไม่ได้

**ความเข้ากันได้กับของเดิม**:

- ชื่อ type เดิม `OrderBookDepth10` ถูกยกเลิกแล้ว แต่ไดเรกทอรีใน catalog ที่ชื่อ `order_book_depth10`
  ยังคง migrate ได้ และไฟล์ Arrow ที่เก็บ type นั้นไว้ใน schema metadata ยังคง transcode ได้
- legacy C FFI และการเข้ารหัส SBE แบบ fixed-depth ต้องมีระดับราคาครบสิบระดับพอดีในแต่ละ side

## ตัวอย่าง [#example]

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::{BookOrder, OrderBookDepth},
    enums::OrderSide,
    identifiers::InstrumentId,
    types::{Price, Quantity},
};

let bids = vec![BookOrder::new(OrderSide::Buy, Price::from("2500.10"), Quantity::from("3.5"), 1)];
let asks = vec![BookOrder::new(OrderSide::Sell, Price::from("2500.20"), Quantity::from("2.0"), 2)];

let depth = OrderBookDepth::new(
    InstrumentId::from("ETHUSDT-PERP.BINANCE"),
    bids,
    asks,
    vec![1],
    vec![1],
    0,
    42,
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);
```

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [QuoteTick](quote_tick.md) อธิบายข้อมูล top-of-book ที่ได้จาก depth
- [Order books](index.md#order-books) อธิบายสถานะของ order book
- [Rust API docs](https://nautilustrader.io/docs/core-latest/) แสดงรายละเอียด API ฝั่ง Rust
