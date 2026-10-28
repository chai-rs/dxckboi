---
title: "ข้อมูล: bar, precision และ timestamp"
description: "สิ่งที่ต้องถูกต้องก่อนข้อมูลจะเข้า engine และข้อผิดพลาดที่เจอจริงเรื่อง price type"
---

strategy ที่ดีแค่ไหนก็ให้ผลผิดถ้าข้อมูลผิด หน้านี้รวบรวมสิ่งที่ engine ตรวจ และสิ่งที่ engine **ไม่ได้** ตรวจให้

## Bar type [#bar-type]

`BarType` ระบุว่า bar นี้เป็นของ instrument ไหน รวบรวมด้วยช่วงอะไร ใช้ราคาแบบไหน และใครเป็นคนสร้าง
รูปแบบ string คือ:

```text
{instrument_id}-{step}-{aggregation}-{price_type}-{source}
AUD/USD.SIM-1-MINUTE-LAST-EXTERNAL
BTCUSDT-LINEAR.BYBIT-1-MINUTE-LAST-EXTERNAL
```

parser ตัด string จากด้านขวา ดังนั้น `/` หรือ `-` ภายใน instrument ID เช่น `AUD/USD` หรือ `BTCUSDT-LINEAR` ไม่เป็นปัญหา
สร้างได้สองแบบ:

```rust
let bar_type = BarType::from("AUD/USD.SIM-1-MINUTE-LAST-EXTERNAL"); // panic ถ้า string ผิด
let bar_type: BarType = "AUD/USD.SIM-1-MINUTE-LAST-EXTERNAL".parse()?; // คืน error แทน
```

ใช้ `BarType::from` กับค่าคงที่ที่เขียนในโค้ด และใช้ `parse()` กับค่าที่มาจาก config หรือ input ภายนอก

ส่วน `source` มีสองค่า:

- **`EXTERNAL`**: bar ที่มาจากภายนอก เช่น kline ของ exchange หรือไฟล์ข้อมูล
- **`INTERNAL`**: bar ที่ Nautilus รวบรวมเองจาก tick ภายใน engine

`BacktestEngine::add_data` ที่เปิด `validate` จะยอมรับเฉพาะ bar แบบ `EXTERNAL` และ instrument ของ bar ต้องถูก `add_instrument`
ไว้ก่อนแล้ว ไม่อย่างนั้นจะได้ error ตั้งแต่ขั้นเพิ่มข้อมูล ไม่ใช่ตอนรัน

## Price type มีผลกับ risk engine [#price-type]

นี่คือปัญหาแรกที่เจอจริงตอนเขียนบทนี้ โค้ดเวอร์ชันแรกใช้ bar แบบ `MID`:

```text
AUD/USD.SIM-1-MINUTE-MID-EXTERNAL
```

ผลคือ order **ทุกตัว** ถูกปฏิเสธ:

```text
[WARN] nautilus_risk::engine: SubmitOrder for O-20240101-003200-001-001-1 DENIED:
       MARKET_PRICE_UNAVAILABLE: order_type=MARKET, instrument_id=AUD/USD.SIM
```

สาเหตุ: ก่อนส่ง MARKET order ไปยัง venue risk engine ต้องรู้ราคาเพื่อคำนวณ notional และตรวจวงเงิน
มันหาราคาตามลำดับนี้:

1. bid/ask จาก quote ล่าสุดใน cache (bid สำหรับ sell, ask สำหรับ buy)
2. ราคา last trade ล่าสุด
3. ราคาปิดของ bar แบบ `EXTERNAL` ที่มี price type เป็น `BID`/`ASK` ตามฝั่งของ order หรือ `LAST`

bar แบบ `MID` ไม่อยู่ในรายการ เมื่อป้อนแต่ bar แบบ `MID` จึงไม่มีราคาให้ risk engine ใช้เลย

ทางแก้มีสามทาง:

- ใช้ bar แบบ `LAST` ซึ่งตรงกับ kline ของ exchange คริปโตส่วนใหญ่อยู่แล้ว (ทางที่โปรเจกต์นี้เลือก)
- ป้อน quote tick เข้า engine ด้วย นอกเหนือจาก bar
- ส่ง entry เป็น LIMIT order ซึ่ง risk engine ใช้ราคาของ order เองคำนวณ notional ได้

## Precision ของราคาและปริมาณ [#precision]

`Price` และ `Quantity` เก็บ precision (จำนวนตำแหน่งทศนิยม) ไว้ในตัว instrument ก็กำหนด precision ของตัวเอง
(`AUD/USD.SIM` ใช้ price precision 5 และ size precision 0) ข้อมูลที่ป้อนต้องใช้ precision เดียวกับ instrument

ในโค้ดของเรา precision ไม่ได้เขียนตายตัว แต่อ่านจาก instrument:

```rust
price_precision: instrument.price_precision(),
```

เมื่อต้องแปลงตัวเลขที่คำนวณเองเป็น `Price` หรือ `Quantity` สำหรับส่ง order ให้ใช้ method ของ instrument
ซึ่งปัดค่าให้ตรงกับ precision และ increment ของ instrument ให้เอง:

```rust
instrument.make_price(take_profit)          // ปัดตาม price increment
instrument.make_qty(units, Some(true))      // Some(true) = ปัดลง ไม่ให้เกินขนาดที่คำนวณไว้
```

