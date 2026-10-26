# MarkPriceUpdate

`MarkPriceUpdate` แทน mark price ของ instrument โดย venue ประเภท derivatives มักใช้ mark price ในการคำนวณ
margin, ตรวจสอบการ liquidation และคำนวณ unrealized PnL

## Field {#fields}

| Field           | Rust type      | Required/default | หมายเหตุ                                  |
| --------------- | -------------- | ---------------- | ---------------------------------------- |
| `instrument_id` | `InstrumentId` | Required         | Instrument ของ mark price นี้             |
| `value`         | `Price`        | Required         | Mark price ปัจจุบัน                        |
| `ts_event`      | `UnixNanos`    | Required         | Timestamp ของ event หน่วย nanosecond      |
| `ts_init`       | `UnixNanos`    | Required         | Timestamp ตอน initialize หน่วย nanosecond |

## พฤติกรรม {#behavior}

- Mark price จะถูก cache แยกตาม instrument เมื่อได้รับเข้ามา
- Backtest สามารถป้อน mark price เข้าไปเพื่อให้พฤติกรรมของ margin และ PnL สอดคล้องกับ venue ที่ publish
  ราคาอ้างอิงแยกจาก trade
- Catalog จัดเก็บ mark price พร้อม metadata ของ instrument ID และ price precision

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::MarkPriceUpdate,
    identifiers::InstrumentId,
    types::Price,
};

let mark = MarkPriceUpdate::new(
    InstrumentId::from("BTCUSDT-PERP.BINANCE"),
    Price::from("65000.10"),
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);
```

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [IndexPriceUpdate](index_price_update.md) อธิบายเรื่อง index reference price
- [FundingRateUpdate](funding_rate_update.md) อธิบายเรื่อง metadata ของ funding สำหรับ perpetual
