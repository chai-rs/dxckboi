---
title: "Databento"
---

NautilusTrader มี adapter สำหรับ API ของ [Databento](https://databento.com/) และสำหรับข้อมูลในรูปแบบ
[Databento Binary Encoding (DBN)](https://databento.com/docs/standards-and-conventions/databento-binary-encoding)
Databento เป็นผู้ให้บริการ market data เท่านั้น adapter จึงไม่มี execution client หากต้องการใช้เพื่อ execution
หรือสร้าง signal สามารถ:

- ใช้คู่กับ sandbox เพื่อจำลอง execution
- route execution ผ่าน adapter อื่น เช่น Interactive Brokers
- ใช้คำนวณ signal จาก asset class ดั้งเดิมสำหรับการเทรด crypto

adapter รองรับ:

- การโหลดข้อมูลย้อนหลังจากไฟล์ DBN และ decode เป็น object ของ Nautilus สำหรับ backtest หรือจัดเก็บใน catalog
- การ request ข้อมูลย้อนหลังที่ decode เป็น object ของ Nautilus สำหรับ live trading และ backtest
- การ subscribe data feed แบบ real-time ที่ decode เป็น object ของ Nautilus สำหรับ live trading และ sandbox

:::tip
[Databento](https://databento.com/signup) ให้เครดิตข้อมูลฟรี $125 สำหรับผู้สมัครใหม่
นำเครดิตไปใช้กับ request ข้อมูลย้อนหลัง หรือหักลบกับแผน subscription ได้ เครดิต
ใช้ร่วมกันภายในทีมและหมดอายุหกเดือนหลังสมัคร

หาก request อย่างระมัดระวัง เครดิตนี้เพียงพอสำหรับการทดสอบและประเมิน ให้ตรวจสอบ
endpoint [metadata.get_cost](https://databento.com/docs/api-reference-historical/metadata/metadata-get-cost)
ก่อน request ข้อมูล
:::

## ภาพรวม [#overview]

adapter ใช้ crate [databento-rs](https://crates.io/crates/databento)
ซึ่งเป็น Rust client library อย่างเป็นทางการของ Databento

:::info
ไม่ต้องติดตั้ง `databento` แยกต่างหาก adapter จะ compile เป็น
static library และ link ให้อัตโนมัติระหว่าง build
:::

adapter class ที่มีให้ใช้ได้แก่:

- `DatabentoDataLoader`: โหลดข้อมูล DBN จากไฟล์
- `DatabentoHistoricalClient`: ดึง market data ย้อนหลังและ instrument definition ผ่าน HTTP API ของ Databento
- `DatabentoLiveClient`: subscribe data feed แบบ real-time ผ่าน raw TCP API ของ Databento
- `DatabentoDataClient`: data client สำหรับ live trading node ซึ่งห่อ historical client และ live client ไว้
- `DatabentoDataClientFactory`: สร้าง data client จาก `DatabentoDataClientConfig` สำหรับ `LiveNode`

:::note
ผู้ใช้ส่วนใหญ่จะตั้งค่า live trading node (อธิบายด้านล่าง) และไม่ได้ทำงานกับ
component เหล่านี้โดยตรง
:::

## ตัวอย่าง [#examples]

ตัวอย่าง Rust อยู่ภายใต้
[`crates/adapters/databento/examples/`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/databento/examples/)
data tester จะ subscribe quote และ trade แบบ live ของ instrument ที่ตั้งค่าไว้เมื่อรัน:

```bash
cargo run --example databento-data-tester --package nautilus-databento
```

## เอกสารของ Databento [#databento-documentation]

ดู [Databento new users guide](https://databento.com/docs/quickstart/new-user-guides)
และใช้อ้างอิงควบคู่กับคู่มือ integration นี้

## Databento Binary Encoding (DBN) [#databento-binary-encoding-dbn]

Databento Binary Encoding (DBN) เป็นรูปแบบการเข้ารหัส message และการจัดเก็บที่รวดเร็วสำหรับ
market data ที่ normalize แล้ว [DBN spec](https://databento.com/docs/standards-and-conventions/databento-binary-encoding)
มี metadata header ที่อธิบายตัวเองได้ และชุดนิยาม struct ที่ตายตัว
ซึ่งกำหนดมาตรฐานการ normalize market data

adapter decode ข้อมูล DBN เป็น object ของ Nautilus โดย Rust decoder ตัวเดียวกันรองรับ:

- การโหลดและ decode ไฟล์ DBN จากดิสก์
- การ decode ข้อมูลย้อนหลังและข้อมูล live แบบ real-time

## Schema ที่รองรับ [#supported-schemas]

NautilusTrader รองรับ schema ของ Databento ต่อไปนี้:

| Databento schema                                                             | Nautilus data type               | คำอธิบาย                     |
| :--------------------------------------------------------------------------- | :------------------------------- | :------------------------------ |
| [MBO](https://databento.com/docs/schemas-and-data-formats/mbo)               | `OrderBookDelta \| TradeTick`    | Market by order (L3)           |
| [MBP_1](https://databento.com/docs/schemas-and-data-formats/mbp-1)           | `(QuoteTick, TradeTick \| None)` | Market by price (L1)           |
| [MBP_10](https://databento.com/docs/schemas-and-data-formats/mbp-10)         | `OrderBookDepth`                 | Market depth (L2)              |
| [BBO_1S](https://databento.com/docs/schemas-and-data-formats/bbo-1s)         | `QuoteTick`                      | best bid/offer ราย 1 วินาที        |
| [BBO_1M](https://databento.com/docs/schemas-and-data-formats/bbo-1m)         | `QuoteTick`                      | best bid/offer ราย 1 นาที        |
| [CMBP_1](https://databento.com/docs/schemas-and-data-formats/cmbp-1)         | `(QuoteTick, TradeTick \| None)` | MBP แบบรวม (consolidated) ข้ามหลาย venue |
| [CBBO_1S](https://databento.com/docs/schemas-and-data-formats/cbbo-1s)       | `QuoteTick`                      | BBO แบบรวม ราย 1 วินาที      |
| [CBBO_1M](https://databento.com/docs/schemas-and-data-formats/cbbo-1m)       | `QuoteTick`                      | BBO แบบรวม ราย 1 นาที      |
| [TCBBO](https://databento.com/docs/schemas-and-data-formats/tcbbo)           | `(QuoteTick, TradeTick)`         | BBO แบบรวมที่สุ่มตาม trade |
| [TBBO](https://databento.com/docs/schemas-and-data-formats/tbbo)             | `(QuoteTick, TradeTick)`         | best bid/offer ที่สุ่มตาม trade   |
| [TRADES](https://databento.com/docs/schemas-and-data-formats/trades)         | `TradeTick`                      | trade tick                    |
| [OHLCV_1S](https://databento.com/docs/schemas-and-data-formats/ohlcv-1s)     | `Bar`                            | bar ราย 1 วินาที                  |
| [OHLCV_1M](https://databento.com/docs/schemas-and-data-formats/ohlcv-1m)     | `Bar`                            | bar ราย 1 นาที                  |
| [OHLCV_1H](https://databento.com/docs/schemas-and-data-formats/ohlcv-1h)     | `Bar`                            | bar ราย 1 ชั่วโมง                    |
| [OHLCV_1D](https://databento.com/docs/schemas-and-data-formats/ohlcv-1d)     | `Bar`                            | bar รายวัน                      |
| [DEFINITION](https://databento.com/docs/schemas-and-data-formats/definition) | `Instrument` (หลายชนิด)     | instrument definition         |
| [IMBALANCE](https://databento.com/docs/schemas-and-data-formats/imbalance)   | `DatabentoImbalance`             | ข้อมูล auction imbalance         |
| [STATISTICS](https://databento.com/docs/schemas-and-data-formats/statistics) | `DatabentoStatistics`            | สถิติของตลาด              |
| [STATUS](https://databento.com/docs/schemas-and-data-formats/status)         | `InstrumentStatus`               | การอัปเดตสถานะตลาด          |

:::note
Databento ยังมีเอกสารของ reference schema อีกหลายชนิด เช่น corporate action,
adjustment factor และ security master data adapter นี้ map เฉพาะ schema
ที่ระบุข้างต้นเป็น data type ของ Nautilus OHLCV รายวันของ Databento ใช้ `ohlcv-1d` และ
record แบบ `ohlcv-eod` ก็ decode เป็น bar รายวันเช่นกัน ราคา settlement อย่างเป็นทางการและ
open interest มาจาก schema `statistics` ไม่ใช่จาก bar แบบ OHLCV
:::

:::info
instrument definition ที่มีค่า `instrument_class` ที่ไม่รองรับ (`'I'` Index, `'B'` Bond) จะถูกข้ามพร้อม
warning แทนที่จะยกเลิกทั้ง batch definition ของ FX spot ที่มีสกุลเงินซึ่ง Nautilus map ไม่ได้ก็จะถูก
ข้ามเช่นกัน definition ของ index ส่วนใหญ่มาจาก Cboe Global Indices Feed (`MAIN.CGIF`) และ publisher ของ OPRA options
`publishers.json` แสดง publisher ID ทุกตัวพร้อม dataset และ venue ของมัน เปิด issue หากต้องการให้
Nautilus รองรับโมเดลของสิ่งเหล่านี้

statistics message ที่มีค่า `stat_type` อยู่นอกช่วงที่รองรับ (1-20) จะ
ถูกข้ามพร้อม warning เช่นกัน รวมถึงค่าเฉพาะ venue
`VenueSpecificVolume1` (10001) และ `VenueSpecificPrice1` (10002) ซึ่งเกิน
ความกว้างของคอลัมน์ Arrow แบบ `u8` ที่ใช้ในการ persist
:::

### ข้อพิจารณาในการเลือก schema [#schema-considerations]

- **TBBO และ TCBBO**: feed ที่สุ่มตาม trade โดยจับคู่ทุก trade กับ BBO
  ณ ขณะ *ก่อน* ที่ trade จะมีผล ใช้เมื่อต้องการ trade ที่ตรงกับ
  quote ณ เวลาเดียวกันโดยไม่ต้องจัดการสอง stream
- **MBP-1 และ CMBP-1 (L1)**: update ระดับ event ที่ปล่อย trade เฉพาะเมื่อเกิด
  trade event เลือกใช้เมื่อต้องการ event tape ของ top-of-book ที่สมบูรณ์ หากต้องการให้ quote และ trade
  ตรงกัน ควรใช้ TBBO หรือ TCBBO
- **MBP-10 (L2)**: 10 ระดับบนสุดพร้อม trade ใช้กับ strategy ที่คำนึงถึง depth
  แต่ไม่ต้องการข้อมูล MBO ทั้งหมด มีจำนวน order ต่อระดับด้วย Databento ให้บริการ
  schema นี้ที่ 10 ระดับเท่านั้น request depth จึงต้องใช้ `depth=10`
- **MBO (L3)**: event ราย order สำหรับการจำลองตำแหน่งในคิวและการสร้าง book
  ขึ้นใหม่อย่างแม่นยำ `subscribe_book_deltas()` ไม่ได้ request snapshot ดังนั้น subscription ของ node
  จะ stream ตั้งแต่จุดที่ subscribe โดยไม่มีสถานะ book เริ่มต้น
  strategy ที่ต้องการ book ที่สมบูรณ์ต้องเริ่มก่อน trading session,
  seed book จาก historical request หรือเรียก `DatabentoLiveClient.subscribe`
  โดยตรงพร้อม `snapshot=True`
- **BBO_1S/BBO_1M และ CBBO_1S/CBBO_1M**: update ของ top-of-book แบบสุ่มตามช่วงเวลา
  คงที่ (1s หรือ 1m) adapter ปล่อยเฉพาะ `QuoteTick` สำหรับ schema เหล่านี้
  ใช้สำหรับ monitoring, spread และ signal ต้นทุนต่ำ ไม่เหมาะ
  กับงานด้าน microstructure
- **TRADES**: trade เท่านั้น หากต้องการบริบทของ quote ควบคู่กับ trade ให้ subscribe ด้วย
  MBP-1 ซึ่งปล่อย `TradeTick` ทุกครั้งที่มี trade event หรือใช้ TBBO หรือ TCBBO
- **OHLCV**: bar ที่ aggregate จาก trade ใช้สำหรับ analytics ใน timeframe ที่สูงขึ้น bar มี timestamp ตอนปิด
  โดย default ตั้ง `bars_timestamp_on_close=False` เพื่อใช้ timestamp ตอนเปิดของช่วง bar รายวันใช้
  `ohlcv-1d` ใช้ `statistics` สำหรับ settlement อย่างเป็นทางการและ open interest
- **Imbalance และ statistics**: ข้อมูลเชิงปฏิบัติการของ venue ที่ไม่มีชนิดข้อมูลเทียบเท่าใน
  Nautilus แบบ built-in subscribe ผ่าน `subscribe_data()` หรือเข้าถึงผ่าน
  historical client, data loader หรือ live client โดยตรง (ดู
  [Imbalance และ statistics](#imbalance-and-statistics))
- **Status**: update สถานะการเทรดของ venue subscribe ผ่าน
  `subscribe_instrument_status`

:::tip
schema แบบรวม (CMBP_1, CBBO_1S, CBBO_1M, TCBBO) จะ aggregate ข้อมูลข้าม
หลาย venue มีประโยชน์สำหรับการวิเคราะห์ข้าม venue
:::

:::info
ดูคู่มือ [Schemas and data formats](https://databento.com/docs/schemas-and-data-formats) ของ Databento ประกอบ
:::

## ความพร้อมใช้งานและการเลือก dataset [#dataset-availability-and-selection]

dataset ID ของ Databento แยกจาก venue identifier ของ Nautilus adapter
รองรับ schema ที่ระบุข้างต้น แต่ dataset แต่ละตัวของ Databento เปิดให้ใช้เพียงบางส่วนเท่านั้น ให้ตรวจสอบ
metadata endpoint ก่อนเพิ่ม dataset หรือ schema ใหม่ลงใน
config ของ live:

```bash
databento_auth="$(printf '%s:' "$DATABENTO_API_KEY" | base64 | tr -d '\n')"

curl --header "Authorization: Basic ${databento_auth}" \
  "https://hist.databento.com/v0/metadata.list_schemas?dataset=EQUS.MINI"

curl --header "Authorization: Basic ${databento_auth}" \
  "https://hist.databento.com/v0/metadata.list_unit_prices?dataset=EQUS.MINI"

curl --header "Authorization: Basic ${databento_auth}" \
  "https://hist.databento.com/v0/metadata.get_cost" \
  --data-urlencode "dataset=EQUS.MINI" \
  --data-urlencode "symbols=AAPL" \
  --data-urlencode "stype_in=raw_symbol" \
  --data-urlencode "schema=bbo-1s" \
  --data-urlencode "start=2026-06-24T14:30:00Z" \
  --data-urlencode "end=2026-06-24T14:31:00Z"
```

สำหรับ dataset ที่นิยมใช้ประเมินสองตัว:

- `GLBX.MDP3` คือ dataset CME Globex MDP 3.0 สำหรับ futures, options on futures และ spread ของ CME, CBOT, NYMEX และ
  COMEX รองรับ MBO, MBP-1,
  MBP-10, TBBO, trades, BBO แบบช่วงเวลา, OHLCV, definitions, statistics และ
  status แต่ไม่มี schema ของหุ้นแบบรวม (`cmbp-1`,
  `cbbo-*` หรือ `tcbbo`)
- `EQUS.MINI` คือ Databento US Equities Mini เป็น dataset top-of-book แบบ aggregate
  ที่ derive มา โดยปกปิด venue ที่เป็นส่วนประกอบ รองรับ MBP-1,
  TBBO, trades, BBO แบบช่วงเวลา, OHLCV และ definitions ไม่รองรับ
  MBO, MBP-10, imbalance, statistics, status หรือ schema แบบรวม

ใช้ `EQUS` เป็น venue ของ Nautilus สำหรับ instrument ของ US Equities Mini เช่น
`AAPL.EQUS`, `MSFT.EQUS` เป็นต้น แผนที่ venue-to-dataset แบบ built-in จะ route
`EQUS` ไปยัง `EQUS.MINI` ส่วนรหัส venue เช่น `XNAS` และ `XNYS` หมายถึง
dataset เฉพาะ venue เว้นแต่จะ override ด้วย `venue_dataset_map`

:::warning
หาก override venue เช่น `XNAS` ไปเป็น `EQUS.MINI` ให้รักษาความสอดคล้องของ
instrument ID ในขั้นถัดไป record ของ Mini มี publisher แบบรวม `EQUS`
และการ decode จากไฟล์หรือข้อมูลย้อนหลังที่ไม่ระบุ `instrument_id` อย่างชัดเจนจะได้
identifier แบบ `*.EQUS`
:::

ค่าใช้จ่ายขึ้นกับ schema, symbol และช่วงเวลา สำหรับการสำรวจ ให้เริ่มด้วย
ช่วงเวลาแคบ ๆ กับ `definition`, `bbo-1s`, `bbo-1m` หรือ `trades` และเรียก
`metadata.get_cost` ก่อนดึงข้อมูล time series ย้อนหลัง หลีกเลี่ยงการ subscribe
quote และ trade ซ้ำซ้อนเมื่อ schema แบบรวมอย่าง `mbp-1` หรือ `tbbo`
มีข้อมูลที่ strategy ต้องการอยู่แล้ว

## Subscription และ request [#subscriptions-and-requests]

method สำหรับ subscribe ของ Nautilus map กับ schema ของ Databento ดังนี้:

| Method สำหรับ subscribe ของ Nautilus    | Schema default | Schema ของ Databento ที่ใช้ได้                                                  | Nautilus data type                          |
| :------------------------------ | :------------- | :--------------------------------------------------------------------------- | :------------------------------------------ |
| `subscribe_instrument()`        | `definition`   | `definition`                                                                 | `Instrument`                                |
| `subscribe_quotes()`            | `mbp-1`        | `mbp-1`, `bbo-1s`, `bbo-1m`, `cmbp-1`, `cbbo-1s`, `cbbo-1m`, `tbbo`, `tcbbo` | `QuoteTick`                                 |
| `subscribe_trades()`            | `trades`       | `trades`, `tbbo`, `tcbbo`, `mbp-1`, `cmbp-1`                                 | `TradeTick`                                 |
| `subscribe_book_deltas()`       | `mbo`          | `mbo`                                                                        | `OrderBookDeltas`                           |
| `subscribe_book_depth()`        | `mbp-10`       | `mbp-10`                                                                     | `OrderBookDepth`                            |
| `subscribe_bars()`              | none           | `ohlcv-1s`, `ohlcv-1m`, `ohlcv-1h`, `ohlcv-1d`                               | `Bar`                                       |
| `subscribe_data()`              | none           | `statistics`, `imbalance`                                                    | `DatabentoStatistics`, `DatabentoImbalance` |
| `subscribe_instrument_status()` | `status`       | `status`                                                                     | `InstrumentStatus`                          |

ส่ง schema ที่ไม่ใช่ค่า default ผ่านพารามิเตอร์ `schema` ของ subscription มีเพียง
`subscribe_quotes()` และ `subscribe_trades()` ที่ให้เลือกได้ `subscribe_bars()` ใช้
schema ตาม bar aggregation และ `subscribe_data()` ใช้ตามชื่อ data type
(`DatabentoStatistics` หรือ `DatabentoImbalance`) method อื่น ๆ ใช้ schema เดียวที่ระบุไว้
เสมอ historical request ที่คู่กัน คือ `request_quotes()` และ `request_trades()` รับค่า `schema`
และค่า default แบบเดียวกัน

`request_instrument()`, `request_quotes()` และ `request_trades()` รับรายการ identifier ได้ adapter
จัดกลุ่ม identifier ตาม dataset และเก็บผลของกลุ่มที่สำเร็จไว้แม้กลุ่มอื่นจะล้มเหลว ให้ตรวจสอบ
`on_data_response()` เพื่อดูสถานะการเสร็จสิ้นและเหตุผลของความล้มเหลว request ของ quote และ trade ที่มี
หลาย ID จะไม่รับค่า limit ของจำนวน record

request ของ book และ bar ต้องมีสมาชิกในรายการเพียงหนึ่งตัว `request_book_deltas()` ที่ใช้ parent symbol อาจ return
book payload หลายชุดภายใต้ correlation ID เดียวกัน โดยการเสร็จสิ้นจะตามหลัง payload สุดท้าย

historical request ของ actor และ strategy รวมขอบเขตทั้งสองด้านของ `ts_init` request ของ bar เลือก bar
ตามเวลาปิดแม้จะตั้ง `bars_timestamp_on_close=False` ไว้ก็ตาม

:::warning
คอลัมน์ "Schema ของ Databento ที่ใช้ได้" แสดงตัวเลือกที่ adapter รองรับสำหรับ
method สำหรับ subscribe นั้น ๆ dataset ที่เลือกต้องรองรับ
schema นั้นด้วย เช่น `EQUS.MINI` ให้บริการ `mbo`, `mbp-10`, `statistics` หรือ
`status` ไม่ได้
:::

### Subscription ของ instrument definition [#instrument-definition-subscriptions]

`subscribe_instrument()` จะ stream definition message ซึ่งช่วยเติม price precision map ของ live ด้วย

### รายการ subscription [#subscription-lists]

instrument subscription ที่ implement แล้วรับ ID เดียวหรือรายการที่ไม่ว่างผ่าน `instrument_id`
external bar subscription รับ bar type แบบเต็มผ่าน `bar_type`

#### การรวม batch ของ quote และ trade [#quote-and-trade-batching]

quote และ trade ใช้หนึ่ง subscription ต่อ dataset adapter ตรวจสอบทุก ID ก่อน dispatch
และใช้ `price_precision` กับแต่ละ symbol

- การ enqueue ของ dataset ที่ล้มเหลวจะถูก log และไม่หยุด dataset ถัดไป
- dataset ที่รับไปแล้วจะไม่ถูก rollback client รายงานเฉพาะ ID ที่ถูกรับไปแล้วอย่างแม่นยำ
- client adapter ติดตามสมาชิกที่ถูกรับแล้ว และจะส่งเฉพาะ ID ที่ขาดไปเมื่อมี subscribe
  command อื่นมาถึง ไม่มีการ retry อัตโนมัติ

:::note
การเรียก subscribe ซ้ำบน actor ตัวเดิมจะไม่ส่ง command ใดออกไปตราบที่ actor นั้นยังถือ
subscription ภายในของตนอยู่ หากต้องการ retry ให้ปล่อย subscription ภายในนั้นก่อน หรือ subscribe จากเจ้าของอื่น
:::

#### Subscription อื่น ๆ [#other-subscriptions]

hook ของ instrument อื่นที่ implement แล้วและ hook ของ external bar จะวนทำงานกับสมาชิกทีละตัวภายในเครื่อง:

- กฎของ dataset และ schema ที่มีอยู่ใช้กับสมาชิกแต่ละตัว
- ความล้มเหลวของสมาชิกจะรายงานชุดย่อยที่ถูกรับแบบ synchronous อย่างแม่นยำ
- bar type แบบเต็มบน instrument เดียวกันยังคงแยกจากกัน

การ unsubscribe ทีละรายการเป็น no-op สำหรับสมาชิกทุกตัวตามที่ระบุไว้ ให้หยุด live session เพื่อลบ subscription

### Subscription ของ quote (MBP และ L1) [#quote-subscriptions-mbp-and-l1]

ตัวเลือกของ `subscribe_quotes()` ได้แก่:

- ค่า default เป็น quote แบบ MBP-1 (ปล่อย trade ด้วยเมื่อเกิด trade event)
- subscription เดียวสำหรับ raw symbol หลายตัวใน `GLBX.MDP3` (เช่น `ESZ6.GLBX` และ `NQZ6.GLBX`) พร้อมพารามิเตอร์ `price_precision`
- ระบุ schema `mbp-1` อย่างชัดเจน
- snapshot ของ BBO ราย 1 วินาทีด้วย `bbo-1s` (adapter ปล่อยเฉพาะ `QuoteTick`)
- quote แบบรวมข้ามหลาย venue ด้วย `cbbo-1s` (หรือ `cmbp-1` สำหรับ MBP แบบรวม)
- BBO ที่สุ่มตาม trade ด้วย `tbbo` ซึ่งจะได้ทั้ง `QuoteTick` และ `TradeTick` บน message bus

### Subscription ของ trade [#trade-subscriptions]

การเรียก trade รับรายการ instrument ตามที่อธิบายใน
[การรวม batch ของ quote และ trade](#quote-and-trade-batching)

ตัวเลือกของ `subscribe_trades()` ได้แก่:

- trade tick เท่านั้น (ค่า default)
- trade จาก feed `mbp-1` (เฉพาะเมื่อเกิด trade event)
- ข้อมูลที่สุ่มตาม trade ด้วย `tbbo` (ให้ quote ณ เวลาที่เกิด trade ด้วย)

### Subscription ของ order book delta (MBO และ L3) [#order-book-deltas-subscriptions-mbo-and-l3]

subscribe update ของ order book แบบเต็ม (market by order) ด้วย `subscribe_book_deltas()` และ book type `L3_MBO`
ซึ่งใช้ schema MBO delta จะ stream ตั้งแต่จุดที่ subscribe โดยไม่มี snapshot ของ book เริ่มต้น

### Subscription ของ instrument status [#instrument-status-subscriptions]

subscribe update สถานะการเทรดของ venue ด้วย `subscribe_instrument_status()`

### Subscription ของ depth, bar และ custom data [#depth-bar-and-custom-data-subscriptions]

`subscribe_book_depth()` stream depth snapshot แบบ `mbp-10` ของ instrument หนึ่งตัว Databento ให้บริการ
depth ที่ 10 เท่านั้น ดังนั้น depth ที่ระบุชัดเจนเป็นค่าอื่นนอกจาก 10 จะถูกปฏิเสธ bar aggregation ใน
`BarType` เป็นตัวเลือก schema OHLCV สำหรับ `subscribe_bars()` (`ohlcv-1s`, `ohlcv-1m`, `ohlcv-1h` หรือ
`ohlcv-1d`) step ต้องเป็น 1 และ price type ต้องเป็น `LAST` ซึ่งเป็น price type เดียวที่ decoder
ปล่อยออกมา `subscribe_data()` stream record แบบ `DatabentoStatistics` หรือ `DatabentoImbalance` ของ
instrument ID ที่ระบุเป็น identifier ของ data type และจะปฏิเสธ data type ที่มี metadata record จะถูก
publish พร้อม identifier นั้น ดังนั้น subscriber จะได้รับเฉพาะ record ของ instrument ของตนเอง

subscription ทั้งสามแบบนี้รับเฉพาะ raw symbol เช่น `ESM4.GLBX` เท่านั้น record จะ resolve ไปยัง
สัญญาที่อยู่เบื้องหลัง ดังนั้น continuous symbol (`ES.c.0`) และ parent symbol (`ES.FUT`) จะถูกปฏิเสธ แทนที่จะ
ถูก publish บน topic ที่ subscriber ไม่ได้ฟังอยู่

subscription แบบ live ของ Databento ไม่สามารถลบทีละรายการได้ ดังนั้น command `unsubscribe_*`
ที่คู่กันจะถูกละเว้น

### Historical request สำหรับ depth และ bar [#historical-requests-for-depth-and-bars]

depth แบบ MBP-10 และ bar แบบ OHLCV สามารถ request ย้อนหลังได้ bar aggregation ใน
`BarType` เป็นตัวเลือก schema OHLCV (`ohlcv-1s`, `ohlcv-1m`, `ohlcv-1h` หรือ `ohlcv-1d`) และ request ของ depth
ใช้ `mbp-10` (Databento ให้บริการเฉพาะ `depth=10`)

## Instrument ID และ symbology [#instrument-ids-and-symbology]

market data ของ Databento มีฟิลด์ `instrument_id` แบบตัวเลข ซึ่งปกติ publisher จะเป็นผู้กำหนด
หาก publisher ไม่ได้ให้มา Databento จะสังเคราะห์ขึ้นเอง Databento รับประกันเพียงว่า ID นี้ไม่ซ้ำ
ภายในวันเดียวกัน ซึ่งต่างจาก `InstrumentId` ของ Nautilus ที่เป็น string ของ symbol + venue คั่นด้วย
จุด: `"{symbol}.{venue}"`

decoder map `raw_symbol` ของ Databento ไปเป็น `symbol` ของ Nautilus publisher
ID จะ map ไปยัง venue default ของ Nautilus ผ่าน `publishers.json` metadata ของ `InstrumentId`
ใน subscription ยังช่วย seed แผนที่ symbol-to-venue ได้ก่อนที่ market data
จะมาถึง

dataset ID เป็นไปตาม
[แบบแผนการตั้งชื่อ dataset](https://databento.com/docs/api-reference-historical/basics/datasets) ของ Databento
ซึ่งแยกจากรหัส venue ใน `InstrumentId` ของ Nautilus

สำหรับ historical request และ live subscription adapter จะส่งส่วน symbol ของ Nautilus
ของแต่ละ `InstrumentId` เป็น symbol ของ Databento และอนุมาน
`stype_in` จาก string นั้น:

- symbol ที่ลงท้ายด้วย `.FUT` หรือ `.OPT` ใช้ parent symbology ของ Databento เช่น
  `ES.FUT.XCME`
- symbol สามส่วนที่ส่วนสุดท้ายเป็นตัวเลขใช้ continuous symbology เช่น
  `ES.c.0.GLBX`
- symbol ที่เป็นตัวเลขทั้งหมดใช้ symbology แบบ `instrument_id` ของ Databento
- symbol อื่น ๆ ทั้งหมดใช้ raw symbol symbology เช่น `ESZ6.XCME` หรือ
  `AAPL.EQUS`

symbol ทั้งหมดใน request หรือ subscription เดียวกันต้องใช้ symbology ชนิดเดียวกัน
รวม `AAPL.EQUS` กับ `MSFT.EQUS` หรือ `ES.FUT.XCME` กับ `NQ.FUT.XCME` ได้ แต่
อย่าผสม raw symbol กับ parent symbol ใน request ของ Databento เดียวกัน

สำหรับ CME Globex MDP 3.0 (`GLBX.MDP3`) ค่า default ของ publisher จะ map ไปยัง venue `GLBX`
เมื่อตั้ง `use_exchange_as_venue=True` definition message สามารถ override
`GLBX` ด้วย MIC ของ exchange ของ instrument ได้:

- `CBCM`: inter-exchange spread ระหว่าง XCME-XCBT
- `NYUM`: inter-exchange spread ระหว่าง XNYM-DUMX
- `XCBT`: Chicago Board of Trade (CBOT)
- `XCEC`: Commodities Exchange Center (COMEX)
- `XCME`: Chicago Mercantile Exchange (CME)
- `XFXS`: CME FX Link spread
- `XNYM`: New York Mercantile Exchange (NYMEX)

:::info
MIC ของ venue อื่น ๆ อยู่ในฟิลด์ `venue` ของ response จาก
endpoint [metadata.list_publishers](https://databento.com/docs/api-reference-historical/metadata/metadata-list-publishers)
:::

## Timestamp [#timestamps]

ข้อมูลของ Databento มีฟิลด์ timestamp ดังนี้:

- `ts_event`: timestamp ที่ matching engine ได้รับ หน่วยนาโนวินาทีนับจาก UNIX epoch
- `ts_in_delta`: timestamp ที่ matching engine ส่ง หน่วยนาโนวินาทีก่อน `ts_recv`
- `ts_recv`: timestamp ที่ capture server ได้รับ หน่วยนาโนวินาทีนับจาก UNIX epoch
- `ts_out`: timestamp ที่ Databento ส่ง (เฉพาะ live)

ข้อมูลของ Nautilus ต้องมี timestamp อย่างน้อยสองตัว (ตาม contract ของ `Data`):

- `ts_event`: UNIX timestamp (นาโนวินาที) ที่ data event เกิดขึ้น
- `ts_init`: UNIX timestamp (นาโนวินาที) ที่ data instance ถูกสร้าง

schema ที่คล้าย quote และ trade จะ map `ts_recv` ของ Databento ไปเป็น `ts_event` ของ Nautilus เพราะเชื่อถือได้มากกว่าและ
เพิ่มขึ้นเสมอต่อ symbol ของ Databento bar ใช้ timestamp ของช่วง bar ใน DBN
`bars_timestamp_on_close` ควบคุมว่า bar ของ Nautilus จะใช้ timestamp ตอนเปิดหรือปิดของช่วง
`InstrumentStatus` ใช้ `ts_event` จาก record header ของ DBN ส่วน `DatabentoImbalance` และ `DatabentoStatistics`
คงฟิลด์ timestamp ของ Databento ไว้ เพราะเป็น type เฉพาะของ adapter

:::info
ดูรายละเอียดในเอกสารของ Databento เหล่านี้:

- [Databento standards and conventions - timestamps](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#timestamps)
- [Databento timestamping guide](https://databento.com/docs/architecture/timestamping-guide)

:::

## Data type [#data-types]

section นี้แสดงการ map schema ของ Databento ไปเป็น data type ของ Nautilus

:::info
ดู [schemas and data formats](https://databento.com/docs/schemas-and-data-formats) ของ Databento
:::

### Instrument definition [#instrument-definitions]

Databento ใช้ schema เดียวสำหรับ instrument ทุก class decoder จะ map แต่ละ class
ไปยัง type `Instrument` ของ Nautilus ที่เหมาะสม

| Instrument class ของ Databento | Code | Instrument type ของ Nautilus |
| -------------------------- | ---- | ------------------------ |
| Stock                      | `K`  | `Equity`                 |
| Future                     | `F`  | `FuturesContract`        |
| Call                       | `C`  | `OptionContract`         |
| Put                        | `P`  | `OptionContract`         |
| Future spread              | `S`  | `FuturesSpread`          |
| Option spread              | `T`  | `OptionSpread`           |
| Mixed spread               | `M`  | `OptionSpread`           |
| FX spot                    | `X`  | `CurrencyPair`           |
| Index                      | `I`  | ยังไม่รองรับ        |
| Bond                       | `B`  | ยังไม่รองรับ        |

### การแก้ไขเวลาหมดอายุของ option [#option-expiration-correction]

option definition ของ OPRA (dataset `OPRA.PILLAR`) ระบุเวลาหมดอายุด้วยความละเอียดระดับวัน: เวลาของวัน
ถูกตั้งเป็นศูนย์ที่เที่ยงคืน UTC ดังนั้น option ที่หมดอายุเวลา 16:00 ตามเวลานิวยอร์กจะถูกประทับเวลาเป็น
ค่ำวันก่อนหน้าตามเวลานิวยอร์ก matching engine จึงถือว่าสัญญาหมดอายุก่อน trading session สุดท้าย
โดย default loader จะแก้เวลาหมดอายุของ OPRA ที่เป็นเที่ยงคืน UTC ดังกล่าวเป็น 16:00 ตามเวลานิวยอร์ก โดยไม่แตะ
dataset อื่นใด และไม่แตะเวลาหมดอายุที่มีเวลาระหว่างวัน เช่น CME Globex

override ค่า default หรือกำหนดเวลาต่อ underlying ได้ด้วย `expiration_overrides` ซึ่ง map dataset ไปยัง
mapping ของ underlying symbol กับเวลา โดย key สงวน `default` ใช้กำหนดเวลาของทั้ง dataset
(เช่น `"OPRA.PILLAR": {"default": "16:00", "SPX": "09:30"}`)

เวลาใช้รูปแบบ `HH:MM` หรือ `HH:MM:SS` ตาม timezone ท้องถิ่นของ exchange (นิวยอร์กสำหรับ OPRA) ปรับได้เฉพาะ dataset ที่มี
กฎการแก้ไขแบบ built-in และ `OPRA.PILLAR` เป็น dataset เดียวที่มี dataset ที่ไม่รู้จักหรือไม่มีกฎ
เช่น `GLBX.MDP3` จะทำให้เกิด error การแก้ไขอิงตาม underlying ของ option จึง
ไม่สามารถแยก series ที่ใช้ underlying ร่วมกันแต่ settle ต่างเวลากันได้ (เช่น SPX ที่ settle ช่วงเช้า
กับ SPXW ที่ settle ช่วงบ่าย) ให้ตั้งเวลาที่ตรงกับสัญญาที่กำลังโหลด

### Price precision [#price-precision]

ราคาดิบของ Databento เป็นจำนวนเต็มแบบ fixed-point ที่ scale ด้วย 1e-9 adapter derive
price precision จาก tick size ของ instrument ใน definition message

สำหรับ live feed feed handler จะดูแล precision map ต่อ instrument ซึ่งเติมข้อมูล
จาก record `InstrumentDefMsg` เมื่อมาถึง handler ของ market data จะ resolve
precision ตามลำดับนี้:

1. precision จาก `InstrumentDefMsg` ที่เคยเห็นแล้วสำหรับ `instrument_id` ของ record ของ Databento
2. precision ที่ส่งมากับ subscription ซึ่ง match กับ `instrument_id` ของ record ผ่าน symbol mapping message
3. precision ที่ส่งมากับ subscription ซึ่ง match โดยตรงกับ symbol ของ Nautilus
4. precision default ของ USD คือ 2

ระบุ precision ด้วยพารามิเตอร์ `price_precision` บน `subscribe_quotes()` หรือ
`subscribe_trades()` หรือด้วย `price_precisions` บน live client โดยตรง method สำหรับ subscribe อื่น
ไม่อ่านพารามิเตอร์นี้ ขั้นที่ 2 และ 3 ใช้ override เดียวกันในสองรูปแบบ: การ match `instrument_id` ของ record
หลัง symbol mapping ทำให้ subscription แบบ parent, continuous และ symbology อื่นที่ไม่ใช่ raw
ใช้ override ได้ก่อนที่ definition metadata จะมาถึง

**Instrument definition ต้องมาถึงก่อน market data** เพื่อให้ precision ถูกต้องสำหรับ
instrument ที่มี tick size ไม่มาตรฐาน (เช่น treasury futures ที่มี tick เป็นเศษส่วน
อย่าง 1/256) ให้ subscribe instrument definition (schema `definition` ของ Databento)
ก่อนหรือพร้อมกับ subscription ของ market data

สำหรับ historical request และการโหลดจากไฟล์ precision จะถูก resolve ต่อ
record ตามลำดับนี้:

1. อาร์กิวเมนต์ `price_precision` ที่ระบุชัดเจนในการเรียก
2. cache ต่อ symbol ที่เติมข้อมูลจากการโหลด definition (`load_instruments`
   บน file loader, `get_range_instruments` บน historical client) หรือ
   จากการเรียก `set_price_precision(symbol, precision)` อย่างชัดเจน

data client จะดึง definition ของ instrument ที่ request เมื่อไม่มี
แหล่งใดให้ precision การค้นหาเริ่มที่เที่ยงคืน UTC ของวันเริ่มต้นของ request เพื่อรวม
definition snapshot ตอนเที่ยงคืนของ Databento ส่วน request ของ market data
ยังใช้ช่วงเวลาเดิม

precision ที่ resolve ไม่ได้จะไม่ default เป็น precision ของ USD การเรียกของ historical client และ loader จะ return error
ส่วน request ของ data client จะ log error และ return response ว่าง

:::tip
เรียก `subscribe_instrument()` สำหรับแต่ละ instrument ตอนเริ่ม strategy เพื่อให้ definition message เติมข้อมูลลงใน
precision map ของ live feed handler จะเก็บ override ของ `price_precision` ต่อ symbol ไว้ตลอด session ของ dataset
override บน subscription ของ quote หรือ trade หนึ่งตัวจะครอบคลุม order book delta, depth, bar,
statistics และ imbalance ของ symbol นั้นด้วย `InstrumentStatus` ไม่มีราคาจึงไม่ต้องใช้ precision
:::

### MBO (market by order) [#mbo-market-by-order]

MBO เป็นข้อมูลที่ละเอียดที่สุดจาก Databento แทน depth ของ order book
แบบเต็ม บาง message มีข้อมูล trade ด้วย decoder จะสร้าง `OrderBookDelta`
และอาจสร้าง `TradeTick` ด้วย live client จะปล่อย `TradeTick` หลังจากที่
instrument ได้รับ delta แรกแล้วเท่านั้น

live client จะ buffer MBO message จนกว่าจะมี record ที่มี flag `F_LAST` ซึ่งปิด match event แล้วจึง
ส่ง container `OrderBookDeltas` หนึ่งตัวไปยัง handler record ที่ decode แล้วไม่ได้ delta (fill attribution,
status action หรือ trade) ก็ยังอาจมี `F_LAST` ได้ client จึงยึดตาม raw flag โดยไม่ขึ้นกับ
payload ที่ decode ได้ มิฉะนั้น event ที่ไม่สมบูรณ์จะค้างอยู่และถูกรวมเข้ากับ event ถัดไป เมื่อ MBO นำ
trade ของ instrument มาด้วย (ดู [สถาปัตยกรรมของ real-time client](#real-time-client-architecture)) trade ที่
ปิด event จะตามหลัง container ที่ถูก flush

record แบบ snapshot (`F_SNAPSHOT`) จะสะสมใน buffer เดียวกันและ flush พร้อมขอบเขต event แรกที่ไม่ใช่ snapshot
ดังนั้น snapshot จะไปถึง handler เป็น container `OrderBookDeltas` ตัวเดียวแทนที่จะเป็น delta
ทีละตัว เมื่อ subscription มี anchor `start` สำหรับ replay client จะกันประวัติที่ replay ไว้นอก live
stream จนกว่า timestamp ของ event จะผ่านเวลาเริ่ม session มันจะถือ delta ที่ replay ไว้และ flush พร้อม
live event แรกนั้น และทิ้ง trade ที่ replay

### MBP-1 (market by price, top-of-book) [#mbp-1-market-by-price-top-of-book]

MBP-1 แทน quote และ trade ของ top-of-book บาง message มีข้อมูล trade
decoder จะสร้าง `QuoteTick` และสร้าง `TradeTick` ด้วยเมื่อ message
เป็น trade

### TBBO และ TCBBO (top-of-book พร้อม trade) [#tbbo-and-tcbbo-top-of-book-with-trades]

TBBO และ TCBBO ให้ทั้งข้อมูล quote และ trade ในแต่ละ message ทั้งสอง schema
ปล่อย `TradeTick` หนึ่งตัวต่อ message พร้อม `QuoteTick` ซึ่งมีประสิทธิภาพกว่าการ subscribe
quote และ trade แยกกัน quote จะถูกข้ามเมื่อราคา bid หรือ ask
ไม่ได้กำหนดไว้ TCBBO ให้ข้อมูลแบบรวมข้ามหลาย venue

#### การ derive trade ID (CMBP-1 และ TCBBO) [#trade-id-derivation-cmbp-1-and-tcbbo]

schema CMBP-1 และ TCBBO ไม่ได้ publish trade identifier ดั้งเดิม decoder จึง derive
`TradeId` แบบ deterministic ด้วยการ hash แบบ FNV-1a จาก instrument ID, `ts_event`, `ts_recv`, ราคา, ขนาด และ aggressor side ของ
trade venue event เดียวกันจะได้ trade ID เดียวกันข้ามการ replay การ dedup ในขั้นถัดไปจึงยังทำงานได้ trade
สองตัวที่แตกต่างกันในเชิงตรรกะแต่มีฟิลด์เหมือนกันจะชนกัน ซึ่งสอดคล้องกับการที่ venue เองก็แยก
มันไม่ได้

### OHLCV (bar aggregate) [#ohlcv-bar-aggregates]

Databento ประทับ timestamp ของ bar message ที่ **ตอนเปิด** ของช่วง โดย default
decoder จะ normalize `ts_event` ของ bar ไปเป็น **ตอนปิด** ของ bar คือ
`ts_event` เดิมบวกด้วยช่วงเวลา `ts_init` ใช้เวลาที่ได้รับแบบ live หรือเวลาปิด
สำหรับการโหลดข้อมูลย้อนหลังและจากไฟล์เมื่อไม่ได้ระบุ init timestamp ไว้ชัดเจน
ตั้ง `bars_timestamp_on_close=False` เพื่อประทับ `ts_event` ของ bar ด้วย
เวลาเปิดของช่วง

### Imbalance และ statistics [#imbalance-and-statistics]

schema `imbalance` และ `statistics` ไม่มีชนิดข้อมูลเทียบเท่าใน Nautilus แบบ built-in
adapter นิยาม `DatabentoImbalance` และ `DatabentoStatistics` ไว้ใน Rust

subscribe record แบบ live ด้วย `subscribe_data()` และ `DataType` ที่ชื่อ `DatabentoStatistics` หรือ
`DatabentoImbalance` ซึ่งมี identifier เป็น instrument ID ส่วน historical request และไฟล์ใช้:

- `DatabentoDataLoader.load_imbalance` และ `load_statistics` สำหรับไฟล์ DBN
- `DatabentoHistoricalClient.get_range_imbalance` และ `get_range_statistics` สำหรับช่วงข้อมูลย้อนหลัง

`DatabentoLiveClient.subscribe` ด้วย schema `imbalance` หรือ `statistics` ก็ stream ข้อมูลเหล่านี้
นอก node ได้เช่นกัน

method `get_range_*` ทั้งสองเป็นแบบ asynchronous ให้ใช้ endpoint
[`metadata.get_cost`](https://databento.com/docs/api-reference-historical/metadata/metadata-get-cost)
ของ Databento Historical ก่อนดึงข้อมูลย้อนหลังจริง

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python (request `statistics` ของ parent symbol `ES.FUT`) จึงตัดออก

historical client ที่สร้างใหม่จะไม่มี precision ใน cache ดังนั้น request จึงต้องมี `price_precision` หรือเรียก
`get_range_instruments` สำหรับช่วงเวลาเดียวกันก่อน มิฉะนั้น record แรกที่ decode จะทำให้ request ถูกยกเลิก
parent symbol เช่น `ES.FUT` ไม่สามารถ seed ด้วย `set_price_precision` ได้ เพราะ record จะ resolve ไปยัง
symbol ของสัญญาแต่ละตัวที่อยู่เบื้องหลัง parent

### การเข้ารหัส Arrow สำหรับ imbalance และ statistics [#arrow-encoding-for-imbalance-and-statistics]

ทั้งสอง type implement การ encode และ decode แบบ Arrow record batch โดย
module `nautilus_databento::arrow` เปิดให้ใช้ใน Rust ภายใต้ feature flag `arrow`:

```rust
use nautilus_databento::arrow::imbalance::{
    decode_imbalance_batch,
    imbalance_to_arrow_record_batch,
};

let batch = imbalance_to_arrow_record_batch(&imbalances)?;

let metadata = batch.schema().metadata().clone();
let decoded = decode_imbalance_batch(&metadata, &batch)?;
```

module `statistics` ใช้รูปแบบเดียวกันด้วย
`decode_statistics_batch` และ `statistics_to_arrow_record_batch`

:::info
ทั้งสอง type ไม่ได้ลงทะเบียนกับ custom data encoder ของ `ParquetDataCatalog` ดังนั้น `write_custom_data` และ
`query_custom_data` จะล้มเหลวด้วย error ว่า type ไม่ได้ลงทะเบียน ทั้งสอง type ไม่ stream ผ่าน `BacktestNode` หรือ
`BacktestEngine` สำหรับงานวิจัยที่ใช้ข้อมูล imbalance หรือ statistics ให้โหลดหรือ request record แล้วประมวลผล
โดยตรง
:::

## ข้อพิจารณาด้านประสิทธิภาพ [#performance-considerations]

ทางเลือกสองแบบสำหรับ backtest ด้วยข้อมูล DBN:

- เก็บข้อมูลเป็นไฟล์ DBN (`.dbn.zst`) และ decode เป็น object ของ Nautilus ทุกครั้งที่รัน
- แปลงไฟล์ DBN เป็น object ของ Nautilus ครั้งเดียว แล้วเขียนลง data catalog (รูปแบบ Nautilus Parquet)

DBN decoder เป็น Rust ที่ optimize แล้ว แต่การเขียนลง catalog ครั้งเดียวจะให้
ประสิทธิภาพ backtest ดีที่สุด

[DataFusion](https://arrow.apache.org/datafusion/) stream ข้อมูล Nautilus Parquet
จากดิสก์ด้วย throughput สูง เร็วกว่าการ decode DBN ทุกครั้งที่รันอย่างน้อยหนึ่งระดับขนาด (order of magnitude)

:::note
throughput ของการ decode และของ client ที่วัดได้สำหรับ adapter นี้บันทึกไว้ใน
[`crates/adapters/databento/benches/BENCHMARKS.md`](https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/adapters/databento/benches/BENCHMARKS.md)
พร้อมคำสั่งสำหรับวัดซ้ำ ตัวเลขสัมบูรณ์แตกต่างกันไปตามเครื่อง จึง
มีเพียงผลต่างบนเครื่องเดียวกันเท่านั้นที่มีความหมาย
:::

สำหรับข้อมูล live การส่งข้อมูลที่ decode แล้วจาก feed handler ไปยัง Nautilus ไม่มีขอบเขตจำกัดโดยเจตนา เพื่อป้องกัน
ไม่ให้ consumer ที่ช้าทำให้เส้นทาง feed ค้าง process ที่อยู่ภายใต้แรงกดดันด้านหน่วยความจำควรล้มเหลวแทนที่จะ block การ decode
ข้อมูล live

## การโหลดข้อมูล DBN [#loading-dbn-data]

`DatabentoDataLoader` decode ไฟล์ DBN เป็น object ของ Nautilus โดยตรง มี method สำหรับ
output type ที่รองรับแต่ละชนิด ได้แก่ `load_instruments`, `load_order_book_deltas`,
`load_order_book_depth`, `load_quotes`, `load_trades`, `load_bars`, `load_status`,
`load_imbalance` และ `load_statistics`

ให้ส่งไฟล์ metadata ของ publisher (เช่น `publishers.json`) เมื่อไฟล์นั้นไม่ได้อยู่ข้าง executable ที่กำลังรัน

เมื่อเขียนลง `ParquetDataCatalog` ให้เขียนข้อมูล definition ก่อน market data เพราะ catalog
ต้องมี instrument ก่อนจึงจะเขียน record ของ instrument นั้นได้

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

ใช้ method เฉพาะ schema สำหรับไฟล์ที่ schema ไม่ใช่ค่า default ของ output type นั้น:

- `load_bbo_quotes` สำหรับ quote แบบ BBO ตามช่วงเวลา
- `load_cmbp_quotes` สำหรับ quote แบบ CMBP-1
- `load_cbbo_quotes` สำหรับ quote แบบ CBBO
- `load_tbbo_trades` สำหรับ trade แบบ TBBO
- `load_tcbbo_trades` สำหรับ trade แบบ TCBBO

เรียก `schema_for_file` เพื่ออ่าน schema ของไฟล์จาก metadata header ของ DBN เมื่อต้องเลือก method
ของ loader

อาร์กิวเมนต์ `instrument_id` และ `price_precision` (optional) ใช้ข้ามการค้นหา symbology หรือ precision เมื่อ
ทราบค่าเหล่านั้นอยู่แล้ว loader ของ bar ยังรับ `timestamp_on_close` ด้วย

## สถาปัตยกรรมของ real-time client [#real-time-client-architecture]

`DatabentoDataClient` ห่อ adapter class อื่น ๆ ของ Databento ไว้ มันสร้าง live feed handler หนึ่งตัวต่อ
dataset เมื่อมี subscription แรกของ dataset นั้น และทุก schema ของ dataset นั้นใช้ handler ร่วมกัน
handler ทำงานด้วย async task เดียวที่แข่งระหว่าง record ถัดไปจาก gateway กับ command ถัดไปจาก engine ดังนั้น
subscription ที่ตามมาภายหลังจะไปถึง session ที่ทำงานอยู่ได้โดยไม่ต้อง reconnect

schema `trades`, `mbp-1`, `tbbo`, `cmbp-1`, `tcbbo` และ `mbo` ล้วนนำ trade ของ instrument มาด้วย เมื่อ
instrument เดียวกันถูก subscribe บนหลาย schema เช่น quote บน `mbp-1` คู่กับ trade บน `trades`
handler จะส่ง trade แต่ละตัวเพียงครั้งเดียว schema แรกที่ส่ง trade ของ instrument มาจะเป็นผู้นำ trade
จนกว่า handler จะ reconnect และ trade ที่ decode จาก schema อื่นจะถูกทิ้ง ส่วน quote และ book delta
จาก schema เหล่านั้นยังคงถูกส่ง schema ที่ส่งมาก่อนอาจต่างกันในแต่ละ session ดังนั้นเมื่อใช้ `mbo`
คู่กับ schema ของ trade อื่น trade จะไม่ได้ตามหลัง book delta ของ event นั้นเสมอไป ข้อจำกัด:

- schema `mbp-1` และ `tbbo` ใช้ record type เดียวกัน ดังนั้นการ subscribe ทั้งสองสำหรับ instrument เดียวกันจะยัง
  ส่ง trade แต่ละตัว และ quote ณ แต่ละ trade ซ้ำสองครั้ง
- continuous symbol จะหยุดนำสัญญาเดิมมาเมื่อเกิดการ roll หาก schema ที่ subscribe บน continuous symbol
  เป็นผู้นำ trade ของสัญญาหนึ่ง trade ของสัญญานั้นจาก schema อื่นจะยังคงถูกทิ้งหลังการ roll

`DatabentoHistoricalClient` ตัวเดียวรองรับ historical request ทุกตัวจาก
data client รวมถึง instrument definition ที่ดึงมาเพื่อ seed price precision

:::warning
Databento จะทิ้ง anchor `start` สำหรับ replay ที่ส่งมาหลังจาก live session เริ่มแล้ว ดังนั้น subscription ที่ทำ
กลาง session จะ stream ตั้งแต่จุดนั้นเป็นต้นไปโดยไม่มีประวัติ feed handler จะ log error สำหรับ
`start` ที่มาช้า และจะตัด `start` ออกจาก subscription ที่เก็บไว้ ดังนั้นการ reconnect จะไม่ replay ประวัติซ้ำอีก
:::

## การตั้งค่า [#configuration]

`DatabentoDataClientConfig` ต้องระบุ API key และ path ของ
`publishers.json`

ดาวน์โหลด
[`publishers.json`](https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/adapters/databento/publishers.json)
ฉบับมาตรฐาน แล้วชี้ `publishers_filepath` ไปยังสำเนาในเครื่อง

| Option                    | Default  | คำอธิบาย                                             |
| ------------------------- | -------- | ------------------------------------------------------- |
| `api_key`                 | Required | API key ของ Databento                                      |
| `publishers_filepath`     | Required | path ในเครื่องของ publisher metadata ของ Databento             |
| `use_exchange_as_venue`   | `False`  | ใช้ MIC ของ exchange เป็น venue สำหรับ instrument ของ GLBX           |
| `bars_timestamp_on_close` | `True`   | ประทับ timestamp ของ bar ตอนปิดแทนตอนเปิดของช่วง   |
| `venue_dataset_map`       | `None`   | override การ map venue-to-dataset จากข้อมูลของ publisher |
| `historical_base_url`     | `None`   | override base URL ของ Historical API                   |
| `live_gateway_addr`       | `None`   | override socket address ของ live gateway (`host:port`) |

ใช้ `DatabentoDataClientConfig` คู่กับ `DatabentoDataClientFactory` ดูตัวอย่าง Rust ได้ที่
[`crates/adapters/databento/examples/`](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/databento/examples/)

### ความเสถียรของการเชื่อมต่อ [#connection-stability]

live client จะ reconnect อัตโนมัติเมื่อเกิด:

- **เครือข่ายขัดข้อง**: ปัญหาการเชื่อมต่อชั่วคราว
- **Gateway restart**: Databento restart live gateway ตามกำหนดการ ดู
  [maintenance schedule](https://databento.com/docs/api-reference-live/basics/maintenance-schedule)
- **ตลาดปิด**: session จบลงในช่วงนอกเวลาทำการ

#### กลยุทธ์การ reconnect [#reconnection-strategy]

live client ที่สร้างจาก factory ใช้หน้าต่างการ reconnect ภายใน 10 นาที พร้อม exponential
backoff เริ่มที่ 1 วินาทีและสูงสุด 60 วินาที เมื่อหน้าต่างนี้หมดลงโดยไม่มี session ที่สำเร็จ client
จะยอมแพ้และรายงาน error แทนการ retry ไปเรื่อย ๆ

การเชื่อมต่อที่ค้างจะถูกตรวจพบโดย Databento client ต้นทาง ซึ่งจะเกิด heartbeat timeout
เมื่อไม่มีข้อมูลมาถึงภายในช่วง heartbeat บวก 5 วินาที feed handler จะถือว่าเป็น
connection error และเข้าสู่ backoff loop เดียวกัน

การ reconnect ทุกครั้งประกอบด้วย:

- **Jitter**: delay แบบสุ่ม (สูงสุด 1 วินาที) เพื่อป้องกันการ reconnect พร้อมกันจำนวนมาก
- **Resubscribe อัตโนมัติ**: กู้คืน subscription ที่ active ทั้งหมดหลัง reconnect
- **Reset รอบ**: session ที่สำเร็จแต่ละครั้ง (>60s) จะ reset นาฬิกา timeout และ backoff delay
- **Buffer command**: command ที่ได้รับระหว่าง backoff จะถูกนำไปใช้กับ session ถัดไป

request unsubscribe ทีละรายการจะ log ที่ระดับ debug และถูกละเว้น เพราะ live session ของ Databento
ไม่รองรับการ unsubscribe ทีละรายการ ให้หยุด session เพื่อลบ
subscription ออกจาก live gateway

#### Maintenance ตามกำหนดการ [#scheduled-maintenance]

Databento restart live gateway ตามกำหนดการนี้ (client ทั้งหมดจะถูกตัดการเชื่อมต่อ):

| Dataset            | เวลา restart      |
| ------------------ | ----------------- |
| CME Globex         | Saturday 02:15 CT |
| All ICE venues     | Sunday 09:45 UTC  |
| All other datasets | Sunday 10:30 UTC  |

timeout ภายใน 10 นาทีครอบคลุมการ restart ทั่วไป ดูรายละเอียดที่
[Databento Maintenance Schedule](https://databento.com/docs/api-reference-live/basics/maintenance-schedule)

## การมีส่วนร่วม [#contributing]

:::info
หากต้องการมีส่วนร่วม ดู
[contributing guide](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
