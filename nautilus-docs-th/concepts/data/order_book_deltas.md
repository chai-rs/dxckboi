---
title: "OrderBookDeltas"
---

`OrderBookDeltas` รวบรวม `OrderBookDelta` หลายรายการ (ไม่ว่าง) ที่มาจาก book event เชิงตรรกะเดียวกันไว้เป็น batch
ช่วยลด overhead ต่อ message เมื่อ adapter ได้รับหรือสร้างการเปลี่ยนแปลงหลายรายการพร้อมกัน

## Field [#fields]

| Field           | Rust type             | Required/default | หมายเหตุ                                 |
| --------------- | --------------------- | ---------------- | --------------------------------------- |
| `instrument_id` | `InstrumentId`        | Required         | Instrument ที่ book กำลังเปลี่ยนแปลง      |
| `deltas`        | `Vec<OrderBookDelta>` | Required         | Batch ของ delta (ต้องไม่ว่าง)             |
| `flags`         | `u8`                  | From last delta  | Flag ของ delta ตัวสุดท้าย                 |
| `sequence`      | `u64`                 | From last delta  | Sequence number ของ delta ตัวสุดท้าย      |
| `ts_event`      | `UnixNanos`           | From last delta  | Event timestamp ของ delta ตัวสุดท้าย      |
| `ts_init`       | `UnixNanos`           | From last delta  | Initialization timestamp ของ delta ตัวสุดท้าย |

## พฤติกรรม [#behavior]

- Batch ต้องมี delta อย่างน้อยหนึ่งรายการ
- `instrument_id` ของ delta แต่ละตัวต้องตรงกับ `instrument_id` ของ batch
- Metadata ของ batch จะสะท้อนค่าจาก delta ตัวสุดท้าย
- Delta ตัวสุดท้ายควรมี `F_LAST` เมื่อเป็นตัวปิดกลุ่ม event เชิงตรรกะ ดู
  [Delta flags and event boundaries](index.md#delta-flags-and-event-boundaries)
- Batch แบบ snapshot มักเริ่มด้วย delta `CLEAR` และจบด้วย `F_SNAPSHOT | F_LAST`

## ตัวอย่าง [#example]

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::{BookOrder, OrderBookDelta, OrderBookDeltas},
    enums::{BookAction, OrderSide, RecordFlag},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};

let instrument_id = InstrumentId::from("ETHUSDT-PERP.BINANCE");
let bid = OrderBookDelta::new(
    instrument_id,
    BookAction::Add,
    BookOrder::new(OrderSide::Buy, Price::from("2500.10"), Quantity::from("3.5"), 1),
    0,
    41,
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);
let ask = OrderBookDelta::new(
    instrument_id,
    BookAction::Add,
    BookOrder::new(OrderSide::Sell, Price::from("2500.20"), Quantity::from("2.0"), 2),
    RecordFlag::F_LAST as u8,
    42,
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);

let deltas = OrderBookDeltas::new(instrument_id, vec![bid, ask]);
```

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [OrderBookDelta](order_book_delta.md) อธิบายประเภทของ update ที่อยู่ภายใน batch
- [Order books](../order_book.md) อธิบายสถานะของ order book ที่รองรับ
