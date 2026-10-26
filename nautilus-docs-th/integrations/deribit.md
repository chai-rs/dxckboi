# Deribit

Deribit ก่อตั้งในปี 2016 เป็น exchange สำหรับ crypto derivative ทั้ง options, futures, perpetual, spot
และ combo instrument เป็นหนึ่งใน exchange crypto options ที่ใหญ่ที่สุดตาม volume และเป็นแพลตฟอร์มชั้นนำ
สำหรับการเทรด crypto derivative

integration นี้รองรับการรับ market data แบบ live และการ execute order กับ Deribit

## ภาพรวม {#overview}

adapter ของ Deribit implement ด้วย Rust Deribit ใช้ JSON-RPC 2.0 บนทั้ง transport แบบ HTTP และ WebSocket
adapter เลือกใช้ WebSocket สำหรับ subscription, ข้อมูล real-time และการทำงานกับ order และใช้ HTTP สำหรับการโหลด
instrument, request ข้อมูลย้อนหลัง และ reconciliation

Component:

- `DeribitRawHttpClient`: HTTP client ระดับ low-level ที่ดูแล JSON-RPC framing, การ sign, rate limit และการ retry
- `DeribitHttpClient`: HTTP client ระดับ domain ที่ parse response ของ venue เป็น type ของ Nautilus เข้าถึง raw client
  ได้ด้วย `inner()`
- `DeribitWebSocketClient`: การเชื่อมต่อ WebSocket ระดับ low-level สำหรับผู้เรียกจาก Rust
- `DeribitDataClient`: ตัวจัดการ market data feed
- `DeribitDataClientFactory`: factory ของ data client
- `DeribitExecutionClient`: gateway สำหรับการจัดการ account และการ execute trade
- `DeribitExecutionClientFactory`: factory ของ execution client

:::note
ผู้ใช้ส่วนใหญ่ตั้งค่า live trading node และไม่จำเป็นต้องใช้ component ระดับ low-level เหล่านี้โดยตรง
:::

## ตัวอย่าง {#examples}

- [ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/deribit/examples/)

## เอกสารของ Deribit {#deribit-documentation}

