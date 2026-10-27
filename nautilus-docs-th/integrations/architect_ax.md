---
title: "AX Exchange"
---

[AX Exchange](https://architect.exchange) เป็น derivatives exchange แบบรวมศูนย์ที่อยู่ภายใต้การกำกับดูแล
สำหรับ asset class อ้างอิงแบบดั้งเดิม Architect Bermuda Ltd. ดำเนินการ AX ภายใต้ใบอนุญาตจาก
[Bermuda Monetary Authority (BMA)](https://www.bma.bm) AX มีสัญญา perpetual และ futures แบบมีวันหมดอายุ
อยู่ทั้งใน catalog ของ production และ sandbox

integration นี้รองรับการรับ market data แบบ live และการส่ง order ไปยัง AX Exchange

## ภาพรวม [#overview]

adapter นี้ implement ด้วย Rust และไม่ต้องใช้ client library ภายนอกของ AX

คู่มือนี้สมมติว่าผู้เทรดกำลังตั้งค่าทั้ง live market data feed และ trade execution
adapter ของ AX Exchange ประกอบด้วยหลาย component ซึ่งใช้ร่วมกันหรือแยกกันได้ตาม use case

- `AxHttpClient`: การเชื่อมต่อ HTTP API ระดับล่าง
- `AxMdWebSocketClient` และ `AxOrdersWebSocketClient`: การเชื่อมต่อ WebSocket ระดับล่างสำหรับ
  ผู้เรียกใช้ฝั่ง Rust
- `AxDataClient`: ตัวจัดการ market data feed
- `AxExecutionClient`: gateway สำหรับจัดการ account และ trade execution
- `AxDataClientFactory`: factory สำหรับ data client ของ AX
- `AxExecutionClientFactory`: factory สำหรับ execution client ของ AX

:::note
ผู้ใช้ส่วนใหญ่จะตั้งค่า live trading node ตามที่แสดงด้านล่าง และไม่จำเป็นต้องใช้ component ระดับล่าง
เหล่านี้โดยตรง
:::

## ตัวอย่าง [#examples]

- [ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/architect_ax/examples/)

## เอกสารของ AX Exchange [#ax-exchange-documentation]

AX Exchange มีเอกสารสำหรับผู้ใช้อยู่ที่
[เว็บไซต์เอกสารของ Architect](https://docs.architect.exchange/)
ให้อ่านเอกสารของ AX Exchange ควบคู่กับคู่มือ integration ของ NautilusTrader นี้

## ผลิตภัณฑ์ [#products]

catalog ของ production มีสัญญา perpetual และ futures แบบมีวันหมดอายุในหมวดหมู่ของ venue ต่อไปนี้:

| หมวดหมู่ของ venue | ตัวอย่าง                      | Nautilus asset class |
| ---------------- | ---------------------------- | -------------------- |
| Foreign exchange | `EURUSD-PERP`, `JPYUSD-PERP` | FX                   |
| Equities         | `AAPL-PERP`, `NVDA-PERP`     | Equity               |
| Metals           | `XAU-PERP`, `XAU-2026-DEC`   | Commodity            |
| Energy           | `WTIOIL-PERP`                | Commodity            |
| Treasuries       | `UST10Y-PERP`                | Debt                 |
| Compute          | `NVDA-H100-2027-MAR`         | Alternative          |

catalog ของ sandbox แตกต่างออกไป โดยมีสัญญา `energy_etfs` เช่น `USO-PERP` และ `UNG-PERP` เพิ่มเข้ามา
ซึ่ง map ไปยัง asset class Equity และมี compute perpetual เช่น `OCPI-H100-PERP`

adapter จะ map หมวดหมู่ venue `crypto` ไปยัง asset class `CRYPTOCURRENCY` และหมวดหมู่ใดที่ไม่รู้จัก
ไปยัง `ALTERNATIVE`

### สัญญา perpetual [#perpetual-contracts]

สัญญา perpetual (perpetual swap) เป็น derivative ที่ติดตามราคาของ asset อ้างอิงโดยไม่มีวันหมดอายุ
ต่างจาก futures ทั่วไปตรงที่ไม่มีวัน settlement ซึ่งช่วยตัดต้นทุนการ rollover และทำให้การจัดการ position
ง่ายขึ้น กลไก funding rate ทำให้ราคาสัญญาสอดคล้องกับราคา index อ้างอิง ผ่านการจ่ายเงินเป็นระยะระหว่าง
ผู้ถือฝั่ง long และ short ดูรายละเอียดกลไก funding rate และ specification ของสัญญาได้ที่
[เอกสารของ Architect](https://docs.architect.exchange/)

คุณลักษณะของสัญญา perpetual บน AX:

- **Cash-settled เป็น USD**: ไม่มีการส่งมอบจริง กำไรขาดทุนทั้งหมด settle เป็น USD
- **Funding rate**: การจ่ายเงินเป็นระยะทำให้ราคาสัญญาสอดคล้องกับ asset อ้างอิง
- **Multiplier เท่ากับ 1**: สัญญาแต่ละสัญญาแทน exposure หนึ่งหน่วยต่อ asset อ้างอิง
- **เฉพาะจำนวนสัญญาเต็มเท่านั้น**: ไม่รองรับ quantity ที่เป็นเศษส่วน
- **Margin**: position ต้องใช้ initial margin ในการเปิด และ maintenance margin เพื่อคง position ไว้

adapter แทน instrument ของ AX ที่ไม่มีวันหมดอายุด้วย `PerpetualContract` และ instrument ที่มี
วันหมดอายุด้วย `FuturesContract` หมวดหมู่ของ venue เป็นตัวกำหนด asset class ของ Nautilus
adapter ใช้ account type แบบ `MARGIN` และการจัดการ order แบบ `NETTING`

## Symbology [#symbology]

adapter คง symbol ของ AX ไว้ตามเดิมและต่อท้ายด้วย venue identifier ของ Nautilus คือ `.AX`
symbol ของ perpetual ใช้ suffix `-PERP` ส่วน symbol แบบมีวันหมดอายุจะมีปีและเดือนของสัญญา

| สัญญา        | AX Symbol      | Nautilus InstrumentId |
| ------------ | -------------- | --------------------- |
| EUR/USD perp | `EURUSD-PERP`  | `EURUSD-PERP.AX`      |
| Gold perp    | `XAU-PERP`     | `XAU-PERP.AX`         |
| Dated gold   | `XAU-2026-DEC` | `XAU-2026-DEC.AX`     |

venue identifier คือ `AX` ดังนั้น `InstrumentId` ของ Nautilus จะอยู่ในรูป `EURUSD-PERP.AX`

## Environment [#environments]

AX Exchange มี trading environment สองแบบ ตั้งค่า environment ที่ต้องการด้วย parameter
`environment` ใน config ของ client

| Environment    | Config                                 | คำอธิบาย                                   |
| -------------- | -------------------------------------- | ------------------------------------------ |
| **Sandbox**    | `environment=AxEnvironment.SANDBOX`    | environment สำหรับทดสอบด้วยเงินจำลอง        |
| **Production** | `environment=AxEnvironment.PRODUCTION` | live trading ด้วยเงินจริง                  |

### Sandbox [#sandbox]

environment เริ่มต้นสำหรับการพัฒนาและทดสอบด้วยเงินจำลอง
endpoint ทั้งหมดของ sandbox จะถูก resolve อัตโนมัติเมื่อ `environment=AxEnvironment.SANDBOX`

#### 1. สร้าง sandbox account [#1-create-a-sandbox-account]

ทำตาม [เอกสารของ Architect](https://docs.architect.exchange/) เพื่อสร้าง sandbox account
ต้องใช้ invite code ระหว่างการลงทะเบียน

#### 2. สร้าง API key และเติมเงินเข้า account [#2-create-api-keys-and-fund-the-account]

ใช้ UI ของ AX sandbox เพื่อสร้าง API key และฝากเงินจำลองเข้า account
เก็บ `api_key` และ `api_secret` ไว้อย่างปลอดภัย

#### 3. ตั้งค่า environment variable [#3-set-environment-variables]

```bash
export AX_API_KEY="your-sandbox-api-key"
export AX_API_SECRET="your-sandbox-api-secret"
```

#### 4. ตั้งค่า live node [#4-configure-the-live-node]

ตั้ง `environment=AxEnvironment.SANDBOX` ใน config ของ data client และ execution client ดูการตั้งค่า
`LiveNode` แบบสมบูรณ์ได้ที่
[ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/architect_ax/examples/)
และ [รัน live trading ด้วย Rust](../how_to/run_rust_live_trading.md)

### Production [#production]

สำหรับ live trading ด้วยเงินจริง ต้องมี account ของ AX Exchange ที่ผ่านการยืนยันตัวตนแล้ว
ให้ตั้ง `environment` ใน config ของ execution client เป็น `PRODUCTION`

:::warning
ตรวจสอบให้แน่ใจว่าใช้ environment ที่ถูกต้องก่อนวาง order
ค่าเริ่มต้นคือ sandbox เพื่อป้องกันการเทรดจริงโดยไม่ตั้งใจ
:::

## Market data [#market-data]

adapter ให้ market data แบบ real-time ผ่าน WebSocket subscription และมี HTTP endpoint สำหรับ
backfill ข้อมูลย้อนหลัง

### ประเภทข้อมูล [#data-types]

| ข้อมูลของ AX       | Nautilus Data Type  | หมายเหตุ                                                         |
| ----------------- | ------------------- | ---------------------------------------------------------------- |
| Order book (L1)   | `QuoteTick`         | best bid/ask ระดับ top-of-book จาก L1 book subscription           |
| Order book (L2)   | `OrderBookDelta`    | ระดับราคาแบบรวม (aggregated)                                     |
| Order book (L3)   | `OrderBookDelta`    | quantity ราย order ในแต่ละ snapshot พร้อม ID สังเคราะห์            |
| Trades            | `TradeTick`         | trade event แบบ real-time จาก WebSocket subscription แบบ trade อย่างเดียว |
| Mark price        | `MarkPriceUpdate`   | ดึงมาจาก L1 ticker subscription                                  |
| Bars/candles      | `Bar`               | ข้อมูล OHLCV (มีเฉพาะ volume รวม ไม่แยกฝั่ง buy/sell)             |
| Funding rates     | `FundingRateUpdate` | poll ผ่าน HTTP ตั้งค่าช่วงเวลาได้                                |
| Instrument status | `InstrumentStatus`  | การเปลี่ยนสถานะจาก L1 ticker subscription                        |

สถานะ instrument ของ AX map ไปยัง `MarketStatusAction` ดังนี้:

| สถานะของ AX                          | `MarketStatusAction`        |
| ----------------------------------- | --------------------------- |
| Pre-open                            | `PRE_OPEN`                  |
| Open                                | `TRADING`                   |
| Closed, closed-frozen               | `CLOSE`                     |
| Halted                              | `HALT`                      |
| Match-and-close auction             | `CROSS`                     |
| Suspended                           | `SUSPEND`                   |
| Delisted หรือสถานะใดที่ไม่รู้จัก       | `NOT_AVAILABLE_FOR_TRADING` |

:::note
AX Exchange ไม่รองรับ request ขอ quote tick ย้อนหลัง มีเพียงข้อมูล quote แบบ real-time ผ่าน
WebSocket L1 book subscription เท่านั้น adapter ไม่ได้เปิดให้ใช้ราคา index ของ AX หรือ event
การปิดของ instrument ดังนั้น subscription เหล่านั้นจะ log warning และไม่ได้ข้อมูลใด ๆ
:::

:::note
snapshot L3 ของ AX มี quantity ราย order แต่ไม่มี venue order ID adapter จะกำหนด ID สังเคราะห์
ภายในแต่ละ snapshot จึงไม่สามารถติดตาม order ตัวเดียวกันข้าม snapshot ได้
การประมวลผล L2 และ L3 ต้องใช้ snapshot เต็ม (`st: true`) adapter จะปฏิเสธ frame แบบ incremental
(`st: false`) เพื่อไม่ให้ล้างระดับราคาของ book ที่ไม่ได้เปลี่ยนแปลง
:::

:::note
AX ไม่ได้เผยแพร่ trade identifier สำหรับ market data ดังนั้น adapter จะสร้าง `TradeTick.trade_id` จาก
timestamp, ราคา, ขนาด และ aggressor side ของ trade REST และ WebSocket จะได้ ID ตรงกันสำหรับ trade
เดียวกันเมื่อทั้งคู่รายงาน aggressor side ส่วน print ที่ AX รายงานเหมือนกันทุกประการจะได้ ID เดียวกัน
เรื่องนี้กระทบเฉพาะผู้ใช้ที่ตัดข้อมูลซ้ำของ market data ด้วย `trade_id` เพราะ fill ใช้ trade ID ของ venue เอง
:::

### พฤติกรรมของ WebSocket subscription [#websocket-subscription-behavior]

WebSocket subscription ของ market data บน AX ใช้ stream ที่ active หนึ่งตัวต่อ symbol adapter จะเลือก
stream ที่เล็กที่สุดที่ครอบคลุม subscription ของ Nautilus ที่ active อยู่:

- subscription แบบ trade อย่างเดียวใช้ AX `level: "TRADES"` ซึ่งส่งเฉพาะ trade print
- subscription แบบ book อย่างเดียวและ quote อย่างเดียวจะตั้ง AX `trades: false` และ `ticker: false`
  เพื่อปิด trade และ ticker event ที่ไม่ได้ร้องขอ
- subscription ของ mark price และ instrument status ต้องใช้ ticker event ของ AX ดังนั้น adapter
  จะเปิดการส่ง ticker บน book stream ที่ active อยู่ หากยังไม่มี book subscription จะเปิด L1 stream
- book delta จะ subscribe ที่ level ของ AX ที่ตรงกับ book type ของ Nautilus `L1_MBP` ไม่มี level
  ของ AX ที่รองรับ delta ดังนั้น adapter จะ log warning และ subscribe ที่ L2 แทน
- หากมี data type ของ Nautilus หลายประเภท active อยู่สำหรับ symbol หนึ่ง adapter จะ resubscribe
  เฉพาะเมื่อเกิดการเปลี่ยนแปลงต่อไปนี้:

  - level ของ AX หรือ delivery flag ที่ต้องใช้เปลี่ยนไป
  - order book ต้องการ snapshot ใหม่ (ดู [การกู้คืน order book](#order-book-recovery))
- การเปลี่ยน subscription จะไปถึง AX ตามลำดับที่ data engine ส่งออกไป ดังนั้นการ unsubscribe
  แล้วตามด้วย resubscribe จะทำให้ stream ยังคง subscribe อยู่

AX ระบุไว้ว่ามี funding rate โดยประมาณใน ticker event และมี request ขอ funding โดยประมาณบน orders
WebSocket model ของ ticker จะเก็บ metadata ของ funding โดยประมาณไว้ Nautilus เปิดให้ใช้ funding-rate update
ที่ settle แล้วผ่าน HTTP polling adapter ไม่ได้ emit data type แยกสำหรับ funding โดยประมาณ หรือ request
ค่าประมาณแยกต่างหาก

### การกู้คืน order book [#order-book-recovery]

data client ติดตาม order book delta subscription แต่ละตัวด้วย
[กลไกกู้คืน book ที่ใช้ร่วมกัน](../developer_guide/adapters.md#order-book-recovery-ownership) message
L2 และ L3 ของ AX มี snapshot เต็มและไม่มี sequence number AX จะส่ง message หนึ่งทันทีหลัง acknowledgement
ของการ subscribe แต่ละครั้ง รวมถึงกรณีที่ book ว่างหรือไม่เปลี่ยนแปลง client ยอมรับทุก message เป็น snapshot
และจะระงับ output ของ book ขณะที่การเขียน subscription ยังอยู่ระหว่างดำเนินการ และทิ้ง frame ของ book
ที่ไม่ได้ subscribe แล้ว

การกู้คืนจะแทนที่ subscription ของ symbol ด้วยการ unsubscribe และ subscribe บน connection เดิม
subscription ที่แทนที่จะคง level ปัจจุบันและ flag ของ trade และ ticker ไว้ AX ส่ง market data ทั้งหมด
ของ symbol หนึ่งบน stream เดียว ดังนั้น trade, quote, mark price และ instrument status ของ symbol นั้น
จะหยุดชั่วคราวระหว่างที่การแทนที่ทำงาน การกู้คืนเริ่มเมื่อ:

- การเขียน subscription ครั้งแรกล้มเหลว
- ไม่มี snapshot มาถึงภายใน `book_snapshot_timeout_secs` (ค่าเริ่มต้น 10 วินาที) หลังจากการเขียน
  subscription ครั้งแรกเสร็จหรือ connection ถูกสร้างใหม่ การ `connect` ของ data client หลัง `disconnect`
  ก็นับด้วย: client จะเก็บ book ไว้ และ WebSocket client จะ replay subscription ของ book เหล่านั้น
- frame L2 หรือ L3 แปลงไม่ได้ เช่น frame แบบ incremental (`st: false`) book จะหยุด emit จนกว่า
  snapshot ทดแทนจะมาถึง และความพยายามปัจจุบันของการกู้คืนที่กำลังทำงานจะล้มเหลวทันทีโดยไม่รอ
  deadline ของ snapshot

subscription ที่ AX ปฏิเสธจะไม่มี snapshot ส่งมา ดังนั้น deadline ของ snapshot จะเป็นตัวเริ่มการกู้คืน
subscription ที่ client ไม่สามารถเข้าคิวได้เพราะ WebSocket handler หยุดทำงานแล้ว จะไม่เริ่มการกู้คืน
และ book ของมันจะไม่ emit อะไรเลย การ subscribe delta ของ book ที่มี stream เปิดอยู่แล้วจะแทนที่ stream
ทำให้ book เริ่มต้นจาก snapshot ใหม่

การกู้คืนแต่ละครั้งจะพยายามได้สูงสุดแปดครั้งภายใน 180 วินาที ด้วย exponential backoff จากนั้นจะ retry
ต่อไปจนกว่าจะยอมรับ snapshot ได้ ช่วงเวลา retry จะเพิ่มเป็นสองเท่าจากหนึ่งนาทีจนถึงสิบห้านาที
การกู้คืนที่กำลังทำงานจะดำเนินต่อข้ามการ reconnect ด้วย budget ที่เหลืออยู่ และการ unsubscribe หรือ
shutdown จะยกเลิกมัน การกู้คืนที่รออยู่ระหว่างความพยายามหลังหมด budget แล้วจะ retry ทันทีบน connection
ใหม่ การกู้คืนจะไม่จบลงในสถานะล้มเหลว

การเขียน subscription จะทำงานตามลำดับ ขณะที่ handler ยังคงประมวลผล command และ frame ต่อไป
การยกเลิกจะข้ามการเขียน book subscription และการกู้คืนที่ยังไม่ได้ส่ง และคง gate ของ snapshot ไว้ในสถานะปิด
การเขียนที่ถูกส่งเข้าเครือข่ายไปแล้วยังอาจเสร็จสิ้นก่อนการเปลี่ยน subscription ที่ตามมาได้

client ไม่ได้จับคู่ acknowledgement ของ subscription กับความพยายามในการกู้คืน snapshot ที่เข้าคิวไว้
ก่อนการแทนที่สามารถทำให้การกู้คืนเสร็จได้เมื่อการเขียนทดแทนเสร็จสิ้น AX ไม่ส่ง frame ใด ๆ ขณะที่ book
ไม่เปลี่ยนแปลง ดังนั้น client จะไม่ถือว่า book ที่เงียบอยู่เป็นข้อมูลเก่า

การตั้ง `book_snapshot_timeout_secs` เป็น `0` จะปิด deadline ของ snapshot การกู้คืนจะเริ่มเฉพาะจาก
การเขียนครั้งแรกที่ล้มเหลวหรือ frame ที่ไม่ถูกต้องเท่านั้น ภายใน retry budget การแทนที่ที่ไม่ได้ส่ง
snapshot มาจะรอจนกว่าจะเกิดเหตุการณ์ใดเหตุการณ์หนึ่งต่อไปนี้:

- client ยอมรับ snapshot
- frame ที่ไม่ถูกต้องทำให้ความพยายามนั้นล้มเหลว
- การกู้คืนถูกยกเลิก
- budget เริ่มต้น 180 วินาทีหมดลง

### การตรวจสอบการกู้คืนแบบ live [#live-recovery-validation]

harness `ax-book-stress` เป็น tool สำหรับการพัฒนาที่ใช้กับการเปลี่ยนแปลงด้าน book synchronization และ
การกู้คืน มันใช้ market data จาก AX sandbox และไม่ส่ง order ใด ๆ โดยจะตรวจ book ของ perpetual ห้าตัว
เทียบกับ contract ของ book stream และ frame L2 ดิบแต่ละ frame ที่ถูกส่งต่อมา

market data stream ของ AX ต้องมีการยืนยันตัวตน ดังนั้น harness จะอ่าน API credential ของ sandbox
จาก `AX_API_KEY` และ `AX_API_SECRET` และรันโดยไม่ใช้ `scripts/strip-adapter-env.bash` จาก root ของ
repository ให้รัน:

```bash
CARGO_BUILD_JOBS=16 cargo test -p nautilus-architect-ax --features examples --test ax-book-stress -- --timeout 10 --rounds 14
```

`--scenario` ใช้เลือกรูปแบบการรัน:

- `churn` (ค่าเริ่มต้น): หมุนเวียนผ่านเงื่อนไขต่อไปนี้:

  - frame ที่ไม่ถูกต้องซึ่งแต่ละ book กู้คืนได้โดยไม่ต้อง reconnect
  - snapshot ที่ถูกกักไว้เกิน deadline หลัง reconnect
  - snapshot ที่มาช้าหลัง reconnect
  - การแทนที่ที่ถูกปฏิเสธ
  - การ reconnect ที่ถูกตัดก่อนที่ snapshot จะมา
  - การ restart ระหว่างการกู้คืน
  - การหยุด traffic 40 วินาทีโดยไม่ปิด socket
- `initial`: ทิ้ง snapshot แรกของแต่ละ book และทำให้ stream เงียบ โดยใช้ session ใหม่ในแต่ละรอบ
  เมื่อใช้ `--timeout 0` book จะมืดอยู่จนกว่าการ reconnect จะ replay subscription ของมัน
- `turnover`: unsubscribe และ resubscribe book ที่กำลังกู้คืน ทันทีหลังการกู้คืนเริ่ม, หลังการแทนที่
  ไปถึง venue หรือหลัง deadline ของการแทนที่ที่ถูกปฏิเสธ subscription ใหม่ต้อง stream ต่อไปได้
  เมื่อ venue นิ่งแล้ว โดยไม่มีการแทนที่เพิ่มเติม
- `boundaries`: ปฏิเสธทุกความพยายามใน retry budget จากนั้นตรวจเพดานของการ retry, การ reconnect
  ที่ยุติการรอเพดาน, การ unsubscribe ระหว่างการกู้คืน และการ shutdown ระหว่างการ reconnect
  ต้องใช้ `--timeout` ที่ไม่เป็นศูนย์ เพราะ deadline ของ snapshot เป็นตัวยุติความพยายามแต่ละครั้งที่ถูกปฏิเสธ
  และต้องมี `--symbols` อย่างน้อยสามตัว

`--timeout` กำหนด snapshot timeout เป็นวินาที โดย `0` จะปิด deadline ของ snapshot และ
`--rounds` กำหนดจำนวนรอบ (ค่าเริ่มต้น 14) `--symbols` รับรายการ symbol คั่นด้วย comma ที่จะตรวจ
market maker ของ sandbox quote เพียงบาง instrument และ book ที่หยุด stream จะทำให้การรันล้มเหลว
ดังนั้นให้เลือก book ที่มีการ stream อยู่

harness ต้องใช้ market data WebSocket และ REST API ของ sandbox ดู
[Stress harnesses](../developer_guide/spec_data_testing.md#stress-harnesses) สำหรับ flag ที่ใช้ร่วมกัน
และรูปแบบ output

### พฤติกรรมของ HTTP API [#http-api-behavior]

- `GET /tickers` คืน metadata ของหน้าแบบ limit/offset และรองรับ query parameter `limit`, `offset`
  และ `sort`
- `GET /ticker` คืน ticker ภายใต้ field `ticker` ระดับบนสุดของ response
- `GET /open-orders` ใช้ pagination แบบ limit/offset การ reconcile open order จะไล่ทุกหน้าและตรวจ
  ยอดรวม, offset, ข้อมูลซ้ำ และความครบถ้วน เพื่อให้ response ที่คลาดเคลื่อนที่ตรวจพบทำให้ request ล้มเหลว
- `GET /fills` และ `GET /funding-rates` ใช้ pagination แบบ cursor adapter จะไล่ cursor chain แต่ละสาย
  เป็นการอ่านข้อมูลย้อนหลังแบบ best-effort การแก้ไขข้อมูลของ AX ระหว่างที่ไล่อยู่ไม่ได้เป็น snapshot แบบ atomic
- `GET /orders` มี metadata ของ cursor และรองรับ filter `order_id`, `order_ids`, `account_id` และ
  filter timestamp ที่เป็นตัวเลือกเสริม การไล่ประวัติ order report จะตาม cursor chain, ยอมรับหน้าที่ไม่เต็ม
  และตรวจ limit และยอดรวมที่ประกาศไว้ หน้าต่อเนื่องที่ว่าง, cursor ซ้ำ, order ID ซ้ำ และยอดรวมที่ประกาศไว้
  แต่ไม่ครบจะทำให้ request ล้มเหลว
- request ขอ report ของ open order, order ย้อนหลัง, fill และ position จะ resolve symbol ที่ยังไม่ได้ cache
  ผ่าน `GET /instrument` และ cache ผลลัพธ์ไว้ หาก request instrument หรือการ parse ล้มเหลว request
  report ทั้งหมดจะล้มเหลว แทนที่จะทิ้ง state ของ venue
- `GET /transactions` ต้องมี `start_timestamp_ns` และ `end_timestamp_ns` ที่มีช่วงไม่เกิน 7 วัน
  client ระดับล่างเปิดให้ใช้ cursor และตัวเลือก account
- `GET /order-status` อาจมี `reject_reason` และ `reject_message` สำหรับ order ที่ถูก reject
- เมื่อไม่ได้ระบุตัวเลือก account AX จะใช้ primary account execution client ระดับสูงเป็นเจ้าของ
  primary account หนึ่งตัว ส่วน request model ระดับล่างเปิดให้ใช้ตัวเลือก account ตามที่ระบุไว้ในเอกสาร

### Bar interval [#bar-intervals]

| Interval | คำอธิบาย  |
| -------- | -------- |
| `1s`     | 1 วินาที  |
| `5s`     | 5 วินาที  |
| `1m`     | 1 นาที    |
| `5m`     | 5 นาที    |
| `15m`    | 15 นาที   |
| `1h`     | 1 ชั่วโมง |
| `1d`     | 1 วัน     |

## ความสามารถด้าน order [#orders-capability]

API สำหรับส่ง order ของ AX ไม่มีตัวเลือก order type รูปแบบ order ดั้งเดิมเพียงแบบเดียวของมันต้องระบุราคา
ซึ่ง adapter map ไปเป็น order `LIMIT` ของ Nautilus adapter จำลอง order `MARKET` ของ Nautilus
โดยขอ preview ราคาแบบ aggressive แล้วส่ง order แบบมีราคานั้นด้วย IOC
fill ที่ติดตามอยู่จะคง order type และ side ของ Nautilus ที่ส่งไว้ รวมถึง market order ที่จำลองขึ้น

schema ของ request ใน [REST place-order](https://docs.architect.exchange/api-reference/order-management/place-order)
และ [orders WebSocket](https://docs.architect.exchange/api-reference/order-management/orders-ws) อย่างเป็นทางการ
ไม่มี field `order_type` หรือ `trigger_price` การส่ง stop-limit บน sandbox ที่ trigger ยังไม่ถูกทะลุ
กลับถูก execute ทันทีที่ราคา limit ที่ active อยู่ เนื่องจากยังยืนยันการ execute แบบมีเงื่อนไขไม่ได้ adapter
จึงปฏิเสธ stop-limit order แบบ native ของ venue ก่อนส่ง

Nautilus ยังคง emulate stop-limit order ภายในเครื่องได้ order emulator ทั่วไปจะรอ trigger ที่ตั้งค่าไว้
จากนั้นจึงส่ง limit order ธรรมดาไปยัง adapter นี้

### Order type [#order-types]

| Order Type             | รองรับ | หมายเหตุ                                           |
| ---------------------- | ------ | -------------------------------------------------- |
| `MARKET`               | ✓      | adapter จำลองด้วยราคา IOC แบบ aggressive           |
| `LIMIT`                | ✓      | map ไปยังรูปแบบ order แบบมีราคาดั้งเดิมของ AX      |
| `STOP_LIMIT`           | -      | *AX Exchange ไม่รองรับ*                            |
| `LIMIT_IF_TOUCHED`     | -      | *AX Exchange ไม่รองรับ*                            |
| `STOP_MARKET`          | -      | *AX Exchange ไม่รองรับ*                            |
| `MARKET_IF_TOUCHED`    | -      | *AX Exchange ไม่รองรับ*                            |
| `TRAILING_STOP_MARKET` | -      | *AX Exchange ไม่รองรับ*                            |

### Execution instruction [#execution-instructions]

| Instruction      | รองรับ | หมายเหตุ                                                          |
| ---------------- | ------ | ----------------------------------------------------------------- |
| `post_only`      | ✓      | maker เท่านั้น ถูก reject หาก order จะเป็นฝั่ง take               |
| `reduce_only`    | -      | ถูก reject ภายในเครื่อง เพราะ AX ไม่มี field reduce-only          |
| `quote_quantity` | -      | ถูก reject ภายในเครื่อง เพราะเส้นทาง wire ของ adapter encode ได้เฉพาะ base |
| `display_qty`    | -      | ถูก reject ภายในเครื่อง เพราะเส้นทาง wire ของ adapter ไม่มี field display |

adapter ไม่ใส่ `rb` ใน request place และ replace จึงใช้พฤติกรรม `rej` ค่าเริ่มต้นของ AX สำหรับ
post-only order ไม่รองรับการเลือก `bo` (ถอยออกหนึ่ง tick จากฝั่งตรงข้าม) หรือ `tbl` (ราคาที่ดีที่สุด
ในฝั่งเดียวกัน) ดู [AX changelog](https://docs.architect.exchange/changelog)

การตรวจ order ที่ cross บน sandbox ยืนยันว่าการไม่ใส่ `rb` และการระบุ `rej` อย่างชัดเจนจะ reject ด้วย
`post-only order would cross the book` external order ที่เลือกใช้ `bo` หรือ `tbl` จะรายงานราคา limit
ที่ปรับแล้วใน `p` และ reconciliation จะใช้ราคานั้น post-only order อาจ fill ได้หลังจากที่วางอยู่ใน book แล้ว

response ของประวัติ order, open order และ WebSocket order จะเก็บ `rb` เป็น metadata แบบมี type ที่เป็น
ตัวเลือกเสริมของ adapter ไว้ใน `AxOrderDetail`, `AxOpenOrder` และ `AxWsOrder` มันอธิบาย request place
หรือ replace ล่าสุด และมีผลเฉพาะเมื่อ request นั้นมี `po: true` ค่าที่หายไปหรือเป็น null จะคงเป็นค่าว่าง
string ที่ไม่รู้จักจะ map ไปเป็น `Unknown` ไม่ใช่ `Reject` `OrderStatusReport` ไม่ได้เปิดให้ใช้ metadata นี้
ดังนั้น reconciliation report จึงไม่สามารถแยกแยะ policy การปรับราคาเหล่านี้สำหรับ order ที่ถูกวางหรือ
replace จากภายนอกได้

ขอบเขตของ reduce-only มีความสำคัญเพราะ AX ไม่มี field reduce-only บน sandbox venue ยอมรับและ fill
order เป็น order ธรรมดาหลังจากที่ instruction reduce-only ถูกตัดออกจาก payload ที่ส่งไป order แบบนี้
อาจเปิดหรือเพิ่ม exposure แทนที่จะปิด ยังไม่ได้ตรวจสอบพฤติกรรมบน production ดังนั้น adapter จึง deny
reduce-only order ก่อนการ submit แทนที่จะส่ง instruction ที่ venue ไม่สามารถปฏิบัติตามได้

adapter ยังปฏิเสธ instruction quote-quantity และ display-quantity เพราะเส้นทาง wire ไปยัง AX ของมัน
encode ความหมายเหล่านั้นไม่ได้ นี่เป็นขอบเขตของ adapter ไม่ได้หมายความว่า AX Exchange ปฏิเสธ
ฟีเจอร์ native ของ venue ที่เทียบเท่ากัน

### Time in force [#time-in-force]

| Time in Force  | รองรับ | หมายเหตุ                          |
| -------------- | ------ | --------------------------------- |
| `GTC`          | ✓      | Good Till Canceled                |
| `GTD`          | -      | adapter reject ภายในเครื่อง       |
| `DAY`          | ✓      | ใช้ได้จนสิ้นวันเทรด               |
| `IOC`          | ✓      | Immediate or Cancel               |
| `FOK`          | -      | adapter reject ภายในเครื่อง       |
| `AT_THE_OPEN`  | -      | adapter reject ภายในเครื่อง       |
| `AT_THE_CLOSE` | -      | adapter reject ภายในเครื่อง       |

venue ประกาศเลิกใช้ `DAY` (deprecated) และแนะนำให้ใช้ `GTC` แทน

### ฟีเจอร์ order ขั้นสูง [#advanced-order-features]

| ฟีเจอร์              | รองรับ | หมายเหตุ                                                              |
| ------------------ | ------ | --------------------------------------------------------------------- |
| Order modification | ✓      | replace แบบ atomic โดย AX จะคืน venue order ID ใหม่                   |
| Cancel order       | ✓      | cancel order ทีละตัว                                                  |
| Cancel all orders  | ✓      | cancel open order ทั้งหมดของ instrument หนึ่ง                         |
| Batch cancel       | -      | adapter ส่ง cancel ทีละตัว                                            |
| Order lists        | ✓      | submit ตามลำดับ (ส่ง order ทีละตัว ไม่เป็น atomic)                     |

adapter จะคืน open order และ in-flight order ภายในเครื่องที่อยู่ใน cache ก่อนประมวลผล orders stream
การส่งหรือ request ที่ล้มเหลวแบบไม่แน่นอนจะยังคงติดตามภายในเครื่องไว้ จนกว่า terminal event หรือ terminal
reconciliation report จะระบุ venue order ที่ active อยู่ได้ หลังการ replace การ query, การแก้ไข และการ
cancel จะใช้ venue order ID ที่ active อยู่ update ของ parent ที่เก่าแล้วและ update ที่ติดตามอยู่แต่มีรูปแบบผิด
จะรอ reconciliation โดยไม่ถูก route เป็น external order

cancel ที่มี venue order ID อยู่แล้วจะถูกส่งต่อไป รวมถึงการ cancel ครั้งที่สองของ order ที่จบแล้ว
strategy จะไม่ส่ง cancel ครั้งที่สองนั้นหลังจากที่ order ภายในเครื่องปิดไปแล้วหรืออยู่ในสถานะ pending cancel
adapter จะ emit `OrderCancelRejected` เฉพาะเมื่อ AX ส่ง `CancelRejected` เท่านั้น sandbox จะตอบการส่ง
cancel ซ้ำของ order ที่ถูก cancel ไปแล้วด้วย WebSocket error `404` (`order not found`) adapter จะ log
error นั้นไว้และไม่แปลงเป็น `OrderCancelRejected`

**Side filter**: cancel-all ของ AX ไม่มี parameter ของ side ดังนั้น command `CancelAllOrders` ที่ตั้ง
`order_side` ไว้จะส่ง cancel request ทีละตัว โดย cancel เฉพาะ open order ในฝั่งนั้นของ instrument
request ที่กรองตาม side จะเลือกจาก open order เท่านั้น ดังนั้น order ที่อยู่ระหว่างดำเนินการ (`SUBMITTED`)
ซึ่ง AX ยังไม่ได้ acknowledge จะรอดจาก request นี้

### การจัดการ position [#position-management]

| ฟีเจอร์           | รองรับ | หมายเหตุ                                  |
| --------------- | ------ | ----------------------------------------- |
| Query positions | ✓      | อัปเดต position แบบ real-time              |
| Position mode   | -      | netting mode เท่านั้น                     |
| Cross margin    | ✓      | cross-margin ข้ามทุก instrument           |

### การ query order [#order-querying]

| ฟีเจอร์                | รองรับ | หมายเหตุ                                                      |
| -------------------- | ------ | ------------------------------------------------------------- |
| Query open orders    | ✓      | แสดงรายการ order ที่ active ทั้งหมด                           |
| Query single order   | ✓      | ด้วย venue order ID หรือ client order ID (ทุกสถานะของ order)  |
| Order status reports | ✓      | การตรวจ open order และ mass status ย้อนหลังตอน startup        |
| Fill reports         | ✓      | ประวัติ execution และ fill                                    |

:::note
การตรวจ open order แบบจำนวนมากจะใช้ `/open-orders` เมื่อเปิด `open_check_open_only` ซึ่งเป็นค่าเริ่มต้น
มิฉะนั้นจะรวม `/orders` เข้ากับ snapshot `/open-orders` แบบไม่จำกัด reconciliation ของ mass-status ตอน
startup ใช้การรวมแบบเดียวกัน รวมถึง terminal order ย้อนหลัง เช่น order ที่ filled และ canceled
ผลลัพธ์ที่รวมแล้วจะใช้ขอบเขตของประวัติเฉพาะกับ report ที่ปิดแล้ว report ที่ active และ in-flight
จะไม่ถูกตัดด้วยขอบเขตนี้ และมีลำดับความสำคัญเหนือแถวประวัติที่มี venue order ID เดียวกัน

การ query order ทีละตัวผ่าน `query_order` ใช้ `/order-status` ซึ่งใช้ได้กับทุกสถานะของ order
order ที่ canceled, expired และ rejected ที่ไม่มี quantity เหลือ จะใช้ `/orders` เพื่อกู้คืน quantity
เดิมด้วย การ probe ของ reconciliation แบบเจาะจงใช้ `/order-status` ด้วย venue order ID หรือ client order ID
จากนั้นดึง record `/orders` ที่สมบูรณ์ มีเพียง error HTTP 404 `no matching orders` ของ AX เท่านั้นที่ยืนยัน
ว่า order ไม่มีอยู่ response error อื่น, ความล้มเหลวของ transport หรือการ decode และ record รายละเอียด
ที่หายไปจะทำให้ probe ล้มเหลว การ resolve instrument, การ parse report และ identity ที่ไม่ตรงกัน
ก็ทำให้ probe ล้มเหลวเช่นกัน

payload ของ open order และ order ย้อนหลังของ AX ไม่ได้เปิดเผย stop order type หรือราคา trigger
reconciliation ที่ได้จาก REST จึงรายงาน external order ทุกตัวที่มองเห็นเป็น limit order adapter
ไม่ได้ submit conditional order แบบ native ของ venue

order report ย้อนหลังมีเหตุผลการ reject ของ venue (`r` หรือถ้าไม่มีจะใช้ `txt`) ดังนั้น event
`OrderRejected` ที่ได้จาก reconciliation จะมี string เหตุผลเดียวกับที่ได้แบบ real-time event
`OrderCanceled` ที่ได้จาก reconciliation ก็อาจมีเหตุผลจาก venue ในกรณีที่การ cancel แบบ live ไม่มี
report จะเก็บ flag post-only ของ venue และ timestamp ระดับต่ำกว่าวินาทีไว้

payload `/orders` ของ AX มีเวลาสร้างแต่ไม่มี timestamp ตอนจบ เมื่อประวัติมี order ที่ปิดแล้วซึ่งมี fill
adapter จะ request `/fills` ในช่วงเวลาเดียวกันด้วย (สูงสุดเจ็ดวัน) report ย้อนหลังจะคง `ts_accepted`
ไว้ที่เวลาสร้าง และเลื่อน `ts_last` ของ order ที่ปิดแล้วไปยัง fill ล่าสุดที่ตรงกัน หากไม่มี fill ที่ตรงกัน
`ts_last` จะคงอยู่ที่เวลาสร้าง เวลาของ fill เป็นขอบล่างของเวลาที่ cancel หรือหมดอายุ การ probe แบบเจาะจง
ใช้ช่วงเจ็ดวันล่าสุดในการค้นหา fill นี้

order ID ย้อนหลังที่ถูก replace ถือเป็น terminal order แบบ day-complete จะ expire และ IOC order ที่
expire จะถูก cancel ซึ่งตรงกับเส้นทางของ WebSocket event สถานะ order ที่ไม่รู้จักจะทำให้ request
reconciliation ล้มเหลว แทนที่จะละ order นั้นออกจาก snapshot fill ที่มีสถานะของ order พี่น้องที่ไม่รู้จัก
ยังคงไปถึง execution

reconciliation ของ mass-status ตอน startup จำกัดขอบเขตของ request `/orders` และ `/fills` ด้วย
`reconciliation_lookback_mins` และ position จะถูกรายงานเป็น snapshot ปัจจุบันเสมอ lookback ที่ยาวกว่า
เจ็ดวันจะยังได้ fill เพียงเจ็ดวัน adapter ตั้ง `lookback_start` ของ mass status เป็นจุดเริ่มของช่วง fill
ที่ใช้จริง รวมถึงขอบเจ็ดวันเมื่อไม่ได้ตั้ง lookback ไว้ หากไม่ได้ตั้ง lookback adapter ยังคงดึง order
ย้อนหลังทั้งหมด lookback ที่ระบุเป็นศูนย์นาทีอย่างชัดเจนจะใช้ช่วง fill หนึ่งนาโนวินาที เพราะ AX ปฏิเสธ
query ที่มีความกว้างเป็นศูนย์

หลังจากได้ snapshot ของ position ที่สมบูรณ์สำเร็จ adapter จะ emit flat report อย่างชัดเจนสำหรับ
instrument ที่ไม่มีใน response ของ AX แต่มีอยู่ใน open position ที่ cache ไว้ของ account นี้ และจะคง
แถว flat และไม่ flat ที่ venue ระบุชัดเจนไว้ flat report จาก cache เหล่านี้ช่วยให้ reconciliation ปิด
position ที่เก็บไว้ซึ่ง resting order ใน cache ถูก fill ระหว่างที่ node offline ได้ โดยไม่ทิ้ง external
position ที่หักล้างกันไว้ fill ของ instrument ที่ไม่มี position report จะอัปเดต order โดยไม่เปิด position
ดังนั้นประวัติที่ไม่ครบของ instrument ที่ flat และไม่ได้ cache ไว้จึงไม่ต้องแก้ไข position แถว position
ที่ไม่ถูกต้องจะทำให้การรวบรวม mass-status ล้มเหลว แทนที่จะสื่อว่า instrument นั้น flat

position report จะหาร cost basis ที่มีเครื่องหมายจาก fill ledger ด้วย net quantity ที่มีเครื่องหมาย
[AX gateway schema](https://docs.architect.exchange/openapi/api-gateway.json) กำหนด `signed_notional`
เป็น alias ที่เลิกใช้แล้วของ `signed_cost_basis` และ report จะใช้ `signed_cost_basis` ก่อนเมื่อมีค่า
ค่าเฉลี่ยจาก ledger นี้อาจเปลี่ยนเมื่อมีการลดหรือกลับ position โดยไม่ขึ้นกับค่าเฉลี่ยของ fill เปิดของ
Nautilus และอาจเป็นศูนย์หรือติดลบได้

report ใช้ [`AvgPxReconciliation::OpeningOnly`](../concepts/execution/reconciliation.md#reported-entry-averages)
ดังนั้น reconciliation จะใช้ค่านี้สำหรับการเปิดและการกลับ position แบบสังเคราะห์ โดยไม่เปรียบเทียบกับ
ค่าเฉลี่ยใน cache report จะคงผลหารทศนิยมที่มีเครื่องหมายไว้โดยไม่ปัดเศษให้ตรงกับ price precision ของ
instrument หรือประกาศ precision ของราคาเฉลี่ย
:::

:::warning[ความเป็นเจ้าของ position ระหว่างการกู้คืน]
fill ปิดที่ผูกกับ order ใน cache จะคงความเป็นเจ้าของของ strategy ไว้ หากไม่มีหลักฐานของ order ที่ปิด
snapshot ที่ flat เพียงอย่างเดียวไม่สามารถระบุได้ว่า strategy ใดเป็นเจ้าของการแก้ไข ในกรณีนั้นให้ตรวจ
position ของ strategy ที่เก็บไว้เทียบกับ AX ก่อนกลับมาเทรดต่อ

special fill ใช้ identity ของ external order settlement ที่ปิด exposure ของ strategy จึงอาจทิ้ง position
ของ strategy และ external position ที่หักล้างกันไว้ ให้ตั้งค่า `external_order_instrument_ids` ของ strategy
เมื่อ strategy ควรเป็นเจ้าของ external order เหล่านั้น ดู
[External order creation](../concepts/execution/reconciliation.md#external-order-creation)

flat report ที่ระบุชัดเจนทำให้ reconciliation ของ position ทำงานได้ หากประวัติเริ่มต้นกลาง lifecycle
reconciliation อาจต้องใช้การแก้ไขแบบสังเคราะห์ เมื่อใช้ `generate_missing_orders=False` ความคลาดเคลื่อน
ที่ยังไม่ได้ข้อสรุปจะทำให้ startup ไม่ได้
:::

:::warning
หลังการ restart ที่ไม่มีประวัติการ replace ใน cache reconciliation อาจนำ state หรือ field ของ order เก่า
ไปใช้กับ order ที่มาแทนที่ ซึ่งอาจทำให้ working order ดูเหมือนถูก cancel ภายในเครื่อง การกู้คืนจะยังไม่สมบูรณ์
ให้ตรวจ order ที่ถูก replace เทียบกับ AX ก่อนกลับมาเทรดต่อ
:::

### Account state [#account-state]

endpoint `/balances` ไม่มีข้อมูล margin ดังนั้น account state จึง request `/risk-snapshot` ด้วย
ค่า `initial_margin_required_total` ที่เป็น USD จะถูกใช้เป็นเงินที่ถูก lock ของ balance USD โดยไม่เกิน
balance USD รายการ `MarginBalance` ของ USD จะจับคู่ initial margin กับ maintenance margin เมื่อ
`/risk-snapshot` ล้มเหลว account state จะถอยกลับไปใช้ balance อย่างเดียวโดย margin ที่ถูก lock เป็นศูนย์
และ log warning

## การยืนยันตัวตน [#authentication]

AX Exchange ใช้การยืนยันตัวตนด้วย bearer token:

1. API key และ secret ใช้ขอ session token ผ่าน `/authenticate`
2. session token ถูกใช้เป็น bearer token สำหรับ request REST และ WebSocket ที่ตามมา
3. adapter ขอ session token อายุหนึ่งชั่วโมง และ refresh ทุก 30 นาที
4. การ refresh จะอัปเดตการยืนยันตัวตนของ REST และ token ที่ใช้ในการ reconnect WebSocket ครั้งถัดไป
   โดยไม่ขัดจังหวะ connection ที่ active อยู่

## Configuration [#configuration]

### Environment และ endpoint [#environments-and-endpoints]

| Environment | HTTP API                                         | HTTP API (orders)                                   | Market Data WS                                   | Orders WS                                            |
| ----------- | ------------------------------------------------ | --------------------------------------------------- | ------------------------------------------------ | ---------------------------------------------------- |
| Sandbox     | `https://gateway.sandbox.architect.exchange/api` | `https://gateway.sandbox.architect.exchange/orders` | `wss://gateway.sandbox.architect.exchange/md/ws` | `wss://gateway.sandbox.architect.exchange/orders/ws` |
| Production  | `https://gateway.architect.exchange/api`         | `https://gateway.architect.exchange/orders`         | `wss://gateway.architect.exchange/md/ws`         | `wss://gateway.architect.exchange/orders/ws`         |

:::info
endpoint สำหรับจัดการ order (place, cancel, replace, cancel-all, order status, open orders,
historical orders และ initial margin requirement) ใช้ base URL ของ orders ส่วน REST endpoint อื่นทั้งหมด
รวมถึง authentication, account state, fills, transactions และ market data ใช้ base URL ของ API
adapter จะ resolve ทั้งสองจาก environment ที่ตั้งค่าไว้
:::

### Option ของ data client [#data-client-configuration-options]

| Option                             | ค่าเริ่มต้น | คำอธิบาย                                                                         |
| ---------------------------------- | --------- | -------------------------------------------------------------------------------- |
| `api_key`                          | `None`    | API key โหลดจาก env var `AX_API_KEY` เมื่อไม่ได้ระบุ                              |
| `api_secret`                       | `None`    | API secret โหลดจาก env var `AX_API_SECRET` เมื่อไม่ได้ระบุ                        |
| `environment`                      | `SANDBOX` | trading environment (`SANDBOX` หรือ `PRODUCTION`)                                |
| `base_url_http`                    | `None`    | override base URL ของ REST                                                       |
| `base_url_ws_public`               | `None`    | override URL ของ market data WebSocket                                           |
| `base_url_ws_private`              | `None`    | override URL ของ orders WebSocket แบบ private                                    |
| `proxy_url`                        | `None`    | URL ของ proxy (ตัวเลือกเสริม) สำหรับ transport HTTP และ WebSocket                |
| `http_timeout_secs`                | `60`      | timeout (วินาที) ของ REST request                                                |
| `max_retries`                      | `3`       | จำนวน retry สูงสุดสำหรับ REST request ที่เป็น idempotent (`GET`, `HEAD`, `OPTIONS`) |
| `retry_delay_initial_ms`           | `1,000`   | delay เริ่มต้น (มิลลิวินาที) ระหว่างการ retry                                     |
| `retry_delay_max_ms`               | `10,000`  | delay สูงสุด (มิลลิวินาที) ระหว่างการ retry (exponential backoff)                  |
| `heartbeat_interval_secs`          | `20`      | ช่วงเวลา heartbeat (วินาที) ของ WebSocket connection                              |
| `recv_window_ms`                   | `5,000`   | สงวนไว้ AX ใช้ bearer token และ adapter ไม่ได้ส่ง window                          |
| `update_instruments_interval_mins` | `60`      | ช่วงเวลา (นาที) ระหว่างการ refresh catalog ของ instrument                         |
| `funding_rate_poll_interval_mins`  | `15`      | ช่วงเวลา (นาที) ระหว่าง request poll funding rate                                |
| `book_snapshot_timeout_secs`       | `10`      | เวลารอ book snapshot ตอนเริ่ม, reconnect และกู้คืน `0` คือปิด                      |
| `transport_backend`                | `Sockudo` | backend ของ WebSocket transport                                                  |

### Option ของ execution client [#execution-client-configuration-options]

| Option                    | ค่าเริ่มต้น | คำอธิบาย                                                                         |
| ------------------------- | --------- | -------------------------------------------------------------------------------- |
| `account_id`              | `AX-001`  | account ID ของ execution client                                                  |
| `api_key`                 | `None`    | API key โหลดจาก env var `AX_API_KEY` เมื่อไม่ได้ระบุ                              |
| `api_secret`              | `None`    | API secret โหลดจาก env var `AX_API_SECRET` เมื่อไม่ได้ระบุ                        |
| `environment`             | `SANDBOX` | trading environment (`SANDBOX` หรือ `PRODUCTION`)                                |
| `base_url_http`           | `None`    | override base URL ของ API REST                                                   |
| `base_url_orders`         | `None`    | override base URL ของ orders REST                                                |
| `base_url_ws_private`     | `None`    | override URL ของ orders WebSocket                                                |
| `proxy_url`               | `None`    | URL ของ proxy (ตัวเลือกเสริม) สำหรับ transport HTTP และ WebSocket                |
| `http_timeout_secs`       | `60`      | timeout (วินาที) ของ REST request                                                |
| `max_retries`             | `3`       | จำนวน retry สูงสุดสำหรับ REST request ที่เป็น idempotent (`GET`, `HEAD`, `OPTIONS`) |
| `retry_delay_initial_ms`  | `1,000`   | delay เริ่มต้น (มิลลิวินาที) ระหว่างการ retry                                     |
| `retry_delay_max_ms`      | `10,000`  | delay สูงสุด (มิลลิวินาที) ระหว่างการ retry (exponential backoff)                  |
| `heartbeat_interval_secs` | `30`      | ช่วงเวลา heartbeat (วินาที) ของ WebSocket connection                              |
| `recv_window_ms`          | `5,000`   | สงวนไว้ AX ใช้ bearer token และ adapter ไม่ได้ส่ง window                          |
| `cancel_on_disconnect`    | `False`   | cancel open order ของ WebSocket session นี้เมื่อหลุดการเชื่อมต่อ                  |
| `transport_backend`       | `Sockudo` | backend ของ WebSocket transport                                                  |

เมื่อ `transport_backend=None` ค่าเริ่มต้นของ Rust ที่ compile ไว้จะเลือก Sockudo เมื่อเปิด Cargo feature
`transport-sockudo` และเลือก Tungstenite ในกรณีอื่น

ใช้ `AxDataClientConfig` คู่กับ `AxDataClientFactory` และใช้ `AxExecutionClientConfig` คู่กับ
`AxExecutionClientFactory` [ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/architect_ax/examples/)
แสดงการตั้งค่า live node สำหรับ data client และ execution client

### API credential [#api-credentials]

มีสองวิธีในการส่ง credential ให้ client ของ AX Exchange
คือส่งค่า `api_key` และ `api_secret` ใน config object หรือตั้ง environment variable ต่อไปนี้:

- `AX_API_KEY`
- `AX_API_SECRET`

:::tip
แนะนำให้ใช้ environment variable ในการจัดการ credential
:::

เมื่อเริ่ม trading node จะได้รับการยืนยันทันทีว่า credential ถูกต้องและมีสิทธิ์เทรดหรือไม่

## หมายเหตุด้าน implementation [#implementation-notes]

- **เฉพาะจำนวนสัญญาเต็มเท่านั้น**: AX ใช้ quantity ของสัญญาเป็นจำนวนเต็ม adapter กำหนด size increment
  และ lot size เป็นหนึ่งสัญญา ขณะเดียวกันก็บังคับใช้ `minimum_order_size` แยกของแต่ละ instrument
  quantity ที่เป็นเศษส่วนจะเกิด `OrderDenied` ภายในเครื่อง
- **การเริ่มใช้งานของ dated futures**: AX เผยแพร่วันหมดอายุแต่ไม่เผยแพร่ timestamp ของการเริ่มใช้งาน adapter
  จะใช้ค่าศูนย์สำหรับเวลาเริ่มใช้งานที่ไม่ทราบ และบันทึกข้อจำกัดนั้นไว้ใน metadata ของ instrument
- **Rate limiting**: adapter ใช้ rate limit แบบระมัดระวังที่ 10 request/วินาที พร้อม exponential backoff
  อัตโนมัติเมื่อได้ response แจ้ง rate limit
- **Market order**: AX ไม่รองรับ market order แบบ native adapter ใช้ preview endpoint เพื่อหาราคา
  take-through และส่ง IOC limit order แบบ aggressive เนื่องจาก book อาจขยับระหว่าง preview และการส่ง
  market order ที่จำลองขึ้นจึงอาจ fill เพียงบางส่วน
- **Stop-limit order**: adapter ปฏิเสธการส่ง stop-limit แบบ native ของ venue เพราะการทดสอบบน sandbox
  ไม่ได้ยืนยันความหมายแบบมีเงื่อนไข ให้ใช้ order emulation ภายในเครื่องเมื่อ strategy ต้องการ stop-limit order
- **การแก้ไข order**: AX รองรับการ replace order แบบ atomic ผ่าน `POST /replace-order` execution client
  map `modify_order` ไปยัง endpoint นี้ HTTP response มี venue order ID ที่เสนอมา ส่วน orders stream
  จะยืนยันว่าการ replace สำเร็จหรือไม่ การ query และ cancel จะคงใช้ working order ID เดิมจนกว่า stream,
  fill ของ order ลูก หรือการ query จะยืนยันการ replace การ query parent ที่ถูก replace จะตรวจ ID ที่เสนอมา
  และคืนสถานะของ order ทดแทนที่ยืนยันแล้ว รวมถึง fill หรือการ cancel และจะปล่อยผลลัพธ์ไว้โดยไม่มีข้อสรุป
  เมื่อยังไม่มีการยืนยันการ replace การ cancel ก่อนการยืนยันจะไปที่ parent และอาจถูก reject จากนั้นการกู้คืน
  จะต้องอาศัยการยืนยันจาก stream ที่มาช้า, fill ของ order ลูก หรือการ query อย่างชัดเจน
  การ replace แบบ post-only ที่ cross จะ emit `OrderModifyRejected` พร้อมเหตุผลจาก venue และปล่อยให้
  order เดิมทำงานต่อ adapter จะ reject การแก้ไขภายในเครื่องหากมีราคา trigger เพราะ AX ไม่มี field
  สำหรับมัน หรือหาก order ยังไม่มี venue order ID
- **การ poll funding rate**: data client จะ poll `GET /funding-rates` สำหรับแต่ละ instrument ที่ subscribe
  ไว้ทุก `funding_rate_poll_interval_mins` โดยขอ lookback เจ็ดวันเพื่อให้หา rate ได้แม้ข้ามวันหยุดสุดสัปดาห์
  และวันหยุดอื่น และจะ emit rate ล่าสุดเฉพาะเมื่อแตกต่างจากค่าที่ emit ครั้งล่าสุด
- **Cancel on disconnect**: ตั้ง `cancel_on_disconnect=True` ใน config ของ execution client
  เพื่อให้ exchange cancel open order ทั้งหมดเมื่อ orders WebSocket หลุดการเชื่อมต่อ
- **อัตราค่าธรรมเนียมของ account**: instrument ไม่มีอัตราค่าธรรมเนียม maker หรือ taker client ที่ยืนยัน
  ตัวตนแล้วจะ resolve อัตราของ account จาก `GET /whoami` และจะ connect ไม่สำเร็จหากการค้นหานั้นล้มเหลว
  execution client จะ cache อัตราแยกตาม AX account โดยแยกจาก instrument และ refresh ทุกครั้งที่ connect
  การ reconnect WebSocket อัตโนมัติจะคงอัตราที่ cache ไว้
- **ค่า commission ของ fill**: fill event แบบ real-time จาก WebSocket ไม่มีข้อมูลค่าธรรมเนียม adapter จะ
  ประมาณค่า commission ทั้งสำหรับ fill event ที่ติดตามอยู่และ fill report ที่ไม่ได้ติดตาม โดยใช้ quantity ของ fill,
  ค่าสัมบูรณ์ของราคา fill, contract multiplier และอัตรา maker หรือ taker ของ account มันใช้ account ID ของ
  trade ก่อน ถ้าไม่มีจะใช้ account ID ของ order หรือถ้าไม่มีทั้งสองจะใช้ account แรกจาก `/whoami` การคำนวณ
  ทำด้วยเลขทศนิยม และปัดเศษเฉพาะค่าธรรมเนียมสุดท้ายเป็น `Money` สกุล USD ซึ่งมี precision เท่ากับ 2

  fill ที่ instrument หรือตารางค่าธรรมเนียมของ account ฉบับสมบูรณ์ยังไม่ถูก cache หรือที่ค่า commission
  แสดงผลไม่ได้ จะถูก log และข้ามไป metadata ของ order ที่ติดตามอยู่ยังคงใช้สำหรับ reconciliation ได้
  การกู้คืนขึ้นอยู่กับการตรวจ reconciliation ที่ตั้งค่าไว้หรือการ query order อย่างชัดเจน fill ที่ถูกข้าม
  จะไม่ trigger REST request ทันที

  reconciliation จะไม่แทนที่ commission ที่นำไปใช้แล้ว ดังนั้นค่าประมาณจาก stream จะคงอยู่แม้ว่าค่าธรรมเนียม
  ที่ venue รายงานใน `/fills` จะต่างออกไป endpoint นั้นให้ค่าธรรมเนียมสำหรับ fill ที่ยังไม่ได้นำไปใช้จาก stream
  adapter แปลงค่าด้วย `Money::from_decimal` เป็น USD ดังนั้นค่าธรรมเนียมที่ต่ำกว่าหนึ่งเซนต์ เช่น
  `0.012188` จะถูกเก็บเป็น `0.01` ค่าธรรมเนียมจาก REST ที่แสดงผลไม่ได้จะทำให้ request fill report,
  request order-status ย้อนหลังที่ต้องใช้ fill และ mass status ล้มเหลว ระหว่าง startup error นั้นจะทำให้
  node เริ่มทำงานไม่ได้
- **ช่วงเวลาของ fill reconciliation**: endpoint `/fills` ต้องระบุช่วงเวลาที่มีขอบเขต และจำกัดช่วงไว้ที่
  เจ็ดวัน request ขอ fill จะครอบคลุมเจ็ดวันโดยค่าเริ่มต้น หรือ lookback ที่ตั้งไว้หากสั้นกว่า ประวัติ order
  ใช้ lookback ที่ตั้งไว้และไม่มีขอบเริ่มต้นโดยค่าเริ่มต้น

  order report ที่เก่ากว่านั้นยังอาจสร้าง fill ที่อนุมานขึ้นโดยมี `commission=None` ประวัติ order
  ไม่มีข้อมูลค่าธรรมเนียม และ adapter ไม่ประมาณค่าธรรมเนียมย้อนหลังจากอัตราปัจจุบันของ account
  การบัญชีของ position จะไม่รวมค่าธรรมเนียมที่ไม่มีข้อมูล ดังนั้น PnL ที่สร้างขึ้นใหม่สำหรับประวัติช่วงนั้น
  จะไม่ครบถ้วน ค่าธรรมเนียมที่รายงานเป็นศูนย์จะยังคงเป็น commission ศูนย์
- **Identity ของ order ของ fill**: AX อาจละ `order_id` สำหรับ block trade และ fill ของ final settlement
  adapter จะสร้าง reconciliation order ID แบบ deterministic จาก `trade_id` สำหรับ record ที่จัดประเภทไว้
  เหล่านั้น และรวม report ของ market order สถานะ `FILLED` ที่ตรงกันไว้ใน mass status ตอน startup
  field การจัดประเภทเป็นตัวเลือกเสริมสำหรับ fill ปกติที่มี `order_id` ถูกต้อง adapter จะปฏิเสธแถวที่ไม่มี
  ทั้ง order ID และการจัดประเภท special fill อย่างชัดเจน และปฏิเสธการจัดประเภทที่ไม่สอดคล้องกัน
- **IOC/FOK ที่ไม่ได้ fill**: AX รายงาน immediate order ที่ไม่ได้ fill ว่าเป็นการหมดอายุ adapter จะ map
  เป็น `OrderCanceled` ให้ตรงกับความหมายของ NautilusTrader
- **Quote ห่างหนึ่ง tick**: tester ในตัวอย่างจะวาง post-only limit ห่างจาก top of book หนึ่ง tick
  quote เหล่านั้นยังอาจ fill ได้ ให้ล้าง position ที่เหลือด้วย
  `cargo run --bin ax-flatten -p nautilus-architect-ax` (`AX_IS_SANDBOX` มีค่าเริ่มต้นเป็น true)
  binary นั้นจะ cancel open order ทั้งหมดของ account จากนั้นปิดทุก position

## การมีส่วนร่วม [#contributing]

:::info
สำหรับฟีเจอร์เพิ่มเติมหรือการมีส่วนร่วมพัฒนา adapter ของ AX Exchange โปรดดู
[contributing guide](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
