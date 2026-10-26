# Binance

Binance ก่อตั้งในปี 2017 และเป็นหนึ่งใน exchange คริปโตที่ใหญ่ที่สุดเมื่อวัดจากปริมาณการซื้อขายรายวันและ
open interest ของสินทรัพย์คริปโตและอนุพันธ์

NautilusTrader มี integration กับ Binance สำหรับ market data แบบ live และ execution โดย adapter
implement ด้วย Rust

ผลิตภัณฑ์ที่รองรับ:

- **Binance Spot** (รวม Binance US)
- **Binance USDT-Margined Futures** (perpetual ทั้งแบบ crypto และ TradFi; สัญญาส่งมอบรายสัปดาห์
  รายเดือน และรายไตรมาส ทั้งงวดปัจจุบันและงวดถัดไป)
- **Binance Coin-Margined Futures** (perpetual และสัญญาส่งมอบรายไตรมาสงวดปัจจุบันหรืองวดถัดไป)

## ตัวอย่าง {#examples}

- [ตัวอย่าง Rust สำหรับ spot](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/binance/examples/spot/)
- [ตัวอย่าง Rust สำหรับ futures](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/binance/examples/futures/)

## ภาพรวม {#overview}

adapter เปิดให้ใช้ component สาธารณะเหล่านี้:

- `BinanceDataClientConfig` และ `BinanceExecutionClientConfig`: config ของ live client
- `BinanceInstrumentProviderConfig`: การเลือก instrument, การกรอง และ policy ของ warning
- `BinanceDataClientFactory` และ `BinanceExecutionClientFactory`: factory ของ client สำหรับ trading node

