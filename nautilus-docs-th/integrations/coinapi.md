---
title: "CoinAPI"
---

[CoinAPI](https://www.coinapi.io/products/market-data-api?ref=nautilus)
โดย [API Bricks](https://www.apibricks.io/?ref=nautilus) ให้บริการ market data ของ cryptocurrency ทั้งแบบ real-time
และย้อนหลังจากหลาย exchange REST และ WebSocket API ของ CoinAPI ให้ข้อมูล trade, quote, order book และ OHLCV bar
ในรูปแบบข้อมูลร่วมเดียวกัน

adapter ของ NautilusTrader รองรับ market data แบบ live, historical request และการโหลด flat file ที่ดาวน์โหลดมา
ไม่มี execution client, การจัดการ account หรือการ route order

## ภาพรวม [#overview]

adapter นี้ implement ใน Rust และไม่ต้องใช้ client library ของ CoinAPI จากภายนอก ตั้งค่าผ่าน live node ด้วย
component เหล่านี้:

- `CoinApiDataClientConfig`: config ของ live data client
- `CoinApiInstrumentSelection`: exchange, instrument และตัวกรอง metadata ที่จะโหลด
- `CoinApiDataClientFactory`: factory ที่ลงทะเบียนกับ node builder

crate ของ Rust ยังมี `CoinApiHttpClient` และ `CoinApiWebSocketClient` สำหรับเข้าถึง API โดยตรงด้วย

## ตัวอย่าง [#examples]

- [ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/coinapi/examples/)

ตัวอย่างจะโหลด BTC/USD ของ Coinbase, stream quote, trade และ bar หนึ่งนาที และ request bar ล่าสุด โดยใช้งบ REST
10 credit และ override price precision เป็น 3 ให้แก้การตั้งค่า instrument และ subscription ให้ตรงกับ feed ของคุณ
และอัปเดตค่า override ของ precision เมื่อเปลี่ยน instrument ข้อมูลย้อนหลังอาจช้ากว่า feed แบบ live ดังนั้น request
bar ล่าสุดอาจไม่ได้ record กลับมาเลย

## เอกสารของ CoinAPI [#coinapi-documentation]

- [Market Data REST API](https://www.coinapi.io/products/market-data-api/docs/rest-api?ref=nautilus)
- [WebSocket V1 API](https://www.coinapi.io/products/market-data-api/docs/websocket?ref=nautilus)
- [Flat Files](https://www.coinapi.io/products/flat-files?ref=nautilus)

## ความสามารถที่รองรับ [#supported-capabilities]

| ความสามารถ                                | Interface                     | สถานะ                                                 |
| ---------------------------------------- | ----------------------------- | ---------------------------------------------------- |
| instrument แบบ spot, perpetual และ futures | REST metadata                 | ใช้ instrument ID ของ venue แบบ native                 |
| Order book deltas                        | WebSocket                     | L2 (`book`) และ L3 (`book_l3`) พร้อม gap recovery     |
| book snapshot แบบความลึกคงที่              | WebSocket                     | L2 สูงสุด 50 ระดับ (`book5`, `book20`, `book50`)       |
| book snapshot ปัจจุบัน                     | REST                          | request snapshot แบบ L2                              |
| book snapshot ย้อนหลัง                     | REST                          | ไม่รองรับ                                              |
| Quote ticks                              | WebSocket และ REST ย้อนหลัง     | live stream และ historical request                   |
| Trade ticks                              | WebSocket และ REST ย้อนหลัง     | live stream และ historical request                   |
| OHLCV bars                               | WebSocket และ REST ย้อนหลัง     | live stream และ historical request                   |
| flat file ที่ดาวน์โหลดมา                     | S3 CSV exports                | loader ของ Rust                                       |
| Execution, order และ account report      | ไม่มี                           | อยู่นอกขอบเขต                                         |

ประเภทของ book:

- L2 market-by-price (MBP): ขนาดรวมตามระดับราคา
- L3 market-by-order (MBO): order แต่ละตัว ในกรณีที่แหล่งข้อมูลและ protocol เปิดเผยให้

subscription แบบความลึกคงที่จะใช้ channel ที่เล็กที่สุดที่ครอบคลุมความลึกที่ขอ L3 ต้องมี ID ของ order แต่ละตัว:
entry ของ `book_l3` แบบรวมที่ไม่มี ID จะแปลงไม่สำเร็จและเริ่มการ recovery book แบบ L3 จะไม่ sync เลยสำหรับแหล่ง
ข้อมูลที่รายงานเฉพาะระดับราคาแบบรวม

option, metric ของ funding และ open interest, FIX, การดาวน์โหลด flat file, ข้อมูล L4 ของ Hyperliquid และ tokenized
stock อยู่นอกขอบเขตของ adapter นี้ ส่วน CoinAPI EMS execution เป็นผลิตภัณฑ์แยกต่างหาก

## การระบุผู้ integrate [#integrator-attribution]

REST request และการเชื่อมต่อ WebSocket จะใส่ header สำหรับอ้างอิงแบบตายตัว `X-Referrer: nautilus` รวมถึงตอน retry
และ reconnect ด้วย header นี้ override ผ่าน config ไม่ได้ และไม่มี credential หรือข้อมูล account ใด ๆ

หากต้องการแนะนำ account ใหม่ให้ NautilusTrader ให้สมัครผ่าน
[API Bricks console](https://console.apibricks.io/?ref=nautilus) การอ้างอิงตอนสมัครและการอ้างอิงใน API request
เป็นกลไกที่แยกกัน ดู[เอกสารการอ้างอิงของ API Bricks](https://www.apibricks.io/platform/docs/general/referral?ref=nautilus)

## การยืนยันตัวตนและ environment [#authentication-and-environments]

ตั้ง `api_key` ใน config ของ data client หรือปล่อยว่างไว้เพื่ออ่านจาก `COINAPI_API_KEY` การสร้าง client จะปฏิเสธ
key ที่ไม่มี และ key ที่มีอักขระนอกช่วง ASCII ที่มองเห็นได้ REST request และการ upgrade WebSocket ยืนยันตัวตนด้วย
`X-CoinAPI-Key` ส่วน frame `hello` ไม่มี credential credential จะถูกปิดบังใน log และ error เสมอ

production ใช้ endpoint ของ REST และ WebSocket ที่เข้ารหัส ยังไม่ได้ตรวจสอบการเข้าถึง sandbox ให้ใช้ API key ของ
Market Data API สำหรับ production ไม่ต้องตั้งค่า execution account

| Interface    | Endpoint ของ production            | บทบาท                       |
| ------------ | --------------------------------- | --------------------------- |
| REST         | `https://rest.coinapi.io/v1/`     | endpoint หลักแบบ GeoDNS      |
| REST         | `https://api-emea.coinapi.io/v1/` | endpoint ตามภูมิภาค (ไม่บังคับ) |
| REST         | `https://api-apac.coinapi.io/v1/` | endpoint ตามภูมิภาค (ไม่บังคับ) |
| REST         | `https://api-ncsa.coinapi.io/v1/` | endpoint ตามภูมิภาค (ไม่บังคับ) |
| WebSocket V1 | `wss://ws.coinapi.io/v1/`         | endpoint หลักสำหรับ streaming |

REST ใช้ GeoDNS เป็นค่าเริ่มต้น ตั้ง `base_url_http` เพื่อเลือก endpoint ตามภูมิภาค adapter ไม่สลับภูมิภาคให้อัตโนมัติ
ดู[คู่มือ availability ของ REST](https://www.coinapi.io/products/market-data-api/docs/rest-api/rest-high-availability?ref=nautilus)
และ[WebSocket endpoints](https://www.coinapi.io/products/market-data-api/docs/websocket/endpoints?ref=nautilus)
ของ CoinAPI

## การใช้ CoinAPI ร่วมกับ adapter แบบ native [#using-coinapi-with-native-adapters]

CoinAPI ใช้ instrument ID ของ venue แบบ native เช่น `BTCUSDT.BINANCE` และใช้ data client ID เป็น `COINAPI`
ใช้ `client_id=COINAPI_CLIENT_ID` สำหรับ request และ subscription ของ CoinAPI เมื่อ node มี data client หลายตัว
request ที่ไม่ระบุ client ID อย่างชัดเจนจะใช้การ routing ตาม venue หรือค่าเริ่มต้น CoinAPI เป็น data client เริ่มต้นได้

:::warning
instrument แต่ละตัวต้องมีแหล่งนิยามเพียงแหล่งเดียวต่อ node ให้ตั้งค่า CoinAPI และ adapter แบบ native ด้วยชุด
instrument ที่ไม่ซ้อนทับกัน รวมถึงตอนโหลดเริ่มต้นและตอน refresh การใช้ instrument ร่วมกันไม่รองรับแม้นิยามจะตรงกันก็ตาม
client ID ที่ต่างกันไม่ได้แยกนิยามหรือ feed ของ instrument เดียวกันออกจากกัน และ adapter ไม่ตรวจจับหรือแก้ความขัดแย้ง
เหล่านี้ให้อัตโนมัติ
:::

ตรวจสอบค่าเริ่มต้นของการโหลด instrument ใน adapter แบบ native ก่อนใช้ผู้ให้บริการร่วมกัน เช่น Binance มีค่าเริ่มต้น
`load_all` เป็น true ให้ตั้งเป็น false และระบุ `load_ids` ที่ไม่ซ้อนทับกัน คง route ของ venue ใน adapter แบบ native
ไว้สำหรับ instrument อื่น ๆ

resolution ของราคาและขนาดใน CoinAPI อธิบายข้อมูลของ CoinAPI เอง และอาจต่างจาก increment การซื้อขายของ venue
tick ขนาด `0.1` และ `0.5` ใช้ทศนิยมหนึ่งตำแหน่งเหมือนกันแต่ยอมรับราคาต่างกัน นิยามแบบ native ยังมีข้อจำกัดการซื้อขาย
รวมถึงขีดจำกัดของปริมาณและ notional ด้วย อย่าแทนที่ข้อจำกัดเหล่านั้นด้วย metadata ของ CoinAPI

## Instrument และ symbology [#instruments-and-symbology]

### Instrument ที่รองรับ [#supported-instruments]

data client โหลด instrument แบบ spot, perpetual และ futures ที่มีวันหมดอายุจาก exchange ที่ระบุด้านล่าง instrument
แบบ spot จะเป็น `CurrencyPair`, perpetual ของ cryptocurrency จะเป็น `CryptoPerpetual` และ futures ของ cryptocurrency
จะเป็น `CryptoFuture` ส่วน perpetual ของ commodity และหุ้นบน Binance USD-M เช่น `XAUUSDT` จะเป็น `PerpetualContract`

นิยามที่ไม่ถูกต้อง ไม่รองรับ หรือกำกวมจะถูกข้าม หากนิยามหลายตัว map ไปยัง instrument ID แบบ native เดียวกัน
จะถูกข้ามทั้งหมด การเลือก instrument ที่โหลดไม่ได้จะทำให้การเชื่อมต่อล้มเหลว

### Metadata [#metadata]

ค่า metadata `price_precision` และ `size_precision` ของ CoinAPI เป็น increment ของราคาและขนาด ไม่ใช่จำนวนตำแหน่ง
ทศนิยม หรือ increment การซื้อขายที่ venue รับประกัน adapter คำนวณ precision จาก resolution เหล่านี้ และ
[override price precision](#price-precision-overrides) ได้ (ไม่บังคับ)

base currency และ quote currency ใช้รหัส asset ที่ CoinAPI normalize แล้ว multiplier มาจาก `future_contract_unit`
และวันหมดอายุของ futures มาจาก `future_delivery_time` activation ใช้วันที่เริ่มมีข้อมูล `data_start` ของ CoinAPI
ซึ่งไม่ใช่เวลาที่ venue เริ่ม list รหัสสกุลเงินและ instrument class อาจต่างจากนิยามของ adapter แบบ native

### การ map symbol [#symbol-mappings]

ใช้ `InstrumentId` แบบ native ตามรูปแบบด้านล่างเมื่อตั้งค่าการเลือก สัญญาแบบ inverse จะ settle ใน base currency
สัญญาแบบ linear จะ settle ใน quote currency adapter กำหนดคุณสมบัติเหล่านี้จากแบบแผนผลิตภัณฑ์ของ venue แทน flag
inverse ของ CoinAPI

| CoinAPI exchange | Symbol type | รูปแบบ symbol ของ exchange | `InstrumentId` แบบ native      | สัญญา                        |
| ---------------- | ----------- | -------------------- | ------------------------------ | ---------------------------- |
| `BINANCE`        | `SPOT`      | `BTCUSDT`            | `BTCUSDT.BINANCE`              | Spot                         |
| `BINANCEFTS`     | `PERPETUAL` | `BTCUSDT`            | `BTCUSDT-PERP.BINANCE`         | Linear                       |
| `BINANCEFTS`     | `FUTURES`   | `BTCUSDT_261225`     | `BTCUSDT_261225.BINANCE`       | Linear                       |
| `BYBITSPOT`      | `SPOT`      | `BTCUSDT`            | `BTCUSDT-SPOT.BYBIT`           | Spot                         |
| `BYBIT`          | `PERPETUAL` | `BTCUSDT`            | `BTCUSDT-LINEAR.BYBIT`         | Linear                       |
| `BYBIT`          | `FUTURES`   | `BTCUSDT-30OCT26`    | `BTCUSDT-30OCT26-LINEAR.BYBIT` | Linear                       |
| `BYBITINVERSE`   | `PERPETUAL` | `BTCUSD`             | `BTCUSD-INVERSE.BYBIT`         | Inverse                      |
| `BYBITINVERSE`   | `FUTURES`   | `BTCUSDZ26`          | `BTCUSDZ26-INVERSE.BYBIT`      | Inverse                      |
| `OKEX`           | `SPOT`      | `BTC-USDT`           | `BTC-USDT.OKX`                 | Spot                         |
| `OKEX`           | `PERPETUAL` | `BTC-USD-SWAP`       | `BTC-USD-SWAP.OKX`             | Inverse สำหรับ USD, นอกนั้น linear |
| `OKEX`           | `FUTURES`   | `BTC-USD-261225`     | `BTC-USD-261225.OKX`           | Inverse สำหรับ USD, นอกนั้น linear |
| `COINBASE`       | `SPOT`      | `BTC-USD`            | `BTC-USD.COINBASE`             | Spot                         |
| `KRAKEN`         | `SPOT`      | `XBT/USD`            | `BTC/USD.KRAKEN`               | Spot                         |
| `KRAKENFTS`      | `PERPETUAL` | `PI_XBTUSD`          | `PI_XBTUSD.KRAKEN`             | `PI_` inverse, `PF_` linear  |

รองรับเฉพาะ exchange และรูปแบบผลิตภัณฑ์ที่ระบุไว้เท่านั้น symbol แบบ native ต้องตรงกับ metadata ของ asset และวัน
หมดอายุของผู้ให้บริการ สำหรับ spot ของ Kraken `XBT` จะ map เป็น `BTC` และ `XDG` จะ map เป็น `DOGE` adapter เก็บ
`symbol_id` ของ CoinAPI ฉบับเต็ม รวมถึง suffix ที่ใช้แยกความกำกวม ไว้สำหรับ data request และ subscription

### สัญญาที่ไม่รองรับ [#unsupported-contracts]

exchange และ symbol type อื่น ๆ ไม่รองรับ รวมถึง Binance COIN-M (`BINANCEFTSC`): CoinAPI รายงานหน่วยสัญญาเป็น
หนึ่งเหรียญสำหรับ `BTCUSD_PERP` ขณะที่สัญญาแบบ native ของ Binance คือ 100 USD

futures แบบมีวันหมดอายุของ Kraken (`KRAKENFTS` `FUTURES`) ก็ไม่รองรับเช่นกัน CoinAPI รายงานเวลาเที่ยงคืน UTC ของ
วันส่งมอบ ขณะที่เวลาซื้อขายสุดท้ายของ Kraken ต่างกันไปตามผลิตภัณฑ์ เช่น 08:00 UTC สำหรับ `FF_` และสำหรับสัญญา
`FI_` คือ 15:00 หรือ 16:00 UTC

รูปแบบอื่นที่ถูกปฏิเสธ ได้แก่ สัญญา USDC ของ Bybit, futures แบบ USD-margined ของ OKX เช่น `BTC-USD_UM-261225`
และ ID การส่งมอบของ Binance ที่กำกวมซึ่ง quote เป็น `USD` เช่น `BTCUSD_261225.BINANCE` symbol ที่ดูคล้าย ID แบบ
native ที่รองรับ ไม่ได้แปลว่าผลิตภัณฑ์นั้นรองรับ

## Data subscription [#data-subscriptions]

การ subscribe ต้องมี client ที่เชื่อมต่อแล้วและ instrument ที่โหลดแล้ว และใช้ `symbol_id` ที่โหลดโดย `connect`
subscription ทั้งหมดใช้การเชื่อมต่อ WebSocket เดียวร่วมกันตามที่อธิบายใน[Raw WebSocket stream](#raw-websocket-streams)

| Subscription   | CoinAPI type                | Event             | `ts_event`        | `ts_init`                   |
| -------------- | --------------------------- | ----------------- | ----------------- | --------------------------- |
| L2 book deltas | `book`                      | `OrderBookDeltas` | `time_exchange`   | `time_coinapi`              |
| L3 book deltas | `book_l3`                   | `OrderBookDeltas` | `time_exchange`   | `time_coinapi`              |
| Book depth     | `book5`, `book20`, `book50` | `OrderBookDepth`  | `time_exchange`   | `time_coinapi`              |
| Quotes         | `quote`                     | `QuoteTick`       | `time_exchange`   | `time_coinapi`              |
| Trades         | `trade`                     | `TradeTick`       | `time_exchange`   | `time_coinapi`              |
| Bars           | `ohlcv`                     | `Bar`             | `time_period_end` | clock ของ adapter ตอน emit   |

### ฝั่ง aggressor ของ trade [#trade-aggressor-sides]

ฝั่ง aggressor ของ trade จะเป็น `BUY` หรือ `SELL` เฉพาะเมื่อ exchange รายงานมาเท่านั้น ค่า `BUY_ESTIMATED` และ
`SELL_ESTIMATED` ของ CoinAPI เป็นการอนุมานของ CoinAPI เอง จึง map เป็น `NoAggressor` เช่นเดียวกับ `UNKNOWN`
และค่าที่ไม่รู้จัก

### Bar [#bars]

subscription ของ bar รับ bar type แบบ external ที่ใช้ราคาล่าสุด ด้วยช่วงเวลา streaming ของ CoinAPI ต่อไปนี้:

- 1, 2, 3, 4, 5, 6, 10, 15, 20 หรือ 30 วินาทีหรือนาที
- 1, 2, 3, 4, 6, 8 หรือ 12 ชั่วโมง
- หนึ่งวัน

client จะปฏิเสธการ aggregate แบบ internal และรูปแบบอื่น ๆ

CoinAPI ส่งยอด OHLCV ระหว่างทางเมื่อเห็น timestamp ในช่วงเวลาถัดไป และส่งทุกห้าวินาทีระหว่างการอัปเดตของช่วงเวลา
ยอดเหล่านี้ไม่มี flag บอกว่าปิดแล้ว client จะเก็บการอัปเดตล่าสุดของแต่ละช่วงเวลาไว้ และ emit bar เมื่อมีการอัปเดตของ
ช่วงเวลาถัดไปเข้ามา client จะทิ้งช่วงเวลาที่เก็บไว้เมื่อ reconnect (เพราะการอัปเดตที่พลาดไปอาจทำให้ข้อมูลไม่ครบ)
และเมื่อเริ่ม subscription ของ bar

### การเปลี่ยน subscription [#subscription-changes]

การเปลี่ยน subscription จะมีผลตามลำดับการเรียก และใช้การเชื่อมต่อ WebSocket เดียวร่วมกัน CoinAPI ไม่ตอบรับการเปลี่ยน
ขอบเขต ข้อมูลแรกที่ตรงกันจะเป็นตัวยืนยันว่า subscription ทำงานแล้ว

การเปลี่ยนจะเว้นระยะห่างกันอย่างน้อยสองวินาทีเพื่อให้เป็นไปตามขีดจำกัด subscription ของ CoinAPI การเปลี่ยนที่รอคิวอยู่
และการ recovery ของ book อาจทำให้ล่าช้าออกไปอีก การ unsubscribe ทุก stream จะปิดการเชื่อมต่อ subscription ครั้งถัดไป
จะเปิดการเชื่อมต่อใหม่และยังคงเว้นระยะเช่นเดิม

ความพยายามเชื่อมต่อที่ล้มเหลวจะ retry หลังหนึ่งวินาที และเพิ่มเป็นสองเท่าจนสูงสุด 30 วินาที จนกว่าจะสำเร็จหรือ
disconnect การเปลี่ยน subscription ที่ตามมาจะรอหลัง batch ที่ค้างอยู่ การ reconnect ของ transport จะคืนค่า subscription
ให้อัตโนมัติ ส่วน `error` จาก CoinAPI จะยุติ stream โดยไม่ subscribe ใหม่ subscription ครั้งถัดไปจะเปิดการเชื่อมต่อใหม่

## Order book [#order-books]

### Channel และความลึก [#channels-and-depths]

subscription แบบ delta ของ `L2_MBP` จะ stream `book` ของ CoinAPI และ subscription ของ `L3_MBO` จะ stream `book_l3`
ส่วน `L1_MBP` จะถูกปฏิเสธ ทั้งสอง channel ส่ง book ทั้งหมด subscription แบบ delta จึงไม่สนใจ `depth`

subscription แบบ depth รับเฉพาะ `L2_MBP` โดยใช้ channel ที่เล็กที่สุดใน `book5`, `book20` และ `book50` ที่ครอบคลุม
ความลึกที่ขอ (ค่าเริ่มต้น 10 ระดับ) ความลึกมากกว่า 50 จะถูกปฏิเสธ message ของ depth แต่ละตัวมีระดับบนสุดครบถ้วน,
ถูก flag ด้วย `F_SNAPSHOT` และมีจำนวน order เป็นศูนย์ เพราะ CoinAPI ไม่ได้รายงานค่านี้

### ความขัดแย้งของ channel [#channel-conflicts]

CoinAPI อนุญาตให้มี book type เดียวต่อการเชื่อมต่อ ดังนั้นทุก subscription ของ book บน client เดียวต้องใช้ channel
เดียวกัน subscription ที่สองจะล้มเหลวหากต้องใช้ channel อื่น เช่น L3 คู่กับ L2 delta หรือ depth 5 คู่กับ depth 20
subscription ที่สองของ instrument เดียวกันที่ใช้ความลึกต่างกันก็ล้มเหลวเช่นกัน ให้ unsubscribe book ที่ทำงานอยู่ก่อน
เพื่อเปลี่ยน channel

### Event แบบ snapshot และ delta [#snapshot-and-delta-events]

stream แบบ delta เริ่มด้วย snapshot: `Clear` ตามด้วย `Add` หนึ่งตัวต่อระดับราคา ทั้งหมดถูก flag ด้วย `F_SNAPSHOT`
และมี `F_LAST` ที่ delta สุดท้าย การอัปเดตแต่ละครั้งจะเป็น event `OrderBookDeltas` หนึ่งตัวที่ลงท้ายด้วย `F_LAST`
สำหรับ book แบบ L3 นั้น CoinAPI รายงานขนาดของ `SUBTRACT` และ `MATCH` เป็นปริมาณที่ถูกลบออก client จึงติดตาม
ขนาดคงเหลือของแต่ละ order และลบ order ออกเมื่อหมด

### Sequence number [#sequence-numbers]

CoinAPI นับเลข book stream แต่ละตัวเริ่มจากหนึ่งในแต่ละการเชื่อมต่อ และเริ่มนับใหม่ทุกครั้งที่เริ่ม stream ใหม่
data client จะทำให้ sequence number ที่ emit ออกไปเพิ่มขึ้นเรื่อย ๆ ข้ามการเริ่มใหม่และการ reconnect เพื่อให้ `OrderBook`
แบบ managed ใช้ stream ได้โดยตรง

### การ recovery [#recovery]

การ recovery ใช้กับ subscription แบบ delta หากเกิด sequence gap, มีการอัปเดตก่อน snapshot แรก หรือระดับราคาไม่ถูกต้อง
output ของ book จะหยุดจนกว่า client จะได้รับ snapshot ทดแทนที่ถูกต้อง การอัปเดตที่ได้รับระหว่างรอ snapshot นั้นจะถูกทิ้ง

subscription แบบ depth ได้รับ snapshot ที่สมบูรณ์ client จะทิ้ง message ของ depth ที่ไม่ถูกต้องพร้อม warning และ
message ถัดไปที่ถูกต้องจะแทนที่ โดยไม่มีการพยายาม recovery หรือ deadline ของ snapshot

### Deadline ของ snapshot และขีดจำกัดการ retry [#snapshot-deadlines-and-retry-limits]

`book_snapshot_timeout_secs` กำหนดเวลารอ snapshot หลัง subscribe, reconnect หรือเริ่ม recovery ค่าเริ่มต้นคือ 10
วินาที `0` ปิด deadline นี้ การไม่ได้ snapshot จะเริ่มหรือ retry การ recovery การ unsubscribe จะยกเลิก recovery และ
`error` จาก CoinAPI จะยุติ book stream ทั้งหมด

การเริ่ม book ใหม่ห่างกันอย่างน้อยห้าวินาที และการเปลี่ยน subscription อื่นจะรอระหว่างที่มีการเริ่มใหม่ค้างอยู่ recovery
จะพยายามสูงสุดแปดครั้งภายใน 180 วินาที พร้อม backoff และ jitter จากนั้นจะ retry ต่อไปจนกว่าจะยอมรับ snapshot ได้
โดย interval ของ retry เพิ่มเป็นสองเท่าจากหนึ่งนาทีจนถึงสิบห้านาที การ reconnect จะรอ snapshot ของ book ชุดใหม่ก่อน
กลับมาส่ง output

### ผลกระทบต่อ stream อื่น [#effects-on-other-streams]

เมื่อมีการ subscribe instrument อื่นอยู่ด้วย การ recovery book จะหยุด stream ทุกตัวของ instrument ที่ได้รับผลกระทบ
ไว้ราวสองวินาที ช่วงเวลา OHLCV ที่เก็บไว้ซึ่งคร่อมช่วงหยุดนั้นจะถูกทิ้ง เพื่อไม่ให้ publish bar ที่ไม่สมบูรณ์

เมื่อ instrument ที่ได้รับผลกระทบเป็น instrument เดียวที่ subscribe อยู่ quote, trade และ bar ของมันจะทำงานต่อไประหว่าง
การ recovery book หาก book เป็น subscription เดียว การ recovery จะคงการเชื่อมต่อไว้โดยไม่ publish trade stream เพิ่มเติม

## Historical request [#historical-requests]

historical request ต้องมี client ที่เชื่อมต่อแล้วและ instrument ที่โหลดแล้ว และใช้ `symbol_id` ที่เก็บไว้ของ instrument นั้น
response จะคง correlation ID, client ID ที่ขอ, instrument หรือ bar type, `start`, `end` และ `params` ไว้ หากไม่ระบุ
client ID จะใช้ ID ของ adapter เป็นค่าเริ่มต้น หากไม่ระบุ `end` ก็จะไม่มีค่า `end` เช่นเดิม

| Request       | CoinAPI endpoint                  | Response       | `ts_event`        | `ts_init`         |
| ------------- | --------------------------------- | -------------- | ----------------- | ----------------- |
| Trades        | `/trades/{symbol_id}/history`     | `TradeTick`s   | `time_exchange`   | `time_exchange`   |
| Quotes        | `/quotes/{symbol_id}/history`     | `QuoteTick`s   | `time_exchange`   | `time_exchange`   |
| Bars          | `/ohlcv/{symbol_id}/history`      | `Bar`s         | `time_period_end` | `time_period_end` |
| Book snapshot | `/orderbooks/{symbol_id}/current` | L2 `OrderBook` | `time_exchange`   | ไม่เกี่ยวข้อง        |

quote, trade และ bar ย้อนหลังใช้เวลาของ event เป็นทั้ง `ts_init` และ `ts_event` การกรอง response จึงคงช่วงเวลาตาม
exchange ที่ขอไว้

### ช่วงเวลาและขีดจำกัด [#time-windows-and-limits]

request ของ trade, quote และ bar ต้องมี `start` และจะไล่หน้าไปตลอดช่วง [`start`, `end`) โดยใช้เวลาปัจจุบันเมื่อไม่ระบุ
`end` ส่วน `limit` จำกัดจำนวน record ที่คืน โดยนับจาก `start` การแบ่งหน้าเป็นไปตาม[Raw HTTP request](#raw-http-requests)
และ client จะ log warning เมื่องบ credit ทำให้ response ถูกตัด

ขอบเขตของ request จะถูกปัดขึ้นตาม resolution ของ timestamp ของ CoinAPI ซึ่งละเอียด 100 นาโนวินาที ทำให้การเลือก
record บน grid เวลานั้นคงเดิม

### ความล่าช้าของข้อมูลย้อนหลัง [#history-lag]

endpoint ข้อมูลย้อนหลังของ CoinAPI ล่าช้ากว่า feed แบบ live ช่วงเวลาล่าสุดอาจไม่ได้ record กลับมาเลย หรือได้ผลลัพธ์
ไม่ครบ แม้ live stream จะทำงานอยู่ หากเกิดกรณีนี้ ให้ขอช่วงเวลาที่เก่ากว่า เช่นวันก่อนหน้า adapter ใช้ endpoint
ย้อนหลังโดยไม่ fallback ไปยัง endpoint ข้อมูลล่าสุด

### Bar ที่ปิดแล้ว [#closed-bars]

request ของ bar รับ bar type แบบเดียวกับ subscription ของ bar response จะมีเฉพาะช่วงเวลาที่ปิดภายใน `end` และภายใน
เวลาที่ response มาถึง ช่วงเวลาที่ยังไม่สิ้นสุดจะไม่ถูกคืนเป็น bar เด็ดขาด `limit` นับช่วงเวลาของ CoinAPI ก่อนการตัดออกนี้

### Book snapshot ปัจจุบัน [#current-book-snapshots]

request ของ book snapshot จะส่ง `depth` เป็น `limit_levels` หรือไม่ระบุเพื่อใช้ความลึกเริ่มต้นของ CoinAPI ระดับราคาที่
ละเอียดกว่า precision ของ instrument จะทำให้ snapshot ทั้งหมดถูกปฏิเสธ แทนที่จะคืน book บางส่วน

### Request ที่ล้มเหลว [#failed-requests]

request ที่ล้มเหลวจะ log error และไม่ส่ง response

## Precision [#precision]

client แปลงราคาและขนาดตาม price precision และ size precision ของ instrument ซึ่งมาจาก resolution ของข้อมูล CoinAPI
ที่อาจตามหลัง stream ดังนั้นค่าที่มีตำแหน่งทศนิยมมากกว่าจะถูก drop พร้อม warning แทนการปัดเศษ ศูนย์ต่อท้ายไม่นับเป็น
ตำแหน่งทศนิยม response ย้อนหลังใช้กฎเดียวกันกับทุก record และ log warning หนึ่งครั้งพร้อมจำนวนที่ถูก drop

volume ของ OHLCV มีข้อยกเว้นแบบมีขอบเขตสำหรับเศษตัวเลขเล็ก ๆ client จะปัดแบบ half to even ตาม size precision ของ
instrument เฉพาะเมื่อการปรับนั้นอยู่ในขีดจำกัดทั้งสองข้อ:

- ไม่เกิน `1e-15` เท่าของ volume ที่รายงาน
- ไม่เกิน `1e-6` เท่าของหนึ่งหน่วยที่ precision นั้น

สำหรับ size precision 8 ขีดจำกัดสัมบูรณ์คือ `0.00000000000001` volume ติดลบและการไม่ตรงกันของ precision ที่มากกว่านี้
จะถูกปฏิเสธ ส่วนราคายังคงใช้กฎการแปลงแบบ exact

ขีดจำกัดสัมบูรณ์คงที่แม้ volume จะเพิ่มขึ้น ดังนั้น volume รวมที่ใหญ่ขึ้นอาจยังถูกปฏิเสธได้ แม้เศษจะอยู่ในขอบเขตสัมพัทธ์

ระดับราคาของ book ที่ละเอียดกว่า precision ที่ตั้งไว้จะทำให้ message ทั้งหมดถูกปฏิเสธ subscription แบบ delta จะเริ่ม
recovery ส่วน subscription แบบ depth จะ drop message นั้น หาก price resolution ของ CoinAPI หยาบเกินไปสำหรับ feed
ให้ตั้ง[override price precision](#price-precision-overrides) ส่วน size precision ไม่มีการ override

### การ override price precision [#price-precision-overrides]

ตั้ง `price_precisions` เพื่อแทนที่ price precision ของ instrument ที่เลือก client จะใช้ค่านี้ตอนสร้าง instrument ดังนั้น
นิยาม, stream และ response ย้อนหลังทั้งหมดจะใช้ค่านี้ override สำหรับ instrument ที่อยู่นอกการเลือก หรือสูงกว่า precision
สูงสุดของ Nautilus จะล้มเหลวตอนสร้าง size precision ไม่มีการ override

ใน Rust ให้ส่ง map จาก instrument ID ไปยัง precision ให้ `CoinApiDataClientConfig::builder().price_precisions(...)`

### Warning เรื่อง precision ของ OHLCV [#ohlcv-precision-warnings]

:::warning
ราคา OHLCV ของ CoinAPI อาจมีตำแหน่งทศนิยมมากกว่าที่ metadata ของ instrument อนุญาต แม้ราคาของ trade จะพอดีก็ตาม
เช่น ราคา OHLCV ของ BTC/USD ที่ `84761.955` ไม่พอดีกับ resolution `0.01` ให้ตั้ง override `price_precisions`
ที่เหมาะสมเพื่อไม่ให้ bar หายไป

volume ของ OHLCV ย้อนหลังอาจมีเศษตัวเลขเล็ก ๆ ด้วย เช่น `1.2582318399999999` สำหรับ `1.25823184` client ยอมรับค่า
เหล่านี้ภายใน[ค่าความคลาดเคลื่อนที่มีขอบเขต](#precision) การไม่ตรงกันที่มากกว่านี้ยังคงทำให้ bar ถูก drop
:::

## Lifecycle ของการเชื่อมต่อ [#connection-lifecycle]

`disconnect` จะยกเลิก subscription และ request ที่ค้างอยู่ก่อนปิดการเชื่อมต่อ subscription ที่รอคิวไม่สามารถเปิดการเชื่อมต่อ
ใหม่ได้ และ request ที่ถูกยกเลิกจะไม่ส่ง response `connect` ครั้งถัดไปจะโหลด instrument ใหม่และอนุญาตให้มี subscription
และ request ใหม่ ส่วน `stop`, `reset` และ `dispose` จะยกเลิกงานที่ค้างอยู่และทิ้งการเชื่อมต่อโดยไม่รอ

## ข้อจำกัดด้านการใช้งานและการเข้าถึง [#usage-and-access-constraints]

### ขีดจำกัดของ account [#account-limits]

การใช้ REST อยู่ภายใต้ขีดจำกัดการทำงานพร้อมกันและ data credit ส่วนการใช้ WebSocket คิดตามปริมาณการรับส่งข้อมูล
ตรวจสอบสิทธิ์ของ account, จำนวนการเชื่อมต่อที่อนุญาต และ
[ราคาปัจจุบัน](https://www.coinapi.io/products/market-data-api/pricing?ref=nautilus) ก่อนเลือก feed ปริมาณการรับส่ง
ที่แผนให้ ไม่ได้รับประกันการเข้าถึงทุก channel หรือทุกแหล่งข้อมูล

หน้าราคาระบุปริมาณที่ได้ต่อวันดังนี้ ณ เดือนตุลาคม 2026 ตรวจสอบหน้าที่ลิงก์ไว้สำหรับการเปลี่ยนแปลงและค่าใช้จ่ายส่วนเกิน

| Plan          | REST credits | Tier 1 transfer | Tier 2 transfer |
| ------------- | ------------ | --------------- | --------------- |
| Pay As You Go | Metered      | Metered         | Metered         |
| Startup       | 1,000        | 32 GiB          | Metered         |
| Streamer      | 10,000       | 128 GiB         | 1 GiB           |
| Pro           | 100,000      | 512 GiB         | 64 GiB          |
| Enterprise    | Custom       | Custom          | Custom          |

CoinAPI จัด trade, quote และ L2 book เป็น Tier 1 และ OHLCV เป็น Tier 2 ดู
[usage FAQ](https://www.coinapi.io/products/market-data-api/faq?ref=nautilus)
[คู่มือราคาของ L3](https://www.coinapi.io/blog/tier-1-vs-tier-2-market-data-level-3-order-book?ref=nautilus)
จัด L3 stream แบบต่อเนื่องเป็น Tier 2 ให้ยืนยันการคิดเงินและการเข้าถึงสำหรับ feed ที่เลือก

client แต่ละตัวใช้การเชื่อมต่อ WebSocket หนึ่งการเชื่อมต่อ node หรือ client อื่นที่ใช้ account เดียวกันอาจใช้ช่องการเชื่อมต่อ
ที่มีอยู่จนหมดและทำให้ได้ response HTTP 429 ให้ลดจำนวนการเชื่อมต่อพร้อมกันให้อยู่ในขีดจำกัดของ account การใช้ REST
quota แบบ hard limit จนหมดก็จะคืน HTTP 429 เช่นกัน

:::warning
CoinAPI รวม data type, symbol และช่วงเวลาของ bar ใน session เข้าด้วยกัน ดังนั้น trade ของสิบ symbol กับ book ของหนึ่ง
symbol อาจทำให้ต้องรับส่ง book ของทั้งสิบ symbol adapter จะกรองคู่ที่ไม่ได้ขอออก แต่ปริมาณการรับส่งยังคงถูกคิดเงิน
ควรคำนึงถึงเรื่องนี้เมื่อรวมหลาย feed ไว้บน client เดียว
:::

### งบ credit ภายใน [#local-credit-budget]

`credit_budget` ของ data client จำกัดจำนวน credit ที่ client instance หนึ่งใช้ได้ ค่าเริ่มต้นคือ 100 การพยายามดึง metadata
ของ symbol แต่ละครั้งใช้หนึ่ง credit ภายใน ไม่ว่า response จะใหญ่แค่ไหน ใช้ `filter_symbol_id` เพื่อลดปริมาณการรับส่งและ
งาน parse เผื่อ credit สำหรับ historical request ทั้งหมดด้วย: range request จะหยุดเมื่อถึงงบและคืน response ที่ถูกตัด
สำหรับข้อมูลย้อนหลังจำนวนมาก [flat file](#flat-files) ที่ดาวน์โหลดมาช่วยเลี่ยง REST request ซ้ำ ๆ ได้ ให้เปรียบเทียบ
ค่ารับส่งของไฟล์กับ REST credit

`max_concurrent_requests` จำกัดจำนวน REST request ที่ทำพร้อมกัน งบ credit ภายในใช้ตลอดอายุของ client และแยกจากการ
คิดเงินของ CoinAPI การพยายามแต่ละครั้งจะจอง `ceil(limit / 100)` credit สำหรับ historical request หรือหนึ่ง credit
สำหรับ request ที่ไม่มีขีดจำกัดจำนวน record

CoinAPI คิดเงิน historical request ต่อทุก 100 record ที่คืนมา ดังนั้นหน้าข้อมูลย้อนหลังที่สำเร็จจะคืน credit ที่จองไว้แต่
ไม่ได้ใช้ส่วนที่เกิน `ceil(records / 100)` ค่าขั้นต่ำภายในคือหนึ่ง credit การพยายามที่ล้มเหลว, ถูก retry และถูกยกเลิกจะไม่
ได้รับคืน client จะ back off เมื่อได้ HTTP 429 และล้มเหลวทันทีเมื่อได้ HTTP 403 ดู
[เอกสารอ้างอิงขีดจำกัดของ API](https://www.coinapi.io/products/market-data-api/docs/api-limits-and-billing-metrics?ref=nautilus)

### Backfill [#backfills]

กำหนดขีดจำกัดจำนวน record และช่วงเวลาอย่างชัดเจนสำหรับ historical request หากไม่มีขีดจำกัดจำนวน record request เดียว
อาจจองงบ credit ที่เหลือทั้งหมดไว้สำหรับหน้าแรก และไม่เหลือให้ request อื่น หน้าข้อมูลย้อนหลังแต่ละหน้ามีงบ retry ของ REST
เป็นของตัวเอง

สำหรับข้อมูลย้อนหลังจำนวนมาก ให้เปรียบเทียบต้นทุน REST credit กับ[flat file](#flat-files) ที่ดาวน์โหลดมา การเรียก HTTP
น้อยครั้งลงไม่ได้แปลว่า credit ที่ถูกคิดเงินจะน้อยลงเสมอไป ให้ดาวน์โหลดไฟล์แยกต่างหากก่อนโหลด adapter ไม่ list หรือ
ดาวน์โหลด object จาก S3

## Raw HTTP request [#raw-http-requests]

สำหรับการเข้าถึงโดยตรงจาก Rust `CoinApiHttpClient` รับ API key แบบ `SecretString` และ `CoinApiHttpConfig` base URL
รวม `/v1/` ไว้แล้ว การยืนยันตัวตนใช้ `X-CoinAPI-Key` redirect จะถูกปฏิเสธ client ไม่สนใจ environment variable ของ proxy
ในระบบ และเชื่อมต่อโดยตรง เว้นแต่จะตั้ง `proxy_url`

raw client เปิดเผย metadata ของ exchange และ symbol ที่ active, trade, quote และ OHLCV ย้อนหลัง และ book L2 และ L3
ปัจจุบันของ symbol เดียว ค่าทางการเงินคงรูปแบบทศนิยมแบบ exact ค่าที่เกิน precision ของ `Decimal` จะ parse ไม่สำเร็จ
แทนการปัดเศษ

### หน้าเดียวและแบบช่วง [#single-pages-and-ranges]

เมธอด `get_trades`, `get_quotes` และ `get_ohlcv` คืนหนึ่งหน้าสำหรับช่วงเวลาและขีดจำกัด record ที่ระบุชัดเจนในช่วง
`[1, 100000]` หน้าที่เต็มไม่ได้แปลว่าช่วงเวลานั้นครบแล้ว ส่วนเมธอด `get_trades_range`, `get_quotes_range` และ
`get_ohlcv_range` จะไล่หน้าตลอดทั้ง `HistoryRange` ซึ่งมี limit แบบไม่บังคับที่จำกัดจำนวน record รวม และเกิน 100,000 ได้

### ขอบเขตของเวลา [#time-boundaries]

ช่วงของ trade และ quote เลือก `[time_start, time_end)` ตามเวลาของ exchange ช่วงของ OHLCV เลือกช่วงเวลาตามเวลาเริ่มต้น
ของช่วงนั้น data client ยังตัดช่วงเวลาที่ยังไม่ปิดออกด้วย record ที่อยู่นอกช่วงเวลาที่ขอจะถูกกรองออก

### การแบ่งหน้า [#pagination]

#### ขนาดหน้าและการจบ [#page-sizes-and-completion]

แต่ละหน้าจะขอ limit ที่ใหญ่ที่สุดที่ยังพอดีกับ limit ของ request ที่เหลือ, ค่าสูงสุด 100,000 record และงบ credit ที่เหลือ
หน้าที่สั้น หรือหน้าเต็มที่ถึง limit ของ request แล้ว จะทำให้ช่วงนั้นจบ เมื่อใช้งบเริ่มต้น 100 credit หนึ่งหน้าจะมีได้ไม่เกิน
10,000 record ดังนั้นการต่อเกินหนึ่งหน้าต้องใช้งบที่ใหญ่ขึ้น

#### Record ที่มี timestamp เดียวกัน [#records-sharing-a-timestamp]

การแบ่งหน้าหลีกเลี่ยง record ซ้ำที่รอยต่อของหน้า โดยอ่านกลุ่ม timestamp ท้ายหน้าซ้ำเมื่อหน้าเต็มต้องต่อ การแบ่งหน้าจะ
ล้มเหลวหาก record ที่ timestamp เดียวเต็มทั้งหน้า ซึ่งทำให้เลื่อน cursor ไม่ได้ หรือหากหน้าไม่ได้เรียงจากน้อยไปมาก

#### Record ที่อยู่นอกช่วงเวลา [#records-outside-the-window]

record ที่อยู่นอกช่วงเวลาจะถูก drop CoinAPI จะคืนช่วงเวลา OHLCV ใกล้เคียงจากวันที่ UTC เดียวกันเมื่อช่วงเวลานั้นไม่มี trade
และหน้าที่มีเฉพาะช่วงเวลาแบบนี้จะทำให้ช่วงนั้นจบ

#### Credit หมดและ range ที่ทำพร้อมกัน [#credit-exhaustion-and-concurrent-ranges]

หากงบ credit ไม่พอสำหรับหน้าถัดไป ผลลัพธ์จะมี record ที่รวบรวมได้จนถึงตอนนั้น และตั้ง `is_truncated=true` งบอาจ
ไม่พอหลังจากการพยายามที่ล้มเหลวยังคงจอง credit ไว้ หากไม่มี credit พอสำหรับหน้าแรก request จะล้มเหลว range request
บน client เดียวจะทำงานทีละตัว ดังนั้นแต่ละตัวจะกำหนดขนาดหน้าหลังจาก range ก่อนหน้าชำระ credit เรียบร้อยแล้ว

## Raw WebSocket stream [#raw-websocket-streams]

สำหรับการเข้าถึงโดยตรงจาก Rust `CoinApiWebSocketClient` รับ API key และ `CoinApiWebSocketConfig` และคืนตัวรับ message
endpoint เริ่มต้นคือ `wss://ws.coinapi.io/v1/` `Subscription` ระบุ data type, `symbol_id` ของ CoinAPI ฉบับเต็มแบบตรงตัว
และช่วงเวลาของ OHLCV ในกรณีที่จำเป็น

client decode `trade`, `quote`, `book`, `book5`, `book20`, `book50`, `book_l3` และ `ohlcv` โดยคงค่าทศนิยมไว้, log และ drop
frame ที่ผิดรูปแบบ และไม่สนใจ feed ของ metadata การเปลี่ยน subscription จะ return เมื่อเข้าคิวแล้ว การได้รับข้อมูลที่ตรงกัน
เป็นการยืนยัน subscription [การเว้นระยะ subscription](#subscription-changes), [ขีดจำกัดของ book channel](#channel-conflicts)
และ[ขอบเขตรวมที่ถูกคิดเงิน](#account-limits) ใช้กับ client นี้ด้วย

### การ reconnect และ error [#reconnects-and-errors]

หากไม่มี message เป็นเวลาห้าวินาทีจะเกิดการ reconnect ความพยายามแรกเกิดขึ้นทันที ความพยายามถัดไปจะ back off จาก
หนึ่งวินาทีถึง 30 วินาที โดยเพิ่มเป็นสองเท่าพร้อม jitter ไม่เกิน 500 มิลลิวินาที บน account ที่มีช่องการเชื่อมต่อเดียว
การ reconnect อาจได้ HTTP 429 จนกว่า CoinAPI จะปล่อยการเชื่อมต่อเก่า

การ reconnect จะคืนขอบเขต subscription และ emit `CoinApiWsMessage::Reconnected` sequence ของ book แบบ raw จะเริ่มใหม่
ทุกการเชื่อมต่อ ดังนั้นให้ล้าง state ของ book หลัง reconnect data client จัดการ state ของ book และความต่อเนื่องของ sequence
ให้อัตโนมัติ

ประกาศ `reconnect` จาก server ไม่ได้ขัดจังหวะการ streaming client จะ reconnect เมื่อ server ปิดการเชื่อมต่อ ส่วน `error`
จาก CoinAPI จะ emit `CoinApiWsMessage::Error`, ล้าง subscription ทั้งหมด และยุติการเชื่อมต่อโดยไม่ reconnect subscription
ครั้งถัดไปจะเปิดการเชื่อมต่อใหม่ error ของการยืนยันตัวตนอาจมาหลังการ upgrade WebSocket สำเร็จแล้ว การปิดหรือ drop
client จะปิดการเชื่อมต่อของมัน

## Flat file [#flat-files]

[Flat Files](https://www.coinapi.io/products/flat-files?ref=nautilus) ของ CoinAPI เป็นไฟล์ export `.csv.gz` รายวันหรือ
รายชั่วโมงที่ให้บริการผ่าน API ที่เข้ากันได้กับ S3 adapter จะโหลดไฟล์ที่ดาวน์โหลดมาเป็นข้อมูลของ Nautilus แต่ไม่ list หรือ
ดาวน์โหลดไฟล์ให้ loader ทำตามแบบ CSV loader ของ Tardis ฟังก์ชัน `load_*` แต่ละตัวคืนข้อมูลทั้งไฟล์ และฟังก์ชัน `stream_*`
แต่ละตัวให้ข้อมูลเป็น chunk ทั้งสองแบบรับ price precision และ size precision แบบไม่บังคับ และ `limit` ของจำนวน record

| Schema                        | Load                            | Stream                       | Output           |
| ----------------------------- | ------------------------------- | ---------------------------- | ---------------- |
| `T-TRADES`                    | `csv::load_trades`              | `csv::stream_trades`         | `TradeTick`      |
| `T-QUOTES`                    | `csv::load_quotes`              | `csv::stream_quotes`         | `QuoteTick`      |
| `T-LIMITBOOK_FULL`            | `csv::load_deltas`              | `csv::stream_deltas`         | `OrderBookDelta` |
| `T-OHLCV_ACTIVE_CONSOLIDATED` | `CoinApiOhlcvLoader::load_bars` | ไม่มี                          | `Bar`            |

ฟังก์ชันเหล่านี้อยู่ใน `nautilus_coinapi::csv`

### ตัวอย่าง order book [#order-book-example]

การเรียก load จะดึงวันที่จาก `D-20250901` ส่วน stream อ่านไฟล์ที่เปลี่ยนชื่อแล้ว จึงส่ง `date` อย่างชัดเจน และตั้ง precision
ตาม increment ของ instrument แทนการอนุมาน precision เหล่านี้ต้องรองรับทุกค่า: ค่าที่ละเอียดกว่าจะทำให้ stream ล้มเหลว
แทนการปัดเศษ

```rust tab="Rust"
use jiff::civil::date;
use nautilus_coinapi::csv::{load_deltas, stream_deltas};
use nautilus_model::identifiers::InstrumentId;

let instrument_id = InstrumentId::from("BTC-USD.COINBASE");

let path = "T-LIMITBOOK_FULL/D-20250901/E-COINBASE/IDDI-27576+SC-COINBASE_SPOT_BTC_USD+S-BTC__002DUSD.csv.gz";
let deltas = load_deltas(path, instrument_id, None, None, None, None)?;

let chunks = stream_deltas(
    "coinbase_btc_usd_book.csv.gz",
    instrument_id,
    Some(date(2025, 9, 1)),
    100_000,    // chunk_size
    Some(2),    // price_precision
    Some(8),    // size_precision
    None,       // limit
)?;

for chunk in chunks {
    let deltas = chunk?;
}
```

### ตัวอย่าง bar [#bars-example]

`CoinApiOhlcvLoader` map ค่า `symbol_id` แบบตัวเลขไปยัง instrument โดย `27576` คือ `symbol_id_int` ใน REST ของ
`COINBASE_SPOT_BTC_USD` loader อ่านช่วงเวลาจาก `TP-1MIN` และข้าม symbol ที่ไม่ได้ map ไว้ นอกจากนี้ยังมี
`CoinApiOhlcvLoader::from_symbols` สำหรับสร้าง map จาก metadata

```rust tab="Rust"
use nautilus_coinapi::csv::CoinApiOhlcvLoader;
use nautilus_model::identifiers::InstrumentId;

let loader = CoinApiOhlcvLoader::new([(27_576, InstrumentId::from("BTC-USD.COINBASE"))]);
let bars = loader.load_bars(
    "T-OHLCV_ACTIVE_CONSOLIDATED/D-20250911/E-COINBASE/TP-1MIN.csv.gz",
    None,   // period_id, read from the file name
    None,   // limit
)?;
```

### โครงสร้างไฟล์ [#file-layout]

#### ไฟล์ราย symbol [#symbol-files]

loader รับไฟล์ที่คั่นด้วยเครื่องหมาย semicolon และมีแถว header ไฟล์ trade, quote และ book เก็บข้อมูลหนึ่ง symbol ไว้ใต้
`T-{SCHEMA}/D-{YYYYMMDD|YYYYMMDDHH}/E-{EXCHANGE}/IDDI-{ID}+SC-{SYMBOL_ID}+S-{EXCHANGE_SYMBOL}.csv.gz`
ให้ส่ง `instrument_id` อย่างชัดเจนสำหรับแต่ละไฟล์

CoinAPI ยังระบุ export แบบอื่นที่คั่นด้วย comma, ใช้ timestamp ของ book แบบวันเวลาเต็ม หรือใช้ประเภทการอัปเดต book
แบบตัวเลข loader เหล่านี้ต้องใช้ตัวคั่น semicolon, timestamp ของ book แบบเวลาในวัน และการอัปเดต book แบบมีชื่อ เช่น
`SNAPSHOT`, `ADD` และ `SET` ให้แปลงรูปแบบอื่นก่อนโหลด ดู
[คู่มือชุดข้อมูล full-book](https://www.coinapi.io/products/flat-files/docs/datasets/limitbook?ref=nautilus)

#### Bucket และระยะเวลาการเก็บ [#buckets-and-retention]

bucket `coinapi` เก็บข้อมูลย้อนหลัง: partition รายวันก่อนวันที่ 2026-06-09 และ partition รายชั่วโมงของ trade, quote และ
full-book ตั้งแต่วันนั้น ส่วน bucket `coinapi-daily-tail` เก็บเฉพาะไฟล์รายวันของ trade, quote และ full-book ของวันก่อนหน้า
โดยเก็บไว้ 24 ชั่วโมง bucket นี้ไม่ใช่ archive และไม่มี OHLCV ดู
[โครงสร้างไฟล์](https://www.coinapi.io/products/flat-files/docs/datasets?ref=nautilus)

#### ไฟล์ OHLCV [#ohlcv-files]

[คู่มือชุดข้อมูล OHLCV](https://www.coinapi.io/products/flat-files/docs/datasets/ohlcv?ref=nautilus)
ระบุไฟล์รายวันระดับ exchange ใน bucket `coinapi` ภายใต้
`T-OHLCV_ACTIVE_CONSOLIDATED/D-{YYYYMMDD}/E-{EXCHANGE}/TP-{PERIOD}.csv.gz` หนึ่งไฟล์ต่อช่วงเวลา `1SEC`, `1MIN`,
`1HRS` หรือ `1DAY` ค่า `symbol_id` แบบตัวเลขของแต่ละแถวคือ `symbol_id_int` ใน REST loader จะ map ค่านี้ไปยัง
instrument, ข้ามแถวที่ไม่ได้ map และอ่านช่วงเวลาจาก `TP-` เว้นแต่จะระบุ `period_id` หน้าโครงสร้างไฟล์ทั่วไปยังระบุ path
ของ OHLCV รายชั่วโมงด้วย ให้ยืนยัน object ที่มีอยู่จริงตอนดาวน์โหลด loader อ่าน timestamp จากแถวได้ทั้งสองรูปแบบ

### ข้อมูล order book [#order-book-data]

ไฟล์ book ที่มี order ID จะให้ delta แบบ L3 market-by-order ไฟล์ที่ไม่มีจะให้ delta แบบ L2 market-by-price เช่น ไฟล์ spot
ของ Coinbase มี order ID ขณะที่ไฟล์ spot ของ Binance อธิบายระดับราคา ให้ใช้ `BookType` ที่ตรงกันเมื่อนำ delta ไปใช้

snapshot เริ่มต้นจะ emit `Clear` ตามด้วย delta แบบ `Add` ทั้งหมดถูก flag ด้วย `F_SNAPSHOT` การอัปเดตที่ตามมาซึ่งอยู่ที่
timestamp ของ exchange เดียวกันจะรวมเป็นหนึ่ง event ที่ลงท้ายด้วย `F_LAST` chunk ของ streaming ปกติจะจบที่ขอบเขตของ
event และอาจเกิน `chunk_size` ได้ `limit` ของจำนวน record อาจตัด event สุดท้ายให้สั้นลง delta สุดท้ายยังคงมี `F_LAST`
ดังนั้นการโหลดแบบจำกัดจำนวนอาจได้ book ที่ไม่สมบูรณ์

การอัปเดต fill ของ L3 จะลดขนาดคงเหลือของ order และลบ order ออกเมื่อเหลือศูนย์ การ add ภายหลังอาจใช้ order ID ซ้ำ
หลังการลบได้ การอัปเดตของ order ที่ไม่รู้จัก ยกเว้นการลบ จะถูก log และข้าม

### Timestamp [#timestamps]

แถวของ trade, quote และ OHLCV มีวันเวลา UTC แบบไม่มี zone เช่น `2025-09-01T00:00:00.1780000` ส่วนแถวของ book มีเพียง
เวลาในวัน เช่น `00:00:00.0140000` loader ของ book จึงกำหนดวันที่ของแต่ละแถวจาก partition `D-` ของไฟล์ สำหรับไฟล์ที่อยู่นอก
partition `D-` จะใช้ `date` snapshot ที่มีเวลาช้ากว่าการอัปเดตแรกจะเป็นเวลาปิดของวันก่อนหน้าและถูกเลื่อนกลับหนึ่งวัน
`time_coinapi` ที่ต่ำกว่า `time_exchange` เกินครึ่งวันถือว่าอยู่หลังเที่ยงคืนและถูกเลื่อนไปข้างหน้าหนึ่งวัน

`ts_event` คือ `time_exchange` และ `ts_init` คือ `time_coinapi` ส่วน bar ใช้ `time_period_end` สำหรับทั้งสองค่า

### Precision [#precision-1]

ราคาและขนาดแปลงจากข้อความแบบ exact `load_*` จะอนุมาน precision ที่ไม่ได้ระบุจากทุกแถวที่อ่าน โดยหยุดที่ `limit`
หากตั้งไว้ `stream_*` จะสุ่มตัวอย่าง 10,000 แถวแรก และคืน error หากค่าที่ตามมาต้องปัดเศษ ให้ส่ง precision อย่างชัดเจน
เมื่อไม่ทราบ precision ของไฟล์ loader ของ OHLCV จะอนุมาน precision ราย instrument โดยมีข้อยกเว้นเรื่อง volume ด้านล่าง

CoinAPI ใช้[โหมด high-precision](../getting_started/installation.md#precision-mode) เป็นค่าเริ่มต้น: ทศนิยม 16 ตำแหน่ง
แทน 9 ตำแหน่ง เพื่อให้รองรับขนาดของ cryptocurrency และ volume ของ CoinAPI `sx_sum` ของ OHLCV อาจมีเศษตัวเลขที่สอดคล้อง
กับ noise ของ floating-point เช่น `689.5507347100003` loader จะปัด volume แบบ half to even ที่ precision สูงสุดของ build
โดยเปลี่ยนเฉพาะค่าที่เกิน precision นั้น เช่นตัวอย่างนี้ใน build แบบ standard-precision loader จะปฏิเสธ volume ต้นทางที่ติดลบ
และ volume บวกที่ปัดแล้วเป็นศูนย์ ส่วน volume ต้นทางที่เป็นศูนย์ยังใช้ได้

## การตั้งค่า [#configuration]

ลงทะเบียน `CoinApiDataClientFactory` และ `CoinApiDataClientConfig` กับ node builder client ID เริ่มต้นคือ `COINAPI`
ชื่อ client ที่กำหนดเองจะกลายเป็น client ID

| Field                        | Default   | วัตถุประสงค์                                                    |
| ---------------------------- | --------- | ----------------------------------------------------------- |
| `api_key`                    | `None`    | API key; `None` จะอ่านจาก `COINAPI_API_KEY`                  |
| `base_url_http`              | `None`    | override URL ของ REST รวม `/v1/`                            |
| `base_url_ws`                | `None`    | override URL ของ WebSocket                                  |
| `proxy_url`                  | `None`    | HTTP(S) proxy แบบระบุชัดเจนสำหรับ REST และ WebSocket           |
| `instruments`                | Empty     | การเลือก instrument ที่โหลดตอน connect                         |
| `credit_budget`              | `100`     | จำนวน REST credit ภายในตลอดอายุของ client นี้                  |
| `max_retries`                | `3`       | จำนวน retry สูงสุดของ REST request ที่ล้มเหลว                   |
| `http_timeout_secs`          | `30`      | timeout ของ REST ต่อการพยายามหนึ่งครั้ง รวมการรอคิวการทำงานพร้อมกัน |
| `max_concurrent_requests`    | `1`       | จำนวน REST request พร้อมกันสูงสุด                              |
| `retry_delay_initial_ms`     | 1,000     | delay เริ่มต้นของการ retry REST                                |
| `retry_delay_max_ms`         | 10,000    | backoff สูงสุดของ REST; delay ที่ผู้ให้บริการกำหนดอาจเกินค่านี้ได้     |
| `retry_max_elapsed_ms`       | 120,000   | งบเวลารวมต่อ REST GET รวมการรอคิวและการ retry                  |
| `transport_backend`          | `Sockudo` | backend ของ WebSocket ที่ใช้ร่วมกัน; มี `Tungstenite` ให้เลือกด้วย   |
| `book_snapshot_timeout_secs` | `10`      | จำนวนวินาทีที่รอ book snapshot; `0` คือปิด                      |
| `price_precisions`           | Empty     | override price precision ตาม instrument ID                   |

config จะปฏิเสธฟิลด์ที่ไม่รู้จัก URL override ที่ไม่ได้ตั้งจะใช้ endpoint ของ production Sockudo เป็น backend ของ WebSocket
เริ่มต้น ตั้ง `transport_backend` เป็น Tungstenite หากต้องการใช้แทน

`proxy_url` route ทั้งสอง transport ผ่าน HTTP(S) proxy ที่ระบุอย่างชัดเจน proxy ของระบบยังคงถูกปิดไว้ URL ต้องมี
`http://` หรือ `https://` URL แบบ SOCKS จะล้มเหลวตอนสร้าง client credential ของ proxy จะถูกปิดบังใน debug output ของ Rust

`credit_budget`, `max_concurrent_requests`, `retry_max_elapsed_ms` และ `http_timeout_secs` ต้องเป็นค่าบวก หน้าข้อมูล
ย้อนหลังแต่ละหน้ามีงบ retry ของ REST เป็นของตัวเอง delay เริ่มต้นของการ retry ต้องเป็นค่าบวก และค่าสูงสุดต้องไม่น้อยกว่า
delay เริ่มต้น การตั้งค่า backoff ที่ไม่ถูกต้องจะล้มเหลวตอนสร้าง client timeout ต่อการพยายามหนึ่งครั้งรวมการรอคิวการทำงาน
พร้อมกัน แต่ไม่รวม delay ของการ retry ส่วนงบเวลารวมนั้นรวมทั้งสองอย่าง

factory จะสร้าง client ระหว่าง `build` ซึ่ง credential, URL ของ endpoint หรือ proxy ที่ไม่ถูกต้อง, ค่าการทำงานพร้อมกัน, งบเวลารวม
หรือ timeout ต่อการพยายามที่เป็นศูนย์ และการตั้งค่า retry backoff ที่ไม่ถูกต้องจะทำให้เกิด error ตัวอย่างอ่าน key จาก
`COINAPI_API_KEY` และ override price precision ของ BTC/USD เพื่อรองรับราคา OHLCV ที่มีทศนิยมสามตำแหน่ง ดู
[Precision](#precision) สำหรับการ setup node ทั่วไป ดู[การรัน Live Trading (Rust)](../how_to/run_rust_live_trading.md)

### การเลือก instrument [#instrument-selections]

`CoinApiInstrumentSelection` แต่ละตัวระบุ exchange ของ CoinAPI, instrument ID แบบ native ที่จะโหลด และตัวกรอง metadata
แบบไม่บังคับ `filter_symbol_id` และ `filter_asset_id` การสร้างจะปฏิเสธ list ของ ID ที่ว่าง, ID ที่ map ไปยัง exchange อื่นของ
CoinAPI และ ID ที่ถูกเลือกมากกว่าหนึ่งครั้ง ใช้ `filter_symbol_id` เพื่อจำกัดการรับส่งและการ parse metadata การพยายาม
request แต่ละครั้งใช้หนึ่ง credit ภายใน ไม่ว่า response จะใหญ่แค่ไหน

การเชื่อมต่อจะโหลดแต่ละการเลือกหนึ่งครั้ง และล้มเหลวหาก instrument ที่เลือกไว้ตัวใดหายไป จะ publish เฉพาะ instrument
ที่เลือกไว้ ใช้การเลือกแบบระบุชัดเจนเพื่อให้ CoinAPI และ adapter แบบ native[ไม่ซ้อนทับกัน](#using-coinapi-with-native-adapters)

request ของ instrument จะดึงข้อมูลจาก CoinAPI เสมอ และ refresh นิยามที่ใช้แปลง stream รวมถึงค่า `symbol_id` ที่ใช้ใน
REST request ส่วน subscription จะคง ID ที่โหลดโดย `connect` ไว้ แม้หลังการ reconnect ของ transport ให้ disconnect
แล้ว reconnect data client เพื่อใช้ ID ที่ refresh แล้วสำหรับ subscription

- `request_instrument` รับเฉพาะ ID ที่ตั้งค่าไว้ และคืน error สำหรับ ID อื่น
- `request_instruments` คืน instrument ที่ตั้งค่าไว้ของ venue ที่ขอ หาก request ไม่ระบุ venue จะคืน instrument ที่ตั้งค่าไว้
  ทั้งหมด response ที่ไม่มี venue ที่ขอจะใช้ venue `COINAPI`
- request สำหรับ venue ที่ไม่มี instrument ตั้งค่าไว้จะคืน response ว่างโดยไม่เรียก REST

## การแก้ปัญหา [#troubleshooting]

| อาการ                                                 | สาเหตุที่เป็นไปได้                                                                                      | สิ่งที่ควรทำ                                                                                     |
| ---------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `CoinAPI API key not configured` ตอน build            | ไม่ได้ตั้งทั้ง `api_key` และ `COINAPI_API_KEY`                                                           | ตั้งค่าใดค่าหนึ่ง                                                                                 |
| `subscription change failed, retrying` พร้อม HTTP 429 | client หรือ node อื่นใช้การเชื่อมต่อเดียวของ account อยู่                                                   | หยุดการเชื่อมต่ออื่น; client จะ retry จนกว่า CoinAPI จะปล่อยช่องการเชื่อมต่อ                          |
| error `WsHelloPerIp` แล้วไม่มีข้อมูล                     | มี message `hello` จาก IP นี้มากเกินไป รวมถึงจาก client อื่น                                             | ลดการเปลี่ยน subscription บน IP นั้น; subscription ครั้งถัดไปจะ reconnect                         |
| `Conflicting CoinAPI book subscription`              | instrument นั้น stream book channel หรือความลึกอื่นอยู่แล้ว                                               | unsubscribe book ของมันก่อน หรือขอ channel และความลึกเดียวกัน                                   |
| `CoinAPI streams one book channel per connection`    | instrument อื่น stream book channel ที่ต่างออกไป                                                       | ใช้ channel ที่ทำงานอยู่ หรือ unsubscribe ทุก book ก่อน                                           |
| warning `Invalid book message` ซ้ำ ๆ                   | ระดับราคาของ book เกิน precision ของ instrument หรือ entry ของ L3 ไม่มี order ID                          | override price precision หากจำเป็น หรือใช้ L2 สำหรับแหล่งข้อมูลที่ไม่มี order ID                      |
| warning `Dropping CoinAPI` ของ quote, trade หรือ bar   | ราคาหรือขนาดเกิน precision ของ instrument; volume ของ OHLCV ต้องปัดเกินค่าความคลาดเคลื่อนที่ยอมรับได้          | override price precision หากจำเป็น; size precision และค่าความคลาดเคลื่อนของ volume override ไม่ได้ |
| historical request ของชั่วโมงล่าสุดคืนค่าว่าง             | ข้อมูลย้อนหลังอาจช้ากว่าข้อมูล live; ข้อมูลย้อนหลังล่าสุดอาจยังไม่พร้อม                                       | ขอช่วงเวลาที่เก่ากว่า                                                                            |
| warning ข้อมูลย้อนหลัง `credit budget truncated`        | งบ credit หมดระหว่าง range request                                                                   | เพิ่ม `credit_budget` หรือแคบช่วงเวลาลง                                                          |

## ข้อจำกัด [#limitations]

- ไม่รองรับ book snapshot ย้อนหลังและการ failover REST ข้ามภูมิภาคอัตโนมัติ
- ยังไม่ได้ตรวจสอบการเข้าถึง sandbox
- size precision มาจาก metadata ของ CoinAPI เท่านั้นและไม่มีการ override
- ไม่รองรับการอัปเดต instrument อัตโนมัติ, status event, metric และ option
- adapter โหลด flat file ที่ดาวน์โหลดมาได้ แต่ไม่ list หรือดาวน์โหลดจาก S3

## การมีส่วนร่วม [#contributing]

:::info
หากต้องการมีส่วนร่วมพัฒนา adapter ของ CoinAPI ดู
[คู่มือการมีส่วนร่วม](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
