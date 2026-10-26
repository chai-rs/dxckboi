# Bar

`Bar` แทนข้อมูลราคาและ volume แบบ OHLCV ของ `BarType` หนึ่ง ๆ โดย venue หรือผู้ให้บริการข้อมูล
อาจส่ง bar มาให้โดยตรง หรือ NautilusTrader จะ aggregate ขึ้นมาเองจาก quote tick, trade tick
หรือ bar ที่มี timeframe เล็กกว่าก็ได้

## ฟิลด์ {#fields}

| Field      | Rust type   | Required/default | หมายเหตุ                                          |
| ---------- | ----------- | ---------------- | ------------------------------------------------ |
| `bar_type` | `BarType`   | Required         | instrument, วิธี aggregation, price type และแหล่งที่มา |
| `open`     | `Price`     | Required         | ราคาแรกในช่วงเวลาของ bar                          |
| `high`     | `Price`     | Required         | ราคาสูงสุดในช่วงเวลาของ bar                        |
| `low`      | `Price`     | Required         | ราคาต่ำสุดในช่วงเวลาของ bar                        |
| `close`    | `Price`     | Required         | ราคาสุดท้ายในช่วงเวลาของ bar                       |
| `volume`   | `Quantity`  | Required         | volume ที่ซื้อขาย หรือค่าประมาณจากจำนวน tick (tick volume) |
| `ts_event` | `UnixNanos` | Required         | timestamp ของ event ของ bar หน่วย nanosecond       |
| `ts_init`  | `UnixNanos` | Required         | timestamp ตอน initialize หน่วย nanosecond          |

## พฤติกรรม {#behavior}

- `high` ต้องมากกว่าหรือเท่ากับ `open`, `low` และ `close`
- `low` ต้องน้อยกว่าหรือเท่ากับ `open` และ `close`
- `bar_type` เป็นตัวกำหนดว่า bar นั้นเป็น internal หรือ external
- bar type แบบ composite ใช้ไวยากรณ์ `@` เพื่อระบุ bar type ต้นทาง

:::warning[Timestamp ของ bar สำหรับการจำลองการ execution]
ในการจำลองการ execution ค่า `ts_init` ต้องแทนเวลาปิดของช่วง bar เพื่อป้องกันไม่ให้ bar ที่สมบูรณ์
มองเห็นได้ก่อนที่มันจะก่อตัวเสร็จจริง ดู
[ข้อตกลงเรื่อง timestamp ของ bar](../backtesting/bar-execution.md#bar-timestamp-convention)
:::

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    data::{Bar, BarType},
    types::{Price, Quantity},
};

let bar = Bar::new(
    BarType::from("AUD/USD.SIM-1-MINUTE-LAST-EXTERNAL"),
    Price::from("0.65000"),
    Price::from("0.65010"),
    Price::from("0.64990"),
    Price::from("0.65005"),
    Quantity::from("1000000"),
    UnixNanos::from(1_000_000_000),
    UnixNanos::from(1_000_000_100),
);
```

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Bar และการ aggregation](index.md#bars-and-aggregation) อธิบายวิธี aggregation แบบต่าง ๆ
- [Bar type](index.md#bar-types) อธิบายไวยากรณ์ string ของ `BarType`
- [Rust API reference](https://nautilustrader.io/docs/core-latest/) รวมรายละเอียดของ type ฝั่ง Rust
