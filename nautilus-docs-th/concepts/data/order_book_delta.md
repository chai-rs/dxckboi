---
title: "OrderBookDelta"
---

`OrderBookDelta` แทนการเปลี่ยนแปลงหนึ่งครั้งของ order book เป็นชนิดข้อมูล book ที่ละเอียดที่สุดในบรรดาที่มี
มาให้ในตัว และรองรับ book type ที่ NautilusTrader ใช้กับ incremental update:

- `L3_MBO`: ข้อมูล Level 3 market-by-order (MBO)
- `L2_MBP`: ข้อมูล Level 2 market-by-price (MBP)
- `L1_MBP`: ข้อมูล Level 1 market-by-price (MBP) เฉพาะ top-of-book

feed ต้นทางและ `BookType` เป้าหมายเป็นตัวกำหนดว่า delta หนึ่งตัวมีความละเอียดระดับใด

ใช้ชนิดนี้เมื่อ venue หรือผู้ให้บริการข้อมูล publish การเปลี่ยนแปลงของ book แบบ incremental และต้องการให้
NautilusTrader ดูแล state ของ book ไว้ในเครื่อง

## ฟิลด์ [#fields]

| ฟิลด์            | Rust type      | Required/default | หมายเหตุ                                        |
| --------------- | -------------- | ---------------- | ---------------------------------------------- |
| `instrument_id` | `InstrumentId` | Required         | instrument ที่ book กำลังเปลี่ยน                   |
| `action`        | `BookAction`   | Required         | `ADD`, `UPDATE`, `DELETE` หรือ `CLEAR`          |
| `order`         | `BookOrder`    | Required         | payload ที่มี price, size, side และ order ID      |
| `flags`         | `u8`           | Required         | bit field ของ `RecordFlag` สำหรับ metadata ของ event |
| `sequence`      | `u64`          | Required         | sequence number จาก venue หรือศูนย์ถ้าไม่มี         |
| `ts_event`      | `UnixNanos`    | Required         | timestamp ของ event หน่วยนาโนวินาที               |
| `ts_init`       | `UnixNanos`    | Required         | timestamp ตอน initialize หน่วยนาโนวินาที           |

## ฟิลด์ของ BookOrder [#bookorder-fields]

ฟิลด์ `order` เก็บ payload `BookOrder` ของ delta

| ฟิลด์       | Rust type           | หมายเหตุ                              |
| ---------- | ------------------- | ------------------------------------ |
| `side`     | `Option<OrderSide>` | ฝั่งของ order                          |
| `price`    | `Price`             | ราคาของ order                         |
| `size`     | `Quantity`          | ขนาดของ order                         |
| `order_id` | `OrderId` (`u64`)   | order ID ที่มากับ feed ต้นทาง            |

order แบบ null/default ใช้ `None` เป็น side พร้อม price, size และ order ID เป็นศูนย์

## variant ของ BookAction [#bookaction-variants]

| Rust variant         | Value | ความหมาย                                |
| -------------------- | ----- | -------------------------------------- |
| `BookAction::Add`    | `1`   | เพิ่ม order เข้า book                     |
| `BookAction::Update` | `2`   | อัปเดต order ที่มีอยู่แล้วใน book            |
| `BookAction::Delete` | `3`   | ลบ order ที่มีอยู่แล้วใน book               |
| `BookAction::Clear`  | `4`   | ล้าง state ของ order book                |

## พฤติกรรม [#behavior]

- delta แบบ `ADD` และ `UPDATE` ต้องมี order size เป็นบวก
- delta แบบ `CLEAR` จะ reset state ของ book และใช้ book order แบบ null
- `flags` เก็บ metadata ของขอบเขต event และ snapshot ดู
  [Delta flags และขอบเขตของ event](index.md#delta-flags-and-event-boundaries)
- ใช้ `OrderBookDelta::clear(...)` เพื่อสร้าง clear delta

## ตัวอย่าง [#example]

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::{BookOrder, OrderBookDelta},
    enums::{BookAction, OrderSide, RecordFlag},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};

let delta = OrderBookDelta::new(
    InstrumentId::from("ETHUSDT-PERP.BINANCE"),
    BookAction::Add,
    BookOrder::new(
        OrderSide::Buy,
        Price::from("2500.10"),
        Quantity::from("3.5"),
        12_345,
    ),
    RecordFlag::F_LAST as u8,
    42,
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);
```

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [OrderBookDeltas](order_book_deltas.md) อธิบายการรวม delta เป็นชุด (batching)
- [Order books](../order_book.md) อธิบาย book type และ state ของ book ในเครื่อง
- [Rust API reference](https://nautilustrader.io/docs/core-latest/) รวมรายละเอียด API ฝั่ง Rust