## Bar ต้องถูกต้องตามโครงสร้าง [#bar-validity]

`Bar::new` จะ panic ถ้า high ต่ำกว่า open/low/close หรือ low สูงกว่า open/close ถ้าข้อมูลมาจากแหล่งที่ไม่แน่ใจ
ให้ใช้ `Bar::new_checked` ซึ่งคืน `anyhow::Result` แทน แล้วนับจำนวน bar ที่เสียไว้รายงาน แทนที่จะปล่อยให้ process ตาย

## Timestamp และ look-ahead bias [#timestamps]

ทุก event ใน Nautilus ใช้ `UnixNanos` (นาโนวินาทีนับจาก Unix epoch) และมีสองค่า:

- `ts_event`: เวลาที่เหตุการณ์เกิดขึ้นจริง
- `ts_init`: เวลาที่ Nautilus สร้าง object นั้น engine ใช้ค่านี้เรียงลำดับข้อมูล

bar หนึ่งแท่งครอบคลุมช่วงเวลา แต่ต้องประทับเวลาด้วยจุดเดียว ข้อมูลในบทนี้ประทับเวลาที่ **เวลาปิด** ของแท่ง:

```rust
// A bar is stamped at its close.
let ts = UnixNanos::from(self.start_ns + (i as u64 + 1) * self.interval_ns);
```

ถ้าประทับ bar 09:00–09:01 ด้วยเวลา 09:00 (เวลาเปิด) strategy จะได้เห็นราคาปิดของนาทีนั้นตั้งแต่ 09:00
ซึ่งเร็วกว่าความเป็นจริงหนึ่งนาที นี่คือ look-ahead bias ที่ทำให้ backtest ดูดีเกินจริงโดยไม่มี error ใดเตือน
ก่อนป้อนข้อมูลจากแหล่งใหม่ ให้ตรวจเสมอว่าแหล่งนั้นประทับเวลาด้วยเวลาเปิดหรือเวลาปิด

## ข้อมูลสังเคราะห์สำหรับพัฒนาและ test [#synthetic-data]

ระหว่างพัฒนา การมีข้อมูลที่สร้างซ้ำได้ทุกครั้งสำคัญกว่าการมีข้อมูลจริง เพราะ:

- test ให้ผลเหมือนเดิมทุกครั้ง จึงจับได้ทันทีเมื่อพฤติกรรมเปลี่ยน
- ไม่ต้องดาวน์โหลดหรือเก็บไฟล์ข้อมูลไว้ใน repository
- ควบคุมคุณสมบัติของตลาดได้ เช่น สร้างตลาดที่มี mean reversion และตลาดที่ไม่มี เพื่อเทียบกัน

`src/data.rs` สร้างราคาด้วย random walk ที่ถูกดึงกลับหา "anchor" (กระบวนการแบบ Ornstein–Uhlenbeck)
และให้ anchor เองเคลื่อนที่แบบสุ่มด้วย เพื่อให้มีช่วงที่ราคาวิ่งเป็น trend ด้วย:

```rust
for i in 0..self.count {
    anchor += self.anchor_drift * rng.normal();
    let open = px;
    let (mut high, mut low) = (px, px);
    for _ in 0..STEPS_PER_BAR {
        px += self.reversion * (anchor - px) + self.volatility * rng.normal();
        high = high.max(px);
        low = low.min(px);
    }
    // ... สร้าง Bar จาก open/high/low/close ...
}
```

จุดที่ควรสังเกต:

- ใช้ตัวสร้างเลขสุ่ม xorshift ที่เขียนเอง แทน crate `rand` เพื่อให้ลำดับเลขสุ่มเหมือนเดิมทุกเวอร์ชันและทุกแพลตฟอร์ม
  (crate สุ่มทั่วไปไม่รับประกันว่าลำดับเลขจะเหมือนเดิมข้ามเวอร์ชัน)
- `reversion = 0.0` ทำให้ได้ random walk ล้วน ซึ่งเป็นตลาดที่ strategy ของเรา **ไม่ควร** ทำกำไรได้
  หน้า [Backtest](backtest.md#control-market) ใช้ตลาดนี้เป็นกลุ่มควบคุม
- high และ low คำนวณจากเส้นทางราคาภายในแท่ง ไม่ได้สุ่มแยก จึงสอดคล้องกับ open และ close เสมอ

## ใช้ข้อมูลจริง [#real-data]

เมื่อ logic นิ่งแล้ว ขั้นต่อไปคือ backtest กับข้อมูลจริง ซึ่งควรเก็บใน Parquet catalog แล้วรันด้วย `BacktestNode`
ดูรายละเอียดได้ที่:

- [Data catalog](../concepts/data/catalog.md)
- [Stream Parquet catalog](../how_to/stream_parquet_catalog.md)
- [รัน backtest ด้วย Rust: BacktestNode](../how_to/run_rust_backtest.md#backtestnode-high-level-api)

ข้อมูลจริงทำให้เจอปัญหาที่ข้อมูลสังเคราะห์ไม่มี เช่น ช่วงตลาดปิด, bar ที่หายไป, ราคากระโดด และค่าผิดปกติ
ทุกข้อควรมี test รองรับก่อนนำ strategy ไปรันกับเงินจริง
