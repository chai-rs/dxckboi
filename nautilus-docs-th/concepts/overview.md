---
title: "ภาพรวม"
---

## บทนำ [#introduction]

NautilusTrader เป็น engine แบบ open-source ระดับ production ที่เขียนด้วย Rust เป็นหลัก (Rust-native)
สำหรับระบบเทรดหลายประเภทสินทรัพย์และหลาย venue

ระบบครอบคลุมทั้งงานวิจัย การจำลองแบบ deterministic และ live execution ภายในสถาปัตยกรรมแบบ event-driven
เดียวกัน ระบบเทรดสามารถเขียนด้วย Rust ทั้งหมดได้สำหรับงานที่ต้องการความเสถียรสูง (mission-critical)

โค้ดของ strategy และ execution algorithm ชุดเดียวกันรันได้ทั้งในระบบ backtest และ live ช่วยลดความต่าง
ตอน deploy อย่างไรก็ตาม live execution ยังมีพฤติกรรมด้าน venue, transport, จังหวะเวลา, persistence,
กิจกรรมจากภายนอก และ reconciliation ที่การจำลองอาจทำซ้ำไม่ได้ ดู
[ความแตกต่างระหว่าง backtest และ live](live.md#backtest-and-live-differences)

NautilusTrader ไม่ผูกกับประเภทสินทรัพย์ใดๆ venue ใดก็ตามที่มี REST API หรือ WebSocket feed สามารถ
integrate ผ่าน adapter แบบ modular ได้ integration ที่มีครอบคลุมทั้ง crypto exchange แบบรวมศูนย์และกระจายศูนย์
(CEX และ DEX), ตลาดแลกเปลี่ยนเงินตรา (FX), หุ้น, futures, options และ betting exchange

## คุณสมบัติ [#features]

- **เร็ว**: แกนหลักเป็น Rust พร้อม networking แบบ asynchronous ด้วย [tokio](https://crates.io/crates/tokio)
- **เชื่อถือได้**: Rust ให้ทั้ง type safety และ thread safety และรองรับการเก็บ state แบบ persistence
  (ไม่บังคับ) ด้วย Redis หรือ PostgreSQL
- **พกพาได้**: รันได้บน Linux, macOS และ Windows และ deploy ด้วย Docker ได้
- **ยืดหยุ่น**: adapter แบบ modular integrate กับ REST API หรือ WebSocket feed ใดก็ได้
- **Order ขั้นสูง**: ตัวเลือก time-in-force มี `IOC`, `FOK`, `GTC`, `GTD`, `DAY`,
  `AT_THE_OPEN` และ `AT_THE_CLOSE` นอกจากนี้ domain model ยังรองรับ conditional trigger,
  `post-only`, `reduce-only`, iceberg และ contingency order แบบ `OCO`, `OUO` และ `OTO` ทั้งนี้การรองรับ
  ของ venue ขึ้นกับแต่ละ adapter
- **ปรับแต่งได้**: สร้าง component ของตัวเองได้ หรือประกอบทั้งระบบขึ้นใหม่ตั้งแต่ต้นโดยใช้
  [cache](cache.md) และ [message bus](message_bus.md)
- **Backtesting**: รันหลาย venue, instrument และ strategy พร้อมกันด้วยข้อมูลย้อนหลังทั้ง quote, trade, bar,
  order book และ custom data ที่ความละเอียดระดับ nanosecond
- **Live**: ใช้ implementation ของ strategy ชุดเดียวกันทั้งตอนวิจัยและตอน deploy แบบ live
- **หลาย venue**: รัน strategy แบบ market-making และแบบข้าม venue บนหลาย venue พร้อมกันได้
- **AI training**: การจำลองที่มี throughput สูงรองรับงานอย่างการฝึก AI trading agent ด้วย
  reinforcement learning (RL) หรือ evolutionary strategies (ES)

## ทำไมต้อง NautilusTrader? [#why-nautilustrader]

โดยทั่วไปงานวิจัย strategy การเทรดมักทำด้วย Python แบบ vectorized ขณะที่ระบบเทรดใน production
ถูกสร้างแยกต่างหากด้วยสถาปัตยกรรมแบบ event-driven ในภาษาที่ compile

NautilusTrader ขจัดการแยกส่วนนี้

แกนหลักที่เป็น Rust-native ให้ runtime แบบ event-driven ที่ deterministic ทั้งสำหรับงานวิจัยและ live
execution สถาปัตยกรรม ความหมายของการ execute และโมเดลเวลาชุดเดียวกันทำงานในทั้งสองสภาพแวดล้อม
ทำให้ย้าย strategy จากงานวิจัยไปสู่ production ได้โดยไม่ต้องเขียนใหม่

## กรณีการใช้งาน [#use-cases]

NautilusTrader รองรับกรณีการใช้งานหลักสามแบบ:

- Backtest ระบบเทรดด้วยข้อมูลย้อนหลัง (`backtest`)
- จำลองระบบเทรดด้วยข้อมูล real-time และการ execute แบบเสมือน (`sandbox`)
- Deploy ระบบเทรดแบบ live บน account จริงหรือ paper account (`live`)

NautilusTrader มี implementation ของ node สำหรับ backtest และ live ใน Rust ส่วน sandbox adapter
ให้การ execute แบบจำลองสำหรับสภาพแวดล้อม `sandbox`

:::note

- ตัวอย่างต่างๆ ใช้ implementation ของ node เหล่านี้ เว้นแต่จะระบุไว้เป็นอย่างอื่น
- strategy การเทรดเป็นเพียง component หนึ่งของระบบเทรดแบบ end-to-end ซึ่งยังรวมถึงชั้น application
  และ infrastructure ด้วย

:::

## ระบบแบบกระจาย [#distributed]

แพลตฟอร์มนี้ integrate เข้ากับระบบแบบกระจาย (distributed system) ที่ใหญ่กว่าได้:

- [external message bus](message_bus.md#encoding) รองรับ JSON และ MessagePack สำหรับ message แบบ native
  และรองรับ Simple Binary Encoding (SBE) สำหรับ market data ที่มี schema รองรับ
- Apache Arrow และ Parquet ใช้สำหรับการแลกเปลี่ยนข้อมูลแบบ columnar และการเก็บข้อมูลผ่าน
  [data catalog](data/catalog.md)

รูปแบบที่รองรับขึ้นกับประเภทของ payload

## แกนหลักร่วม [#common-core]

แกนหลักของระบบชุดเดียวกันถูกใช้โดย node ใน[บริบทสภาพแวดล้อม](architecture.md#environment-contexts)
ทุกแบบ ได้แก่ `backtest`, `sandbox` และ `live` actor, strategy และ execution algorithm ที่ผู้ใช้สร้างเองจะใช้
lifecycle เดียวกันในทุกบริบทเหล่านี้

## Backtesting [#backtesting]

ป้อนข้อมูลให้ `BacktestEngine` โดยตรง หรือผ่าน `BacktestNode` และ `ParquetDataCatalog` ที่เป็นระดับสูงกว่า
แล้วรันข้อมูลผ่านระบบที่ความละเอียดระดับ nanosecond ดู [Backtesting](backtesting/) สำหรับ API และ
โมเดลการ execute

## Live trading [#live-trading]

`LiveNode` รับข้อมูลและ event จาก data client และ execution client หลายตัว รองรับทั้ง account แบบ demo,
paper และ account จริง node แบบ Rust-native รัน event loop ของ kernel บน thread ที่เรียกใช้ ส่วน I/O
แบบ asynchronous และ background task ใช้ Tokio runtime แบบ multi-threaded ร่วมกัน

ดู [Live trading](live.md) สำหรับ lifecycle ของ node และข้อพิจารณาด้านความเสี่ยง และ
[Execution reconciliation](execution/reconciliation.md) สำหรับการกู้คืน state

## Domain model [#domain-model]

domain model ด้านการเทรดประกอบด้วย [value type](value_types.md) เช่น `Price` และ `Quantity`
รวมถึง [order](orders/) และ [position](positions.md) ที่รวบรวม event เพื่อกำหนด state

## Timestamp [#timestamps]

NautilusTrader แทน timestamp ของระบบเป็น **UNIX nanoseconds** formatter มาตรฐานแบบ ISO 8601
(RFC 3339) ใช้ UTC และเก็บทศนิยมครบทั้งเก้าหลัก ส่วน formatter แบบ millisecond เก็บทศนิยมสามหลัก
สำหรับการแสดงผลบางอย่าง เช่น เวลาหมดอายุของ good-till-date (GTD)

string ของ timestamp ประกอบด้วย:

- ส่วนวันที่เต็มรูปแบบซึ่งมีเสมอ: `YYYY-MM-DD`
- ตัวคั่น `T` ระหว่างส่วนวันที่และเวลา
- ทศนิยมเก้าหลักสำหรับผลลัพธ์แบบ nanosecond หรือสามหลักสำหรับแบบ millisecond
- timezone UTC ที่ระบุด้วยตัวท้าย `Z`

ตัวอย่าง: `2024-01-05T15:30:45.123456789Z`

ดูข้อกำหนดฉบับเต็มได้ที่
[RFC 3339: Date and Time on the Internet](https://datatracker.ietf.org/doc/html/rfc3339)

## UUID [#uuids]

value type `UUID4` ให้ค่า Universally Unique Identifier (UUID) เวอร์ชัน 4 แบบสุ่ม สำหรับ event, command,
report และ message ภายในอื่นๆ โดยใช้ crate `uuid` ตรวจสอบบิตของ version และ variant ตอน parse string

UUID v4 ที่ถูกต้องตาม RFC 9562 ประกอบด้วย:

- เลขฐานสิบหก 32 หลัก แสดงเป็น 5 กลุ่ม
- กลุ่มคั่นด้วยขีดกลางในรูปแบบ `8-4-4-4-12`
- ระบุเวอร์ชัน 4 (กลุ่มที่สามขึ้นต้นด้วย "4")
- ระบุ variant แบบ IETF (กลุ่มที่สี่ขึ้นต้นด้วย "8", "9", "a" หรือ "b")

ตัวอย่าง: `2d89666b-1a1e-4a75-b193-4eb3b454c757`

ดูข้อกำหนดฉบับเต็มได้ที่
[RFC 9562: Universally Unique Identifiers (UUIDs)](https://www.rfc-editor.org/rfc/rfc9562.html)

## ประเภทข้อมูล [#data-types]

NautilusTrader กำหนดประเภทข้อมูลตลาดและข้อมูลอ้างอิงในตัวไว้ดังนี้ การรองรับ request ข้อมูลย้อนหลัง
และ subscription แบบ live ขึ้นกับผู้ให้บริการข้อมูลและ adapter ดู [Data](data/index.md) สำหรับฟิลด์และพฤติกรรม

- `OrderBookDelta` (การเปลี่ยนแปลงของ order book หนึ่งรายการ)
- `OrderBookDeltas` (ประเภท container)
- `OrderBookDepth` (snapshot ที่ความลึกแปรผันได้ พร้อมความจุแบบ inline สำหรับสิบระดับ)
- `QuoteTick`
- `TradeTick`
- `Bar`
- `MarkPriceUpdate`
- `IndexPriceUpdate`
- `FundingRateUpdate`
- `OptionGreeks`
- `Instrument`
- `InstrumentStatus`
- `InstrumentClose`

ใช้ [custom data](custom_data.md) สำหรับประเภทข้อมูลเฉพาะของแอปพลิเคชัน

ตัวเลือก `PriceType` ต่อไปนี้ใช้เลือกข้อมูลละเอียดสำหรับการรวม bar ภายใน (internal bar aggregation):

- `BID`
- `ASK`
- `MID`
- `LAST`

`BID`, `ASK` และ `MID` ใช้ข้อมูล `QuoteTick` ส่วน `LAST` ใช้ข้อมูล `TradeTick`
ส่วน bar type แบบ composite จะรวมจาก bar ที่เล็กกว่าแทน

## Bar aggregation [#bar-aggregations]

method `BarAggregation` ที่มีให้ใช้:

- `MILLISECOND`
- `SECOND`
- `MINUTE`
- `HOUR`
- `DAY`
- `WEEK`
- `MONTH`
- `YEAR`
- `TICK`
- `VOLUME`
- `VALUE` (หรือที่เรียกว่า dollar bar)
- `RENKO` (brick ตามราคา)
- `TICK_IMBALANCE`
- `TICK_RUNS`
- `VOLUME_IMBALANCE`
- `VOLUME_RUNS`
- `VALUE_IMBALANCE`
- `VALUE_RUNS`

aggregation ทั้งหมดที่ระบุไว้ถูก implement สำหรับ internal aggregation แล้ว
aggregation แบบ information-driven ต้องใช้ข้อมูล `TradeTick`

`BarSpecification` ประกอบด้วย price type, method ของ aggregation และขนาด step ที่เป็นบวก time bar แบบ
fixed-subunit มีข้อจำกัดเรื่องการหารลงตัว ดู [Bar types](data/index.md#bar-types) สำหรับกฎการตรวจสอบ
internal aggregation รันระหว่าง live trading ได้เมื่อมีข้อมูล input ที่ต้องการ

## ประเภทของ account [#account-types]

[accounting engine](accounting.md#account-types) รองรับการตั้งค่าต่อไปนี้ทั้งในสภาพแวดล้อม live และ backtest:

- `Cash` สกุลเงินเดียว (base currency)
- `Cash` หลายสกุลเงิน
- `Margin` สกุลเงินเดียว (base currency)
- `Margin` หลายสกุลเงิน
- `Betting` สกุลเงินเดียว
- `Wallet` หลายสกุลเงิน (blockchain wallet โดย execution client ยังอยู่ระหว่างพัฒนา)

## ประเภทของ order [#order-types]

[order model](orders/) รองรับประเภทต่อไปนี้ ขึ้นกับการรองรับของ venue adapter:

- `MARKET`
- `LIMIT`
- `STOP_MARKET`
- `STOP_LIMIT`
- `MARKET_TO_LIMIT`
- `MARKET_IF_TOUCHED`
- `LIMIT_IF_TOUCHED`
- `TRAILING_STOP_MARKET`
- `TRAILING_STOP_LIMIT`

## Value type [#value-types]

value type แบบ fixed-point ต่อไปนี้ใช้ integer ดิบขนาด 128 บิตหรือ 64 บิต ขึ้นกับ
[precision mode](../getting_started/installation.md#precision-mode) ที่ใช้ตอน compile

- `Price`
- `Quantity`
- `Money`

build แบบ Rust ล้วนใช้ standard precision เป็นค่าเริ่มต้น เว้นแต่จะเปิด feature `high-precision`

### High-precision mode (128 บิต) [#high-precision-mode-128-bit]

เมื่อ**เปิด** feature flag `high-precision` ค่าต่างๆ จะใช้ข้อกำหนดดังนี้:

| Type       | Raw backing | Max precision | Min value           | Max value          |
| :--------- | :---------- | :------------ | :------------------ | :----------------- |
| `Price`    | `i128`      | 16            | -17,014,118,346,046 | 17,014,118,346,046 |
| `Money`    | `i128`      | 16            | -17,014,118,346,046 | 17,014,118,346,046 |
| `Quantity` | `u128`      | 16            | 0                   | 34,028,236,692,093 |

### Standard-precision mode (64 บิต) [#standard-precision-mode-64-bit]

เมื่อ**ปิด** feature flag `high-precision` ค่าต่างๆ จะใช้ข้อกำหนดดังนี้:

| Type       | Raw backing | Max precision | Min value      | Max value      |
| :--------- | :---------- | :------------ | :------------- | :------------- |
| `Price`    | `i64`       | 9             | -9,223,372,036 | 9,223,372,036  |
| `Money`    | `i64`       | 9             | -9,223,372,036 | 9,223,372,036  |
| `Quantity` | `u64`       | 9             | 0              | 18,446,744,073 |