- [API reference](https://docs.deribit.com/)
- [Rate limits](https://docs.deribit.com/articles/rate-limits)
- [Connection management](https://docs.deribit.com/articles/connection-management-best-practices)
- [Market data collection](https://docs.deribit.com/articles/market-data-collection-best-practices)
- [Order management](https://docs.deribit.com/articles/order-management-best-practices)
- [Access scope](https://docs.deribit.com/articles/access-scope)

อ่านเอกสารของ Deribit ควบคู่กับคู่มือ integration ของ NautilusTrader ฉบับนี้

## ผลิตภัณฑ์ที่รองรับ {#product-support}

client แต่ละตัวโหลดเฉพาะกลุ่มผลิตภัณฑ์ที่ระบุใน configuration `product_types` ของตัวเอง ครั้งเดียวตอน connect
ไม่มี client ตัวใด refresh ชุด instrument หลังจากนั้น จึงต้อง restart node เพื่อรับ instrument ที่เพิ่ง list ใหม่
ฝั่ง data client นั้น `auto_load_missing_instruments` จะดึง instrument ที่ไม่อยู่ใน cache ตอน subscribe ครั้งแรกแทน
ดู [Lazy-load ตอน subscribe](#lazy-load-on-subscribe)

| ประเภทผลิตภัณฑ์       | Data feed | Trading | หมายเหตุ                                        |
| ----------------- | --------- | ------- | ---------------------------------------------- |
| Perpetual futures | ✓         | ✓       | โหลดด้วย product type `FUTURE`                  |
| Dated futures     | ✓         | ✓       | โหลดด้วย product type `FUTURE`                  |
| Options           | ✓         | ✓       | โหลดด้วย product type `OPTION`                  |
| Spot              | ✓         | ✓       | โหลดด้วย product type `SPOT`                    |
| Future combos     | ✓         | ✓       | โหลดด้วย product type `FUTURE_COMBO`            |
| Option combos     | ✓         | ✓       | โหลดด้วย product type `OPTION_COMBO`            |

## Symbology {#symbology}

Deribit ใช้ข้อตกลงการตั้งชื่อ symbol เฉพาะสำหรับ instrument แต่ละประเภท
instrument ID ทุกตัวควรมี suffix `.DERIBIT` เมื่ออ้างถึง
(เช่น `BTC-PERPETUAL.DERIBIT` สำหรับ BTC perpetual)

### หน่วยของ quantity {#quantity-units}

quantity ของ Nautilus map ไปยังฟิลด์ `amount` ของ Deribit ไม่ใช่ฟิลด์ `contracts` ที่เป็นทางเลือก Deribit
รายงาน amount ของ perpetual และ inverse futures เป็นหน่วย USD และรายงาน amount ของ options และ linear futures
เป็นสกุลเงิน base ของสินทรัพย์อ้างอิง ฟิลด์ `contract_size` ของ Deribit ใช้แปลงระหว่าง `amount` กับจำนวนสัญญา
adapter ไม่ได้นำค่านี้ไปใช้ซ้ำเป็น multiplier ของ Nautilus

### Perpetual futures {#perpetual-futures}

รูปแบบ: `{Currency}-PERPETUAL`

ตัวอย่าง:

- `BTC-PERPETUAL` - Bitcoin perpetual swap
- `ETH-PERPETUAL` - Ethereum perpetual swap

instrument ID ที่ใช้ subscribe BTC perpetual ใน strategy คือ `BTC-PERPETUAL.DERIBIT`

### Dated futures {#dated-futures}

รูปแบบ: `{Currency}-{DDMMMYY}`

ตัวอย่าง:

- `BTC-25DEC26` - Bitcoin future หมดอายุ 25 ธันวาคม 2026
- `ETH-26MAR27` - Ethereum future หมดอายุ 26 มีนาคม 2027

instrument ID: `BTC-25DEC26.DERIBIT`

### Options {#options}

รูปแบบ: `{Currency}-{DDMMMYY}-{Strike}-{Type}`

ตัวอย่าง:

- `BTC-25DEC26-100000-C` - Bitcoin call option, strike $100,000, หมดอายุ 25 ธันวาคม 2026
- `BTC-25DEC26-80000-P` - Bitcoin put option, strike $80,000, หมดอายุ 25 ธันวาคม 2026
- `ETH-26MAR27-4000-C` - Ethereum call option, strike $4,000, หมดอายุ 26 มีนาคม 2027

โดยที่:

- `C` = Call option
- `P` = Put option

instrument ID: `BTC-25DEC26-100000-C.DERIBIT`

### Spot {#spot}

รูปแบบ: `{BaseCurrency}_{QuoteCurrency}`

ตัวอย่าง:

- `BTC_USDC` - Bitcoin เทียบกับ USDC
- `ETH_USDC` - Ethereum เทียบกับ USDC

instrument ID: `BTC_USDC.DERIBIT`

### Future combos {#future-combos}

รูปแบบ: `{Currency}-FS-{LegA}_{LegB}`

leg เป็น dated futures หรือ perpetual (ใช้ `PERP` ในชื่อ combo แม้ว่า instrument เดี่ยวจะชื่อ `BTC-PERPETUAL`)
combo หมดอายุพร้อมกับ leg ที่หมดอายุเร็วที่สุด

ตัวอย่าง:

- `BTC-FS-25DEC26_PERP` - calendar spread ระหว่าง future เดือนธันวาคม 2026 กับ perpetual
- `BTC-FS-26MAR27_25DEC26` - inter-month spread ระหว่าง dated futures สองตัว

instrument ID: `BTC-FS-25DEC26_PERP.DERIBIT`

adapter จำลอง future combo เป็น `CryptoFuturesSpread` ตั้งราคาเป็น USD ตาม spread ระหว่าง leg มีสกุลเงิน settle
เป็น crypto และตั้ง `is_inverse` ตาม `instrument_type` จากต้นทาง

### Option combos {#option-combos}

รูปแบบ: `{Currency}-{Strategy}-{DDMMMYY}-{Strikes}`

รหัส strategy ได้แก่ CS (call spread), PS (put spread), STRG (strangle), STRD (straddle), BOX (box) และ RR
(risk reversal) ส่วน strike คั่น strike หลายตัวด้วย `_`

ตัวอย่าง:

- `BTC-CS-25DEC26-70000_75000` - call spread 70k / 75k หมดอายุ 25 ธันวาคม 2026
- `BTC-STRG-26MAR27-72000_80000` - strangle 72k / 80k หมดอายุ 26 มีนาคม 2027
- `BTC-STRD-26MAR27-77000` - straddle 77k หมดอายุ 26 มีนาคม 2027
- `BTC-BOX-26MAR27-58000_60000` - box 58k / 60k หมดอายุ 26 มีนาคม 2027

instrument ID: `BTC-STRG-26MAR27-72000_80000.DERIBIT`

adapter จำลอง option combo เป็น `CryptoOptionSpread` ตั้งราคาเป็นสกุลเงิน base ตามข้อตกลง inverse-option
ของ Deribit `size_increment` ที่เป็นเศษส่วน (เช่น `0.1`) ถูกรักษาไว้ตลอดทาง

## วันหมดอายุที่มีการเทรด {#traded-expirations}

Deribit เปิดเผยวันหมดอายุที่มีการเทรดอยู่ผ่าน HTTP endpoint `public/get_expirations` loader ของ option chain
ใช้ HTTP client ระดับสูงเพื่อ refresh option series ที่ active ได้โดยไม่ต้อง scan ทุก instrument

```rust tab="Rust"
use nautilus_deribit::http::models::DeribitCurrency;

let expirations = client
    .request_option_expirations(DeribitCurrency::BTC)
    .await?;
```

method ระดับสูงคืนเฉพาะวันหมดอายุของ option สำหรับ request ระดับ low-level ใน Rust ให้เรียก
`client.inner().get_expirations(...)` พร้อม `GetExpirationsParams` Deribit คืนผลลัพธ์ที่มี key เป็นสกุลเงินสำหรับ
สกุลเงินที่ระบุชัดเจน เช่น `BTC` และคืนผลลัพธ์ที่มี key เป็น kind โดยตรงสำหรับ `currency=any` adapter
รองรับทั้งสองรูปแบบ

## Combo instrument {#combo-instruments}

การโหลด instrument จะรวม combo เมื่อ `product_types` มี variant แบบ future-combo หรือ option-combo
Deribit เปิดเผยองค์ประกอบ leg ของทุก combo ที่ active ผ่าน `/public/get_combos` และ metadata การเทรดของ combo
(tick size, contract size, วันหมดอายุ, min trade amount) ผ่าน response มาตรฐานของ
`/public/get_instruments?kind=option_combo|future_combo` adapter รวมทั้งสองเข้าด้วยกัน โดยแนบ
`deribit_combo_id`, `deribit_combo_state` และ `deribit_combo_legs` ไว้ใน map `info` ของ instrument ที่โหลด
เพื่อให้ strategy resolve leg ได้โดยไม่ต้องเรียก venue ซ้ำ

### การ publish trade {#trade-publishing}

Deribit publish trade ของ combo แต่ละรายการสองครั้ง:

- บน trade channel ของ combo (`trades.{combo_name}.{interval}`): trade หลักพร้อม array `legs[]` ที่อธิบาย
  fill ของแต่ละ leg
- บน trade channel ของแต่ละ leg (`trades.{leg_instrument}.{interval}`): trade เดี่ยวของ leg ที่ติด tag
  `combo_id` และ `combo_trade_id` ชี้กลับไปยัง trade หลัก

ผู้ subscribe option หรือ future ธรรมดาจึงเห็น fill ที่มาจาก combo บน trade stream เดิมของตน และผู้ subscribe
combo เองจะเห็น trade ระดับ combo adapter ไม่แตก message หลักของ combo ออกเป็น tick ของ leg เพิ่ม แต่จะส่งต่อ
message หลักและ message ราย leg จากต้นทางเป็น `TradeTick` แยกกันตาม `InstrumentId` ของแต่ละตัว ผู้ที่ subscribe
ทั้ง combo และ leg อ้างอิงจึงเห็น tick ของ combo หนึ่งตัวและ tick ของ leg หนึ่งตัวสำหรับ trade combo นั้น โดยไม่ได้รับ
tick ซ้ำสำหรับ instrument เดียวกัน

หากต้องการให้ data client ของ Deribit เปิด trade channel จริงของ leg ไปพร้อมกับ trade subscription ของ combo
ให้ส่ง param `subscribe_combo_legs` เป็น true ให้ `subscribe_trades` เมื่อ unsubscribe trade stream ของ combo นั้น
Nautilus จะปิด subscription ของ leg ที่เปิดจาก opt-in นี้ด้วย

Deribit publish block trade และ Block RFQ ราย leg อยู่แล้ว adapter จึงส่งต่อผ่านเส้นทาง trade แบบ 1:1 มาตรฐาน
ดู [ที่มาของ trade ID](#trade-id-provenance) สำหรับวิธีติด tag trade ที่มาจาก block และ RFQ บน `TradeTick`

### Trade ย้อนหลังของ combo {#historical-combo-trades}

endpoint trade ราย instrument มาตรฐานรับชื่อ combo instrument ได้ หากต้องการดึง combo ทั้งหมดของผลิตภัณฑ์ประเภท
หนึ่งในการเรียกครั้งเดียว ให้ใช้ `get_last_trades_by_currency` ผ่าน `DeribitHttpClient::inner()`:

```rust
use nautilus_deribit::http::{
    models::{DeribitCurrency, DeribitProductType},
    query::GetLastTradesByCurrencyParams,
};

let params = GetLastTradesByCurrencyParams::builder()
    .currency(DeribitCurrency::BTC)
    .kind(DeribitProductType::FutureCombo)
    .count(50_u32)
    .include_old(true)
    .build()?;
let resp = client.inner().get_last_trades_by_currency(params).await?;
```

`DeribitPublicTrade` แต่ละตัวที่คืนมามี `legs: Option<Vec<DeribitTradeLeg>>` พร้อมฟิลด์ `combo_id` และ
`combo_trade_id` ที่ใช้ correlate trade ราย leg

## ที่มาของ trade ID {#trade-id-provenance}

`TradeTick` สาธารณะที่ adapter ปล่อยจะเติม prefix หน้า trade ID ของ venue เมื่อ trade มาจาก Block RFQ,
block trade หรือ combo strategy ที่ต้องแยก trade เหล่านี้ออกจาก trade ธรรมดาสามารถ match prefix บน
`TradeTick.trade_id` ได้ `trade_id` ดิบของ Deribit ยังอยู่หลัง prefix การ reconcile กับ ID ของ Deribit
จึงทำได้ด้วยการตัด prefix ออก

| Prefix       | ฟิลด์ต้นทาง        | ความหมาย                                                     |
| ------------ | ---------------- | ------------------------------------------------------------ |
| `RFQ-`       | `block_rfq_id`   | trade มาจาก Block RFQ                                         |
| `BLK-`       | `block_trade_id` | trade เป็น block trade ที่ไม่ใช่ RFQ                             |
| `COMBO-`     | `combo_id`       | trade ราย leg ที่ trade หลักมาจาก combo instrument              |
| *ไม่มี prefix* | (ไม่มีข้างต้น)     | trade มาตรฐาน                                                 |

ลำดับความสำคัญเมื่อมีหลาย tag: `RFQ-` > `BLK-` > `COMBO-` บน Deribit นั้น Block RFQ เองก็เป็น block trade
tag ของ RFQ จึงชนะ ส่วน combo ที่ execute เป็น block trade จะติด tag `BLK-` เพราะ flow ของ block เป็นสัญญาณที่
สำคัญกว่าสำหรับ reconciliation

หลักนี้ใช้กับ public trade (`TradeTick`) เท่านั้น `FillReport.trade_id` ไม่เปลี่ยน การ reconcile กับ
`get_user_trades_*` จึงยังทำงานได้

:::note
นี่เป็นข้อตกลงทางเดียวที่เริ่มใช้ใน NautilusTrader 1.228.0 ข้อมูล replay ที่บันทึกก่อน release นั้นไม่มี prefix
ถ้าคุณเก็บและเปรียบเทียบ string ของ `trade_id` ข้าม release ให้ตัด prefix ฝั่งข้อมูลใหม่ออก หรือกรองตาม prefix
เฉพาะกับข้อมูลที่บันทึกหลังการอัปเกรด
:::

## Market data {#market-data}

subscription แบบ live ให้บริการจาก WebSocket channel ส่วน request ข้อมูลย้อนหลังใช้ HTTP ทุก channel ที่รับ
`{interval}` จะ resolve ค่านี้ผ่าน subscription parameter `interval` และกฎการเลือกค่าเริ่มต้นที่อธิบายไว้ใน
[Order book subscription](#order-book-subscriptions)

| Nautilus subscription | Deribit channel                                                           | หมายเหตุ                                                                      |
| --------------------- | ------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Instruments           | `instrument.state.{kind}.{currency}`                                      | param `kind` และ `currency` มีค่าเริ่มต้นเป็น `any` ทั้งคู่                        |
| Instrument status     | `instrument.state.{kind}.{currency}`                                      | channel ได้จาก instrument ID; ทุก instrument บน channel จะถูกปล่อยออกมา          |
| Book deltas           | `book.{instrument}.raw` or `book.{instrument}.{group}.{depth}.{interval}` | `L2_MBP` เท่านั้น; ดู [Order book subscription](#order-book-subscriptions)     |
| Book depth            | `book.{instrument}.{group}.10.{interval}`                                 | ใช้ grouped channel เสมอ ที่ depth คงที่ 10                                    |
| Quotes                | `quote.{instrument}`                                                      | top of book; channel นี้ไม่รับ interval                                       |
| Trades                | `trades.{instrument}.{interval}`                                          | ดู [การ publish trade](#trade-publishing) สำหรับพฤติกรรมของ combo              |
| Bars                  | `chart.trades.{instrument}.{resolution}`                                  | ดู [Bar](#bars)                                                             |
| Mark prices           | `ticker.{instrument}.{interval}`                                          | parse จาก payload ของ ticker                                                |
| Index prices          | `ticker.{instrument}.{interval}`                                          | parse จาก payload ของ ticker                                                |
| Option greeks         | `ticker.{instrument}.{interval}`                                          | เฉพาะ options                                                               |
| Funding rates         | `perpetual.{instrument}.{interval}`                                       | เฉพาะ perpetual; instrument อื่นจะถูกปฏิเสธ                                    |
| Volatility index      | `deribit_volatility_index.{index_name}`                                   | custom data; ดู [Volatility index](#volatility-index)                       |

| Nautilus request             | Deribit endpoint                                | หมายเหตุ                                               |
| ---------------------------- | ----------------------------------------------- | --------------------------------------------------- |
| Instruments                  | `public/get_instruments`                        | จำกัดตาม `product_types` ที่ตั้งค่าไว้                    |
| Trades                       | `public/get_last_trades_by_instrument_and_time` | รับชื่อ combo instrument ได้                          |
| Bars                         | `public/get_tradingview_chart_data`             | ดู [Bar](#bars)                                      |
| Book snapshot                | `public/get_order_book`                         | ต้องมี instrument อยู่ใน cache ของ client               |
| Option-chain reference price | `public/ticker`                                 | ใช้ภายในเพื่อตั้งต้นช่วง strike แบบ dynamic                |
| `DeribitBookSummary`         | `public/get_book_summary_by_currency`           | custom data; ดู [Book summary](#book-summaries)       |

### Bar {#bars}

resolution ของ chart ของ Deribit คือ `1`, `3`, `5`, `10`, `15`, `30`, `60`, `120`, `180`, `360` และ
`720` นาที รวมถึง `1D` ทั้งเส้นทาง live และย้อนหลังต้องใช้ aggregation แบบ `EXTERNAL`

request ข้อมูลย้อนหลังเข้มงวด: bar type ที่ map ไปยัง resolution ข้างต้นไม่ได้จะถูกปฏิเสธ เช่นเดียวกับ aggregation
ใดๆ ที่ไม่ใช่ minute, hour และ day

subscription แบบ live จะปัดค่าแทน step ที่ขอจะถูกปัด**ขึ้น**เป็น resolution ที่รองรับถัดไป step 2 นาทีจึงเปิด
channel 3 นาทีของ Deribit และ step 6 นาทีเปิด channel 10 นาที ค่าที่ยาวกว่า 720 นาทีจะกลายเป็น `1D`
aggregation ที่ไม่ใช่ minute, hour และ day จะใช้ 1 นาทีแทนพร้อม warning

candle แต่ละแท่งที่ได้รับจะถูกติด `BarType` ที่ได้จาก resolution ของ venue โดยมี price type เป็น `LAST` เสมอ
ให้ subscribe ด้วย bar type ที่ระบุ resolution ที่รองรับและ `LAST` มิฉะนั้น bar จะ publish ภายใต้ `BarType`
ที่ต่างจาก subscription ของคุณและไม่มีวันถึง handler

`Bar.volume` มาจากฟิลด์ `cost` ของ chart (USD) สำหรับ inverse perpetual และ inverse futures และมาจากฟิลด์
`volume` (สกุลเงิน base) สำหรับอย่างอื่นทั้งหมด รวมถึง options และ option spread ที่ติด flag inverse ทำให้
`Bar.volume` และ `TradeTick.size` ใช้หน่วยเดียวกันต่อ instrument

## Order book subscription {#order-book-subscriptions}

Deribit publish ข้อมูล book แบบ L2 (market-by-price) เท่านั้น `subscribe_book_deltas` และ
`subscribe_book_depth` จึงปฏิเสธ book type ใดๆ ที่ไม่ใช่ `L2_MBP` มี feed ให้เลือกสองกลุ่ม แต่ละกลุ่มเหมาะกับ
กรณีการใช้งานต่างกัน

### Raw feed (tick-by-tick) {#raw-feeds-tick-by-tick}

raw channel ส่งทุกการอัปเดตเป็น message แยกกัน การ subscribe raw order book จะได้รับ notification สำหรับ
ทุกการเพิ่ม, อัปเดต หรือลบ order ใน book

- ต้องใช้ connection ที่ยืนยันตัวตนแล้ว (เพื่อป้องกันการใช้งานในทางที่ผิด)
- ใช้เมื่อต้องการทุกการเปลี่ยนแปลงของ price level สำหรับ HFT หรือ market making
- ปริมาณ message สูงกว่า
- ส่ง snapshot เริ่มต้นของ book ทั้งหมดหนึ่งครั้ง แล้วตามด้วย delta แบบ incremental

### Aggregated feed (แบบ batch) {#aggregated-feeds-batched}

aggregated channel ส่งการอัปเดตเป็น batch ตาม interval คงที่ (เช่น ทุก 100ms) โดยรวมการเปลี่ยนแปลงของ order book
หลายรายการไว้ใน message เดียว

- ใช้ได้โดยไม่ต้องยืนยันตัวตน
- แนะนำสำหรับกรณีการใช้งานส่วนใหญ่
- ปริมาณ message ต่ำกว่า ประมวลผลง่ายกว่า
- interval ค่าเริ่มต้นเมื่อไม่ยืนยันตัวตน: 100ms
- ทุก message เป็น snapshot ที่สมบูรณ์แบบจำกัดความลึก การอัปเดตแต่ละครั้งจึงแทนที่ book แทนการ patch

`subscribe_book_depth` ปล่อย snapshot `OrderBookDepth` ได้สูงสุด 10 ระดับต่อฝั่ง และมีจำนวน order เป็นศูนย์
เพราะ Deribit ไม่ส่งจำนวนมาให้ snapshot เหล่านี้ไม่แทนที่ book แบบ raw-delta หรือเปลี่ยนการติดตาม sequence ของมัน
subscription `subscribe_book_deltas` แบบ grouped จะปล่อย delta ที่แทนที่ book ที่ subscribe ไว้ รวมถึงเมื่อใช้
channel ร่วมกับ depth subscription

### Subscription parameter {#subscription-parameters}

adapter ของ Nautilus รองรับ feed ทั้งสองแบบผ่าน subscription parameter:

| Parameter  | ค่า                     | หมายเหตุ                                                                    |
| ---------- | ---------------------- | ------------------------------------------------------------------------- |
| `interval` | `raw`, `100ms`, `agg2` | `agg2` รวม batch ราว 1 วินาที `raw` ต้องยืนยันตัวตน                            |
| `group`    | `none`, price group    | ค่าเริ่มต้น: `none` ใช้เฉพาะกับ grouped book channel ที่ไม่ใช่ raw               |
| `depth`    | `1`, `10`, `20`        | ค่าเริ่มต้น: `10` จำนวน price level ต่อฝั่งสำหรับ grouped book channel          |

data client เลือก interval ของ order book ดังนี้:

1. ใช้ `params["interval"]` เมื่อระบุไว้
2. ใช้ `raw` เมื่อ WebSocket connection ยืนยันตัวตนแล้วและไม่ได้ระบุ interval
3. ใช้ grouped feed สาธารณะ `100ms` ของ Deribit เมื่อ connection ไม่ได้ยืนยันตัวตน

:::note
raw order book feed ต้องใช้ WebSocket connection ที่ยืนยันตัวตนแล้ว ตรวจให้แน่ใจว่าตั้งค่า API credential
ไว้ก่อน subscribe raw feed
:::

:::tip
สำหรับ strategy ส่วนใหญ่ aggregated feed 100ms ให้ความละเอียดเพียงพอโดยมี overhead ของ message ต่ำกว่า
ตั้ง param `interval` เป็น `100ms` เมื่อคุณให้ credential ไว้แต่ไม่ต้องการการอัปเดต book แบบ raw tick-by-tick
:::

### การกู้คืนเมื่อ sequence ขาดช่วง {#sequence-gap-recovery}

adapter บันทึก `change_id` ของ book message แบบ non-grouped แต่ละตัวที่นำไปใช้ และตรวจ `prev_change_id`
ของการอัปเดต incremental แต่ละตัวเทียบกับค่านั้น snapshot แบบ grouped ไม่อ่านหรืออัปเดต state ของ sequence นี้

เมื่อตรวจพบช่องว่าง (message หายไป) หรือ level ใน book message แปลงเป็น `Price` หรือ `Quantity` ไม่ได้
(เช่น ค่าเกินช่วงที่รองรับ) adapter จะทำโดยอัตโนมัติ:

1. ทิ้ง message นั้น และทุก delta ที่ตามมาของ instrument ที่ได้รับผลกระทบ
2. unsubscribe จาก book channel แบบ non-grouped ของ instrument นั้น
3. subscribe ใหม่เมื่อการ unsubscribe ได้รับการยืนยัน เพื่อรับ snapshot ใหม่
4. กลับมาประมวลผลตามปกติเมื่อแต่ละ channel ที่ subscribe ใหม่ส่ง snapshot ที่แปลงได้ และตั้ง sequence ใหม่จากมัน

ระหว่าง resync strategy จะไม่ได้รับการอัปเดต book ที่เก่าหรือไม่สมบูรณ์ การ unsubscribe ที่ผู้ใช้เริ่มเองขณะที่
resync ค้างอยู่จะยกเลิกการ subscribe ใหม่แทนการเปิด channel อีกครั้ง

grouped book channel (`book.{instrument}.{group}.{depth}.{interval}`) ส่ง snapshot เต็มทุก interval adapter
จึงไม่เคย resync channel นี้ มันจะทิ้ง snapshot ที่แปลงไม่ได้ และ snapshot ถัดไปจะแทนที่ book

:::warning[Subscribe แต่ละ instrument กับ book channel แบบ non-grouped เพียงช่องเดียว]
`DeribitWebSocketClient::subscribe_book` เปิด channel แบบ non-grouped เช่น `book.{instrument}.raw`
และ `book.{instrument}.100ms` ได้ channel เหล่านี้ใช้ sequence และ state ของ resync ร่วมกันต่อ instrument
ถ้า resync เริ่มขณะที่ channel หนึ่งยังรอ snapshot เริ่มต้น channel นั้นอาจค้างอยู่ในสถานะ unsubscribe
จนถึงการ reconnect ครั้งถัดไป คำเตือนนี้ไม่เกี่ยวกับ grouped depth subscription
:::

adapter จะ log warning และข้าม level ที่ผิดรูปแบบ รวมถึงค่าที่ parse เป็น decimal ไม่ได้ level ที่ amount ปัดเป็นศูนย์
ตาม size precision ของ instrument จะถือว่าไม่มีอยู่: snapshot จะข้าม การอัปเดต incremental จะลบ และ adapter
จะ log warning

## Funding rate {#funding-rates}

Deribit แลกเปลี่ยน funding อย่างต่อเนื่อง (ทุกไม่กี่วินาที) แทนการใช้ interval คงที่เหมือน exchange อื่นส่วนใหญ่
ฟิลด์ `interval` ของ `FundingRateUpdate` จึงเป็น `None` สำหรับ Deribit เพราะโมเดลแบบต่อเนื่องนี้ map ไปยัง
ช่วงเวลาแบบไม่ต่อเนื่องไม่ได้

## ข้อมูลเฉพาะของ Deribit {#deribit-specific-data}

### Book summary {#book-summaries}

request custom data `DeribitBookSummary` เพื่อดึง snapshot แบบ bulk หนึ่งชุดที่กรองตามสกุลเงินและประเภทผลิตภัณฑ์
แต่ละรายการใน response มี instrument ID ของ Nautilus, implied volatility, open interest, ราคา, volume และฟิลด์อื่น
ที่ `public/get_book_summary_by_currency` คืนมา

actor หรือ strategy จะได้รับ response ทั้งหมดผ่าน callback `on_historical_data` ครั้งเดียว แต่ละรายการเป็น
wrapper `CustomData` ที่มี `DeribitBookSummary` อยู่ในฟิลด์ `data`

ฟิลด์ metadata `currency` เป็นฟิลด์บังคับ ส่วนฟิลด์ `kind` ที่เป็นทางเลือกมีค่าเริ่มต้นเป็น `option`
response ที่ว่างจะเรียก `on_historical_data` หนึ่งครั้งพร้อม list ว่าง request ไปยัง venue ที่ล้มเหลวก็จะเรียก
callback พร้อม list ว่างหลังจาก log error request ที่ถูกปฏิเสธก่อนถึง venue เช่น request ที่ไม่มี `currency`
จะไม่เรียก callback

### Volatility index {#volatility-index}

adapter ปล่อย custom data `DeribitVolatilityIndex` จาก WebSocket channel
`deribit_volatility_index.{index_name}` ของ Deribit ซึ่งมี stream ของ volatility index เช่น `btc_usd` และ `eth_usd`

| Field        | คำอธิบาย                                                 |
| ------------ | -------------------------------------------------------- |
| `index_name` | ชื่อ volatility index ของ Deribit เช่น `btc_usd`           |
| `volatility` | ค่า volatility index ปัจจุบัน                               |
| `ts_event`   | UNIX timestamp เป็น nanosecond ของเวลาที่การอัปเดตเกิดขึ้น      |
| `ts_init`    | UNIX timestamp เป็น nanosecond ของเวลาที่สร้าง object        |

subscribe จาก actor หรือ strategy ด้วย data type `DeribitVolatilityIndex` บน client `DERIBIT`
โดยต้องระบุ metadata key `index_name` (เช่น `btc_usd`)

## ความสามารถด้าน order {#orders-capability}

adapter ตรวจสอบ parameter ของ order และเข้าคิว `OrderSubmitted` ก่อนส่ง order ถ้าการตรวจสอบหรือการเข้าคิว
event ล้มเหลว adapter จะคืน error โดยไม่ส่ง order และ execution engine จะจัดการ error นั้นเป็น `OrderDenied`

สำหรับ order list นั้น adapter ตรวจสอบทุกสมาชิกและเข้าคิว event `OrderSubmitted` ทั้งหมดก่อนส่งสมาชิกตัวใด
ความล้มเหลวของการตรวจสอบหรือการเข้าคิวจะกัน request ไปยัง venue ทั้งหมดของ list นั้น

ด้านล่างคือประเภท order, คำสั่งการ execute และตัวเลือก time-in-force ที่รองรับบน Deribit

### ประเภทของ order {#order-types}

| ประเภท order ของ Nautilus | ประเภท order ของ Deribit | รองรับ | หมายเหตุ                                   |
| ---------------------- | ------------------ | ------ | --------------------------------------- |
| `MARKET`               | `market`           | ✓      | execute ทันทีที่ราคาตลาด                   |
| `LIMIT`                | `limit`            | ✓      | execute ที่ราคาที่กำหนดหรือดีกว่า                |
| `STOP_MARKET`          | `stop_market`      | ✓      | market order แบบมีเงื่อนไขเมื่อ trigger        |
| `STOP_LIMIT`           | `stop_limit`       | ✓      | limit order แบบมีเงื่อนไขเมื่อ trigger         |
| `MARKET_IF_TOUCHED`    | `take_market`      | ✓      | market order แบบ take-profit              |
| `LIMIT_IF_TOUCHED`     | `take_limit`       | ✓      | limit order แบบ take-profit               |
| `TRAILING_STOP_MARKET` | `trailing_stop`    | -      | *ยังไม่ได้ implement*                       |
| `TRAILING_STOP_LIMIT`  | N/A                | -      | *Deribit ไม่รองรับ*                        |
| `MARKET_TO_LIMIT`      | `market_limit`     | -      | *ยังไม่ได้ implement*                       |

### คำสั่งการ execute {#execution-instructions}

| คำสั่ง          | รองรับ | หมายเหตุ                                                                          |
| ------------- | ------ | -------------------------------------------------------------------------------- |
| `post_only`   | ✓      | order จะถูกปฏิเสธถ้าจะไปกิน liquidity ใช้ `reject_post_only=true`                   |
| `reduce_only` | ✓      | order ลด position ที่มีอยู่ได้เท่านั้น                                               |

### Time in force {#time-in-force}

| Time in force | รองรับ | หมายเหตุ                                                |
| ------------- | ------ | ---------------------------------------------------- |
| `GTC`         | ✓      | Good till canceled (`good_til_cancelled`)            |
| `GTD`         | ✓      | Good till day หมดอายุเวลา 8:00 UTC (`good_til_day`)  |
| `IOC`         | ✓      | Immediate or cancel (`immediate_or_cancel`)          |
| `FOK`         | ✓      | Fill or kill (`fill_or_kill`)                        |

Deribit ใช้ time in force กับ order แบบ limit adapter จึงละ `time_in_force` สำหรับ order แบบ `MARKET`,
`STOP_MARKET` และ `MARKET_IF_TOUCHED` เพราะ Deribit ปฏิเสธ parameter นั้นสำหรับ order แบบ market

:::note
**GTD บน Deribit**: ต่างจาก exchange อื่นที่ GTD รับเวลาหมดอายุใดก็ได้ `good_til_day` ของ Deribit หมดอายุ
เวลา 8:00 UTC ของวันเดียวกันหรือวันถัดไปเสมอ เวลาหมดอายุที่กำหนดเองจะถูก log เป็น warning และ order
จะใช้พฤติกรรมเวลาหมดอายุคงที่ของ exchange
:::

### ประเภทของ trigger {#trigger-types}

conditional order (stop order) รองรับแหล่งราคา trigger ต่างกัน:

| ประเภท trigger  | รองรับ | หมายเหตุ                                                |
| ------------- | ------ | ---------------------------------------------------- |
| `last_price`  | ✓      | ใช้ราคาที่เทรดล่าสุด และใช้กับ `DEFAULT` ด้วย              |
| `mark_price`  | ✓      | ใช้ mark price                                       |
| `index_price` | ✓      | ใช้ราคา index ของสินทรัพย์อ้างอิง                          |

trigger type อื่นของ Nautilus จะถูกตัดออกจาก request แทนการ map Deribit ต้องการ `trigger` สำหรับ order
แบบ stop-loss และ take-profit จึงต้องใช้หนึ่งในสามแบบข้างต้นสำหรับ conditional order

### การทำงานแบบ batch {#batch-operations}

| การทำงาน                  | รองรับ | หมายเหตุ                                                                    |
| ------------------------ | ------ | ------------------------------------------------------------------------ |
| Submit order list        | ✓      | ส่งแต่ละ order เป็น order เดี่ยวของ Deribit ไม่มี batch แบบ atomic ที่ venue     |
| Batch cancel by order ID | ✓      | ส่ง request `private/cancel` แยกสำหรับแต่ละ venue order ID                 |
| Cancel all by instrument | ✓      | ใช้ `private/cancel_all_by_instrument` เมื่อไม่ได้ระบุ filter ด้าน side       |
| Side-filtered cancel all | ✓      | กรอง order ที่เปิดอยู่ใน cache ฝั่ง local แล้ว cancel ทีละ order ที่ตรงเงื่อนไข     |
| Batch modify             | -      | *ยังไม่ได้ implement*: รองรับการแก้ไข order เดี่ยว                           |

### พฤติกรรมของ post-only {#post-only-behavior}

Deribit มีโหมด post-only สองแบบ:

1. **ปรับราคา (ค่าเริ่มต้นของ Deribit)**: ถ้า order แบบ post-only จะข้าม spread และถูก execute Deribit
   จะปรับราคาให้อยู่ภายใน spread หนึ่ง tick โดยอัตโนมัติ
2. **โหมดปฏิเสธ**: order จะถูกปฏิเสธทันทีถ้าจะข้าม spread

adapter ของ Nautilus ใช้**โหมดปฏิเสธ** (`reject_post_only=true`) เพื่อให้พฤติกรรม deterministic ถ้า order
แบบ post-only จะไปกิน liquidity จะถูกปฏิเสธด้วย error code `11054` และปล่อย event `OrderRejected` ที่ตั้ง flag
`due_post_only` เป็น `true`

ทำให้ strategy แยกความแตกต่างได้ระหว่าง:

- order ที่ถูกปฏิเสธเพราะละเมิด post-only (พยายามกิน liquidity)
- order ที่ถูกปฏิเสธด้วยเหตุผลอื่น (margin ไม่พอ, ราคาไม่ถูกต้อง ฯลฯ)

### การแก้ไข order {#order-modification}

adapter ใช้ endpoint `private/edit` แบบ native ของ Deribit แทนการ cancel แล้ววางใหม่ ซึ่งมีข้อดีหลายประการ:

| ข้อดี                         | คำอธิบาย                                                             |
| --------------------------- | ----------------------------------------------------------------- |
| Request เดียว                 | execute เร็วกว่า latency ต่ำกว่าการ cancel + order ใหม่                |
| คงลำดับในคิว                    | คงตำแหน่งในคิวเมื่อลด quantity อย่างเดียวหรือใช้ราคาเดิม                    |
| คงประวัติ fill                   | partial fill ยังผูกกับ order ID เดิม                                 |

**กฎลำดับในคิว**:

- **ลด quantity อย่างเดียว**: คงตำแหน่งในคิว
- **ราคาเดิม**: คงตำแหน่งในคิว
- **เพิ่ม quantity หรือเปลี่ยนราคา**: เสียตำแหน่งในคิว (ถือเป็น order ใหม่)

คำสั่ง modify ต้องมี venue order ID และราคา ถ้าขาดอย่างใดอย่างหนึ่ง adapter จะปฏิเสธคำสั่งฝั่ง local โดยไม่ส่งไปถึง
Deribit quantity เป็นทางเลือก และจะใช้ quantity ปัจจุบันของ order ใน cache แทนเมื่อไม่ได้ระบุ

### การจัดการ position {#position-management}

| ความสามารถ        | รองรับ | หมายเหตุ                                                              |
| ---------------- | ------ | ------------------------------------------------------------------ |
| Query positions  | ✓      | ดึงจาก `private/get_positions` สำหรับ reconciliation                 |
| Position mode    | -      | *Deribit ไม่รองรับ*: มีเฉพาะโหมด net position                         |
| Leverage control | -      | *Deribit ไม่รองรับ*: ไม่มีการตั้ง leverage โดยตรง                        |
| Margin mode      | -      | *ยังไม่ได้ implement*: Deribit มีโหมด margin ระดับ account              |

execution client subscribe `user.orders`, `user.trades` และ `user.portfolio` (ยอดคงเหลือและ margin)
ไม่ได้ subscribe channel `user.changes` แบบรวมของ Deribit จึงไม่มี position ถูก push มาที่ adapter Nautilus
ดูแลสถานะ position แบบ live จาก fill บน `user.trades` และ `private/get_positions` ให้ snapshot ฝั่ง venue
ที่ reconciliation ใช้เปรียบเทียบ

### การ query order {#order-querying}

| ความสามารถ            | รองรับ | หมายเหตุ                          |
| -------------------- | ------ | ------------------------------ |
| Query open orders    | ✓      | แสดง order ที่ active ทั้งหมด       |
| Query order history  | ✓      | ข้อมูล order ย้อนหลัง              |
| Order status updates | ✓      | การเปลี่ยนสถานะ order แบบ real-time |
| Trade history        | ✓      | execution report และ fill report |

### Contingent order {#contingent-orders}

| ความสามารถ                      | รองรับ | หมายเหตุ                                                                          |
| ------------------------------ | ------ | ------------------------------------------------------------------------------ |
| Order lists                    | ✓      | ส่งตามลำดับเป็น order อิสระ ไม่มี list แบบ atomic ที่ venue                          |
| Native linked orders           | -      | *ยังไม่ได้ implement*: Deribit มี `linked_order_type`                            |
| OCO orders                     | -      | *ยังไม่ได้ implement*: Deribit มี `one_cancels_other`                            |
| Bracket orders                 | -      | *ยังไม่ได้ implement*: Deribit มี `one_triggers_one_cancels_other`               |
| Conditional stop orders        | ✓      | order แบบ stop market และ stop limit                                           |
| Conditional take-profit orders | ✓      | order แบบ market-if-touched และ limit-if-touched                               |

### การจัดการ liquidation {#liquidation-handling}

Deribit ติด tag ทุก trade ที่เกิดจาก liquidation บน stream `user.trades` และ endpoint
`private/get_user_trades_*` ฟิลด์ `liquidation` ที่เป็นทางเลือกระบุว่าฝั่งใดถูก liquidate:

| ค่า     | ความหมาย                       |
| ------ | ----------------------------- |
| `"M"`  | ฝั่ง maker ถูก liquidate         |
| `"T"`  | ฝั่ง taker ถูก liquidate         |
| `"MT"` | ทั้งสองฝั่งถูก liquidate          |
| absent | trade ปกติที่ไม่ใช่ liquidation    |

สำหรับ fill แต่ละตัวที่ติด tag liquidation adapter จะ log warning พร้อม instrument, trade ID, order ID และฝั่งที่ถูก
liquidate จากนั้นปล่อย `FillReport` ผ่าน pipeline ปกติ Deribit ไม่มีกลไก ADL ที่แยกจากกระบวนการ liquidation +
insurance fund / portfolio margin จึงไม่มีสัญญาณ ADL แยกให้แสดง

แหล่งอ้างอิงต้นทาง:

- [channel `user.trades.{instrument_name}.{interval}`](https://docs.deribit.com/subscriptions/user/usertradesinstrument_nameinterval)
- [เอกสารเรื่อง liquidation](https://support.deribit.com/hc/en-us/articles/25944769313309-Liquidations)

## Rate limiting {#rate-limiting}

Deribit ใช้ rate limit แบบ credit และแบบเฉพาะ endpoint ขีดจำกัดทางการของ Deribit เป็นแหล่งข้อมูลที่ถูกต้อง
และอาจต่างกันตาม endpoint, ระดับ account และนโยบายปัจจุบันของ venue adapter เพิ่ม token bucket ฝั่ง local
เพื่อลดการถูก throttle ที่หลีกเลี่ยงได้ แต่ไม่ได้แทนที่การตรวจฝั่ง server ของ Deribit

### ขีดจำกัดของ HTTP {#http-limits}

ทุก HTTP request ใช้ทั้ง bucket ระดับ global และ bucket ราย method ที่ใช้ key `deribit:{method}` ซึ่งใช้ quota
ระดับ global ถ้า JSON-RPC method อยู่ในหมวด order หรือ account ก็จะใช้ bucket นั้นด้วย

| Bucket / key       | Bucket ของ adapter      | หมายเหตุ                                                          |
| ------------------ | --------------------- | -------------------------------------------------------------- |
| `deribit:global`   | 20 req/sec, 100 burst | ใช้กับทุก HTTP request                                           |
| `deribit:orders`   | 5 req/sec, 20 burst   | method ของ matching engine เมื่อใช้ HTTP client โดยตรง              |
| `deribit:account`  | 5 req/sec, 5 burst    | endpoint ด้าน account, position, สถานะ order และ user trade       |
| `deribit:{method}` | 20 req/sec, 100 burst | bucket ราย method; ใช้ quota ระดับ global แทน                      |

### ขีดจำกัดของ WebSocket {#websocket-limits}

adapter รวม subscribe request ที่อยู่ติดกันในคิวและใช้ method สาธารณะหรือส่วนตัวเดียวกันเข้าด้วยกันโดยไม่เพิ่ม delay
และแบ่งชุด subscribe และ unsubscribe ขนาดใหญ่ เพื่อให้ JSON-RPC request ที่ serialize แล้วแต่ละตัวไม่เกิน 32,000 byte
รวม envelope subscription ที่กู้คืนหลัง reconnect ก็ใช้ขีดจำกัดขนาดเดียวกัน

| การทำงาน               | Bucket ของ adapter    | หมายเหตุ                                                          |
| --------------------- | ------------------- | -------------------------------------------------------------- |
| Subscribe/unsubscribe | 3 req/sec, 10 burst | เป็น bucket ค่าเริ่มต้นสำหรับ WebSocket request ทุกตัวที่ไม่ใช่ order     |
| Order operations      | 5 req/sec, 20 burst | buy, sell, edit และ cancel ผ่าน WebSocket                        |

:::note
adapter ของ Nautilus ใช้ WebSocket ในการส่ง order (ไม่ใช่ HTTP) เพื่อ latency ที่ต่ำกว่า
การทำงานกับ order ถูกจำกัดอัตราด้วย `DERIBIT_WS_ORDER_QUOTA` (5 req/sec, 20 burst)
:::

### รายละเอียดของระบบ credit {#credit-based-system-details}

Deribit เติม credit ที่ไม่ใช่ของ matching engine อย่างต่อเนื่อง เอกสารสาธารณะปัจจุบันระบุ pool ค่าเริ่มต้นของ
request ที่ไม่ใช่ matching engine ดังนี้:

**Request ที่ไม่ใช่ matching engine**:

| Parameter        | ค่า                 | หมายเหตุ                           |
| ---------------- | ------------------ | ------------------------------- |
| Cost per request | 500 credits        | การเรียก API แต่ละครั้งใช้ credit     |
| Maximum pool     | 50,000 credits     | burst ได้ 100 request            |
| Refill rate      | 10,000 credits/sec | ราว 20 request/วินาทีต่อเนื่อง       |

**Request ของ matching engine (ระดับค่าเริ่มต้น)**:

| Parameter      | ค่า             | หมายเหตุ                            |
| -------------- | -------------- | -------------------------------- |
| Sustained rate | 5 requests/sec | rate limit ต่อเนื่อง                |
| Burst capacity | 20 requests    | burst สูงสุดก่อนถูก throttle          |

ขีดจำกัดของ matching engine ที่สูงกว่ามีให้สำหรับ market maker และเทรดเดอร์ที่มี volume สูง ตามระดับ volume การเทรด
7 วัน

endpoint บางตัวของ Deribit มีขีดจำกัดเฉพาะ method ที่เข้มงวดกว่า เช่น เอกสาร venue ปัจจุบันระบุ
`public/get_instruments` ไว้ที่ 1 request ต่อวินาทีพร้อม burst 50 request และ method ของ subscription ราว 3.3 request
ต่อวินาทีพร้อม burst 10 request ให้จำกัด `product_types` เฉพาะกลุ่มที่ต้องใช้ และหลีกเลี่ยงการโหลด instrument ใหม่
ทั้งหมดซ้ำๆ ในระบบ live

adapter ของ Nautilus implement token bucket rate limiter แบบกว้างที่ตั้งค่าไว้ดังนี้:

- `DERIBIT_HTTP_REST_QUOTA`: 20 req/sec พร้อม 100 burst (HTTP ระดับ global และราย method)
- `DERIBIT_HTTP_ORDER_QUOTA`: 5 req/sec พร้อม 20 burst (HTTP ของ matching engine)
- `DERIBIT_HTTP_ACCOUNT_QUOTA`: 5 req/sec พร้อม 5 burst (HTTP ด้าน account)
- `DERIBIT_WS_ORDER_QUOTA`: 5 req/sec พร้อม 20 burst (WebSocket ของ matching engine)
- `DERIBIT_WS_SUBSCRIPTION_QUOTA`: 3 req/sec พร้อม 10 burst (subscribe, unsubscribe และ request อื่นที่ไม่ใช่ order)

ดูรายละเอียดเพิ่มเติมได้ที่
[บทความ Rate limits](https://docs.deribit.com/articles/rate-limits)

:::warning
Deribit คืน error code `10028` (too_many_requests) เมื่อคุณใช้เกิน quota ที่อนุญาต การละเมิดซ้ำๆ
อาจทำให้ถูก throttle ชั่วคราว
:::

## การจัดการ connection {#connection-management}

### ขีดจำกัดของแพลตฟอร์ม {#platform-limits}

| ขีดจำกัด                                           | แนวทางปัจจุบันของ Deribit |
| ------------------------------------------------ | ---------------------- |
| Active sessions per API key or login             | 16                     |
| Session and connection-scoped connections per IP | 32                     |
| Web app connections per browser session          | 2                      |

### การยืนยันตัวตนแบบ session {#session-based-authentication}

adapter ใช้ **WebSocket session แยกกัน** สำหรับ data client และ execution client แต่ละตัวมีขอบเขตการยืนยัน
ตัวตนของตัวเอง:

| Client           | ชื่อ session           | วัตถุประสงค์                                             |
| ---------------- | -------------------- | --------------------------------------------------- |
| Data client      | `nautilus-data`      | market data subscription (raw feed ต้องยืนยันตัวตน)     |
| Execution client | `nautilus-execution` | การทำงานกับ order (buy, sell, edit, cancel)            |

**Flow การยืนยันตัวตน**:

1. WebSocket เชื่อมต่อกับ Deribit
2. client ยืนยันตัวตนด้วย grant type `client_signature` พร้อม session scope
3. token ถูก refresh ก่อนหมดอายุ
4. เมื่อ reconnect การยืนยันตัวตนใหม่จะ retry ด้วย exponential backoff (สูงสุด 3 ครั้ง)
   ถ้าล้มเหลวทั้งหมด จะกู้คืนเฉพาะ subscription ของ channel สาธารณะ

แนวทางแบบ session นี้ทำให้:

- จัดการ token แยกกันตามประเภท client
- แยกขอบเขตความล้มเหลว (การยืนยันตัวตนฝั่ง data ล้มเหลวไม่กระทบ execution)
- มี audit trail ชัดเจนใน session log ของ Deribit

### แนวปฏิบัติที่ดี {#best-practices}

adapter ทำตาม[แนวปฏิบัติด้าน connection ที่ Deribit แนะนำ](https://docs.deribit.com/articles/connection-management-best-practices):

1. **ใช้ WebSocket subscription** สำหรับข้อมูล real-time แทนการ poll ผ่าน REST ทำให้ request น้อยลง
   latency ต่ำลง และใช้ rate limit น้อยลง
2. **ยืนยันตัวตนทุก connection** เมื่อมี credential ผู้ใช้ที่ยืนยันตัวตนแล้วได้ rate limit สูงกว่าและมีโอกาสถูกจำกัด
   อัตราตาม IP น้อยกว่า
3. **ใช้ heartbeat** (interval 30 วินาทีโดยค่าเริ่มต้น) เพื่อรักษาสุขภาพของ connection และตรวจจับการหลุดได้เร็ว
4. **จัดการการ reconnect** โดยอัตโนมัติพร้อมการยืนยันตัวตนใหม่และการกู้คืน subscription

:::tip
ควรให้ API credential เสมอแม้จะเข้าถึงข้อมูลสาธารณะ connection ที่ยืนยันตัวตนแล้วมี rate limit สูงกว่า และ
Deribit จะติดต่อ client ที่ยืนยันตัวตนแล้วก่อนใช้ข้อจำกัดในช่วงที่ภาระสูง
:::

:::note
adapter ใช้ heartbeat interval 30 วินาทีโดยค่าเริ่มต้น Deribit กำหนดให้ heartbeat interval ของ WebSocket
ต้องไม่น้อยกว่า 10 วินาที
:::

## การยืนยันตัวตน {#authentication}

Deribit ใช้การยืนยันตัวตนด้วย API key พร้อมลายเซ็น HMAC-SHA256 สำหรับ endpoint ส่วนตัว

วิธีสร้าง API credential:

1. Log in เข้า account Deribit ที่ [deribit.com](https://www.deribit.com)
   (หรือ [test.deribit.com](https://test.deribit.com) สำหรับ testnet)
2. ไปที่ **Account** -> **API**
3. คลิก **Add new key** แล้วตั้งค่าสิทธิ์:
   - เปิด **read** สำหรับการเข้าถึง market data
   - เปิด **trade** สำหรับการ execute order
   - เปิด **wallet** ถ้าต้องการเข้าถึงยอดคงเหลือของ account
4. จด **Client ID** (API key) และ **Client Secret** (API secret) ไว้

:::warning
เก็บ API secret ให้ปลอดภัย อย่าแชร์หรือ commit เข้า version control
:::

### Scope ของ API key {#api-key-scopes}

API key แต่ละตัวบน Deribit ได้รับ access scope ค่าเริ่มต้นซึ่งกำหนดสิทธิ์สูงสุด ตั้งค่าสิทธิ์ที่เหมาะสมเมื่อ
[สร้าง API key](https://docs.deribit.com/articles/creating-api-key):

| Scope              | จำเป็นสำหรับ                              |
| ------------------ | -------------------------------------- |
| `account:read`     | ข้อมูล account และข้อมูล portfolio         |
| `trade:read`       | ดู order และ position                    |
| `trade:read_write` | วาง, แก้ไข และ cancel order              |
| `wallet:read`      | ดูยอดคงเหลือและประวัติธุรกรรม                |

**ขั้นต่ำที่แนะนำสำหรับการเทรด**: `account:read`, `trade:read_write`, `wallet:read`

:::tip
ยึดหลัก least privilege สำหรับการเข้าถึงเฉพาะข้อมูล (market data, ไม่เทรด) ให้สร้าง key แบบอ่านอย่างเดียว
ที่ไม่มี `trade:read_write`
:::

## Testnet {#testnet}

Deribit มีสภาพแวดล้อม testnet สำหรับทดสอบ strategy โดยไม่ใช้เงินจริง หากต้องการใช้ testnet ให้ตั้ง
`environment` เป็น `TESTNET` ใน configuration ของ client (ทั้ง data client และ execution client)

เมื่อเปิดโหมด testnet:

- HTTP request ใช้ `https://test.deribit.com`
- WebSocket connection ใช้ `wss://test.deribit.com/ws/api/v2`
- โหลด credential จาก environment variable `DERIBIT_TESTNET_API_KEY` และ `DERIBIT_TESTNET_API_SECRET`

:::note
API key ของ testnet แยกจาก key ของ production ให้สร้าง API key สำหรับ testnet โดยเฉพาะผ่านหน้า testnet
ที่ [test.deribit.com](https://test.deribit.com)
:::

## Configuration {#configuration}

### Option ของ data client {#data-client-configuration-options}

| Option                             | ค่าเริ่มต้น   | คำอธิบาย                                                           |
| ---------------------------------- | ---------- | ------------------------------------------------------------------ |
| `api_key`                          | `None`     | API key ของ Deribit โหลดจาก environment variable เมื่อไม่ได้ระบุ       |
| `api_secret`                       | `None`     | API secret ของ Deribit โหลดจาก environment variable เมื่อไม่ได้ระบุ    |
| `product_types`                    | `[FUTURE]` | ประเภทผลิตภัณฑ์ที่จะโหลด                                               |
| `environment`                      | `MAINNET`  | enum ของ environment (`MAINNET` หรือ `TESTNET`)                    |
| `base_url_http`                    | `None`     | override base URL ของ HTTP JSON-RPC                               |
| `base_url_ws`                      | `None`     | override base URL ของ WebSocket                                   |
| `proxy_url`                        | `None`     | proxy URL สำหรับ transport แบบ HTTP และ WebSocket (ไม่บังคับ)         |
| `http_timeout_secs`                | `60`       | timeout ของ request เป็นวินาทีสำหรับการเรียก HTTP                       |
| `max_retries`                      | `3`        | จำนวนครั้งสูงสุดของการ retry สำหรับ error ที่กู้คืนได้                      |
| `retry_delay_initial_ms`           | `1,000`    | delay เริ่มต้นเป็น millisecond ก่อน retry                             |
| `retry_delay_max_ms`               | `10,000`   | delay สูงสุดเป็น millisecond ระหว่างการ retry                          |
| `heartbeat_interval_secs`          | `30`       | heartbeat interval ของ WebSocket                                  |
| `auth_timeout_secs`                | `None`     | จำนวนวินาทีที่รอผลการยืนยันตัวตนของ WebSocket; ไม่ตั้งหมายถึง 30          |
| `update_instruments_interval_mins` | `60`       | รับค่าได้แต่ยังไม่มีผล; instrument โหลดครั้งเดียวตอน connect               |
| `auto_load_missing_instruments`    | `False`    | lazy-load instrument ที่ไม่อยู่ใน cache ตอน subscribe                 |
| `transport_backend`                | `Sockudo`  | backend ของ WebSocket transport                                   |

#### Lazy-load ตอน subscribe {#lazy-load-on-subscribe}

คำสั่ง `subscribe_*` จะค้นหา instrument ใน cache ฝั่ง local ก่อนส่ง WebSocket subscribe เพื่อให้ handler
parse frame ขาเข้าได้ เมื่อ `auto_load_missing_instruments = False` (ค่าเริ่มต้น) subscription จะคืน error
ถ้า instrument ของมันไม่ได้ถูกโหลดไว้ล่วงหน้าตาม `product_types` ที่ตั้งค่าไว้ มันจะไม่สำเร็จแบบเงียบๆ แล้วไปทิ้ง
frame ที่ตามมาที่ handler

หากต้องการโหลด instrument ที่ขาดไปตอน subscribe ครั้งแรก ให้ตั้ง `auto_load_missing_instruments = True`
client จะดึง instrument ผ่าน HTTP, ตั้งต้น cache ของ WebSocket handler แล้วส่งต่อ subscribe ความล้มเหลวของ
HTTP จะถูก log และข้าม WebSocket subscribe ไป

### Option ของ execution client {#execution-client-configuration-options}

| Option                   | ค่าเริ่มต้น   | คำอธิบาย                                                           |
| ------------------------ | ---------- | ------------------------------------------------------------------ |
| `account_id`             | Required   | account ID ของ Nautilus สำหรับ report และ event ที่สร้างขึ้น            |
| `api_key`                | `None`     | API key ของ Deribit โหลดจาก environment variable เมื่อไม่ได้ระบุ       |
| `api_secret`             | `None`     | API secret ของ Deribit โหลดจาก environment variable เมื่อไม่ได้ระบุ    |
| `product_types`          | `[FUTURE]` | ประเภทผลิตภัณฑ์ที่จะโหลด                                               |
| `environment`            | `MAINNET`  | enum ของ environment (`MAINNET` หรือ `TESTNET`)                    |
| `base_url_http`          | `None`     | override base URL ของ HTTP JSON-RPC                               |
| `base_url_ws`            | `None`     | override base URL ของ WebSocket                                   |
| `proxy_url`              | `None`     | proxy URL สำหรับ transport แบบ HTTP และ WebSocket (ไม่บังคับ)         |
| `http_timeout_secs`      | `60`       | timeout ของ request เป็นวินาทีสำหรับการเรียก HTTP                       |
| `max_retries`            | `3`        | จำนวนครั้งสูงสุดของการ retry สำหรับ error ที่กู้คืนได้                      |
| `retry_delay_initial_ms` | `1,000`    | delay เริ่มต้นเป็น millisecond ก่อน retry                             |
| `retry_delay_max_ms`     | `10,000`   | delay สูงสุดเป็น millisecond ระหว่างการ retry                          |
| `auth_timeout_secs`      | `None`     | จำนวนวินาทีที่รอผลการยืนยันตัวตนของ WebSocket; ไม่ตั้งหมายถึง 30          |
| `transport_backend`      | `Sockudo`  | backend ของ WebSocket transport                                   |

`transport_backend` เลือกระหว่าง WebSocket transport แบบ `Sockudo` และ `Tungstenite` ค่าเริ่มต้นคือ
`Sockudo` เมื่อเปิด Cargo feature `transport-sockudo` ส่วน build ของ Rust ที่ไม่มี feature นั้นจะใช้
`Tungstenite` เป็นค่าเริ่มต้น

### Configuration สำหรับ production {#production-configuration}

ดูตัวอย่าง live node ที่ใช้ data client และ execution client ของ Deribit ได้ใน
[ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/deribit/examples/)
และดูวิธีประกอบ live node จาก Rust ได้ที่ [รัน live trading (Rust)](../how_to/run_rust_live_trading.md)

### API credential {#api-credentials}

มีหลายวิธีในการส่ง credential ให้ client ของ Deribit ส่งค่าที่ตรงกันให้ object ของ configuration หรือ
ตั้ง environment variable ต่อไปนี้:

สำหรับ client ของ Deribit แบบ live (production):

- `DERIBIT_API_KEY`
- `DERIBIT_API_SECRET`

สำหรับ client ของ Deribit testnet:

- `DERIBIT_TESTNET_API_KEY`
- `DERIBIT_TESTNET_API_SECRET`

:::tip
แนะนำให้ใช้ environment variable ในการจัดการ credential
:::

### ประเภทผลิตภัณฑ์ {#product-types}

option `product_types` ของ configuration ควบคุมว่าจะโหลดกลุ่มผลิตภัณฑ์ใดของ Deribit ตัวเลือกที่มีผ่าน enum
`DeribitProductType`:

- `FUTURE` - perpetual และ dated futures
- `OPTION` - call และ put option
- `SPOT` - คู่เทรดแบบ spot
- `FUTURE_COMBO` - instrument แบบ future spread
- `OPTION_COMBO` - instrument แบบ option spread

สามารถโหลดหลายประเภทพร้อมกันได้ เช่น `FUTURE` และ `OPTION`

### Base URL {#base-urls}

option `environment` เลือกค่าเริ่มต้นเหล่านี้ ตั้ง `base_url_http` หรือ `base_url_ws` เพื่อชี้ transport ใดตัวหนึ่ง
ไปที่อื่น เช่น local proxy หรือ capture harness

| Environment | HTTP URL                   | WebSocket URL                      |
| ----------- | -------------------------- | ---------------------------------- |
| Production  | `https://www.deribit.com`  | `wss://www.deribit.com/ws/api/v2`  |
| Testnet     | `https://test.deribit.com` | `wss://test.deribit.com/ws/api/v2` |

## โครงสร้างพื้นฐานของ server {#server-infrastructure}

matching engine ของ Deribit ตั้งอยู่ที่ **Equinix LD4, Slough, UK** สำหรับ strategy ที่ไวต่อ latency ควรพิจารณา
hosting ในหรือใกล้ลอนดอน Deribit มีตัวเลือก colocation และ cross-connect ให้ลูกค้าสถาบันโดยตรง

สำหรับผู้ใช้ส่วนใหญ่ที่เชื่อมต่อผ่านอินเทอร์เน็ต logic การ retry, การติดตาม heartbeat และการจัดการ reconnect
อัตโนมัติที่มีในตัว adapter ให้การเชื่อมต่อที่เชื่อถือได้

ดูรายละเอียดเพิ่มเติมได้ที่[บทความ Server Infrastructure](https://support.deribit.com/hc/en-us/articles/25944617582877-Server-Infrastructure)

## การมีส่วนร่วม {#contributing}

:::info
หากต้องการ feature เพิ่มเติมหรือร่วมพัฒนา adapter ของ Deribit โปรดดู
[contributing guide](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