:::note
ผู้ใช้ส่วนใหญ่ต้องการเพียง config และ factory ซึ่งต่อเข้ากับ live trading node ตามที่แสดงใน
[การตั้งค่า live node](#live-node-configuration)
:::

### การรองรับผลิตภัณฑ์ {#product-support}

| ประเภทผลิตภัณฑ์                            | รองรับ | หมายเหตุ                                              |
| --------------------------------------- | --------- | -------------------------------------------------- |
| Spot Markets (incl. Binance US)         | ✓         |                                                    |
| Margin Accounts (Cross & Isolated)      | -         | *ยังไม่ได้ implement*                                 |
| USDT-Margined Futures (PERP & Delivery) | ✓         | สัญญาส่งมอบรายสัปดาห์ รายเดือน และรายไตรมาส |
| Coin-Margined Futures (PERP & Delivery) | ✓         | สัญญาส่งมอบรายไตรมาส                      |

:::note
ฟีเจอร์ของ margin account เช่น borrow, repay และการจัดการ isolated margin ยังไม่ได้ implement
:::

:::info
Binance client แต่ละ instance รองรับ product type เดียว config ใช้ฟิลด์ `product_type` แบบค่าเดียว และ
live factory จะสร้าง data client หรือ execution client หนึ่งตัวจาก config หนึ่งชุด หากต้องการรัน Spot และ Futures ใน
node เดียวกัน ให้ตั้งค่า client แยกกันโดยใช้ ID ต่างกัน เช่น `BINANCE_SPOT` และ `BINANCE_FUTURES` จากนั้นเมื่อ
strategy subscribe หรือ submit order ให้ส่ง `client_id` ที่ตรงกัน เมื่อใช้ execution client ทั้งสองตัว
ให้ตั้งค่า route ของ venue `BINANCE` อย่างชัดเจนหรือกำหนด default client ตามที่อธิบายใน
[Execution client routing](../concepts/live.md#execution-client-routing) ดู[ตัวอย่าง](#examples)ประกอบ
:::

## Data type {#data-types}

integration นี้มี custom data type หลายชนิด:

- `BinanceSpotTicker`: ข้อมูล ticker 24 ชั่วโมงของ Spot รวมถึงราคา ปริมาณ และสถิติการซื้อขาย
- `BinanceFuturesTicker`: ข้อมูล ticker 24 ชั่วโมงของ Futures รวมถึงราคาและสถิติ
- `BinanceBar`: ข้อมูล bar ที่มี metric ด้านปริมาณเพิ่มเติม ใช้ได้ทั้งแบบย้อนหลังและ real-time
- `BinanceFuturesMarkPriceUpdate`: ข้อมูล mark ของ Futures รวมถึงราคา settlement โดยประมาณ
- `BinanceFuturesLiquidation`: event การ liquidation ของ Futures จาก stream `forceOrder`
- `BinanceFuturesOpenInterest`: snapshot ของ open interest ปัจจุบันของ Futures (request เท่านั้น)
- `BinanceFuturesOpenInterestHist`: ประวัติ open interest ของ Futures ในช่วงเวลาหนึ่ง (request เท่านั้น)

ดูนิยามทั้งหมดได้ที่ [Rust API docs](https://nautilustrader.io/docs/core-latest/)

`BinanceBar`, `BinanceFuturesTicker`, `BinanceFuturesOpenInterest` และ
`BinanceFuturesLiquidation` รองรับการ persist ลง catalog แบบ Arrow/Parquet ภายใต้
`data/custom/{TypeName}/{identifier}` build ของ Rust ต้องเปิด feature flag `arrow` ของ `nautilus-binance`
สำหรับการ persist นี้ catalog ที่ยังใช้ layout เดิม `data/custom_<snake_case>` ที่เขียนโดยฝั่ง Python
(เช่น `data/custom_binance_bar`) ให้แปลงด้วย
[nautilus catalog migrate-parquet](../how_to/migrate_parquet_catalog.md)

## Symbology {#symbology}

ใช้ symbol ดั้งเดิมของ Binance เท่าที่ทำได้สำหรับสัญญา spot และ futures
เนื่องจาก NautilusTrader รองรับการเทรดหลาย venue จึงต้องแยกแยะระหว่าง
`BTCUSDT` ที่เป็นคู่ spot กับ `BTCUSDT` ที่เป็นสัญญา perpetual futures (Binance
ใช้ symbol เดียวกันสำหรับทั้งสองอย่าง)

Nautilus จะต่อท้าย `-PERP` ให้กับ symbol ของ USD-M perpetual เช่น `BTCUSDT` perpetual ของ Binance
USD-M จะกลายเป็น `BTCUSDT-PERP` listing แบบ `TRADIFI_PERPETUAL` ของ USD-M
ใช้ suffix เดียวกัน ดังนั้น `XAUUSDT` จะกลายเป็น `XAUUSDT-PERP`

adapter map listing แบบ `TRADIFI_PERPETUAL` ไปเป็น
`PerpetualContract` และ derive asset class จาก `underlyingType` ของ Binance:

| Binance `underlyingType`                                     | Nautilus asset class |
| ------------------------------------------------------------ | -------------------- |
| `EQUITY`, `CN_EQUITY`, `KR_EQUITY`, `HK_EQUITY`, `PREMARKET` | Equity               |
| `COMMODITY`                                                  | Commodity            |

listing ที่มีค่าอื่นหรือไม่มีค่าจะถูกข้ามพร้อม warning

adapter คง suffix `_PERP` ดั้งเดิมของ Binance ไว้สำหรับ COIN-M
perpetual ดังนั้น `BTCUSD_PERP` จะไม่เปลี่ยนแปลง

symbol ของสัญญาส่งมอบคง suffix `_YYMMDD` ของ Binance ไว้ เช่น
`BTCUSDT_270326` และ `BTCUSD_270326` จะไม่เปลี่ยนแปลงภายใน Nautilus USD-M
รองรับ contract type ตามเอกสาร ได้แก่ `CURRENT_MONTH`, `NEXT_MONTH`, `CURRENT_QUARTER` และ
`NEXT_QUARTER` รวมถึง `CURRENT_WEEK` และ `NEXT_WEEK` แบบรายสัปดาห์
ซึ่งไม่ได้ระบุไว้ใน common definitions ของ Binance ส่วน COIN-M รองรับ
`CURRENT_QUARTER` และ `NEXT_QUARTER` ความพร้อมของสัญญาแตกต่างกันไปตาม
environment และรอบการ listing

instrument แบบส่งมอบของ USD-M เป็นแบบ linear และ settle ด้วย margin asset ส่วน instrument แบบส่งมอบ
ของ COIN-M เป็นแบบ inverse, settle ด้วย margin asset (base
currency) และใช้ `contractSize` ของ Binance เป็น multiplier ของ instrument ทั้งสองแบบ
ใช้ `onboardDate` และ `deliveryDate` สำหรับการเริ่ม active และการหมดอายุ ดู
[USD-M common definitions](https://developers.binance.com/en/docs/products/derivatives-trading-usds-futures/common-definition)
และ [COIN-M common definitions](https://developers.binance.com/en/docs/products/derivatives-trading-coin-futures/common-definition)
อย่างเป็นทางการของ Binance

Rust Futures data tester รับ instrument แบบส่งมอบได้โดยไม่ต้องแก้ source:

```bash
BINANCE_FUTURES_INSTRUMENT_ID=BTCUSDT_270326.BINANCE \
  cargo run -p nautilus-binance --example binance-futures-data-tester --features examples
```

## ข้อจำกัด notional ของ Spot {#spot-notional-constraints}

instrument ของ Spot ที่โหลดผ่าน SBE หรือ JSON จะเติมฟิลด์ `min_notional` และ
`max_notional` จาก filter `MIN_NOTIONAL` และ `NOTIONAL` เมื่อมีทั้งสอง filter
parser จะใช้ขอบเขตที่เข้มงวดที่สุด ฟิลด์เหล่านี้เก็บค่า `Money` ในสกุลเงิน quote ตาม
precision ของสกุลเงินนั้น

risk engine ตรวจสอบฟิลด์เหล่านี้ของ instrument โดยใช้ราคาประมาณการของตน ส่วน `info` ของ instrument เป็น
metadata สำหรับ actor และ strategy ปลายทาง และไม่มีผลต่อการตัดสินใจด้าน risk Binance ใช้
flag ของ market order และกฎของราคาอ้างอิงของตนเองเมื่อตรวจสอบ order ที่ venue

PostgreSQL จะเก็บ metadata ของ instrument ไว้เมื่อกู้คืน instrument หลังจากอัปเกรด
database ที่มีอยู่ ให้รัน `nautilus database init --schema "$PWD/schema/sql"` จาก root ของ repository เพื่อเพิ่ม
ที่เก็บ metadata metadata ที่เคยถูกทิ้งไปก่อนหน้าต้องโหลด instrument definition ใหม่

## ความสามารถด้าน order {#order-capability}

ตารางต่อไปนี้แสดงรายละเอียดของ order type, execution instruction และ
ตัวเลือก time-in-force ของผลิตภัณฑ์ Binance ที่รองรับ

### Order type {#order-types}

| Order Type             | Spot | USDT Futures | Coin Futures | หมายเหตุ                                   |
| ---------------------- | ---- | ------------ | ------------ | --------------------------------------- |
| `MARKET`               | ✓    | ✓            | ✓            | รองรับ quote quantity: เฉพาะ Spot      |
| `LIMIT`                | ✓    | ✓            | ✓            |                                         |
| `STOP_MARKET`          | ✓    | ✓            | ✓            | Spot ส่งเป็น `STOP_LOSS` ของ Binance         |
| `STOP_LIMIT`           | ✓    | ✓            | ✓            | Spot ส่งเป็น `STOP_LOSS_LIMIT` ของ Binance   |
| `MARKET_IF_TOUCHED`    | ✓    | ✓            | ✓            | Spot ส่งเป็น `TAKE_PROFIT` ของ Binance       |
| `LIMIT_IF_TOUCHED`     | ✓    | ✓            | ✓            | Spot ส่งเป็น `TAKE_PROFIT_LIMIT` ของ Binance |
| `TRAILING_STOP_MARKET` | -    | ✓            | ✓            | เฉพาะ Futures                           |

Binance Spot publish ชุด order type ที่รองรับต่อ symbol ไว้ใน `exchangeInfo` adapter ไม่ได้กรองตาม
ชุดนี้ ดังนั้น venue จะ reject conditional order ของ Spot ที่ใช้ type ที่ symbol นั้นไม่รองรับ
adapter ไม่ได้ reject ภายในเครื่อง

### Execution instruction {#execution-instructions}

| Instruction   | Spot | USDT Futures | Coin Futures | หมายเหตุ                                                     |
| ------------- | ---- | ------------ | ------------ | --------------------------------------------------------- |
| `post_only`   | ✓    | ✓            | ✓            | ดูข้อจำกัดด้านล่าง                                   |
| `reduce_only` | -    | ✓            | ✓            | เฉพาะ Futures; แปลงเป็น `positionSide` ใน Hedge Mode |

ใน One-way Mode adapter จะส่งฟิลด์ `reduceOnly` ของ Binance แต่ Binance ไม่รับ
ฟิลด์นั้นใน Hedge Mode adapter จึงเลือก `positionSide` ฝั่งที่ปิดแทน ทำให้
order อยู่บน leg ที่ระบุและไม่เปิด leg ฝั่งตรงข้าม ดู
[New Order API](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/New-Order) ของ Binance
สำหรับข้อจำกัดของ wire format

#### ข้อจำกัดของ post-only {#post-only-restrictions}

เฉพาะ order type แบบ *limit* เท่านั้นที่รองรับ `post_only`

| Order Type   | Spot | USDT Futures | Coin Futures | หมายเหตุ                                               |
| ------------ | ---- | ------------ | ------------ | --------------------------------------------------- |
| `LIMIT`      | ✓    | ✓            | ✓            | ใช้ `LIMIT_MAKER` สำหรับ Spot และ TIF `GTX` สำหรับ Futures |
| `STOP_LIMIT` | -    | ✓            | ✓            | เฉพาะ Futures                                       |

### Time in force {#time-in-force}

| Time in force | Spot | USDT Futures | Coin Futures | หมายเหตุ                                     |
| ------------- | ---- | ------------ | ------------ | ----------------------------------------- |
| `GTC`         | ✓    | ✓            | ✓            | Good Till Canceled                       |
| `GTD`         | ✓*   | ✓            | ✓*           | *map ภายในเครื่องผ่าน `GTC` (ไม่ใช่ค่า default) |
| `FOK`         | ✓    | ✓            | ✓            | Fill or Kill                             |
| `IOC`         | ✓    | ✓            | ✓            | Immediate or Cancel                      |

#### GTD policy {#gtd-policy}

[ค่า time-in-force ของ Binance Spot](https://github.com/binance/binance-spot-api-docs/blob/master/enums.md) คือ
`GTC`, `IOC` และ `FOK` Spot ไม่มี `GTD` หรือ `goodTillDate` แบบ native

USD-M รองรับ `GTD` แบบ native สำหรับ `LIMIT` และรูปแบบ limit ของ `STOP` และ `TAKE_PROFIT` adapter route
order ปกติผ่าน HTTP หรือ WebSocket trading, batch อิสระผ่าน HTTP `batchOrders` และ
conditional algo order ผ่าน HTTP `algoOrder` schema ของ algo บน WebSocket ของ Binance มี `goodTillDate`
แต่ไม่มี `GTD` ใน enum `timeInForce` adapter จึงไม่ route GTD algo order ผ่าน
endpoint นั้น

COIN-M ไม่มีค่า `GTD` แบบ native หรือพารามิเตอร์ `goodTillDate` ใน order API ตามเอกสาร ดู
[USD-M trade API](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/trade)
และ
[COIN-M common definitions](https://developers.binance.com/en/docs/products/derivatives-trading-coin-futures/common-definition) อย่างเป็นทางการ

`goodTillDate` ของ USD-M เป็น epoch timestamp หน่วยมิลลิวินาที แต่ Binance ไม่สนใจส่วนที่ต่ำกว่า
วินาที Nautilus จะ reject expiry ที่ไม่ได้อยู่บนขอบวินาทีพอดี แทนที่จะปัดเศษ
อย่างเงียบ ๆ expiry ต้องมากกว่าเวลาปัจจุบันบวก 600 วินาทีอย่างเคร่งครัด และ
น้อยกว่า `253402300799000` อย่างเคร่งครัด GTD แบบ native ยัง reject market order, post-only order และ
order ที่ไม่มี expiry ด้วย

`use_gtd=True` เป็นค่า default ซึ่งใช้ GTD แบบ native ของ USD-M และ reject GTD แบบ native บน Spot และ COIN-M
ตั้ง `use_gtd=False` เฉพาะเมื่อ strategy ที่ submit มี `manage_gtd_expiry=True` เท่านั้น adapter
จะ warn และส่ง `GTC` แทน ส่วน Nautilus จะ cancel order เมื่อถึงเวลาหมดอายุภายในเครื่อง

### ฟีเจอร์ order ขั้นสูง {#advanced-order-features}

| ฟีเจอร์            | Spot | USDT Futures | Coin Futures | หมายเหตุ                                       |
| ------------------ | ---- | ------------ | ------------ | ------------------------------------------- |
| Order Modification | ✓    | ✓            | ✓            | ราคาและจำนวนสำหรับ `LIMIT` order เท่านั้น |
| OCO Orders         | ✓    | -            | -            | Spot OCO ส่งผ่าน `orderList/oco`     |
| Bracket Orders     | -    | -            | -            | *มีแผน* ปัจจุบันถูกปฏิเสธตอน submit  |
| Iceberg Orders     | ✓    | -            | -            | Spot `icebergQty` จาก `display_qty`       |

### Batch operation {#batch-operations}

| Operation    | Spot | USDT Futures | Coin Futures | หมายเหตุ                                   |
| ------------ | ---- | ------------ | ------------ | --------------------------------------- |
| Batch Submit | ✓    | ✓            | ✓            | Spot OCO หรือ Futures `batchOrders`      |
| Batch Modify | -    | -            | -            | ยังไม่ได้ implement                        |
| Batch Cancel | -*   | ✓            | ✓            | *Spot fallback ไปเป็นการ cancel ทีละตัว |

#### พฤติกรรมของ cancel all orders {#cancel-all-orders-behavior}

โดย default `cancel_all_orders()` ของ strategy จะส่ง cancel ทีละตัวสำหรับ order ที่เกี่ยวข้องกับ
strategy นั้น เมื่อใช้ `strategy_only=False` strategy จะส่ง command `CancelAllOrders` แบบกว้าง
ไปยัง adapter adapter จะรวม order ทั้งในสถานะ open และ inflight (SUBMITTED) เพื่อให้
cancel order ที่ Binance ยังไม่ได้ยืนยันด้วย

**ความปลอดภัยเมื่อมีหลาย strategy**: เมื่อหลาย strategy เทรด instrument เดียวกัน
adapter จะเปรียบเทียบ order ที่เกี่ยวข้องกับ strategy ที่ร้องขอกับ order ทั้งหมด
ของ instrument นั้น หาก order ทั้งหมดเป็นของ strategy นั้น จะใช้การเรียก cancel-all API
ครั้งเดียว มิฉะนั้นจะส่ง cancel ราย strategy (batch สำหรับ order ปกติ
และทีละตัวสำหรับ algo order) เพื่อไม่ให้กระทบ strategy อื่น

**Side filter**: command `CancelAllOrders` ที่ตั้ง `order_side` ไว้จะ cancel เฉพาะ open order ฝั่งนั้นของ
instrument Spot ส่ง cancel หนึ่งครั้งต่อ order ที่ตรงกัน ส่วน Futures จะ batch order ปกติและ cancel
algo order ทีละตัว request ที่กรองตาม side จะเลือกจาก open order เท่านั้น ดังนั้น order ที่ inflight (`SUBMITTED`)
ซึ่ง Binance ยังไม่ได้ยืนยันจะรอดไป ให้ใช้ cancel-all แบบไม่กรองเพื่อรวม order นั้นด้วย

**Futures algo order**: conditional order type (`STOP_MARKET`, `STOP_LIMIT`,
`TAKE_PROFIT`, `TAKE_PROFIT_MARKET`, `TRAILING_STOP_MARKET`) ต้องใช้
cancel endpoint ต่างออกไป adapter จะ route ไปยัง endpoint ที่ถูกต้อง
โดยอัตโนมัติ เมื่อ algo order ถูก trigger และกลายเป็น order ปกติ
จะใช้ cancel endpoint มาตรฐาน

**Endpoint ที่ใช้**:

| Product      | Regular Orders                  | Algo Orders (batch)              | Algo Orders (individual)    |
| ------------ | ------------------------------- | -------------------------------- | --------------------------- |
| Spot         | `DELETE /api/v3/openOrders`     | N/A                              | N/A                         |
| USDT Futures | `DELETE /fapi/v1/allOpenOrders` | `DELETE /fapi/v1/algoOpenOrders` | `DELETE /fapi/v1/algoOrder` |
| Coin Futures | `DELETE /dapi/v1/allOpenOrders` | `DELETE /dapi/v1/algoOpenOrders` | `DELETE /dapi/v1/algoOrder` |

#### Retry policy ของ submit, modify และ cancel {#submit-modify-and-cancel-retry-policy}

execution client ส่ง command submit, modify หรือ cancel แต่ละตัวเพียงครั้งเดียว request แรกอาจไปถึง
matching engine แล้ว จึงไม่ retry แบบสุ่มสี่สุ่มห้าหลังเกิด timeout, network failure หรือ response สถานะ unknown
ของ Binance การ retry อาจสร้าง order ซ้ำหรือแก้ไข order ซ้ำเป็นครั้งที่สอง

- การตรวจสอบ submit ภายในเครื่องจะปล่อย `OrderDenied` ก่อน submit การตรวจสอบ modify ภายในเครื่องจะปล่อย
  `OrderModifyRejected` การ reject ที่ชัดเจนจาก venue จะปล่อย rejection event ที่ตรงกัน
- ผลลัพธ์ด้าน transport ที่กำกวมจะยังคงเป็น inflight และถูก resolve โดย private stream หรือ REST
  reconciliation adapter จะไม่ปล่อย rejection ปลอมขณะที่ยังไม่ทราบผลลัพธ์ที่ venue
- cancel ของ Futures algo อาจ fallback จาก algo endpoint ก่อน trigger ไปยัง endpoint ของ order ปกติ ซึ่ง
  เป็นการเปลี่ยน endpoint หลังจาก order ถูก trigger แล้ว ไม่ใช่การส่ง cancel เดิมซ้ำไปยัง endpoint เดิม
- โค้ดของ strategy ต้องไม่ resubmit command ขณะที่ผลลัพธ์ยังกำกวม ให้รอ reconciliation
  หรือ query order ด้วย client order ID

`BinanceDataClientConfig` และ `BinanceExecutionClientConfig` มี `max_retries`,
`retry_delay_initial_ms` และ `retry_delay_max_ms` สำหรับ HTTP GET request ความล้มเหลวชั่วคราวในการอ่าน
จะ retry ด้วย exponential backoff ที่มีขอบเขตและฟิลด์ authentication ใหม่ header `Retry-After`
ของ venue จะกำหนด delay ขั้นต่ำ ซึ่งอาจเกิน `retry_delay_max_ms` ได้ งบ retry ทั้งหมดคงที่
ที่ 180 วินาที เมื่อ delay ที่ต้องรอเกินงบที่เหลือ request จะ return error ของ venue
โดยไม่รอ

การตั้งค่าเหล่านี้ไม่ retry order command เพราะการส่ง command ที่กำกวมซ้ำอาจทำให้
order หรือการแก้ไขซ้ำซ้อน

### การจัดการ position {#position-management}

| ฟีเจอร์          | Spot | USDT Futures | Coin Futures | หมายเหตุ                                   |
| ---------------- | ---- | ------------ | ------------ | --------------------------------------- |
| Query positions  | -    | ✓            | ✓            | update position แบบ real-time             |
| Position mode    | -    | ✓            | ✓            | One-Way vs Hedge mode (position ID)   |
| Leverage control | -    | ✓            | ✓            | ปรับ leverage แบบ dynamic ต่อ symbol |
| Margin mode      | -    | ✓            | ✓            | Cross vs Isolated margin ต่อ symbol    |

Binance Futures จะ log symbol ของ position ที่อยู่นอกขอบเขตที่ระดับ debug และทิ้งก่อน parse สำหรับ
แถวที่เหลือ การสร้าง position report จะ warn เมื่อ parse จำนวนไม่ได้ หลังจากลบ position ที่
flat ออกแล้ว instrument ที่ resolve ไม่ได้และความล้มเหลวในการแปลงอื่น ๆ ก็จะ warn เช่นกัน หากมี
position ใดล้มเหลว `generate_position_status_reports` จะ return error พร้อมจำนวนที่ล้มเหลว แทนที่จะ
return ชุด report ที่ไม่สมบูรณ์ ดู
[ความพร้อมของ instrument](../concepts/execution/reconciliation.md#instrument-availability)

### Risk event {#risk-events}

| ฟีเจอร์              | Spot | USDT Futures | Coin Futures | หมายเหตุ                              |
| -------------------- | ---- | ------------ | ------------ | ---------------------------------- |
| Liquidation handling | -    | ✓            | ✓            | การปิด position ที่ exchange บังคับ |
| ADL handling         | -    | ✓            | ✓            | event ของ Auto-Deleveraging          |

Binance Futures อาจสร้าง order ที่ exchange สร้างขึ้นเองเพื่อตอบสนองต่อ risk event:

- **Liquidation**: เมื่อ margin ไม่พอรักษา position Binance จะบังคับปิด
  ที่ราคา bankruptcy order เหล่านี้มี client ID ขึ้นต้นด้วย `autoclose-`
- **ADL (Auto-Deleveraging)**: เมื่อ insurance fund หมด Binance จะปิด position ที่มีกำไร
  เพื่อชดเชยการขาดทุน order เหล่านี้ใช้ prefix ของ client ID เป็น `adl_autoclose`
- **Settlement (USD-M)**: order ของ funding และ margin settlement ใช้ client ID ขึ้นต้นด้วย
  `settlement_autoclose-`
- **Delivery (COIN-M)**: สัญญาส่งมอบที่หมดอายุจะปิดอัตโนมัติด้วย client ID ขึ้นต้นด้วย
  `delivery_autoclose-`
- **Insurance fund**: การเข้ารับช่วงโดย insurance fund ใช้สถานะ `NEW_INSURANCE` (ประกาศ deprecated ใน
  changelog สาธารณะแล้ว แต่ยังพบได้ใน wire)

adapter ตรวจจับ order ชนิดพิเศษเหล่านี้จากรูปแบบของ client ID
(ตรวจก่อน execution type) จากนั้น:

1. log warning พร้อมรายละเอียดของ order เพื่อการ monitor
2. สร้าง `FillReport` ที่มีรายละเอียด fill ถูกต้องและ liquidity side เป็น TAKER
3. สร้าง `OrderStatusReport` สำหรับ reconciliation

เอกสารอ้างอิงต้นทาง:

- [USDT-M `ORDER_TRADE_UPDATE`](https://developers.binance.com/docs/derivatives/usds-margined-futures/user-data-streams/Event-Order-Update)
- [COIN-M `ORDER_TRADE_UPDATE`](https://developers.binance.com/docs/derivatives/coin-margined-futures/user-data-streams/Event-Order-Update)

execution engine จะสร้าง external order จาก runtime status report เมื่อ
order ยังไม่อยู่ใน cache ซึ่งครอบคลุม order ที่ exchange สร้างขึ้นที่เพิ่งพบครั้งแรก
(กรณีทั่วไปของ event liquidation หรือ ADL แบบ live) engine
จะกำหนด order ผ่าน external order claim ที่ active ของ instrument นั้น
ซึ่งตั้งค่าเริ่มต้นด้วย `external_order_instrument_ids` หรือกำหนดให้ strategy `EXTERNAL`
โดย default

:::note
status report และ fill report จะถูกปล่อยรวมกันเป็น execution report แบบ
`OrderWithFills` ตัวเดียว engine จะสร้าง external order
จาก status report แล้วจึงนำ fill จริงไปใช้ โดยคง
`trade_id` และ `commission` ของ venue ไว้ จำนวนที่เหลือซึ่งไม่ได้ครอบคลุมโดย
fill ที่รวมมาจะถูกปิดด้วย fill ที่อนุมานจาก `avg_px` ของ status report
:::

#### การประมาณค่า commission {#commission-estimation}

เมื่อ Binance ละฟิลด์ commission (`N`/`n`) ออกจาก fill event
adapter จะประมาณ commission เป็น `default_taker_fee * qty * price` โดยใช้
quote currency ใช้กับสัญญา linear ของ USD-M เท่านั้น สัญญา inverse ของ COIN-M
ใช้ commission เป็นศูนย์เป็นค่า fallback เพราะสูตร linear ไม่ได้คำนึงถึงขนาดสัญญา
ตั้งค่า `default_taker_fee` บน
`BinanceExecutionClientConfig` ให้ตรงกับ fee tier ของคุณ (default: 0.0004 / 0.04%)

### การ query order {#order-querying}

| ฟีเจอร์              | Spot | USDT Futures | Coin Futures | หมายเหตุ                          |
| -------------------- | ---- | ------------ | ------------ | ------------------------------ |
| Query open orders    | ✓    | ✓            | ✓            | แสดง order ที่ active ทั้งหมด        |
| Query order history  | ✓    | ✓            | ✓            | ข้อมูล order ย้อนหลัง         |
| Order status updates | ✓    | ✓            | ✓            | การเปลี่ยนสถานะ order แบบ real-time |
| Trade history        | ✓    | ✓            | ✓            | execution และ fill report    |

#### ระยะเวลาเก็บประวัติ trade ของ Futures {#futures-trade-history-retention}

Binance เก็บ account trade ของ USD-M และ COIN-M ไว้สามเดือนย้อนหลัง Binance ไม่ได้ระบุว่า
หมายถึงเดือนตามปฏิทินหรือระยะเวลาคงที่ adapter จึงใช้ 88 วันล่าสุดเป็นหน้าต่างประวัติ fill
ของ Futures ที่สมบูรณ์

`generate_fill_reports` จะปฏิเสธ start ที่ระบุชัดเจนซึ่งอยู่ก่อนหน้าต่างนั้น ขอบเขตใช้
`ts_init` ของ command โดยจำกัดไว้ให้ช้ากว่าเวลาปัจจุบันของ client ได้ไม่เกิน 12 ชั่วโมง การระบุ end
time ต้องมี start time ด้วย

สำหรับ mass status หากไม่ได้ตั้ง `reconciliation_lookback_mins` หรือตั้งค่ายาวกว่าหน้าต่างที่สมบูรณ์ จะใช้
หน้าต่างนั้น `ExecutionMassStatus` ที่ return จะตั้ง `lookback_start` เป็นขอบเขตที่ใช้ และ
`reports_complete` เป็น `false` ดู
[mass-status history contract](../concepts/execution/reconciliation.md#mass-status-history-contract) Binance
Spot ไม่ได้รับผลกระทบ ดู
[Futures change log](https://developers.binance.com/en/docs/products/derivatives-trading-usds-futures/change-log) ของ Binance

### Contingent order {#contingent-orders}

| ฟีเจอร์            | Spot | USDT Futures | Coin Futures | หมายเหตุ                                        |
| ------------------ | ---- | ------------ | ------------ | -------------------------------------------- |
| Order lists        | ✓    | ✓            | ✓            | Spot OCO list; Futures batch อิสระ |
| OCO orders         | ✓    | -            | -            | เฉพาะ Spot ผ่าน `orderList/oco`              |
| Bracket orders     | -    | -            | -            | *มีแผน* ปัจจุบันถูกปฏิเสธตอน submit   |
| Conditional orders | ✓    | ✓            | ✓            | stop และ market-if-touched order           |

สำหรับ Binance Global Spot event list-status แบบ SBE จะเติม `OrderAccepted` ที่ขาดไปพร้อม venue order ID
ให้กับ child ที่ติดตามอยู่เมื่อ list เริ่ม execute การ reject list พร้อมเหตุผลจาก venue จะเติม
`OrderRejected` ให้กับ child ที่ยังไม่ถูก accept หรือ fill ส่วน execution report ของ child จะขับเคลื่อน fill,
cancellation, expiration และสถานะสิ้นสุดอื่น ๆ

### พารามิเตอร์ของ order {#order-parameters}

ปรับแต่ง order แต่ละตัวได้โดยตั้ง `Params` บน command `SubmitOrder`
(ดูตัวอย่าง Rust ด้านล่าง) Binance execution client รู้จักพารามิเตอร์ต่อไปนี้:

| Parameter        | Type   | Products          | วัตถุประสงค์                                          | ข้อจำกัด                                                                              |
| ---------------- | ------ | ----------------- | ------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| `price_match`    | `str`  | USDT/COIN Futures | ให้ Binance เป็นผู้เลือกราคา             | `LIMIT` เท่านั้น; ใช้ร่วมกับ `post_only` ไม่ได้                                                       |
| `close_position` | `bool` | USDT/COIN Futures | ปิดทั้ง position เมื่อ trigger ทำงาน | `StopMarket` และ `MarketIfTouched` เท่านั้น; ต้องมี `reduce_only=true`; ใช้ใน order list ไม่ได้ |
| `rpi`            | `bool` | USDT Futures      | submit order แบบ Retail Price Improvement         | `LIMIT` เท่านั้น; ต้องมี `post_only=true`; order เดี่ยวเท่านั้น                          |

ดูพฤติกรรมทั้งหมดได้ที่ [Price match](#price-match), [RPI](#rpi) และ [Close position](#close-position)

### Price match {#price-match}

Binance Futures รองรับ price matching แบบ BBO (Best Bid/Offer) ผ่าน
พารามิเตอร์ `priceMatch` ซึ่งให้ exchange เป็นผู้เลือกราคา limit
order จะเข้าร่วม order book แบบ dynamic ที่ราคาที่เหมาะสมโดยไม่ต้องระบุ
ระดับราคาที่แน่นอน

เมื่อใช้ `price_match` ให้ submit limit order พร้อมราคาอ้างอิงสำหรับการตรวจ risk ภายในเครื่อง Binance จะกำหนด
ราคาที่ใช้งานจริงจากสถานะตลาดและโหมดของ price match

#### ค่า price match ที่ใช้ได้ {#valid-price-match-values}

| ค่า         | พฤติกรรม                                                       |
| ------------- | -------------------------------------------------------------- |
| `OPPONENT`    | เข้าร่วมที่ราคาดีที่สุดของฝั่งตรงข้ามใน book          |
| `OPPONENT_5`  | เข้าร่วมที่ราคาฝั่งตรงข้ามแต่ยอมให้ offset ได้ถึง 5 tick  |
| `OPPONENT_10` | เข้าร่วมที่ราคาฝั่งตรงข้ามแต่ยอมให้ offset ได้ถึง 10 tick |
| `OPPONENT_20` | เข้าร่วมที่ราคาฝั่งตรงข้ามแต่ยอมให้ offset ได้ถึง 20 tick |
| `QUEUE`       | เข้าร่วมที่ราคาดีที่สุดของฝั่งเดียวกัน (คงความเป็น maker)             |
| `QUEUE_5`     | เข้าร่วมคิวฝั่งเดียวกันแต่ offset ได้ถึง 5 tick             |
| `QUEUE_10`    | เข้าร่วมคิวฝั่งเดียวกันแต่ offset ได้ถึง 10 tick            |
| `QUEUE_20`    | เข้าร่วมคิวฝั่งเดียวกันแต่ offset ได้ถึง 20 tick            |

:::info
ดูรายละเอียดเพิ่มเติมได้ที่ [เอกสารอย่างเป็นทางการ](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api)
:::

#### ลำดับของ event {#event-sequence}

เมื่อ submit order พร้อม `price_match`:

1. Nautilus ส่ง order พร้อมพารามิเตอร์ `priceMatch` และละราคา limit ออกจาก API
   request
2. Binance accept order และกำหนดราคาที่ใช้งานจริง
3. Nautilus สร้าง event `OrderAccepted`
4. หากราคาที่ Binance accept ต่างจากราคาอ้างอิง Nautilus จะสร้าง
   event `OrderUpdated` พร้อมราคาที่ใช้งานจริง
5. ราคาของ order ใน cache ของ Nautilus จะตรงกับราคาที่ Binance accept

#### ตัวอย่าง {#example}

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python (limit order ของ `BTCUSDT-PERP.BINANCE` ที่ราคาอ้างอิง 65000 พร้อม `price_match` เป็น `QUEUE`) จึงตัดออก

:::note
หาก Binance accept order ที่ราคาอื่น (เช่น 64,995.50) คุณจะ
ได้รับ event `OrderAccepted` ตามด้วย event `OrderUpdated` พร้อม
ราคาใหม่
:::

### RPI {#rpi}

Binance RPI (Retail Price Improvement) ใช้ `timeInForce=RPI` เป็นแบบ post-only และ match เฉพาะกับ
retail order ที่เข้าเกณฑ์จาก Binance App หรือ Web

Nautilus เปิดให้ใช้ผ่านพารามิเตอร์ `rpi` เฉพาะของ Binance ใช้กับ LIMIT order ของ USD-M ที่
`post_only=true` เท่านั้น รองรับเฉพาะ command `SubmitOrder` แบบเดี่ยว `SubmitOrderList` จะถูกปฏิเสธ
หากไม่มี `rpi` post-only order ปกติจะยังใช้ `GTX` ดู
[USD-M Futures API definitions](https://developers.binance.com/docs/derivatives/usds-margined-futures/common-definition) ของ Binance
สำหรับรายละเอียดของ venue

RPI ใช้ได้เฉพาะ symbol ที่ `permissionSets` มี `RPI` ใน response ของ `GET /fapi/v1/exchangeInfo`
ให้ตรวจสอบสิทธิ์ของ symbol ก่อน submit RPI order ดู
[RPI guide](https://www.binance.com/en/support/faq/detail/92c83c53173947c4a44f9a7277c3b9ce) ของ Binance

ตัวอย่าง Rust สมมติว่า `order` เป็น post-only LIMIT order ของ symbol ที่เข้าเกณฑ์

```rust tab="Rust"
use nautilus_core::params::Params;

let params = Params::new([("rpi".to_string(), true.into())].into());
self.submit_order(order, None, None, Some(params))?;
```

### Close position {#close-position}

conditional order ของ Binance Futures รองรับ `closePosition` ซึ่งปิดทั้ง position
เมื่อ trigger ทำงาน Binance จะ resolve จำนวนฝั่ง server จากขนาด position ปัจจุบัน
ณ เวลาที่ trigger ดู
[USD-M Algo Service API](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/New-Algo-Order)
และ [COIN-M Algo Service API](https://developers.binance.com/docs/derivatives/coin-margined-futures/trade/rest-api/New-Algo-Order) อย่างเป็นทางการ

ต่างจาก `reduce_only` ตรงที่ `closePosition` ปรับตามการเปลี่ยนแปลงขนาด position และ Binance
จะ cancel order อัตโนมัติเมื่อ position ถูกปิดด้วยวิธีอื่น

ตั้ง `reduce_only=true` บน order แบบ `StopMarket` หรือ `MarketIfTouched` ของ Nautilus แล้วส่ง
`close_position=true` ใน `params` ของมัน flag reduce-only จะบันทึกเจตนาการปิดของ order และ
จำเป็นเพื่อให้ order ผ่านได้ขณะที่ trading state เป็น `REDUCING` adapter จะแปลง
การรวมกันนี้เป็นคำสั่ง close-all ของ Binance และปฏิเสธ `close_position` ใน order list

อนุญาตการปิดทั้ง position ของ Binance ใน config ของ risk engine โดยใส่ venue `BINANCE` ไว้ใน
`full_position_exit_venues`

allowlist นี้มีค่า default เป็นว่าง หากไม่มี entry นี้ จำนวน placeholder จะถูกตรวจสอบ
จำนวนต่ำสุด จำนวนสูงสุด และ notional เหมือน order ทั่วไป ให้ส่ง position ID
ที่เปิดอยู่เมื่อ submit order เพื่อให้ risk engine ตรวจสอบได้ว่าการปิดนี้ลด position นั้น

```rust tab="Rust"
let params = Params::from([("close_position", true.into())]);
self.submit_order(order, Some(position.id), None, Some(params))?;
```

:::info
order ของ Nautilus ต้องตั้ง `reduce_only=true` แต่ Binance ไม่อนุญาตให้ใช้ฟิลด์ `reduceOnly`
ร่วมกับ `closePosition=true` adapter จึงส่ง `closePosition=true` โดยละ
`quantity` และ `reduceOnly` ออกจาก request ของ Binance ใน Hedge Mode จะส่ง
`positionSide` ฝั่งที่ปิดด้วย

สำหรับ venue ที่อยู่ใน allowlist risk engine ยังคงตรวจสอบ precision และความเป็นบวกของจำนวน,
trigger price, รูปร่างและ side ของ order และ position ที่เปิดอยู่ซึ่งผูกไว้ แต่ไม่ได้ใช้
ขอบเขตจำนวนต่ำสุดหรือสูงสุดและ notional กับจำนวน placeholder
:::

:::warning
เพิ่ม venue เฉพาะเมื่อ execution client ที่ตั้งค่าไว้บังคับใช้ความหมายของการปิดทั้ง position เท่านั้น
execution client ที่ไม่ตีความ `close_position` อาจ submit เพียงจำนวน placeholder
ผ่านเส้นทาง reduce-only มาตรฐาน แทนที่จะปิดทั้ง position
:::

### Trailing stop {#trailing-stops}

สำหรับ trailing stop market order บน Binance:

- ใช้ `activation_price` (optional) เพื่อระบุว่ากลไก trailing จะเริ่มทำงานเมื่อใด
- หากไม่ระบุ Binance จะใช้ราคาตลาดปัจจุบัน ณ เวลาที่ submit
- ใช้ `trailing_offset` สำหรับ callback rate โดยใช้ `TrailingOffsetType.BASIS_POINTS` adapter
  จะปฏิเสธ offset type อื่น และปฏิเสธ callback rate ที่อยู่นอกช่วง 0.1% ถึง 10% ของ Binance

:::warning
อย่าใช้ `trigger_price` สำหรับ trailing stop order เพราะจะล้มเหลวพร้อม
error ให้ใช้ `activation_price` แทน
:::

## Link & Trade {#link--trade}

adapter จะเติม prefix ของ integration ID ของ NautilusTrader ให้กับ client order ID ที่รองรับ สำหรับ
โปรแกรม [Link and Trade](https://www.binance.com/en/support/faq/detail/a78a065d0c4846aaa1af474d8e712ab9)
ของ Binance โดยผู้ใช้ไม่ต้องตั้งค่าใด ๆ

adapter บีบอัดค่า `ClientOrderId` ขาออกให้พอดีกับขีดจำกัด 36 ตัวอักษรของ Binance และ
decode event ขาเข้าก่อนถึง strategy รูปแบบที่รองรับได้แก่ ID แบบตัวเลขจาก factory,
UUID และ ID จาก factory แบบมีขีดกลางพร้อม tag ของ trader หรือ strategy ที่เป็นตัวอักษรและตัวเลขสั้น ๆ เช่น
`O-20260922-160119-V2-000-8`

รูปแบบ tag สั้นรองรับตัวอักษรหรือตัวเลข ASCII หนึ่งหรือสองตัวใน tag หนึ่งตัว ส่วน tag อีกตัวต้อง
เป็นค่าตัวเลขใน `[0, 1023]` เติมให้มีอย่างน้อยสามหลัก ตัวนับรองรับ `[0, 4194303]`
โดยไม่มีศูนย์นำหน้า และ timestamp ครอบคลุม `[2020-01-01 00:00:00, 2156-02-07 06:28:15]` UTC
การเปิด `use_uuid_client_order_ids` เป็นทางเลือก

custom ID ที่ยาวไม่เกิน 24 byte ไม่ต้องบีบอัด สำหรับ ID ที่ยาวกว่าและไม่รองรับ adapter จะ log
warning และส่ง ID เดิมโดยไม่มี prefix
ข้อจำกัดด้านความยาวและตัวอักษรของ Binance ยังคงมีผล

### Order ที่มีอยู่แล้วหลังการอัปเกรด {#existing-orders-after-an-upgrade}

การเข้ารหัสแบบตัวเลข, UUID และแบบมี raw prefix ที่มีอยู่แล้วจะไม่เปลี่ยนแปลง decoder ยังรับ
ID แบบเก่าที่ไม่มี prefix ด้วย เมื่อ operation กับ order ที่มีอยู่แล้วมี ID แบบ tag สั้นแต่ไม่มี venue order
ID adapter จะ query ทั้งรูปแบบที่เข้ารหัสและรูปแบบเก่าที่ไม่มี prefix หากทั้งสองระบุ
order ต่างกัน จะปฏิเสธ operation นั้นเพราะกำกวม

การแก้ไขและการ cancel ใช้ venue order ID ที่กู้คืนได้ cancel-replace ของ Spot จะคง
wire client order ID เดิมไว้เมื่อแทนที่ order เชิงตรรกะเดียวกัน รวมถึง order ที่ submit
ด้วย ID แบบเก่าที่ไม่มี prefix การค้นหาเหล่านี้ใช้กับกรณีที่การแก้ไขใช้ transport แบบ WebSocket ด้วย

### การ decode client order ID {#decoding-client-order-ids}

เมื่อ query Binance โดยตรง (REST API, web UI หรือโค้ด HTTP ของคุณเอง)
ฟิลด์ `clientOrderId` จะเป็นรูปแบบที่เข้ารหัสแล้ว มี utility function สองตัวที่กู้คืน
`ClientOrderId` เดิมของ Nautilus ได้ คือ `decode_binance_spot_client_order_id` และ
`decode_binance_futures_client_order_id` เช่น `x-TD67BGP9-T0000000000000` (Spot) และ
`x-aHRE4BCj-T0000000000000` (Futures) จะ decode กลับเป็น `O-20200101-000000-000-000-0`

string ที่ไม่มี prefix ของ broker จะผ่านไปโดยไม่เปลี่ยนแปลง จึง
เรียกใช้กับค่า `clientOrderId` ใดก็ได้อย่างปลอดภัย

:::note
adapter จะ decode ให้อัตโนมัติทุกที่ที่ return type ของ Nautilus เช่น `OrderStatusReport`
การ decode เองจำเป็นเฉพาะเมื่อทำงานนอก adapter เท่านั้น เช่น query REST โดยตรง, web UI ของ Binance
หรือ payload ดิบจาก venue
:::

## Order book {#order-books}

order book สามารถดูแลได้ทั้งแบบ depth เต็มหรือบางส่วน stream ของ diff-depth และอัตราการ update
แตกต่างกันตามผลิตภัณฑ์และ transport ของ Spot:

| Product / transport | Diff-depth stream    | อัตราการ update      |
| ------------------- | -------------------- | ---------------- |
| Spot SBE            | `<symbol>@depth`     | 25ms             |
| Spot JSON           | `<symbol>@depth`     | 1000ms (default) |
| Futures             | `<symbol>@depth@0ms` | ไม่ throttle      |

book subscription จะปล่อย snapshot ไม่ว่าจะ seed จาก REST หรือได้รับเป็น frame แบบ partial-depth
พร้อม flag ดังนี้:

- delta ทุกตัวของ snapshot มี `F_SNAPSHOT` และ delta สุดท้ายมี `F_SNAPSHOT | F_LAST`
- snapshot ที่ไม่มีระดับราคาจะเป็น `Clear` ตัวเดียวที่ล้าง book

### Futures L2 subscription {#futures-l2-subscriptions}

depth ของ subscription แบบ `L2_MBP` เป็นตัวเลือก stream:

| Depth                       | Stream                         | แหล่งของ book               |
| --------------------------- | ------------------------------ | ------------------------- |
| 5, 10, or 20                | `<symbol>@depth<levels>@100ms` | snapshot ในทุก message  |
| None, 50, 100, 500, or 1000 | `<symbol>@depth@0ms`           | REST snapshot แล้วตามด้วย diff |
| ค่าอื่น ๆ                    | None                           | ถูกปฏิเสธ                  |

- **Partial depth**: Binance มี stream แบบ partial-depth เฉพาะที่ 5, 10 และ 20 ระดับ แต่ละ message
  จะ snapshot ทั้งสองฝั่งด้วย delta `Clear` ตามด้วยระดับราคาของ snapshot จึงลบราคาที่ไม่มีอยู่
  และเก็บระดับราคาไว้ไม่เกินจำนวนที่ request ต่อฝั่ง subscription เหล่านี้ไม่เคย request REST
  snapshot รวมถึงหลัง reconnect ด้วย
- **Diff depth**: depth จำกัด REST snapshot ตอนเริ่มต้น ตอน reconnect และตอน recovery ไม่ได้จำกัด book
  ที่ดูแลอยู่ หากไม่ระบุจะเลือก snapshot 1000 ระดับ update ที่ตามมาอาจเพิ่มระดับราคาเกิน depth นั้นได้

accessor `OrderBook.bids(depth=...)` และ `OrderBook.asks(depth=...)` จำกัดผลลัพธ์
ที่ return โดยไม่ลบระดับราคาที่เก็บไว้ ให้ unsubscribe ก่อนเปลี่ยน depth ของ subscription ของ
instrument

### Spot L2 subscription {#spot-l2-subscriptions}

subscription แบบ partial-depth ของ Spot ส่ง snapshot top-N ที่สมบูรณ์ในตัว depth ที่รองรับ
ขึ้นกับ [Spot market data mode](#spot-market-data-mode):

| Depth                                   | JSON                     | SBE                |
| --------------------------------------- | ------------------------ | ------------------ |
| 5 or 10                                 | `<symbol>@depth<levels>` | ถูกปฏิเสธ ให้ใช้ JSON |
| 20                                      | `<symbol>@depth20`       | `<symbol>@depth20` |
| None (diff depth)                       | `<symbol>@depth`         | `<symbol>@depth`   |
| ค่าอื่น ๆ รวมถึง 50, 100, 500, 1000 | ถูกปฏิเสธ                 | ถูกปฏิเสธ           |

- depth ที่ถูกปฏิเสธจะล้มเหลวก่อน subscribe ในโหมด JSON error จะแสดงรายการ depth ที่ใช้ได้
- subscription แบบ diff-depth จะ seed ด้วย REST snapshot 5000 ระดับ
- ให้ unsubscribe ก่อนเปลี่ยน depth ของ subscription ของ instrument subscription แบบ partial-depth ใหม่จะไม่
  ลบ stream เดิม

### L1 top-of-book subscription {#l1-top-of-book-subscriptions}

subscription แบบ `L1_MBP` ต้องใช้ depth 1 และใช้ stream `bestBidAsk` หรือ `bookTicker` ของ Spot
และ stream `bookTicker` ของ Futures แต่ละ update จะปล่อย `QuoteTick` ตามปกติ
และ batch `OrderBookDeltas` สองฝั่งพร้อม flag `F_MBP` เพื่อให้ L1 book ที่ดูแลอยู่ได้รับ
สถานะ top-of-book เดียวกัน

subscription ของ quote และ L1 ใช้ stream ของ venue ร่วมกันผ่าน reference counting client
จะปฏิเสธ subscription แบบ L1 และ L2 พร้อมกันสำหรับ instrument เดียวกัน

### Snapshot request {#snapshot-requests}

request snapshot ของ order book แบบระบุชัดเจนรองรับแยกจากการ synchronize ของ subscription:

- **Spot**: depth ใน [1, 5000]
- **Futures**: depth 5, 10, 20, 50, 100, 500 หรือ 1000

### การ synchronize และ recovery ของ snapshot {#snapshot-synchronization-and-recovery}

subscription แบบ diff-depth ของ Futures และ subscription แบบ `L2_MBP` ของ Spot ที่ไม่ระบุ depth จะคง diff-depth
stream ไว้ และ seed book จาก REST snapshot ผ่าน
[กลไก recovery ของ book ที่ใช้ร่วมกัน](../developer_guide/adapters.md#order-book-recovery-ownership)

#### การ synchronize {#synchronization}

การ synchronize เริ่มที่ diff แรกหลังการ subscribe หรือหลังการ reconnect ของ data WebSocket:

1. diff จะถูก buffer และ request REST snapshot
1. diff ใน buffer ที่ครอบคลุมโดย `lastUpdateId` ของ snapshot จะถูกทิ้ง
1. diff ที่เหลือต้องต่อเนื่องจาก snapshot มิฉะนั้นจะ request snapshot ใหม่
1. snapshot จะถูกส่งไปยัง `DataEngine` ตามด้วย diff ที่เหลือใน buffer
1. diff แต่ละตัวที่ตามมาจะถูกตรวจสอบเทียบกับตัวก่อนหน้าก่อนส่ง

diff แต่ละตัวต้องต่อเนื่องจาก snapshot หรือ diff ก่อนหน้า:

| Product | diff แรกหลัง snapshot | diff ถัด ๆ ไป           |
| ------- | ----------------------------- | --------------------- |
| Spot    | `U <= lastUpdateId + 1`       | `U <= previous u + 1` |
| Futures | `U <= lastUpdateId <= u`      | `pu == previous u`    |

diff ที่ผิดกฎเหล่านี้ถือเป็น sequence gap: การส่ง book จะหยุด diff จะถูก buffer และ
request snapshot ใหม่โดยไม่ต้อง resubscribe diff ที่ parse ไม่ได้จะปรากฏเป็น gap ที่
diff ถัดไป

#### ขีดจำกัดของ recovery {#recovery-limits}

- **Snapshot timeout**: `book_snapshot_timeout_secs` (default **10 วินาที**) จำกัดเวลาของ snapshot
  request แต่ละครั้ง ตั้งเป็น `0` เพื่อให้ใช้ timeout ของ HTTP client
- **งบ retry**: recovery แต่ละครั้งพยายาม snapshot **ได้สูงสุดแปดครั้งภายใน 180 วินาที** ด้วย exponential
  backoff และ jitter จากนั้นจะ retry ด้วยช่วงเวลาที่เพิ่มเป็นสองเท่าจากหนึ่งนาทีถึงสิบห้านาที ความล้มเหลว
  ของ request แบบถาวรจะข้ามไปใช้ช่วงเวลานั้นทันที
- **Reconnect**: การ reconnect จะเริ่มการ synchronize ใหม่จาก stream ใหม่ และคง recovery ที่กำลังทำงานไว้พร้อม
  งบที่เหลือ recovery ที่รอระหว่างความพยายามหลังหมดงบแล้วจะ retry ทันที snapshot ถัดไปของ recovery นั้น
  อาจ seed book ได้ก่อนที่ stream ใหม่จะส่ง diff มา diff แรกจึงต้องต่อเนื่อง
  จาก snapshot
- **ความล้มเหลวต่อเนื่อง**: recovery จะดำเนินต่อไปจนกว่า snapshot จะเชื่อม diff ใน buffer ได้หรือ
  book ถูก unsubscribe book อื่น ๆ ทำงานต่อไปอย่างอิสระ

#### การ pace snapshot {#snapshot-pacing}

snapshot request ใช้ส่วนแบ่งต่อ client ของงบ request weight ของ venue ดังนั้นการ resync จำนวนมากพร้อมกัน
เช่นหลัง reconnect จะรองบแทนที่จะใช้เกิน:

| Product | งบ snapshot  | Burst (ครึ่งหนึ่งของงบ) | ต้นทุน snapshot เต็ม |
| ------- | ---------------- | ----------------------- | ------------------ |
| Spot    | 3,000 per minute | 1,500                   | 250 (5000 levels)  |
| Futures | 1,200 per minute | 600                     | 20 (1000 levels)   |

- การรอคิวสำหรับ snapshot แรกไม่นับรวมในจำนวนความพยายามหรืองบ 180 วินาทีของ recovery
- snapshot request แบบระบุชัดเจนใช้งบเดียวกัน
- การ retry ของ HTTP client สำหรับ snapshot request ที่ล้มเหลวไม่ถูก pace

### การตรวจสอบ recovery แบบ live {#live-recovery-validation}

harness `binance-book-stress` เป็นเครื่องมือพัฒนาสำหรับการเปลี่ยนแปลงการ synchronize และ recovery ของ book มัน
เชื่อมต่อกับ market data ของผลิตภัณฑ์ที่เลือกโดยไม่ submit order ใด ๆ และตรวจสอบ book ที่ปล่อยออกมาเทียบกับ
contract ของ book stream และ oracle อิสระสองตัว:

- stream `<symbol>@depth20@100ms` ของ venue ที่ update ID ตรงกัน
- book อ้างอิงที่สร้างขึ้นใหม่จาก diff ดิบและ REST snapshot ที่ส่งต่อมา

harness จะทิ้ง diff เพื่อบังคับให้เกิด gap ทำให้ REST snapshot ล้มเหลว หน่วง และถูกปฏิเสธ ตัดและ freeze
การเชื่อมต่อ ร้องขอการ reconnect และหมุนเวียน subscription REST proxy ของมันจะปฏิเสธ snapshot request
ก่อนที่ request weight ของ venue จะใกล้ถึงขีดจำกัด และการรันจะล้มเหลวหาก venue throttle

จาก root ของ repository ให้รัน:

```bash
CARGO_BUILD_JOBS=16 bash scripts/strip-adapter-env.bash \
  cargo test -p nautilus-binance --features examples --test binance-book-stress -- --product futures
```

`--product` เลือก venue:

| Product          | Venue                                                                       |
| ---------------- | --------------------------------------------------------------------------- |
| `spot` (default) | JSON stream ของ Spot mainnet                                                  |
| `spot-sbe`       | SBE stream ของ Spot mainnet; อ่าน `BINANCE_API_KEY` และ `BINANCE_API_SECRET` |
| `futures`        | USD-M testnet                                                              |
| `coinm`          | COIN-M testnet                                                             |

ให้รัน `spot-sbe` โดยไม่ใช้ `strip-adapter-env.bash` เพราะสคริปต์นั้นจะ unset API key ที่ต้องใช้

`--scenario` เลือกรูปแบบการรัน:

- `churn` (default): หมุนเวียน fault แบบ gap, reconnect, subscription churn, cut และ freeze
- `boundaries`: ทดสอบ deadline ของ snapshot, การ retry จนหมดจนถึงเพดาน retry และการ
  ปฏิเสธแบบถาวร
- `resubscribe`: ให้ unsubscribe แข่งกับการ resubscribe ทันทีหนึ่งครั้งต่อรอบ
- `quiet`: เฝ้าดู book ที่มีการซื้อขายน้อย หนึ่งรอบต่อนาที
- `crowd`: subscribe book ที่มีสภาพคล่องและ reconnect หนึ่งครั้งต่อรอบเพื่อให้การ pace snapshot ทำงาน

`--timeout` ตั้ง snapshot timeout เป็นวินาที โดย `0` คือปิด deadline ของ snapshot และ
`--rounds` ตั้งจำนวนรอบ (default 14) `--books` ตั้งจำนวน book ที่ `quiet` และ
`crowd` เลือกตามจำนวน trade ใน 24 ชั่วโมง (default 4)

harness ต้องเข้าถึง REST API และ WebSocket market data stream ของผลิตภัณฑ์ที่เลือกได้
test lifecycle ของ book แบบอัตโนมัติใช้ mock server ภายในเครื่อง ดู
[Stress harnesses](../developer_guide/spec_data_testing.md#stress-harnesses) สำหรับ flag ที่ใช้ร่วมกัน
และรูปแบบ output

## Timestamp ของ quote {#quote-timestamps}

ฟิลด์ `ts_event` บน `QuoteTick` แตกต่างกันตาม transport Spot SBE ใช้
event timestamp หน่วยไมโครวินาที message `bookTicker` แบบ JSON สาธารณะของ Spot อาจไม่มี event
timestamp ซึ่งในกรณีนั้น adapter จะใช้ `ts_init` ส่วน Futures ใช้ transaction time

## Bar และ market data ย้อนหลัง {#bars-and-historical-market-data}

Spot รองรับ kline ราย 1 วินาทีสำหรับ subscription และ historical request kline subscription
แบบ real-time ของ Spot ต้องใช้ `spot_market_data_mode=Json` เพราะ Binance ไม่ได้
publish stream ของ kline หรือ ticker ผ่าน Spot SBE ส่วน Binance Futures ปฏิเสธ kline
ระดับวินาที เพราะ Futures API ไม่มีให้

kline ของ venue ที่ปิดแล้วจะปล่อย `Bar` ของ core และ event custom data แบบ `BinanceBar` โดย `BinanceBar` เก็บ quote volume,
จำนวน trade, taker-buy base volume และ taker-buy quote volume ไว้ historical request ของ core bar จะ return `Bar`
ให้ request custom data แบบ `BinanceBar` พร้อม metadata `bar_type` เพื่อเก็บฟิลด์เพิ่มเติมไว้ใน historical
response

trade subscription แบบ real-time ใช้ stream `<symbol>@aggTrade` บน Futures เพราะ Binance
publish เฉพาะ aggregated trade บน WebSocket ของ Futures และใช้ stream `<symbol>@trade` แบบรายตัวบน
Spot

historical trade request ที่ไม่มีขอบเขตจะใช้ endpoint recent-trades request ที่มีขอบเขตเวลาจะใช้
aggregate trades และรับได้สูงสุด 1000 record ดังนั้นแหล่งข้อมูลจึงขึ้นกับ request ไม่ใช่ตัวเลือกใน config
Spot ส่งขอบเขตที่ระบุไปยัง `/api/v3/aggTrades` ส่วน Futures กำหนดให้ขอบเขตที่ระบุแต่ละตัวต้อง
อยู่ภายใน 48 ชั่วโมงล่าสุด และเมื่อระบุทั้งสองขอบเขต ช่วงเวลาต้องสั้นกว่าหนึ่งชั่วโมง

historical request ของ core bar รับ time bar ที่ aggregate ภายนอก และใช้ kline endpoint ของ venue
ที่ตรงกัน ส่วน bar ที่ aggregate ภายในจะถูกสร้างโดย `DataEngine` จาก response ของ trade, quote หรือ source-bar ดิบ
ผ่านพารามิเตอร์ request `bar_types` โดย Binance data client ไม่ได้ aggregate ให้

## ข้อมูลเฉพาะของ Binance {#binance-specific-data}

bar, mark price, index price และ funding rate subscribe ได้ตามปกติ custom
data type ด้านล่างเปิดให้เข้าถึงฟิลด์เฉพาะ venue เพิ่มเติมที่ data type ของ core ไม่มี

payload ของ mark-price ใน Binance Futures เก็บราคา settlement โดยประมาณ `P` ของ venue ไว้ใน
`BinanceFuturesMarkPriceUpdate` Nautilus ยังปล่อย update มาตรฐานของ mark-price, index-price และ
funding-rate จาก stream เดียวกันด้วย ฟิลด์ moving-average `ap` (optional) ของ USD-M จะถูก
parse ที่ขอบเขตของ transport แต่ไม่ได้เปิดให้ใช้เป็น domain data หรือ custom data

### `BinanceSpotTicker` {#binancespotticker}

custom data ของ ticker 24 ชั่วโมงของ Spot ต้องใช้ market-data mode แบบ JSON สาธารณะ และ
ค่า metadata `instrument_id` (subscribe ด้วย `DataType` ชื่อ `BinanceSpotTicker` พร้อม metadata
`{"instrument_id": "BTCUSDT.BINANCE"}` และ client ID `BINANCE`)

adapter จะ subscribe stream `@ticker` ของ instrument โหมด SBE จะปฏิเสธ
subscription นี้เพราะ Binance Spot SBE ไม่มี stream นี้

### `BinanceFuturesTicker` {#binancefuturesticker}

subscribe สถิติ ticker 24 ชั่วโมงของ Futures instrument ที่ระบุ ด้วย `DataType` ชื่อ
`BinanceFuturesTicker` พร้อม metadata `{"instrument_id": "BTCUSDT-PERP.BINANCE"}`

adapter จะ subscribe stream `@ticker` ของ instrument และปล่อย custom data แบบ `BinanceFuturesTicker` พร้อม
`metadata={"instrument_id": "<instrument_id>"}` custom data ของ ticker ต้องมี `instrument_id` ไม่รองรับ
ticker subscription แบบทั้งตลาด

### `BinanceFuturesMarkPriceUpdate` {#binancefuturesmarkpriceupdate}

subscribe `BinanceFuturesMarkPriceUpdate` (รวมข้อมูล funding rate)
จาก actor หรือ strategy (เช่นใน `on_start`) ด้วย `DataType` ชื่อ `BinanceFuturesMarkPriceUpdate`
พร้อม metadata `instrument_id` ของ instrument

object `BinanceFuturesMarkPriceUpdate` ที่ได้รับจะถูกส่งไปยัง `on_data`
โดยห่อไว้ใน `CustomData` ให้ดึง payload ออกมาและตรวจสอบ
type เพราะ method นี้รับ custom/generic data ทุกชนิด

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

### `BinanceFuturesLiquidation` {#binancefuturesliquidation}

subscribe update ของ liquidation ได้ทั้งแบบ:

- instrument ที่ระบุ (`<symbol>@forceOrder`) หรือ
- ทุก symbol (`!forceOrder@arr`) โดยไม่ระบุ `instrument_id`

ใช้ `DataType` ชื่อ `BinanceFuturesLiquidation` พร้อม metadata `instrument_id` สำหรับแบบระบุ instrument
หรือไม่ใส่ metadata สำหรับแบบทั้งตลาด

สำหรับ subscription แบบระบุ instrument `CustomData.data_type` จะมี
`metadata={"instrument_id": "<instrument_id>"}` สำหรับ subscription แบบทั้งตลาด
data type จะไม่มี metadata

เมื่อ subscribe ทั้งสองโหมดพร้อมกัน แบบทั้งตลาดจะมีลำดับความสำคัญสูงกว่า
adapter จะพัก liquidation stream ราย symbol ขณะที่แบบทั้งตลาด active และ
กู้คืน stream ราย symbol ที่ active หลังจาก unsubscribe แบบทั้งตลาดแล้ว

### Futures open interest {#futures-open-interest}

open interest เป็นแบบ request เท่านั้น Futures data client ไม่มี subscription ของ open interest ทั้งสอง type ต้องมี
metadata `instrument_id` และ `BinanceFuturesOpenInterestHist` ยังต้องมี string `period` ของ Binance เช่น
`"5m"` (request ผ่าน `request_data` ด้วย `DataType` ชื่อ `BinanceFuturesOpenInterest` สำหรับ snapshot ปัจจุบัน
หรือ `BinanceFuturesOpenInterestHist` สำหรับชุดข้อมูลย้อนหลัง)

`BinanceFuturesOpenInterestHist` return batch ของจุดข้อมูล แต่ละจุดมีผลรวมของ open interest
และมูลค่า notional ของมันสำหรับหนึ่ง bucket ประวัติของ COIN-M ใช้ pair และ contract type เป็น key ซึ่ง
adapter derive จาก symbol สำหรับ perpetual และจาก instrument definition ใน cache สำหรับ
สัญญาส่งมอบ

## Funding rate {#funding-rates}

adapter ปล่อย `FundingRateUpdate` เป็น data type ระดับหลักผ่าน
`subscribe_funding_rates` ข้อมูลมาจาก WebSocket endpoint
[Mark Price Stream](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams/Mark-Price-Stream)
ซึ่งให้ funding rate ปัจจุบันและเวลา funding ถัดไป
พร้อมกับ mark price และ index price subscription ทั้งสาม
(`subscribe_mark_prices`, `subscribe_index_prices`, `subscribe_funding_rates`)
ใช้ stream `@markPrice@1s` ร่วมกันโดยจัดการ subscription แบบ ref-counted

funding rate ย้อนหลังใช้ได้ผ่าน `request_funding_rates` ซึ่ง
query REST endpoint
[Get Funding Rate History](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Get-Funding-Rate-History)
(`GET /fapi/v1/fundingRate` สำหรับ USD-M, `GET /dapi/v1/fundingRate`
สำหรับ COIN-M) แต่ละแถวของประวัติ map ไปเป็น `FundingRateUpdate` โดยตั้ง `ts_event`
เป็นเวลา funding ฟิลด์ `next_funding_ns` เป็น `None` สำหรับแถวย้อนหลัง
เพราะ endpoint ไม่ได้ให้ค่านี้

adapter ยังเปิดให้เข้าถึง payload ของ venue ผ่าน subscription ของ custom data แบบ `BinanceFuturesMarkPriceUpdate`
(ดู [ข้อมูลเฉพาะของ Binance](#binance-specific-data))

สำหรับ Binance `FundingRateUpdate.interval` เป็น `None` Mark Price Stream และ endpoint ประวัติ funding rate
ไม่มีฟิลด์ funding interval Binance เปิดให้ใช้ `fundingIntervalHours` ผ่าน REST endpoint
[Get Funding Rate Info](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Get-Funding-Rate-Info)
แต่ adapter ไม่ได้ใช้

## การ poll สถานะ instrument {#instrument-status-polling}

data client จะ poll `exchangeInfo` ของ Binance เป็นระยะเพื่อตรวจจับการเปลี่ยนแปลง
สถานะการซื้อขายของ instrument เมื่อ symbol เปลี่ยนสถานะ (เช่น
Trading เป็น Halt หรือ Trading เป็น Delivering สำหรับสัญญา futures ที่ใกล้
หมดอายุ) adapter จะปล่อย event `InstrumentStatus`

ช่วงเวลา poll มีค่า default 3,600 วินาที (60 นาที) และตั้งค่าได้
ผ่าน `instrument_status_poll_secs` ใน config ของ data client ตั้งเป็น `0` เพื่อ
ปิดการ poll ทั้งหมด

ตอนเชื่อมต่อครั้งแรก adapter จะ seed status cache จาก response ของ exchange info
โดยไม่ปล่อย event เฉพาะการ poll ครั้งถัดไปที่ตรวจพบการเปลี่ยนสถานะ
เท่านั้นที่จะปล่อย event `InstrumentStatus` หาก symbol หายไปจาก exchange
info (เช่น หลัง delist หรือสัญญาหมดอายุ) adapter จะปล่อย
`NotAvailableForTrading`

การ poll สถานะไม่ได้โหลด instrument definition ใหม่ task `instrument_refresh_interval_secs` ที่แยกต่างหาก
จะโหลด catalog ที่กรองแล้วทั้งหมด และแทนที่ lookup map ของ data client และ WebSocket แบบ atomic
จากนั้นจะอัปเดต status snapshot และส่ง instrument ที่ refresh แล้วไปยัง data engine และยัง refresh
precision cache ของ execution client ด้วย

ช่วงเวลา refresh เต็มมีค่า default 3,600 วินาที ตั้งเป็น `0` เพื่อปิด การ disconnect จะ cancel task นี้
และการ reconnect จะเริ่ม task ทดแทนหนึ่งตัวพร้อม cancellation token ใหม่

การ refresh catalog และการ poll สถานะจะเรียงลำดับ request และการอัปเดต snapshot ของตน เพื่อไม่ให้ response ที่ซ้อนกัน
เขียนทับสถานะที่ใหม่กว่าหรือทำให้ event `InstrumentStatus` ซ้ำ แต่ละ task มีช่วงเวลา poll ของตัวเอง

### การ map สถานะ {#status-mapping}

#### Spot {#spot}

| Binance status   | MarketStatusAction     |
| ---------------- | ---------------------- |
| Trading          | Trading                |
| EndOfDay         | Close                  |
| Halt             | Halt                   |
| Break            | Pause                  |
| CancelOnly       | Halt                   |
| NonRepresentable | NotAvailableForTrading |

Binance US จะ poll exchange info แบบ JSON สาธารณะแทน ซึ่ง map `TRADING` เป็น `Trading`, `BREAK`
เป็น `Pause` และค่าอื่นทั้งหมดเป็น `NotAvailableForTrading`

#### Futures (USD-M) {#futures-usd-m}

| Binance status    | MarketStatusAction |
| ----------------- | ------------------ |
| Trading           | Trading            |
| PendingTrading    | PreOpen            |
| PreTrading        | PreOpen            |
| PostTrading       | PostClose          |
| EndOfDay          | Close              |
| Halt              | Halt               |
| AuctionMatch      | Cross              |
| Break             | Pause              |
| PreDelivering     | PreClose           |
| Delivering        | Close              |
| Delivered         | Close              |
| PreSettle         | PreClose           |
| Settling          | Close              |
| Close             | Close              |
| TradingHalt       | Halt               |
| TradingCancelOnly | Halt               |

#### Futures (COIN-M) {#futures-coin-m}

| Binance status    | MarketStatusAction     |
| ----------------- | ---------------------- |
| Trading           | Trading                |
| PendingTrading    | PreOpen                |
| PreDelivering     | PreClose               |
| Delivering        | Close                  |
| Delivered         | Close                  |
| PreSettle         | PreClose               |
| Settling          | Close                  |
| Close             | Close                  |
| PreDelisting      | PreClose               |
| Delisting         | Suspend                |
| Down              | NotAvailableForTrading |
| TradingHalt       | Halt                   |
| TradingCancelOnly | Halt                   |

ค่าสถานะของ Futures ที่ไม่รู้จักหรือไม่มีในเอกสารจะ map เป็น `NotAvailableForTrading`

:::note
จะติดตามเฉพาะ instrument ที่อยู่ในสถานะซื้อขายได้ ณ เวลาที่เชื่อมต่อเท่านั้น
symbol ที่เริ่มต้นในสถานะที่ซื้อขายไม่ได้ (เช่น halt อยู่ตอนเชื่อมต่อ) จะไม่
ปรากฏใน instruments cache จึงไม่ได้ monitor การเปลี่ยนสถานะของ symbol เหล่านั้น
:::

## Rate limiting {#rate-limiting}

Binance ใช้ระบบ rate limiting แบบตามช่วงเวลา โดยติดตาม request weight
ในหน้าต่างเวลาคงที่ (ทุกนาที reset ที่วินาที :00) API
endpoint แต่ละตัวมีต้นทุน weight ที่กำหนดไว้ และการใช้ weight ทั้งหมดจะถูกติดตาม
ต่อ IP address

### ขีดจำกัด weight ของ venue {#venue-weight-limits}

โควตา weight ต่อ IP ของ Binance เอง ซึ่งต้นทุนของ endpoint ด้านล่างดึงไปใช้:

| Product     | Weight limit | Interval |
| ----------- | ------------ | -------- |
| Spot/Margin | 6,000        | 1 minute |
| Futures     | 2,400        | 1 minute |

### ต้นทุน weight ของ endpoint {#endpoint-weight-costs}

Binance คิด weight ต่อ request ดังนี้:

| Endpoint                  | Weight | หมายเหตุ                                  |
| ------------------------- | ------ | -------------------------------------- |
| `/api/v3/order`           | 1      | การวาง order ของ Spot                  |
| `/api/v3/allOrders`       | 20     | order ย้อนหลังของ Spot (แพง)    |
| `/api/v3/klines`          | 2+     | เพิ่มตามพารามิเตอร์ `limit`         |
| `/api/v3/depth`           | 5+     | เพิ่มตาม `limit`; 250 ที่ 5000      |
| `/fapi/v1/order`          | 1      | การวาง order ของ Futures               |
| `/fapi/v1/algoOrder`      | 0      | ใช้ขีดจำกัดจำนวน order               |
| `/fapi/v1/allOrders`      | 20     | order ย้อนหลังของ Futures (แพง) |
| `/fapi/v1/commissionRate` | 20     | query อัตรา commission ของ Futures         |
| `/fapi/v1/klines`         | 5+     | เพิ่มตามพารามิเตอร์ `limit`         |
| `/fapi/v1/depth`          | 2+     | เพิ่มตาม `limit`; 20 ที่ 1000       |

`POST /fapi/v1/algoOrder` ของ USD-M Futures ใช้ `1` จากทั้ง `X-MBX-ORDER-COUNT-10S` และ
`X-MBX-ORDER-COUNT-1M` Binance ไม่คิด IP request weight สำหรับ endpoint นี้ แต่ adapter ยังคงเข้าคิว
ผ่าน global bucket เป็นส่วนหนึ่งของโมเดลการ pace ภายในเครื่อง

### ขีดจำกัดของ WebSocket API {#websocket-api-limits}

WebSocket API (ใช้สำหรับการส่ง order และ user data stream) ใช้โควตา weight เดียวกับ
REST API:

| ประเภทขีดจำกัด       | ค่า  | หมายเหตุ                                 |
| ---------------- | ------ | ------------------------------------- |
| Request weight   | Shared | นับรวมกับโควตา weight ของ REST API |
| Handshake        | 5      | ต้นทุน weight ต่อความพยายามเชื่อมต่อหนึ่งครั้ง   |
| Ping/pong frames | 5/sec  | อัตรา ping/pong สูงสุด                |

### การ pace ของ adapter {#adapter-pacing}

adapter รัน token bucket limiter ของตัวเองก่อนการนับของ venue ทุก HTTP request
จะดึง token หนึ่งตัวจาก request bucket ต่อผลิตภัณฑ์ และทุก order operation จะดึง
token เพิ่มอีกหนึ่งตัวจาก order-count bucket:

| Product                    | Requests     | Order operations             |
| -------------------------- | ------------ | ---------------------------- |
| Spot                       | 1,200/minute | 10/second, 100,000/day       |
| Futures (USD-M and COIN-M) | 2,400/minute | 300/10 seconds, 1,200/minute |

order-count bucket ครอบคลุมทุก order operation ที่ผลิตภัณฑ์รองรับ ไม่ใช่เฉพาะการวาง order:
submit, OCO submit, batch submit, algo submit, modify (Spot modify ผ่าน cancel-replace), batch
modify, cancel, cancel-all, batch cancel และ algo cancel strategy ที่ cancel หรือ modify บ่อย
ก็จะถูก throttle ด้วย bucket เหล่านี้เช่นกัน request ที่ authenticate แล้วแต่ไม่ใช่ order เช่นการเปลี่ยน leverage และ
margin type และ keepalive ของ listen-key จะดึงจาก request bucket เท่านั้น

request bucket นับจำนวนการเรียก ไม่ใช่ weight จึงไม่ได้สะท้อนการนับ weight ของ venue การเรียก
endpoint ที่มี weight สูงหรือ weight แบบ dynamic ติดต่อกัน (`/api/v3/allOrders` ที่ weight 20 หรือ `/klines` ที่เพิ่มตาม
`limit`) จะใช้ weight ของ venue เร็วกว่าที่ bucket ภายในเครื่องนับ history request ขนาดใหญ่อาจต้อง pace
เอง ให้ monitor header `X-MBX-USED-WEIGHT-*` ใน response เพื่อติดตามการใช้งานจริงที่ venue snapshot request
ของ order book ยังรองบที่คำนึงถึง weight ด้วย ดู [การ pace snapshot](#snapshot-pacing)

:::warning
Binance return HTTP 429 เมื่อใช้ weight เกินที่อนุญาต การละเมิด
ซ้ำจะทำให้ IP ถูกแบนชั่วคราว (เพิ่มจาก 2 นาทีไปจนถึง 3 วัน
สำหรับผู้ละเมิดซ้ำ)
:::

:::info
สำหรับ rate limit ล่าสุด ให้ query `/api/v3/exchangeInfo` (Spot) หรือ `/fapi/v1/exchangeInfo` (Futures) หรือดู:

- [Spot API Limits](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits)
- [Futures API Limits](https://developers.binance.com/docs/derivatives/usds-margined-futures/general-info)

:::

## การตั้งค่า {#configuration}

### Data client {#data-client}

| Option                             | Default   | คำอธิบาย                                                                    |
| ---------------------------------- | --------- | ------------------------------------------------------------------------------ |
| `product_type`                     | `Spot`    | หนึ่งใน `Spot`, `UsdM` หรือ `CoinM`                                             |
| `environment`                      | `Live`    | หนึ่งใน `Live`, `Testnet` หรือ `Demo`                                           |
| `base_url_http`                    | `None`    | override HTTP endpoint (optional)                                               |
| `base_url_ws`                      | `None`    | override market WebSocket endpoint (optional)                                   |
| `api_key` / `api_secret`           | `None`    | จำเป็นสำหรับ Spot SBE; optional สำหรับ JSON สาธารณะและข้อมูล Futures              |
| `spot_market_data_mode`            | `Sbe`     | `Json` คงเส้นทาง Global Spot ที่ไม่ต้องใช้ credential ส่วน Binance US ต้องใช้ `Json` |
| `instrument_provider`              | default   | การโหลด, filter, parser-warning และ commission policy                       |
| `instrument_refresh_interval_secs` | `3,600`   | ช่วงเวลา refresh catalog เต็ม; `0` คือปิด                                |
| `instrument_status_poll_secs`      | `3,600`   | ช่วงเวลา poll exchange-info เฉพาะสถานะ; `0` คือปิด                      |
| `book_snapshot_timeout_secs`       | `10`      | deadline ของ REST snapshot request แบบ diff-depth แต่ละครั้ง; `0` คือปิด           |
| `proxy_url`                        | `None`    | proxy ที่ใช้กับ HTTP และทุกการเชื่อมต่อ market WebSocket                   |
| `recv_window_ms`                   | `5,000`   | receive window ของ HTTP ที่ sign แล้ว ช่วงรวมปลาย `1..=60000`                       |
| `max_retries`                      | `3`       | จำนวน retry สูงสุดของ HTTP GET request                                         |
| `retry_delay_initial_ms`           | `1,000`   | delay เริ่มต้นของการ retry การอ่าน HTTP หน่วยมิลลิวินาที                                 |
| `retry_delay_max_ms`               | `10,000`  | delay แบบ exponential สูงสุด; ค่าขั้นต่ำจาก venue อาจเกินค่านี้ได้                      |
| `us`                               | `False`   | route live Spot JSON client ไปยัง Binance US                                   |
| `transport_backend`                | `Sockudo` | backend ของ WebSocket transport                                                   |

### Execution client {#execution-client}

| Option                             | Default   | คำอธิบาย                                                             |
| ---------------------------------- | --------- | ----------------------------------------------------------------------- |
| `account_id`                       | Required  | identity ของ account ใน Nautilus                                              |
| `product_type`                     | `Spot`    | หนึ่งใน `Spot`, `UsdM` หรือ `CoinM`                                      |
| `environment`                      | `Live`    | หนึ่งใน `Live`, `Testnet` หรือ `Demo`                                    |
| `base_url_http`                    | `None`    | override HTTP endpoint (optional)                                        |
| `base_url_ws`                      | `None`    | override private stream (optional)                                       |
| `base_url_ws_trading`              | `None`    | override WebSocket trading ของ Global Spot หรือ USD-M (optional)               |
| `use_ws_trading`                   | `True`    | ใช้การส่ง order ผ่าน Global WebSocket เมื่อรองรับ; Binance US ใช้ HTTP |
| `ws_trading_setup_timeout_ms`      | `10,000`  | timeout ของ authentication และ setup ของ WebSocket trading                     |
| `instrument_provider`              | default   | การโหลด, filter, parser-warning และ commission policy                |
| `instrument_refresh_interval_secs` | `3,600`   | ช่วงเวลา refresh precision cache ของ execution; `0` คือปิด            |
| `proxy_url`                        | `None`    | proxy ที่ใช้กับ HTTP, private stream และ WebSocket trading          |
| `recv_window_ms`                   | `5,000`   | receive window ของ HTTP และ WebSocket ที่ sign แล้ว ช่วงรวมปลาย `1..=60000`  |
| `max_retries`                      | `3`       | จำนวน retry สูงสุดของ HTTP GET request                                  |
| `retry_delay_initial_ms`           | `1,000`   | delay เริ่มต้นของการ retry การอ่าน HTTP หน่วยมิลลิวินาที                          |
| `retry_delay_max_ms`               | `10,000`  | delay แบบ exponential สูงสุด; ค่าขั้นต่ำจาก venue อาจเกินค่านี้ได้               |
| `us`                               | `False`   | route live Spot execution client ไปยัง Binance US                       |
| `api_key` / `api_secret`           | `None`    | Global ใช้ WebSocket auth แบบ Ed25519; Binance US ใช้การ sign HTTP แบบ HMAC  |
| `use_gtd`                          | `True`    | ใช้ GTD แบบ native ของ USD-M; ดู [GTD policy](#gtd-policy)                    |
| `use_position_ids`                 | `True`    | เปิดเผย ID ของ Futures บน order, fill และ hedge REST report              |
| `oms_type`                         | `None`    | `None` เลือก Futures netting; ใช้ `Hedging` สำหรับโหมด dual-side       |
| `default_taker_fee`                | `0.0004`  | ค่า fallback สำหรับ fill ของ Futures ที่ exchange สร้างขึ้น                          |
| `futures_leverages`                | `None`    | leverage เริ่มต้นตาม Futures symbol                                     |
| `futures_margin_types`             | `None`    | margin type เริ่มต้นตาม Futures symbol                                  |
| `treat_expired_as_canceled`        | `False`   | map execution event `EXPIRED` เป็น canceled event                      |
| `use_trade_lite`                   | `False`   | ใช้ fill stream แบบ trade-lite ของ USD-M ที่มี latency ต่ำกว่า                     |
| `bnfcr_currency`                   | `USDT`    | สกุลเงินที่ใช้ resolve ยอดคงเหลือและค่าธรรมเนียมแบบ `BNFCR`                     |
| `transport_backend`                | `Sockudo` | backend ของ WebSocket transport                                            |

### การตั้งค่า live node {#live-node-configuration}

ใช้ `BinanceDataClientConfig` คู่กับ `BinanceDataClientFactory` และ `BinanceExecutionClientConfig` คู่กับ
`BinanceExecutionClientFactory` ดู[ตัวอย่าง Rust](#examples) สำหรับการตั้งค่า live node
ที่ใช้ data client และ execution client

### Futures Credits Trading Mode (BNFCR) {#futures-credits-trading-mode-bnfcr}

Binance Futures Credits Trading Mode เป็นโหมดตามข้อกำหนดของ EU ซึ่งระบุมูลค่าของ wallet, margin, PnL และค่าธรรมเนียม
ของ USD-M futures เป็น `BNFCR` หน่วยเครดิตภายในนี้ใช้แทนยอดคงเหลือ stablecoin และผูกค่า 1:1 กับ
USD

`BNFCR` ไม่ใช่สินทรัพย์ที่ซื้อขายได้ adapter จึง map ไปยัง `bnfcr_currency` ใน execution config (default
`USDT`) ดังนั้นยอดคงเหลือของ account และ commission จะ reconcile กับ settlement stablecoin ของสัญญาที่ซื้อขาย
ตั้ง `bnfcr_currency` เป็น `USDC` เมื่อเทรด perpetual ที่ใช้ USDC เป็น margin สินทรัพย์ futures อื่นที่ไม่รู้จักจะถูก
ลงทะเบียนเป็นสกุลเงินคริปโตทั่วไปแทนที่จะล้มเหลว

### Spot market data mode {#spot-market-data-mode}

`spot_market_data_mode` บน `BinanceDataClientConfig` เลือก transport ของข้อมูล Spot มีผลกับ Spot เท่านั้น
Futures ไม่เปลี่ยนแปลง

| Mode   | Credentials        | Quotes       |
| ------ | ------------------ | ------------ |
| `Sbe`  | Ed25519 (required) | `bestBidAsk` |
| `Json` | None (public)      | `bookTicker` |

`Sbe` (default) ใช้ stream แบบ Simple Binary Encoding ของ Binance และต้องใช้ key แบบ Ed25519 (ดู
[ชนิดของ key](#key-types)) client จะปฏิเสธการเชื่อมต่อหากไม่มี key ส่วน `Json` ใช้ stream สาธารณะที่ไม่ต้องใช้
credential

subscription แบบ `BookDeltas` เต็มของ Spot ใช้ diff-depth stream `<symbol>@depth` บน
transport ที่เลือก พร้อมการ synchronize ด้วย REST snapshot ส่วน subscription ที่ระบุ depth ใช้ snapshot ของ book บางส่วน
(ดู [Order book](#order-books))

### ชนิดของ key {#key-types}

adapter sign ด้วย **Ed25519** หรือ **HMAC-SHA256** โดยตรวจจับชนิดของ key จากรูปแบบของ API secret
อัตโนมัติ จึงไม่ต้องตั้งค่าใด ๆ secret ที่ parse ได้เป็น Ed25519 private key แบบ PKCS#8 จะ sign ด้วย
Ed25519 นอกนั้นจะถือเป็น HMAC secret

**แนะนำอย่างยิ่งให้ใช้ Ed25519** Binance แนะนำ Ed25519 เพราะ
ประสิทธิภาพและความปลอดภัยที่เหนือกว่า NautilusTrader เวอร์ชันในอนาคตจะ
รองรับเฉพาะ Ed25519 เท่านั้น

| Key Type | Data Clients | Execution Clients | สถานะ                                           |
| -------- | ------------ | ----------------- | ------------------------------------------------ |
| Ed25519  | ✓            | ✓                 | **แนะนำ**                                  |
| HMAC     | ✓            | ✓                 | Deprecated จะถูกนำออกในเวอร์ชันอนาคต |
| RSA      | -            | -                 | ไม่รองรับ ให้ลงทะเบียน key แบบ Ed25519 แทน  |

:::tip
เปลี่ยนไปใช้ key แบบ Ed25519 ตั้งแต่ตอนนี้ สร้าง keypair แบบ Ed25519 และลงทะเบียนกับ
Binance ดู [การสร้าง key แบบ Ed25519](#generating-ed25519-keys) ด้านล่าง
:::

:::note
key แบบ Ed25519 ต้องอยู่ในรูปแบบ PEM ที่ไม่เข้ารหัส (ASN.1/DER ที่ encode ด้วย base64)
implementation จะดึง seed ขนาด 32 byte ออกจากโครงสร้าง DER อัตโนมัติ
ไม่รองรับ PEM key ที่เข้ารหัส (ป้องกันด้วยรหัสผ่าน) หาก key ของคุณเข้ารหัสอยู่
ให้ถอดรหัสก่อน: `openssl pkey -in encrypted.pem -out decrypted.pem`
:::

#### การสร้าง key แบบ Ed25519 {#generating-ed25519-keys}

**ทางเลือกที่ 1: OpenSSL (แนะนำ)**

```bash
# Generate private key (PKCS#8 PEM format)
openssl genpkey -algorithm ed25519 -out binance_ed25519_private.pem

# Extract public key
openssl pkey -in binance_ed25519_private.pem -pubout -out binance_ed25519_public.pem
```

**ทางเลือกที่ 2: Binance Key Generator**

ดาวน์โหลด
[Binance Asymmetric Key Generator](https://github.com/binance/asymmetric-key-generator)
จากหน้า releases แล้วรันเพื่อสร้าง keypair

**การลงทะเบียนกับ Binance**

1. login เข้า Binance แล้วไปที่ **Profile** -> **API Management**
2. คลิก **Create API** และเลือก **Self-generated**
3. วางเนื้อหาของไฟล์ public key รวมทั้ง header และ footer `-----BEGIN PUBLIC KEY-----`
4. ตั้งค่า permission (Enable Spot & Margin Trading เป็นต้น)

**การใช้กับ NautilusTrader**

ตั้ง private key เป็น API secret:

```bash
export BINANCE_API_KEY="your-api-key-from-binance"
export BINANCE_API_SECRET="$(cat binance_ed25519_private.pem)"
```

หรือส่งเนื้อหา PEM ใน config โดยตรง

:::warning
เก็บ private key ไว้อย่างปลอดภัย ห้ามแชร์หรือ commit ลง version control
:::

### API credential {#api-credentials}

ส่ง credential ให้กับ config object โดยตรง หรือตั้ง environment variable
ที่เหมาะสม (ดู [Environment](#environments) สำหรับตัวแปรของแต่ละ
environment)

:::tip
ใช้ key แบบ Ed25519 กับทุก client key แบบ HMAC ยังใช้ได้ทั้งกับ data และ
execution client แต่ Ed25519 ให้ประสิทธิภาพดีกว่าและจะกลายเป็น
ชนิด key เดียวที่รองรับในเวอร์ชันอนาคต ดู [ชนิดของ key](#key-types)
:::

:::warning
environment variable `BINANCE_ED25519_*` และ `BINANCE_*_ED25519_*` ถูกนำออกแล้วสำหรับ Spot client
ที่พบตัวแปรเหล่านี้จะ log error และถือว่าไม่มี credential สำหรับ Futures ตัวแปรเหล่านี้ deprecated แล้ว ยัง
ใช้ได้พร้อม warning และจะถูกนำออกในเวอร์ชันอนาคต ให้เปลี่ยนชื่อเป็น `BINANCE_API_KEY` /
`BINANCE_API_SECRET` (ตอนนี้ตรวจจับ key แบบ Ed25519 ได้อัตโนมัติ)
:::

เมื่อ trading node เริ่มทำงาน คุณจะได้รับการยืนยันว่า
credential ถูกต้องและมีสิทธิ์ในการเทรดหรือไม่

### Product type {#product-type}

config เลือกผลิตภัณฑ์ที่รองรับหนึ่งตัวด้วยฟิลด์ `product_type` และ
enum `BinanceProductType`:

- `SPOT`
- `USD_M` (collateral เป็น USDT, USDC หรือ BNFCR)
- `COIN_M` (collateral เป็นคริปโต)

:::note
ยังไม่ได้ implement margin trading enum variant อื่น ๆ จะถูกปฏิเสธโดย live client และ
instrument loader แบบ standalone ดู [การรองรับผลิตภัณฑ์](#product-support)
:::

### การ override base URL {#base-url-overrides}

override base URL default ของทั้ง HTTP REST และ WebSocket API ได้ ซึ่ง
มีประโยชน์สำหรับการตั้งค่า API cluster หรือเมื่อ Binance ให้
endpoint เฉพาะมา

### Binance US {#binance-us}

ตั้ง `us=True` บน config เพื่อ route ไปยัง Binance US Spot แบบเต็มรูปแบบ Binance US ไม่ใช่ alias ของ custom URL
switch นี้จะเลือก endpoint และ transport ต่อไปนี้:

- `api.binance.us`
- JSON stream สาธารณะ
- HTTP execution ที่ sign ด้วย HMAC
- private stream แบบ listen-key บนพอร์ต 443 พร้อม keepalive เป็นระยะ

ดูเอกสารอย่างเป็นทางการของ Binance US ได้แก่ [REST API](https://github.com/binance-us/binance-us-api-docs/blob/master/rest-api.md),
[market streams](https://github.com/binance-us/binance-us-api-docs/blob/master/web-socket-streams.md)
และ [user data stream](https://github.com/binance-us/binance-us-api-docs/blob/master/web-socket-api.md)
สำหรับ contract ของ venue เบื้องหลังการ route นี้

ชุดค่าที่รองรับถูกกำหนดไว้อย่างจงใจ:

- Data: `product_type=Spot`, `environment=Live`, `spot_market_data_mode=Json`
- Execution: `product_type=Spot`, `environment=Live` การส่ง order ใช้ HTTP และ private event ใช้
  listen-key stream
- config แบบ Futures, Testnet, Demo และ Spot SBE ที่มี `us=True` จะไม่ผ่านการตรวจสอบ

JSON สาธารณะของ Binance US ครอบคลุม live market data, depth snapshot, ประวัติ trade แบบล่าสุดและแบบ aggregate
และประวัติ kline ใช้อัตรา maker และ taker ระดับ account ส่วน Global Binance ยังคง
พฤติกรรม Spot JSON ที่ไม่ต้องใช้ credential เดิมเมื่อตั้ง `us=False` และ
`spot_market_data_mode=Json`

### Environment {#environments}

Binance มี trading environment สามแบบ แต่ละแบบมี API
credential และ endpoint แยกกัน ตัวเลือก config `environment` ใช้เลือกว่าจะ
ใช้แบบใด

| Environment | ค่าใน config                 | คำอธิบาย                                         |
| ----------- | ---------------------------- | --------------------------------------------------- |
| **Live**    | `BinanceEnvironment.LIVE`    | การเทรดจริงด้วยเงินจริง (default)       |
| **Demo**    | `BinanceEnvironment.DEMO`    | Demo Trading ด้วยเงินจำลองทั้ง Spot และ Futures |
| **Testnet** | `BinanceEnvironment.TESTNET` | test network แบบเดิมของ Spot และ Futures               |

#### Live (production) {#live-production}

environment default สำหรับ live trading ด้วยเงินจริง ใช้ credential ของบัญชี Binance
หลักของคุณ

| Variable             | คำอธิบาย      |
| -------------------- | ---------------- |
| `BINANCE_API_KEY`    | API key ของ live    |
| `BINANCE_API_SECRET` | API secret ของ live |

#### Demo trading {#demo-trading}

ฝึกเทรดด้วยเงินจำลองบนโครงสร้างพื้นฐาน production บัญชี demo
ใช้ login ของ Binance เดียวกับบัญชี live แต่เทรดด้วย
ยอดเงินเสมือน

**วิธีรับ credential ของ demo**:

1. login ที่ [binance.com/en/demo-trading](https://www.binance.com/en/demo-trading)
2. ไปที่ **API Management** แล้วสร้าง demo API key
3. demo key ใช้ได้กับ demo endpoint ของ Spot และ Futures

| Endpoint    | URL                        |
| ----------- | -------------------------- |
| Spot HTTP   | `demo-api.binance.com`     |
| Spot WS     | `demo-stream.binance.com`  |
| USD-M HTTP  | `demo-fapi.binance.com`    |
| USD-M WS    | `demo-fstream.binance.com` |
| COIN-M HTTP | `demo-dapi.binance.com`    |
| COIN-M WS   | `demo-dstream.binance.com` |

| Variable                  | คำอธิบาย      |
| ------------------------- | ---------------- |
| `BINANCE_DEMO_API_KEY`    | API key ของ demo    |
| `BINANCE_DEMO_API_SECRET` | API secret ของ demo |

#### Testnet {#testnet}

test network แบบเดิมที่มีบัญชีผู้ใช้ ยอดเงิน และ order book ของตัวเอง สำหรับการตั้งค่าการเทรดจำลองใหม่ควรใช้
`environment=BinanceEnvironment.DEMO` Spot testnet ยังอยู่ที่
`testnet.binance.vision` ส่วน endpoint ของ Futures testnet อาจ route ผ่านโครงสร้างพื้นฐานของ Demo Trading

**วิธีรับ credential ของ Spot testnet**:

1. ไปที่ [testnet.binance.vision](https://testnet.binance.vision/)
2. login ด้วย GitHub
3. สร้าง API key แบบ Ed25519 หรือ HMAC (adapter ไม่รองรับ key แบบ RSA)

**Futures testnet**: config เดิมที่ใช้ `BinanceEnvironment.TESTNET`
ยังใช้งานได้ แต่การทดสอบ Futures ใหม่ควรใช้ `BinanceEnvironment.DEMO`

| Variable                             | คำอธิบาย                 |
| ------------------------------------ | --------------------------- |
| `BINANCE_TESTNET_API_KEY`            | API key ของ Spot testnet       |
| `BINANCE_TESTNET_API_SECRET`         | API secret ของ Spot testnet    |
| `BINANCE_FUTURES_TESTNET_API_KEY`    | API key ของ Futures testnet    |
| `BINANCE_FUTURES_TESTNET_API_SECRET` | API secret ของ Futures testnet |

:::note
credential ของ testnet แยกจากบัญชี live โดยสิ้นเชิง market
data และสภาพคล่องแตกต่างจาก production
:::

### การโหลด instrument {#instrument-loading}

instrument provider ควบคุมการเลือกและการกรอง ฟิลด์ที่ใช้ได้แก่ `load_all`, `load_ids`, `filters`,
`log_warnings` และ `query_commission_rates`

`load_all=False` จะเลือกเฉพาะ `load_ids` จากนั้น filter ของ venue จะใช้แบบ intersection filter ที่รองรับคือ
`symbols`, `bases` และ `quotes` รวมถึง `contract_types` สำหรับ Futures ค่าเป็น string หรือรายการ string
ที่ไม่ว่าง และการ match ไม่สนใจตัวพิมพ์เล็กใหญ่

instrument ที่ parse แล้วไม่มีอัตรา fee ของ maker หรือ taker
`query_commission_rates` ไม่ได้คัดลอก commission ของ account ลงใน instrument

### Parser warning {#parser-warnings}

instrument บางตัวของ Binance ไม่สามารถ parse เป็น object ของ Nautilus ได้หากมี
ค่าฟิลด์เกินกว่าที่แพลตฟอร์มรองรับ instrument เหล่านี้จะถูกข้าม
พร้อม warning

symbol ที่ไม่ได้ซื้อขายจะถูกข้ามพร้อม debug log ระหว่างการโหลดจำนวนมาก แต่ยังคง
warn เมื่อถูกเลือกอย่างชัดเจนผ่าน `load_ids` หรือ filter `symbols`

หากต้องการปิด warning เหล่านี้ ให้ตั้ง `log_warnings=False` บน instrument provider config

### Futures hedge mode {#futures-hedge-mode}

Hedge mode ของ Binance Futures อนุญาตให้ถือทั้ง long และ short position บน
instrument เดียวกันพร้อมกันได้

เมื่อเปิด `use_position_ids` (default) order และ fill report ของ Futures จะมี
`venue_position_id` ที่ derive จาก instrument และ position side ของ Binance REST position
report ใน hedge mode ใช้ ID เดียวกัน เช่น `ETHUSDT-PERP.BINANCE-LONG` identity นี้ถูกรักษาไว้ตลอด
ประวัติ REST, update จาก user stream, การกู้คืน stream, fill ที่ exchange สร้างขึ้น และ fill แบบ
`TRADE_LITE` ที่ติดตามอยู่

position, order และ fill แบบ `BOTH` ใน One-way ยังคงไม่มี key และใช้ netting reconciliation ตั้ง
`use_position_ids` เป็น false เฉพาะสำหรับ virtual position ที่ใช้ `OmsType.HEDGING` ซึ่ง engine
เป็นผู้จัดการ identity ของ position เมื่อ `use_position_ids=True` adapter จะปฏิเสธ custom
position ID ที่ submit มาซึ่งต่างจาก hedge-leg ID มาตรฐานของ Binance ก่อนส่ง order

หากต้องการใช้ hedge mode ให้ตั้งค่าบน Binance, ตั้ง
`oms_type=OmsType.HEDGING` บน `BinanceExecutionClientConfig` และคง `use_position_ids=True` ไว้เพื่อติดตาม
position ทั้งสองฝั่งของ venue

การตั้งค่านี้จำเป็นเพื่อให้ startup reconciliation เก็บ leg แบบ `LONG` และ `SHORT`
แยกกัน

หาก hedge position ของ Binance ที่เปิดอยู่ใน cache มี ID ที่สร้างภายในเครื่องต่างออกไป adapter จะปฏิเสธ
แถว position นั้น โดยรายงานทั้ง ID ใน cache และ ID ที่คาดหวัง ให้ reconcile สถานะใน cache ก่อนลอง startup ใหม่
adapter ไม่ได้ทำ alias ให้ ID เดิมหรือสร้าง venue position ซ้ำ

### สถาปัตยกรรม COIN-M / USD-M {#coin-m--usd-m-architecture}

Binance COIN-M Futures (CM / DAPI) และ USD-M Futures (UM / FAPI) ใช้
สถาปัตยกรรมร่วมกัน section นี้อธิบายผลกระทบต่อ adapter

ดูรายละเอียดทั้งหมดที่ [Important CM-UM Integration Notice](https://developers.binance.com/docs/derivatives/coin-margined-futures/Important-CM-UM-Integration-Notice)

#### WebSocket stream {#websocket-streams}

payload ของ market-data stream มี `st` (symbol type: `1` = UM, `2` = CM) บน
`<symbol>@aggTrade`, `<symbol>@ticker`, `<symbol>@bookTicker`,
`<symbol>@depth<levels>`, `<symbol>@miniTicker` และทุก stream แบบ `!*@arr`
stream แบบ symbol เดียวฝั่ง UM ยังมี `ps` (pair symbol) บน
`<symbol>@bookTicker`, `<symbol>@depth<levels>`, `<symbol>@miniTicker` และ
`<symbol>@rpiDepth`

adapter decode JSON ด้วย `serde` ซึ่งไม่สนใจฟิลด์ที่ไม่รู้จักโดย default ฟิลด์เหล่านี้
จึงถูกทิ้งไปอย่างเงียบ ๆ

stream แบบ array ทั้งตลาด (`!ticker@arr`, `!miniTicker@arr`, `!bookTicker`,
`!forceOrder@arr`, `!contractInfo`) ส่งเนื้อหา UM + CM ที่รวมกันทั้งบน
`fstream` และ `dstream`

#### REST และ WebSocket API {#rest-and-websocket-api}

- response ยืนยันการวางและแก้ไข order ไม่มี
  `avgPrice` / `cumQuote` / `cumBase` adapter ดึง fill จาก user
  data stream ส่วน query endpoint (`GET /{f,d}api/v1/order`, `userTrades`) ยังคง
  return ฟิลด์เหล่านี้
- `PUT /dapi/v1/order` (COIN-M modify) ต้องมีทั้ง `price` และ `quantity`
  adapter จะส่งทั้งสองฟิลด์เสมอ โดย fallback ไปใช้ค่าของ order ใน cache
  สำหรับฟิลด์ที่ modify command ไม่ได้ระบุ
- conditional order ของ COIN-M (STOP, TAKE_PROFIT เป็นต้น) ใช้
  endpoint `/dapi/v1/algoOrder` adapter route conditional order ของ futures
  ทั้งหมดผ่าน algo order API
- `GET /dapi/v1/openOrders` ที่ใช้ symbol ไม่ถูกต้องจะ return error `-1121`

#### Rate-limit pool {#rate-limit-pools}

UM และ CM ใช้ rate-limit pool ของ Binance ร่วมกัน: 2400 weight/นาทีต่อ IP บวก 1200 order/นาที และ 300 order/10 วินาทีต่อ
account Rust futures HTTP client ใน process เดียวกันใช้สถานะ request-weight ร่วมกันระหว่าง UM และ CM สำหรับ
environment หรือขอบเขต custom endpoint เดียวกันและเส้นทาง egress ที่ตั้งค่าไว้เดียวกัน และใช้สถานะ order-count ร่วมกันระหว่าง UM
และ CM เมื่อ authenticate ด้วย API key เดียวกัน ไม่ว่าเส้นทาง egress จะเป็นอย่างไร

ขอบเขตของ live, testnet, demo และ custom endpoint ที่ไม่เกี่ยวข้องกันจะแยกจากกัน
เส้นทาง egress ที่ตั้งค่าต่างกันมีสถานะ request-weight แยกกัน ส่วน
API key ต่างกันมีสถานะ order-count แยกกัน process ที่แยกกันและ
API key หลายตัวสำหรับบัญชี Binance เดียวยังคงต้องประสานงานจากภายนอก

#### dualSidePosition {#dualsideposition}

UM และ CM ใช้การตั้งค่า `dualSidePosition` เดียวกัน การเปลี่ยนค่าฝั่งใด
ฝั่งหนึ่งจะมีผลกับทั้งสองฝั่ง ตรวจสอบให้แน่ใจว่าทั้ง UM และ CM ไม่มี open order หรือ position
ก่อนเปลี่ยนการตั้งค่านี้

## การมีส่วนร่วม {#contributing}

:::info
หากต้องการมีส่วนร่วมกับ Binance adapter ดู
[contributing guide](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
