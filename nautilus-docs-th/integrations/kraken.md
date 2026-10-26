# Kraken

adapter ของ Kraken รองรับ market data แบบ live และการส่ง order สำหรับ Kraken
Spot และ Kraken Derivatives (Futures)

## ภาพรวม {#overview}

adapter นี้ implement ด้วย Rust และไม่ต้องใช้ client library ภายนอกของ Kraken
config ของ data หรือ execution แต่ละตัวจะเลือก client แบบ Spot หรือ Futures ผ่าน `product_type`

component หลักได้แก่:

- `KrakenDataClientConfig` และ `KrakenExecutionClientConfig`: config ของ live
  client
- `KrakenDataClientFactory` และ `KrakenExecutionClientFactory`: factory ที่ trading node builder
  ใช้
- `KrakenSpotHttpClient` และ `KrakenFuturesHttpClient`: การเข้าถึง HTTP ระดับล่าง
  สำหรับการส่ง request โดยตรง

crate ของ Rust ยังมี `KrakenSpotWebSocketClient` และ `KrakenFuturesWebSocketClient` สำหรับ
การเข้าถึง WebSocket ระดับล่างด้วย

ตั้งค่า adapter ผ่าน live trading node หรือใช้ client ระดับล่าง
เพื่อเข้าถึง API โดยตรง

## ตัวอย่าง {#examples}

- [ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/kraken/examples/)

## เอกสารของ Kraken {#kraken-documentation}

- [เอกสาร API ของ Kraken](https://docs.kraken.com/)
- [Kraken Spot REST API](https://docs.kraken.com/exchange/guides/rest/introduction)
- [Kraken Derivatives API](https://docs.kraken.com/exchange/guides/futures/introduction)

## ผลิตภัณฑ์ {#products}

adapter รองรับหมวดผลิตภัณฑ์ต่อไปนี้:

| ประเภทผลิตภัณฑ์           | รองรับ | หมายเหตุ                                                 |
| --------------------- | ----- | ------------------------------------------------------ |
| คู่สกุลเงิน Spot          | ✓     | การเทรดแบบ cash และ margin สำหรับคู่ที่เข้าเงื่อนไข             |
| Spot tokenized asset  | ✓     | โหลดจาก asset class `tokenized_asset` ของ Kraken        |
| Futures               | ✓     | instrument ที่ Kraken Futures API คืนมา                   |

:::warning
Kraken Futures อาจคืนนิยามของ instrument ที่ต้องการทศนิยมมากกว่าเก้าตำแหน่งของโหมด standard-precision
ให้เปิด[โหมด high-precision](../getting_started/installation.md#precision-mode)ไว้สำหรับ Futures โหมด standard-precision
ยังรองรับ Spot ได้ แต่ client ของ Futures จะ start ไม่สำเร็จหรือคืน instrument ไม่ได้เมื่อมีนิยามใดที่
parse ไม่ได้ request ของ catalog ฝั่ง Futures จะไม่คืนผลลัพธ์เพียงบางส่วน และจะไม่ปัด ตัด หรือละนิยามที่
ไม่รองรับเลย
:::

:::note
**หนึ่งประเภทผลิตภัณฑ์ต่อหนึ่ง client**: data client หรือ execution client ของ Kraken แต่ละตัวถูกตั้งค่าให้ใช้
`product_type` เดียว (`SPOT` หรือ `FUTURES`) client ตัวเดียวไม่ครอบคลุมทั้งสองตลาด
:::

## ค่าธรรมเนียมของ instrument ฝั่ง Spot {#spot-instrument-fees}

การโหลด instrument ฝั่ง Spot จะไม่เติมอัตราค่าธรรมเนียม maker หรือ taker ของ venue และ
ไม่เรียก endpoint `TradeVolume` ของ Kraken

API key ของ Spot ที่ไม่มีสิทธิ์ `Funds permissions - Query` (**Query Funds**) จะล้มเหลวเมื่อ
execution client ขอสถานะของ account

## การ stream bar {#bar-streaming}

### Interval ที่รองรับ {#supported-intervals}

สำหรับ Spot ให้ subscribe bar แบบ `EXTERNAL` เพื่อรับข้อมูล OHLC ผ่าน WebSocket ของ Kraken
interval ที่มีให้ใช้มีดังนี้:

| Interval   | BarType specification |
| ---------- | --------------------- |
| 1 นาที      | `1-MINUTE-LAST`       |
| 5 นาที      | `5-MINUTE-LAST`       |
| 15 นาที     | `15-MINUTE-LAST`      |
| 30 นาที     | `30-MINUTE-LAST`      |
| 1 ชั่วโมง    | `1-HOUR-LAST`         |
| 4 ชั่วโมง    | `4-HOUR-LAST`         |
| 1 วัน       | `1-DAY-LAST`          |
| 1 สัปดาห์    | `1-WEEK-LAST`         |
| 15 วัน      | `15-DAY-LAST`         |

:::note
**ข้อจำกัดของ Futures**: Kraken Futures ไม่รองรับการ stream bar ผ่าน
WebSocket ให้ใช้ `request_bars()` สำหรับข้อมูล bar ย้อนหลังแทน
:::

### Latency ในการส่ง bar {#bar-emission-latency}

[OHLC channel ของ Spot WebSocket](https://docs.kraken.com/exchange/api-reference/spot-websocket-v2/ohlc) ของ Kraken
จะอัปเดต bar ปัจจุบันที่ยังไม่สมบูรณ์เมื่อมี trade event และไม่มี
ฟิลด์ที่ระบุว่า bar ปิดแล้ว

ระหว่างการ stream ตามปกติ adapter จะ buffer bar ปัจจุบันไว้และส่งออกไปหลังจากได้รับการอัปเดตที่มี
`interval_begin` ใหม่ ดังนั้นความล่าช้าจึงขึ้นอยู่กับ trade แรกใน interval ถัดไป สำหรับตลาดที่ไม่มี
trade ความล่าช้าจะไม่ถูกจำกัดไว้ที่หนึ่งช่วงของ bar เมื่อ message handler ของ WebSocket หยุดทำงาน adapter จะ flush
bar ที่ buffer ไว้ออกไป รวมถึง bar ปัจจุบันที่อาจยังไม่สมบูรณ์

adapter ใช้การ buffer แทนการส่งออกตาม timer เพราะ:

- การส่งออกตาม timer อาจพลาดการอัปเดตสุดท้ายก่อน bar ปิด
- ไม่มีการรับประกันว่าการอัปเดตของ Kraken จะมาถึงตรงขอบของ interval พอดี

:::tip
หากต้องการ latency ของ bar ที่ต่ำกว่า ให้ subscribe trade tick และใช้การรวมแบบ `INTERNAL`
ด้วย `BarAggregator` time bar ใช้ timer ในเครื่อง จึงไม่ต้องรอ trade ใน interval ถัดไปของ Kraken
และใช้พฤติกรรมการรวมแบบเดียวกันในทุก venue
:::

## Symbology {#symbology}

### การ normalize symbol ของ Spot {#spot-symbol-normalization}

adapter ใช้ convention ของ symbol Bitcoin ที่ต่างกันระหว่าง Spot และ Futures:

| ตลาด     | รูปแบบ symbol | ตัวอย่าง             | หมายเหตุ                                            |
| ------- | ------------ | ------------------ | ------------------------------------------------- |
| Spot    | `BTC`        | `BTC/USD.KRAKEN`   | adapter normalize XBT เป็น BTC ตอนโหลด              |
| Futures | `XBT`        | `PI_XBTUSD.KRAKEN` | symbol ของ instrument คง XBT แบบดั้งเดิมของ Kraken ไว้ |

:::note
REST API ของ Kraken อาจคืน `XBT` สำหรับ Bitcoin ในขณะที่ WebSocket v2 API ต้องการ `BTC` adapter
จะ normalize symbol ของ Spot เป็น `BTC` ตอนโหลด instrument ไม่ว่า `XBT` จะปรากฏเป็น base currency (เช่น
`XBT/USD` เป็น `BTC/USD`) หรือ quote currency (เช่น `ETH/XBT` เป็น `ETH/BTC`) ส่วน symbol ของ instrument
ฝั่ง Futures คงรูปแบบ `XBT` ดั้งเดิมของ Kraken ไว้ แต่รหัสสกุลเงินของ Futures ไม่ได้คงไว้ และใช้
[การ map สกุลเงิน](#currency-codes)แบบเดียวกับ Spot
:::

Kraken ยังใช้ `XDG` สำหรับ Dogecoin ใน response บางตัวของ Spot adapter
จะ normalize เป็น `DOGE` รวมถึงใน symbol ของ quote currency ด้วย

### รหัสสกุลเงิน {#currency-codes}

adapter จะ map รหัส asset แบบเก่าของ Kraken เป็นรหัสมาตรฐานอย่างสม่ำเสมอทั้งใน
instrument, ยอดเงินคงเหลือ, ค่าธรรมเนียม และการตั้งค่าสกุลเงิน:

- `XXBT` และ `XBT` เป็น `BTC`
- `XXDG` และ `XDG` เป็น `DOGE`
- `ZEUR` เป็น `EUR` และ `ZUSD` เป็น `USD`

การ map ใช้ตารางที่ระบุไว้อย่างชัดเจนแบบเดียวกับ CLI ของ Kraken เอง แทนการตัด prefix ทิ้ง `XTZ`, `XRP`,
`XLM`, `XAUT`, `ZRX` และ `ZEC` เป็นรหัสที่ถูกต้อง รหัสที่ไม่อยู่ในรายการจะผ่านไปโดยไม่เปลี่ยนแปลง

การตั้งค่ารับได้ทั้งสองการสะกด ตัวอย่างเช่น
`spot_positions_quote_currency="ZEUR"` และ `"EUR"` จะตรงกับ instrument ที่ quote เป็นยูโรเหมือนกัน

ค่าธรรมเนียมใช้สกุลเงินที่ venue รายงานเมื่อมี fill ของ Futures อาจระบุสกุลเงินของค่าธรรมเนียม
ซึ่งในสัญญาแบบ inverse จะเป็น base ไม่ใช่ quote ส่วน `TradesHistory` ของ Spot
รายงานเพียงจำนวนค่าธรรมเนียม ดังนั้นค่าคอมมิชชันเหล่านั้นจึงใช้ quote currency ของ instrument

#### การ migrate จากรหัสสกุลเงินแบบเดิม {#migration-from-earlier-currency-codes}

:::warning
ให้ migrate หรือสร้าง cache และ database ที่เขียนไว้ก่อนการ normalize สกุลเงินใหม่
instrument, fill, position และยอดเงินที่จัดเก็บไว้อาจใช้รหัสแบบเดิมด้านล่าง
:::

การ map แบบเดิมแตกต่างกันตามแหล่งที่มา:

- instrument คง `XXBT`, `XETH`, `XXDG`, `ZUSD` และ `ZEUR` ไว้ fill
  และ position ของมันก็เช่นกัน ปัจจุบันรหัสเหล่านี้จะเป็น `BTC`, `ETH`, `DOGE`, `USD` และ `EUR`
- ยอดเงิน Spot และ asset ของ margin summary ถูกตัด `X` หรือ `Z` ตัวหน้าออกหนึ่งตัว ทำให้เหลือ
  `XBT` และ `XDG` แทน `BTC` และ `DOGE` และเหลือ `TZ`, `RX` และ `AUT` แทน
  `XTZ`, `ZRX` และ `XAUT` ส่วน `KFEE` จะเป็น `FEE` ภายใต้การ map แบบชัดเจน
- ยอดเงิน Futures คงการสะกดของ wallet ไว้: wallet แบบ cash และ margin ใช้ `xbt` ในขณะที่ flex wallet
  ใช้ `XBT` ทั้งสองจะเป็น `BTC` และ `usd` จะเป็น `USD`

#### การรวมยอดของ Futures wallet {#futures-wallet-aggregation}

ยอดเงิน Futures ที่มีรหัสหลัง normalize เดียวกันจะรวมเป็นรายการเดียว ยอดรวมและยอดที่ถูกล็อกจะบวกกันข้าม
wallet ยอดที่ถูกล็อกของแต่ละ wallet จะถูกจำกัดให้อยู่ในช่วง `[0, total]` เมื่อยอดรวมไม่ติดลบ ยอดรวมที่ติดลบ
จะคงยอดที่ถูกล็อกตามที่รายงานไว้ ผลรวมจะไม่ถูก clamp ดังนั้น free balance อาจติดลบได้เมื่อ
ยอดที่จองไว้เกินยอดถือครองรวม ซึ่งเป็นการคงยอดขาดจริงไว้

ก่อนหน้านี้ รายการของแต่ละ wallet จะเขียนทับกันใน account ทำให้เหลือเพียงรายการสุดท้ายที่อ่าน

#### Precision ของ money {#money-precision}

รหัสที่ normalize แล้วอาจ resolve เป็นสกุลเงิน built-in ที่มี precision ต่างกัน
ก่อนหน้านี้รหัสที่ไม่รู้จักอย่าง `ZEUR` และ `ZUSD` ถูกลงทะเบียนเป็นสกุลเงิน crypto
ที่มีทศนิยมแปดตำแหน่ง ส่วน `EUR` และ `USD` แบบ built-in ใช้สองตำแหน่ง และ `JPY` ไม่มีทศนิยม
สิ่งนี้ส่งผลต่อ quote currency ของ instrument, ค่าคอมมิชชันของ fill จาก REST และ PnL ที่ derive มา

ยอดเงินของ account ยังคงทศนิยมแปดตำแหน่ง เพราะ parser ของมันสร้างสกุลเงินโดยตรงแทนการ
ใช้ registry รายการ `portfolioValue` ของ Futures flex และรายการ margin แบบ USD ทั้ง account ก็ใช้
สกุลเงินทศนิยมแปดตำแหน่งนี้เช่นกัน ก่อนหน้านี้ใช้ `USD` แบบสองตำแหน่ง ดังนั้น precision จึงเพิ่มขึ้นโดยไม่สูญเสียข้อมูล

### ตลาด Spot {#spot-markets}

NautilusTrader ใช้ symbol ที่ normalize แล้วและคั่นด้วยเครื่องหมาย slash สำหรับ instrument ของ Kraken Spot
adapter จะแปลงเป็นรูปแบบดั้งเดิมของ Kraken ภายใน

**รูปแบบ instrument ID**:

- `BTC/USD.KRAKEN`: Spot BTC/USD
- `ETH/USD.KRAKEN`: Spot ETH/USD
- `SOL/USD.KRAKEN`: Spot SOL/USD
- `BTC/USDT.KRAKEN`: Spot BTC/USDT
- `ETH/BTC.KRAKEN`: Spot ETH/BTC (normalize มาจาก ETH/XBT)

### ตลาด Futures {#futures-markets}

symbol ของ Kraken Futures ใช้ prefix ของสัญญา:

- `PI_`: สัญญา perpetual แบบ inverse (เช่น `PI_XBTUSD`)
- `PF_`: สัญญา perpetual แบบ linear multi-collateral (เช่น `PF_XBTUSD`)
- `PV_`: สัญญา perpetual แบบ vanilla (เช่น `PV_XRPXBT`)
- `FI_`: สัญญาแบบมีวันครบกำหนดแบบ inverse (เช่น `FI_XBTUSD_230929`)
- `FF_`: สัญญาแบบมีวันครบกำหนดแบบ linear

[ข้อมูลอ้างอิง ticker symbol](https://support.kraken.com/articles/360022835891-ticker-symbols-derivatives) ของ Kraken
แสดง prefix หลักของสัญญา ให้ใช้ catalog ของ instrument ที่คืนมาเพื่อดูว่ามีสัญญาใดให้ใช้บ้าง
symbol ตัวอย่างอาจอ้างถึงสัญญาที่หมดอายุแล้วหรือไม่ได้ list ไว้

**รูปแบบ instrument ID**:

- `PI_XBTUSD.KRAKEN`: BTC แบบ perpetual inverse
- `PI_ETHUSD.KRAKEN`: ETH แบบ perpetual inverse
- `PF_XBTUSD.KRAKEN`: BTC แบบ perpetual linear multi-collateral

## ความสามารถด้านข้อมูล {#data-capabilities}

### Subscription (real-time) {#subscriptions-real-time}

| Data type           | Spot | Futures | หมายเหตุ                                      |
| ------------------- | ---- | ------- | ------------------------------------------- |
| `QuoteTick`         | ✓    | ✓       | Spot จาก ticker; Futures จาก L2 book         |
| `TradeTick`         | ✓    | ✓       |                                             |
| `OrderBookDeltas`   | ✓    | ✓       | การอัปเดต L2/L3 ของ Spot และ L2 ของ Futures     |
| `OrderBookDepth`    | -    | -       | ใช้ `OrderBookDeltas` ที่ depth `10`           |
| `Bar`               | ✓    | -       | OHLC channel ของ Spot WS ดูส่วนเรื่อง bar      |
| `MarkPriceUpdate`   | -    | ✓       | จาก ticker feed ของ futures                  |
| `IndexPriceUpdate`  | -    | ✓       | จาก ticker feed ของ futures                  |
| `FundingRateUpdate` | -    | ✓       | เฉพาะ perpetual                             |
| `InstrumentStatus`  | -    | -       | live client ไม่ส่งการอัปเดตสถานะ               |

### Request (ย้อนหลัง) {#requests-historical}

| Data type              | Spot | Futures | หมายเหตุ                                    |
| ---------------------- | ---- | ------- | ----------------------------------------- |
| `TradeTick`            | ✓    | ✓       |                                           |
| `Bar`                  | ✓    | ✓       |                                           |
| `OrderBook` (snapshot) | ✓    | ✓       | ผ่าน HTTP depth endpoint                   |
| `FundingRateUpdate`    | -    | ✓       | กรอง start/end/limit ที่ฝั่ง client           |

### การตรวจสอบ checksum ของ L2 book {#l2-book-checksum-validation}

[L2 `book` channel ของ Spot](https://docs.kraken.com/exchange/api-reference/spot-websocket-v2/book) ของ Kraken
ส่ง CRC32 checksum ของสิบระดับบนสุดในแต่ละฝั่งมากับทั้ง snapshot และการอัปเดต
adapter จะตรวจสอบค่านี้เป็นค่าเริ่มต้นเทียบกับ shadow book ในเครื่อง โดยใช้ `pair_decimals`
สำหรับราคาและ `lot_decimals` สำหรับ quantity จาก `AssetPairs` บางคู่มี scale ของราคาบน wire
ละเอียดกว่า tick size หนึ่งหลัก instrument จะเก็บ scale ที่ละเอียดกว่านั้นไว้ใน metadata
เพื่อให้การ render checksum ไม่ตัดตัวเลขของราคา
ข้อความ `book` ของ Kraken Futures ไม่มี checksum

#### การกู้คืนและ identity ของ stream {#recovery-and-stream-identity}

เมื่อ checksum ไม่ตรงกัน adapter จะส่ง delta แบบ `Clear` ทิ้ง shadow book
แล้ว unsubscribe และ resubscribe symbol นั้นที่ depth ที่ตั้งค่าไว้พร้อม snapshot
และจะไม่สนใจการอัปเดตจนกว่า snapshot นั้นจะมาถึง การ unsubscribe `book` ทุกครั้งจะระบุ
depth ด้วย เพราะ Kraken ใช้ symbol และ depth เป็น key ของ subscription และใช้ค่าเริ่มต้น `10` เมื่อ
ไม่ระบุ depth ตอน unsubscribe

การกู้คืนทำงานตามลำดับร่วมกับการเปลี่ยนแปลง subscription ของผู้ใช้ ดังนั้นการกู้คืนที่ค้างเก่าจะไม่มีวัน
ยกเลิก subscription ตัวใหม่ที่เข้ามาแทน request subscribe จะระบุ identity ของแต่ละ stream
adapter รับ frame เฉพาะจาก stream ล่าสุดของ symbol นั้นหลังจาก Kraken ยืนยัน
request นั้นแล้ว และทิ้ง frame จาก subscription ที่ถูกแทนที่และการกู้คืนที่ถูกแทนที่ไปแล้ว
snapshot จะเปลี่ยน book ก็ต่อเมื่อ parse และนำไปใช้สำเร็จเท่านั้น request กู้คืน
ที่ทำก่อน snapshot ที่ได้รับการยอมรับหรือก่อนการ reconnect จะไม่ส่งอะไรหลังจาก event นั้น

adapter ยังส่ง delta แบบ `Clear` เมื่อทิ้ง shadow book นอกการประมวลผล frame ซึ่งเกิดขึ้นใน
event ต่อไปนี้:

- การ reconnect
- การยืนยัน stream ใหม่
- การปฏิเสธ request subscribe ล่าสุด
- snapshot watchdog ตรวจพบ request ล่าสุดที่ยังไม่ได้รับการยืนยัน

#### Snapshot ที่หายไปและ subscription ที่ถูกปฏิเสธ {#missing-snapshots-and-rejected-subscriptions}

snapshot watchdog เป็นกลไก retry เพียงอย่างเดียวสำหรับการกู้คืน `book` มันทำงานแม้
จะปิดการตรวจสอบ checksum ไว้ เพราะมันยังกู้คืน stream ที่ไม่เคย
ได้รับ snapshot เลยด้วย subscription ที่เข้ามาแทนแต่ละตัวจะเริ่มการรอใหม่ แทนที่จะ
สืบทอดจำนวนครั้ง retry ของตัวก่อนหน้า

หากไม่มี snapshot มาถึงภายใน 10 วินาที data client จะขอใหม่อีกครั้ง โดยเพิ่มระยะรอเป็นสองเท่าระหว่าง
แต่ละ request (`10`, `20`, `40`, `80` และ `160` วินาที) สูงสุดห้า request เมื่อถึงขีดจำกัด จะ log error และ
หยุด retry book จะยังคงถูก clear และการอัปเดตจะถูกทิ้งจนกว่า snapshot จะมาถึง การเปลี่ยน subscription
หรือการ reconnect จะเริ่มการกู้คืนใหม่อีกครั้ง

การที่ Kraken ปฏิเสธ request subscribe ล่าสุดจะ clear book ที่ถืออยู่ และ log
เหตุผลของ venue และเวลาของ request ถัดไปที่ระดับ error การปฏิเสธนับเป็น request ที่ล้มเหลว
หนึ่งครั้ง ดังนั้นความพยายามถัดไปจะตามมาหลัง 20 วินาที การปฏิเสธอีกสี่ครั้ง
จะใช้โควตาหมดในเวลาประมาณห้านาที การปฏิเสธชั่วคราวจึงสามารถ
กู้คืนได้ในความพยายามถัดไปหลัง 20 วินาที การปฏิเสธของ request ที่ถูกแทนที่แล้วจะถูกเพิกเฉย

เมื่อ reconnect adapter จะยกเลิกรายการ subscribe ที่บันทึกไว้และ replay การ subscribe แต่ละรายการ
ด้วย request ID เดิม response ของการ replay จะไม่ตรงกับ request ที่ active อยู่ หาก Kraken
ปฏิเสธการ replay watchdog จะตรวจพบ snapshot ที่หายไปภายในระยะรอพื้นฐาน
10 วินาที

#### Checksum ไม่ตรงกันซ้ำๆ {#repeated-checksum-mismatches}

การไม่ตรงกันสามครั้งบน instrument เดียวโดยไม่มีการอัปเดตที่ถูกต้องคั่นระหว่างกัน จะปิด
การตรวจสอบสำหรับ instrument นั้นและ log error adapter จะคง book ไว้ตามที่
ได้รับ เพื่อป้องกันการ resubscribe วนซ้ำเมื่อ Kraken คำนวณ hash ต่างออกไป
snapshot ที่ถูกต้องจะไม่ reset จำนวนครั้งที่ไม่ตรงกัน

หากต้องการปิดการตรวจสอบ checksum ของ L2 ให้ตั้ง `validate_l2_checksum=False` ใน `KrakenDataClientConfig`

## L3 order book (market-by-order) {#l3-order-book-market-by-order}

Kraken เปิดข้อมูล book ราย order ของ Spot ผ่าน WebSocket v2
[`level3` channel](https://docs.kraken.com/exchange/api-reference/spot-websocket-v2/level3) ที่
`wss://ws-l3.kraken.com/v2` ซึ่งให้ venue order ID, quantity ราย order
และ event แบบ incremental จริง (`add`, `modify`, `delete`) adapter จะ hash
venue order ID แต่ละตัวเป็นฟิลด์ `BookOrder.order_id` แบบ `u64` ที่ NautilusTrader ใช้

### สิ่งที่ต้องมีก่อน {#prerequisites}

subscription แบบ L3 ต้องใช้ credential ของ Spot API ที่มีสิทธิ์ `WebSocket interface - On`
ซึ่ง adapter ใช้เพื่อขอ
[authentication token](https://docs.kraken.com/api-reference/trading/get-websockets-token)
ให้ส่ง credential (`api_key` และ `api_secret`) ไปยัง `KrakenDataClientConfig`

จากนั้น subscribe order book delta ด้วย book type `L3_MBO`

depth ที่ใช้ได้คือ `10`, `100` และ `1000` ค่า `depth` เป็น `0` จะใช้ `1000`

### การตรวจสอบ CRC32 checksum {#crc32-checksum-validation}

โดยค่าเริ่มต้น adapter จะตรวจสอบ CRC32 checksum ของ snapshot และการอัปเดต L3 แต่ละรายการ
เมื่อ Kraken ส่งมาให้ เมื่อไม่ตรงกัน จะส่ง delta แบบ `Clear` ล้างสถานะ L3
ในเครื่อง refresh auth token และ resubscribe เพื่อให้ Kraken
ส่ง snapshot ใหม่มา หากต้องการปิดการตรวจสอบเพื่อทำ benchmark ให้ตั้ง `validate_l3_checksum=False`
ใน `KrakenDataClientConfig`

### คำแนะนำในการจัดเก็บ {#storage-recommendations}

`OrderBookDelta` มี `order_id: u64` ใน Arrow schema ดังนั้น `ParquetDataCatalog`
จึงเก็บข้อมูล L3 ด้วย schema เดียวกับ L2 L3 สร้าง event ต่อ instrument มากกว่า

- ใช้ `chunk_size` ของ query session ที่เล็กลง (เช่น `50_000`) เพื่อให้ได้ streaming batch ที่เล็กลง
  batch ที่จัดตาม timestamp อาจเกินเป้าหมายนั้นได้ การตั้งค่านี้ไม่ได้ควบคุมขนาดไฟล์หรือการอ่านแบบขนาน
- ตั้ง `compression="zstd"` ใน `DataCatalogConfig` เพื่อบีบอัดข้อมูลที่จัดเก็บ
- Parquet catalog แบ่ง partition ของข้อมูลตาม instrument เป็นค่าเริ่มต้น

## ความสามารถด้าน order {#order-capabilities}

### ประเภทของ order {#order-types}

| ประเภทของ order         | Spot | Futures | หมายเหตุ                                                                |
| ---------------------- | ---- | ------- | --------------------------------------------------------------------- |
| `MARKET`               | ✓    | ✓       | execute ทันทีที่ราคาตลาด                                                 |
| `LIMIT`                | ✓    | ✓       | execute ที่ราคาที่ระบุหรือดีกว่า                                            |
| `STOP_MARKET`          | ✓    | ✓       | market order แบบมีเงื่อนไข (stop-loss)                                   |
| `MARKET_IF_TOUCHED`    | ✓    | ✓       | market order แบบมีเงื่อนไข (take-profit)                                 |
| `STOP_LIMIT`           | ✓    | ✓       | limit order แบบมีเงื่อนไข (stop-loss-limit)                              |
| `LIMIT_IF_TOUCHED`     | ✓    | ✓       | Spot: `take-profit-limit`; Futures: `take_profit` พร้อม `limit_price` |
| `TRAILING_STOP_MARKET` | ✓    | -       | trailing stop พร้อม `trailing_offset`                                  |
| `TRAILING_STOP_LIMIT`  | ✓    | -       | trailing stop-limit พร้อม `limit_offset`                               |

### Time in force {#time-in-force}

| Time in force | Spot | Futures | หมายเหตุ                                                     |
| ------------- | ---- | ------- | ---------------------------------------------------------- |
| `GTC`         | ✓    | ✓       | Good Till Canceled                                         |
| `GTD`         | ✓    | -       | Good Till Date (เฉพาะ Spot ต้องระบุ `expire_time`)           |
| `IOC`         | ✓    | ✓       | Immediate or Cancel                                        |
| `FOK`         | ✓    | -       | เฉพาะ limit order ของ Spot                                  |

:::note
**Market order** execute ทันทีโดยธรรมชาติและไม่รองรับ time-in-force
`IOC` ใช้กับ order ประเภท limit เท่านั้น
:::

### คำสั่งการ execute {#execution-instructions}

| คำสั่ง            | Spot | Futures | หมายเหตุ                                                              |
| ---------------- | ---- | ------- | ------------------------------------------------------------------- |
| `post_only`      | ✓    | ✓       | ใช้ได้กับ limit order                                                 |
| `reduce_only`    | ✓    | ✓       | Spot ต้องมี margin account และ leverage ที่ resolve แล้ว                  |
| `quote_quantity` | ✓    | -       | เฉพาะ Spot ปริมาณเป็น quote currency (`viqc`); ส่งผ่าน REST              |
| `display_qty`    | ✓    | -       | เฉพาะ Spot iceberg order (`displayvol`)                               |

### ประเภทของ trigger {#trigger-types}

order แบบมีเงื่อนไขรองรับราคาอ้างอิงของ trigger ต่อไปนี้:

| ประเภทของ trigger | Spot | Futures | หมายเหตุ                         |
| --------------- | ---- | ------- | ------------------------------ |
| `LAST_PRICE`    | ✓    | ✓       | ค่าเริ่มต้น ราคาที่ซื้อขายล่าสุด       |
| `INDEX_PRICE`   | ✓    | ✓       | ราคา index ของตลาดในวงกว้าง       |
| `MARK_PRICE`    | -    | ✓       | เฉพาะ Futures                  |

:::note
adapter จะปฏิเสธประเภทของ trigger ที่ไม่รองรับ (เช่น `BID_ASK`) ตอนส่ง order
แทนที่จะแปลงเป็นค่าอื่นแบบเงียบๆ
:::

### การดำเนินการแบบ batch {#batch-operations}

| การดำเนินการ   | Spot | Futures | หมายเหตุ                                                     |
| ------------ | ---- | ------- | ---------------------------------------------------------- |
| Batch submit | ✓    | ✓       | Spot แบ่งชุดละ 15 order ส่วน Futures แบ่งชุดละ 10              |
| Batch modify | -    | ✓       | เฉพาะ HTTP method ของ Futures ฝั่ง execution ส่งทีละคำสั่ง       |
| Batch cancel | ✓    | ✓       | แบ่งเป็น batch ละ 50 อัตโนมัติ                                 |

:::note
**ยกเลิก order ทั้งหมด**:

- Spot จะเลือก order ที่เปิดอยู่และที่อยู่ระหว่างดำเนินการ (in-flight) ที่ตรงกับ instrument ที่ร้องขอ ไม่ว่าจะมีตัวกรองฝั่งหรือไม่
  และยกเลิกด้วย order ID ที่ระบุชัดเจน ดังนั้น request จะไม่ไปถึง instrument อื่นเลย order ที่ in-flight
  ถูกรวมไว้ด้วยเพราะ venue อาจยอมรับ order ที่ cache ยังบันทึกว่าเป็น submitted อยู่แล้ว
- Futures ใช้การยกเลิกแบบกลุ่มที่จำกัดตาม symbol ของ venue เมื่อไม่ได้ระบุตัวกรองฝั่ง
  และเลือก order ที่เปิดอยู่และ in-flight ใน cache ที่ตรงกันด้วย order ID ที่ระบุชัดเจน
  เมื่อมีการระบุตัวกรองฝั่ง
- ID ที่เลือกจะส่งผ่าน endpoint batch-cancel และแบ่งเป็น batch ละ 50 อัตโนมัติ Kraken แยก key ของ
  identifier สองประเภท ดังนั้น venue order ID จะส่งเป็น `orders` และ client order ID เป็น
  `cl_ord_ids` ขีดจำกัดของ batch นับรวมทั้งสองประเภท การยกเลิกแต่ละรายการจะคง strategy เจ้าของของ order ที่
  มันมุ่งเป้าไว้ ส่วน response แบบรวมหรือกำกวมจะปล่อยให้ reconciliation จัดการแทนการสร้างผลลัพธ์
  ราย order

:::

### การแก้ไข order {#order-modification}

quantity ของการ modify คือยอดรวมใหม่ของ order ซึ่งรวม fill ไว้ด้วย ตามที่การแก้ไขของ Kraken
Spot รับ `order_qty` ส่วน Kraken Futures อ่าน `size` ของการ edit แบบไม่ระบุโหมดว่าเป็นขนาดที่ยังเปิดอยู่
adapter จึงส่ง `qtyMode=ABSOLUTE` ไปกับการ edit ของ futures ทุกครั้งที่เปลี่ยนขนาด
order ทั้งแบบเดี่ยวและแบบ batch Kraken Futures ไม่ได้ระบุไว้ในเอกสารว่าจะตอบสนองอย่างไรต่อ
ยอดรวมที่เท่ากับหรือต่ำกว่า quantity ที่ fill ไปแล้ว

### การจัดการ position {#position-management}

| ฟีเจอร์            | Spot | Futures | หมายเหตุ                                                     |
| ---------------- | ---- | ------- | ---------------------------------------------------------- |
| Query position   | ✓    | ✓       | Spot margin ผ่าน `OpenPositions`; spot cash ต้องเปิดใช้เอง    |
| Position mode    | -    | -       | position เดียวต่อ instrument                                |
| การควบคุม leverage | ✓    | -       | ระดับ (tier) ผ่าน REST ของ Spot; ราย order `params={"leverage": N}` |
| Margin mode      | ✓    | ✓       | cross margin ทั้ง Spot/Futures; ไม่มี isolated margin ของ spot |

### การ query order {#order-querying}

| ฟีเจอร์                 | Spot | Futures | หมายเหตุ                                               |
| --------------------- | ---- | ------- | ---------------------------------------------------- |
| Query open order      | ✓    | ✓       | แสดง order ที่ active ทั้งหมด                          |
| Query ประวัติ order     | ✓    | ✓       | Spot แบ่งหน้า; Futures อ่านประวัติหน้าเดียว               |
| การอัปเดตสถานะ order    | ✓    | ✓       | การเปลี่ยนสถานะ order แบบ real-time ผ่าน WebSocket     |
| ประวัติ trade           | ✓    | ✓       | report ของการ execute และ fill                       |

### Contingent order {#contingent-orders}

| ฟีเจอร์              | Spot | Futures | หมายเหตุ                                       |
| ------------------ | ---- | ------- | -------------------------------------------- |
| Linked order list  | -    | -       | list ที่ส่งไปประกอบด้วย order ที่เป็นอิสระต่อกัน       |
| OCO order          | -    | -       | *ไม่รองรับ*                                   |
| Bracket order      | -    | -       | *ไม่รองรับ*                                   |
| Conditional order  | ✓    | ✓       | order แบบ stop และ take-profit               |

### Maker Protection (Futures) {#maker-protection-futures}

Kraken Futures ใช้ [Maker Protection](https://docs.kraken.com/exchange/guides/futures/maker-protection) กับ
ตลาดที่เลือกไว้: การวางและการแก้ไข order ที่อาจกิน liquidity จะถูกพักไว้ตามช่วงเวลาที่ตั้งค่าของตลาดนั้น
ก่อนไปถึง matching engine การจัดประเภทอิงตามประเภทของ order ดังนั้นการวาง limit order ที่ไม่ใช่ post-only
จะถูกพักไว้แม้ว่ามันจะไปพักอยู่ใน book ก็ตาม การวาง trigger แบบ stop และ take-profit จะไม่ถูกพัก แต่ order ที่
มัน trigger ออกมาจะถูกพักเว้นแต่จะเป็น post-only การวางแบบ post-only และการยกเลิกทั้งหมดจะไม่ถูกพักเลย และไม่มี API ใด
เปิดเผยสถานะของ order ที่ถูกพัก

venue ใช้การพักต่อตลาดกับทุก client adapter จะ decode ช่วงเวลาต่อตลาด
(`makerProtectionMillis`) บน raw instrument model ของ venue และไม่เปิดให้ตั้งค่าใดๆ

การจัดการสถานะของ order คำนึงถึงความหมายของ order ที่ถูกพัก:

- การยกเลิกที่ได้รับการตอบรับสำหรับการวาง limit order ที่ถูกพักนั้นยังไม่ใช่สถานะสุดท้าย order จะถูก
  ปล่อยออกไปเป็น IOC และยังอาจ fill ได้ ส่วน market order และ IOC order ที่ถูกพักไม่สามารถเรียกคืนได้:
  การยกเลิกจะคืน `ORDER_NOT_FOUND` และ order ยังคงไปถึงการ matching เมื่อถูกปล่อย
  การอัปเดต order จาก venue เป็นตัวขับเคลื่อน fill และสถานะสุดท้าย ไม่ใช่การตอบรับการยกเลิกเอง
- order ที่ไม่สามารถเทรดได้หลังการปล่อยดังกล่าวจะถูกรายงานด้วยสถานะของ venue
  `iocWouldNotExecute` บน REST (`IOC_WOULD_ENTER_BOOK` บน market data)
  ซึ่ง adapter ถือว่าเป็นการปฏิเสธขั้นสุดท้าย บน order-update feed
  ผลลัพธ์เดียวกันจะมาถึงเป็นการยกเลิกขั้นสุดท้ายโดยที่ adapter คงเหตุผลของ venue
  ไว้
- order ที่ถูกปล่อยจะยกเลิก order ที่พักอยู่ใน book ของ account เดียวกันที่มันจะ
  match ด้วย โดย override self-trade strategy ที่ตั้งค่าไว้ order ที่พักอยู่จะถูก
  รายงานว่าถูกยกเลิกด้วยเหตุผล `CANCELLED_BY_SELF_TRADE`

#### Order-update feed {#order-update-feed}

การลบที่มี `is_cancel=true` และเหตุผล `partial_fill` จะทิ้งส่วนที่เหลือ
และเป็นสถานะสุดท้าย (การพักที่ถูกแปลง หรือ order แบบ IOC ใดๆ) delta จะมี
ยอด fill สะสมของ venue

- สำหรับ order ที่ติดตามอยู่ adapter จะปิดจาก feed เมื่อ fill stream
  นับ quantity นั้นครบแล้ว fill ที่ยังอยู่ระหว่างทางจะไม่มีวันถูกทิ้งค้าง และ
  order ที่ติดตามอยู่จะไม่ถูกปล่อยให้เปิดค้างหลังจากนับ fill ครบแล้ว
- สำหรับการลบที่ไม่สามารถจับคู่ได้ adapter จะข้ามไปและปล่อยให้
  reconciliation ทำให้สถานะสอดคล้องกัน

| การลบที่จับคู่ไม่ได้               | เหตุผล                             |
| ----------------------------- | -------------------------------- |
| ข้อความที่เป็นการยกเลิกเท่านั้น        | ไม่มียอด fill สะสม                  |
| ไม่มี client order ID ที่ resolve ได้ | ไม่สามารถจับคู่กับ order ที่ติดตามอยู่ได้ |

#### Reconciliation {#reconciliation}

order ที่ถูกพักจะไม่ไปถึง book จึงไม่ปรากฏใน `/openorders` ทั้ง mass
status, การรัน report แบบเฉพาะ open และการ query order เดี่ยวแบบเจาะจงจะตรวจสอบ
[`POST /orders/status`](https://docs.kraken.com/api-reference/order-management/get-specific-orders-status)
ก่อนถือว่า order หายไป ช่วงเวลานั้น (window) รายงาน
order ที่เปิดอยู่ หรือที่ fill หรือถูกยกเลิกภายใน 5 วินาทีที่ผ่านมา

venue จะส่ง client order ID ที่ order ถูกส่งมาด้วยกลับมา (echo) และจะมีการสอบถาม window
ด้วยทั้งสองรูปแบบที่ venue อาจเก็บไว้ (ดู
[Client order ID](#client-order-ids)) mass status และการรัน report แบบเฉพาะ open จะ map
report แต่ละรายการใน window กับ order ใน cache:

- venue order ID ใน cache ระบุ order ได้โดยตรง
- มิฉะนั้น ต้องมี order เพียงหนึ่งรายการที่ query ด้วย client order ID ที่ถือ ID ที่ echo กลับมา
  และต้องมี report เพียงหนึ่งรายการที่มี venue order ID ที่ไม่รู้จักซึ่งมี ID นั้น
  instrument และฝั่งต้องตรงกัน และเวลาวาง order ของ venue ต้องไม่เกิน
  ห้าวินาทีก่อนเวลาสร้าง order เพื่อเผื่อ clock skew
  วิธีนี้กัน order ที่เก่ากว่าซึ่ง ID ถูกย่อให้เหมือนกันหรือนำ ID มาใช้ซ้ำออกไป
  report สองรายการที่จะระบุ order เดียวกัน โดยแต่ละรายการ echo คนละรูปแบบ จะไม่ระบุ order ใดเลย

adapter จะกัน report อื่นใน window ไว้พร้อม warning ที่ระบุเหตุผล
เพราะไม่สามารถผูก report นั้นเข้ากับ order ใน cache เพียงหนึ่งรายการได้

request ด้วย client order ID จะคืนทุก order ที่ถือ ID นั้น report ใน window
ของ order ที่การอ่าน open order หรือประวัติคืนมาแล้ว จะระบุ
report นั้นด้วย order ใน cache ที่มันอธิบาย แล้วถูกละไว้ ดังนั้น order จะ
ไปถึง reconciliation เพียงครั้งเดียวด้วยฟิลด์จากการอ่านของ venue เช่น time in force,
post-only และฟิลด์ของ trigger ซึ่ง window ไม่มี เมื่อ report นั้นเปิดอยู่
และ report ใน window เป็นการอัปเดตที่ใหม่กว่า report จะรับสถานะ,
quantity, ราคา, เหตุผลการยกเลิก และเวลาอัปเดตจาก window และกฎของ window ด้านล่าง
จะใช้กับมัน
report ที่ปิดแล้วจะคงสถานะของมันไว้ การรันที่จำกัดเฉพาะ instrument เดียวจะเก็บเฉพาะ report ใน window
ของ instrument นั้น

request ด้วย venue order ID และ client order ID ทำงานต่อกันทีละตัว ดังนั้น
order ที่สอบถามด้วย venue order ID อาจกลับมาจาก request ด้วย client order ID
ด้วย ในสถานะที่ต่างกัน adapter จะเลือกสถานะล่าสุดโดยไม่สนว่า
request ใดทำงานทีหลัง: สถานะปิดมาก่อนสถานะเปิด จากนั้นคือการอัปเดตของ venue
ที่ใหม่กว่า แล้วจึงเป็น quantity ที่ fill มากกว่า กฎการระบุชื่อข้างต้นและ
กฎของราคา fill ด้านล่างจะใช้กับ report ที่เก็บไว้

ในการรันเหล่านั้น window ไม่มีราคา fill ดังนั้น order ที่ execute ครบแล้วจะถูกละไว้พร้อม warning
การ execute ของมันจะไม่ถูกอนุมานจากราคาของ order การ query order เดี่ยวแบบเจาะจงจะหาราคาจาก fill ล่าสุด
แทน หรือล้มเหลวเพื่อให้ reconciliation เลื่อนออกไป mass status จะระบุว่าชุดข้อมูลไม่สมบูรณ์เมื่อละ
report ใน window ไว้ order ที่ถูกยกเลิกแต่มีการ execute จะถูกรายงาน เพราะ reconciliation รอ fill ของมันแทน
การอนุมาน

order ที่ถูกพักแล้ว fill หลังการตอบรับการยกเลิก จะ reconcile ไปยังสถานะสุดท้ายที่แท้จริง
พร้อมยอด fill สะสมของ venue ไม่ใช่การยกเลิกก่อนเวลาอันควร

## การ routing order (Spot) {#order-routing-spot}

execution client ของ Spot จะ route การส่ง, การแก้ไข, การยกเลิก
และการยกเลิกแบบ batch ของ order ผ่าน trade channel ของ WebSocket v2 ที่ยืนยันตัวตนแล้วของ Kraken
เป็นค่าเริ่มต้น และจะถอยกลับไปใช้ REST เมื่อ WebSocket ไม่ active ตั้ง
`use_ws_trade=False` บน `KrakenExecutionClientConfig` เพื่อ route การดำเนินการเหล่านี้
ผ่าน REST

### รูปแบบ order ที่ route ผ่าน REST {#order-shapes-routed-via-rest}

[method `add_order` ของ Spot WebSocket v2](https://docs.kraken.com/exchange/api-reference/spot-websocket-v2/add_order) ของ Kraken
รองรับรูปแบบเหล่านี้ แต่ adapter route ผ่าน REST:

| รูปแบบ                      | พฤติกรรมของ adapter                                                |
| -------------------------- | ---------------------------------------------------------------- |
| time in force แบบ `FOK`     | ตัวสร้างพารามิเตอร์ของ WebSocket ไม่ได้ encode `FOK`                   |
| Trailing stop / stop-limit | ตัวสร้างพารามิเตอร์ของ WebSocket ไม่ได้ encode trailing offset         |
| Iceberg (`display_qty`)    | ตัวสร้างพารามิเตอร์ของ WebSocket ไม่ได้ encode iceberg order          |
| order แบบ quote quantity    | WS รองรับ buy market order ที่ไม่ใช่ margin; adapter ใช้ REST         |

order list ที่มีหลาย symbol ก็ใช้ REST เช่นกัน เพราะ request `batch_add` ของ WebSocket ของ Kraken
ต้องใช้ symbol เดียวกันทั้งหมด ราคาอ้างอิงของ trigger ที่ไม่รองรับจะถอยกลับไปใช้
เส้นทาง REST ซึ่งจะปฏิเสธในเครื่องก่อนส่ง request ไปยัง Kraken
สำหรับ order แบบ Spot margin ให้ใช้ REST ตามที่อธิบายไว้ใน [การเทรดแบบ Spot margin](#spot-margin-trading)

การ override ราย call ด้วย `params={"use_ws_trade": False}` จะบังคับให้
คำสั่งเดียวผ่าน REST ไม่ว่าค่าเริ่มต้นที่ตั้งไว้จะเป็นอะไร ตั้งได้บน
`SubmitOrder`, `ModifyOrder`, `CancelOrder`, `SubmitOrderList` หรือ
`BatchCancelOrders`

### Timeout ของ request ผ่าน WebSocket {#websocket-request-timeout}

เมื่อการรับส่งไปกลับผ่าน WebSocket ใช้เวลาเกิน `ws_request_timeout_secs` (ค่าเริ่มต้น `5`)
ผลลัพธ์ที่ venue จะยังไม่ทราบ request แบบ submit, modify, cancel และ batch-add
จะยังคงอยู่ระหว่างดำเนินการ (in flight) โดยไม่มีการปฏิเสธขั้นสุดท้าย dispatcher จะเก็บ
request ID ไว้ เพื่อให้ response ที่ตรงกันซึ่งมาช้ายังนำไปใช้ในการจัดการแบบปกติได้ ทั้ง
กรณีสำเร็จหรือการปฏิเสธที่ชัดเจน

timeout ของ submit และ batch-add ยังส่งการยกเลิกเพื่อชดเชยแบบ best-effort ผ่าน
WebSocket เดียวกันสำหรับทุก client order ID ที่ได้รับผลกระทบ การยกเลิกนี้จำกัด
ความเสี่ยงในกรณีที่ Kraken ยอมรับ order แล้วแต่ตอบกลับช้า แต่ไม่ได้
แทนที่ผลลัพธ์ที่ไม่ทราบด้วยสถานะสุดท้ายในเครื่อง

การอัปเดตจาก stream และ live execution reconciliation engine จะ resolve order เมื่อ
ไม่มี response ที่ตรงกันมาถึง การ query สถานะแบบเจาะจงสามารถ resolve request แบบ modify หรือ
cancel ที่มี venue order ID แล้ว response ที่ตรงกันหรือการปิด execution client
จะยุติการผูก correlation ของ request ที่เก็บไว้

:::tip
ตั้ง `ws_request_timeout_secs` ให้สูงกว่า latency ไปกลับที่สังเกตได้ค่อนข้างมาก
timeout ที่เร็วเกินไปอาจส่งการยกเลิกเพื่อชดเชยสำหรับ submit หรือ
batch add ที่ Kraken ยอมรับไปแล้ว
:::

### ตัวเลือกการ routing order ผ่าน WebSocket {#websocket-order-routing-options}

`KrakenExecutionClientConfig` มีตัวเลือก:

| ตัวเลือก                    | ค่าเริ่มต้น | คำอธิบาย                                                     |
| ------------------------- | -------- | ---------------------------------------------------------- |
| `use_ws_trade`            | `True`   | route order ผ่าน WS เมื่อ trade channel active               |
| `ws_request_timeout_secs` | `5`      | จำนวนวินาทีที่รอ response ของ order ผ่าน Spot WS              |

## Reconciliation {#reconciliation-1}

execution client ทั้งสองตัวจะ reconcile order, fill, position และสถานะ account
ในเครื่องกับ Kraken ตอน startup และระหว่างการทำงาน

### Report ที่มีขอบเขต {#bounded-reports}

เมื่อ reconciliation ระบุ lookback ไว้ execution client ทั้งสองตัวจะ derive จุดตัด (cutoff) หนึ่งจุด
สำหรับทุก query ย้อนหลัง และบันทึกไว้บน mass status ผ่าน `set_report_window`
จุดตัดที่เลื่อนไปเรื่อยๆ อาจรวม report ที่ไม่เคยมีอยู่พร้อมกันที่ venue เข้าด้วยกันได้

การประกาศจุดตัดจะเปิดใช้กฎประวัติแบบมีขอบเขตของ engine ชุดข้อมูลที่ไม่สมบูรณ์ยังคง reconcile
report ที่ได้รับมาพร้อม warning flag ความสมบูรณ์ใช้ระบุคุณลักษณะของชุดข้อมูล ไม่ได้ขัดขวาง reconciliation

#### ขอบเขตและการอ่านที่ล้มเหลว {#scope-and-read-failures}

open order หรือ position ที่อยู่ในขอบเขตแต่ไม่สามารถ resolve เป็น instrument หรือ parse ได้
จะทำให้การอ่านล้มเหลวบน client ทั้งสองตัว การละไว้จะเป็นการบอก reconciliation ว่า venue
ไม่เคยถือ order หรือ position นั้น

Futures มีข้อยกเว้นหนึ่งข้อ: stop หรือ take-profit แบบ reduce-only ที่ยังไม่ถูกแตะ อาจไม่มี
ขนาดที่ยังไม่ fill จนกว่าจะถูก trigger หากสัญญาของมันไม่มี open position ที่จะใช้กำหนดขนาด
การอ่านจะข้าม order นั้นและแถวประวัติของมันพร้อม warning และระบุว่าชุดข้อมูล
ไม่สมบูรณ์ การอ่าน open position ที่ล้มเหลวจะทำให้การอ่าน order ล้มเหลวด้วย การ lookup สถานะของ
order เดี่ยวสำหรับ trigger นั้นจะล้มเหลวขณะที่ถูกข้าม ดังนั้น reconciliation จะเลื่อนออกไป

client ทั้งสองตัวไม่คืนแถวใดเลยสำหรับ query ที่จำกัดเฉพาะ instrument ที่ตัวเองไม่ได้ถือ Spot และ Futures ใช้
venue `KRAKEN` ร่วมกัน ดังนั้น client ตัวใดตัวหนึ่งอาจได้รับ query สำหรับ order ที่ตัวเองไม่ได้ route การ lookup
สถานะของ order เดี่ยวและ `query_order` จะตรวจสอบเฉพาะ instrument ของคำสั่งนั้น record ที่ resolve ไม่ได้หรือ
parse ไม่ได้บน instrument อื่นจะไม่ทำให้การอ่านที่จำกัดขอบเขตนั้นล้มเหลว แม้ว่าการ lookup จะไม่สามารถตัด
order ที่ record ดังกล่าวอาจระบุถึงออกไปได้

ช่องว่างอื่นจะทำให้ชุดข้อมูลถูกระบุว่าไม่สมบูรณ์: order และ fill ที่ปิดแล้วหรือย้อนหลังซึ่ง parse ไม่ได้ หรือ
resolve instrument ไม่ได้ record ของ position ไม่มีส่วนต่อ flag ความสมบูรณ์ ความล้มเหลวของ position
ที่อยู่ในขอบเขตจะทำให้การอ่านล้มเหลว

#### การ lookup order เดี่ยว {#single-order-lookups}

การ lookup สถานะของ order เดี่ยวที่ไม่พบ report ใดเลยจะบอก reconciliation ว่า venue
ไม่มี order นั้น และ reconciliation จะ resolve order ตามคำตอบนั้น การ lookup
จะให้คำตอบนั้นได้ก็ต่อเมื่อมาจากการอ่านที่ตัด order ออกได้แล้วเท่านั้น มิฉะนั้นจะล้มเหลว
เพื่อให้ reconciliation เลื่อน order ออกไป:

- การอ่านที่สมบูรณ์จะตัด order ใดๆ ที่มันอาจคืนมาได้ออก order ของ futures ที่ถูกพัก
  จะปรากฏเฉพาะใน orders-status window ซึ่งการ lookup ของ Futures จะอ่านสำหรับ order ใน cache
  ก่อนตอบ
- การอ่านที่ข้ามบางแถว รวมถึงแถวที่ client resolve instrument ไม่ได้ จะตัด order ที่มี
  venue order ID ที่ไม่มีแถวที่ถูกข้ามใดมีออกได้ แถวที่ถูกข้ามซึ่งไม่ได้ระบุ order ใด
  เช่น ชนิดของ history event ที่ไม่มีเอกสาร จะไม่ตัด order ใดออก
- การ lookup บน instrument ของผลิตภัณฑ์อีกประเภทจะถือว่าอ่านครบ เพราะ client อีกตัว
  เป็นผู้ตอบสำหรับ order ของตัวเอง ส่วนการ lookup บน instrument ของผลิตภัณฑ์ของ client เอง
  ที่มันไม่ได้ถือจะไม่อ่านอะไรเลยและไม่ตัด order ใดออก
- การอ่านที่ถูกตัดกลางคันจะตัด order ที่วางหลัง event ที่เก่าที่สุดที่มันถืออยู่ออกได้ โดยต้องหลังมากกว่า
  ค่าเผื่อ clock skew 60 วินาที โดยใช้เวลาที่เร็วกว่าระหว่างเวลาสร้าง
  และเวลาที่ได้รับการยอมรับของ order หน้าประวัติของ Futures เรียงจากใหม่ไปเก่า ดังนั้นหน้าที่
  คืน continuation token มาจะย้อนไปถึง event ที่เก่าที่สุดของหน้า การอ่าน closed order ของ spot
  ที่หยุดเพราะถึงขีดจำกัดจำนวนหน้าจะไม่ตัด order ใดออก

order ที่การอ่านไม่สามารถตัดออกได้จะถูกเลื่อนออกไปอีกในการตรวจสอบ open order ทุกครั้ง บน Futures การ lookup
แต่ละครั้งแบบนี้ใช้ token ของ order-history หนึ่ง token

#### ขีดจำกัดการแบ่งหน้าของ Spot {#spot-pagination-limits}

การอ่าน closed order และ fill ของ Spot จะเลื่อน offset ไปเรื่อยๆ จนกว่า Kraken จะคืน
หน้าว่าง โดยจำกัดที่ 500 หน้าต่อการอ่าน การถึงขีดจำกัดจะ log warning:

- mass status ตอน startup บันทึกความสมบูรณ์ของการอ่าน order และ fill ไว้ใน report
  window ของมัน ดังนั้น engine จะมองชุดข้อมูลที่ถูกตัดว่าไม่สมบูรณ์
- `generate_order_status_reports` และ `generate_fill_reports` คืน record ที่
  อ่านได้จนถึงขีดจำกัดโดยไม่เปิดเผย flag ความสมบูรณ์
- การ lookup สถานะของ order เดี่ยวที่ไม่พบอะไรในการอ่าน closed order ที่ถูกตัดกลางคัน
  เพราะขีดจำกัดจะล้มเหลว เพื่อให้ reconciliation เลื่อน order ออกไป

### Reconciliation ของ Spot {#spot-reconciliation}

#### Order status report {#order-status-reports}

- Open order: ดึง order ที่ active อยู่ทั้งหมดในปัจจุบัน
- Closed order: ดึง order ย้อนหลังพร้อมรองรับการแบ่งหน้า
- Query แบบมีขอบเขตเวลา: รองรับการกรองด้วย timestamp เริ่มต้น/สิ้นสุด
- mass status ตอน startup อ่าน closed order ควบคู่กับ open order ดังนั้น order ที่ไปถึงสถานะ
  สุดท้ายระหว่างที่ node หยุดทำงานจะถูก reconcile lookback ของ reconciliation จำกัดขอบเขตการอ่าน และ
  การอ่าน closed order ที่ถูกตัดกลางคันเพราะขีดจำกัดจำนวนหน้าจะทำให้ mass status ไม่สมบูรณ์
- report จะมีราคาของ order เอง ไม่ใช่ `price` ระดับบนสุดของแถว ฟิลด์นั้นคือราคา fill เฉลี่ย
  และอ่านได้เป็นศูนย์สำหรับ order ที่ยังไม่ fill ส่วน `avg_px` ของ report มาจาก `cost` และ `vol_exec`

  ราคา limit มาจาก `descr.price` สำหรับ limit order และ `descr.price2` สำหรับ stop-loss-limit หรือ
  take-profit-limit order หรือ trailing-stop-limit order เมื่อ `misc` ระบุว่าเป็น `stopped` แล้ว
  `limitprice` ของ trailing-stop-limit เก็บราคาสูงสุด (peak) ไม่ใช่ limit ของมัน order แบบ market, settle-position,
  stop-loss, take-profit หรือ trailing-stop ไม่มีราคา limit

  ราคา trigger มาจาก `stopprice` ซึ่ง order แบบ stop-loss, take-profit, stop-loss-limit หรือ take-profit-limit
  จะเติมค่าก็ต่อเมื่อถูก trigger แล้ว ดังนั้นก่อนหน้านั้นจะใช้ `descr.price` ตามที่ AddOrder รับเข้าไปแทน ราคาที่เป็นศูนย์หรือ
  แบบสัมพัทธ์ถือว่าไม่มีค่า
- trailing-stop-limit order ที่ยังไม่ถูก trigger ไม่มี limit แบบสัมบูรณ์ ดังนั้น report ของมันจึงไม่มีราคา
  และ order ที่ cache ไม่ได้ถือไว้จะไม่สามารถสร้างขึ้นใหม่จาก report นี้ได้จนกว่าจะถูก trigger
- frame ของ executions จะระบุราคาของ order ก็ต่อเมื่อมี `limit_price` เท่านั้น ไม่ใช่จาก
  ราคา fill

#### Fill report {#fill-reports}

- ประวัติ trade: ดึงประวัติการ execute พร้อมการแบ่งหน้า
- Query แบบมีขอบเขตเวลา: รองรับการกรองด้วย timestamp เริ่มต้น/สิ้นสุด
- fill ทุกประเภท: fill ของ market, limit และ conditional order

#### การสะกดชื่อคู่ {#pair-spelling}

- Kraken สะกดชื่อคู่ได้สองแบบ: key ของ `AssetPairs` (`XXBTZEUR`) ซึ่งใช้เป็น `raw_symbol`
  ของ instrument และ altname (`XBTEUR`) `OpenPositions` คืน key ในขณะที่ `OpenOrders`
  และ `TradesHistory` คืน altname
- adapter resolve ได้ทั้งสองการสะกด ดังนั้น order หรือ fill บนคู่ที่ใช้ชื่อแบบเก่าจะถูกรายงาน
- การอ่านที่จำกัดเฉพาะ instrument เดียวจะ resolve แต่ละแถวแล้วเปรียบเทียบ instrument ID แทนการใช้
  `raw_symbol` ใน cache และการสะกดของ venue ดังนั้นจึงคืน record ของคู่นั้นเองไม่ว่า Kraken จะสะกดแบบใด
- การอ่านที่จำกัดเฉพาะ instrument ที่ client ไม่ได้ถือจะไม่คืนอะไรเลย ID ของ spot และ futures ใช้
  venue `KRAKEN` ร่วมกัน ดังนั้น futures ID อาจไปถึง spot client และกลับกัน ทั้งสองจะไม่ถอยกลับไปคืน
  record ของทุก instrument การ lookup order เดี่ยวบน instrument ของผลิตภัณฑ์ของ client เองที่ไม่ได้ถือ
  จะไม่ตัด order ใดออก
- spot instrument ที่ altname ต่างจาก key ของ `AssetPairs` จะเก็บ altname ไว้ใน map
  `info` ดังนั้น client ที่ได้ instrument มาผ่าน `cache_instrument` หรือ
  `cache_instruments` จะ resolve record ที่สะกดด้วย altname ได้โดยไม่ต้องดึง `AssetPairs` ใหม่
- ในการอ่านแบบไม่จำกัดขอบเขต open order ที่ resolve คู่เป็น instrument ใน cache ไม่ได้จะทำให้
  การอ่านล้มเหลว แทนที่จะถูกละไว้จากการอ่านที่สำเร็จ การอ่านแบบจำกัดขอบเขตจะข้าม record ที่
  resolve ไม่ได้ ซึ่งอาจเป็นของ instrument อื่น และการ lookup order เดี่ยวจะไม่สามารถตัด
  order ที่มันระบุออกได้
- closed order หรือ fill ที่ resolve ไม่ได้จะถูก log เป็น warning และข้ามไป โดยคง
  record ที่ resolve ได้ไว้ record ย้อนหลังมักมีอายุยาวกว่าชุด instrument ที่โหลดไว้เป็นปกติ

#### ยอดเงินของ account {#account-balances}

- ยอดเงินใน wallet: ดึงจาก
  [`POST /0/private/BalanceEx`](https://docs.kraken.com/api-reference/account-data/get-extended-balance)
  ซึ่งรายงานยอดรวมและยอดที่ถูกพัก (`hold_trade`) ต่อ asset ยอดที่ถูกพักจะเติมลงใน
  `AccountBalance.locked` ดังนั้น `free` จะไม่รวมเงินที่ Kraken จองไว้สำหรับ
  Spot order ที่ไม่ใช่ margin ซึ่งพักอยู่ใน book สำหรับ account ที่มีวงเงินเครดิต เครดิตสุทธิ (`credit - credit_used`)
  จะรวมอยู่ใน `AccountBalance.total` ดังนั้น `free` จะตรงกับยอดที่ใช้ได้ของ Kraken
  คือ `balance + credit - credit_used - hold_trade`
- ยอดเงินเป็นศูนย์: asset ที่ Kraken แสดงเป็นศูนย์จะถูกรายงานเป็นศูนย์แทนที่จะถูกละไว้
  ทั้งบน spot และ futures engine จะเพิ่มยอดเงินเท่านั้น ดังนั้นสกุลเงินที่ถูกละไว้
  จาก snapshot จะคงค่าเดิมไว้ บน futures ค่าศูนย์จะเข้าร่วมในผลรวมต่อสกุลเงิน
  ดังนั้น wallet ที่มีเงินคู่กับ wallet ว่างของ asset เดียวกันจะรายงานยอดที่มีเงิน
  asset ที่ Kraken ตัดออกจาก response ทั้งหมดจะยังคงค่าที่รายงานล่าสุดไว้

#### Margin position report {#margin-position-reports}

เมื่อใช้ `spot_account_type=AccountType.MARGIN`:

- Open position: ดึงจาก `POST /0/private/OpenPositions` แล้วรวม
  ตามคู่เป็นรายการ `PositionStatusReport` Kraken คืนหนึ่งรายการต่อ lot
  ดังนั้น lot ที่สวนทางกันของคู่เดียวกันจะหักกลบเป็น report เดียว
- ราคาเข้าเฉลี่ย: แต่ละ report มี `avg_px_open` ซึ่ง derive จากฟิลด์ `cost`
  และ `vol` ของ lot และถ่วงน้ำหนักด้วยปริมาณที่ยังเปิดอยู่ lot ฝั่ง long และ short จะ
  หาค่าเฉลี่ยแยกกัน ดังนั้นค่าเฉลี่ยที่รายงานจะอธิบายฝั่งที่เหลืออยู่หลัง
  การหักกลบ reconciliation ต้องใช้ค่านี้ในการเปิด position จาก report เมื่อ
  cache ไม่มีประวัติ order หรือ fill ของ position นั้น
- ค่าเฉลี่ยนี้ถูกระบุเป็น `AvgPxReconciliation::OpeningOnly` เพราะ Kraken ปิด margin lot
  แบบ FIFO และตัด lot ที่ปิดครบแล้วออกจาก `OpenPositions` หลังการปิดบางส่วน ค่าเฉลี่ย
  จึงอธิบาย lot ที่ยังเปิดอยู่แทนที่จะเป็น fill ตอนเปิด และจะไม่ตรงกับ
  ค่าเฉลี่ยของ netting position reconciliation ใช้ค่านี้เพื่อเปิด position จากสถานะ flat และไม่เคย
  เปรียบเทียบมัน Futures คงค่าเริ่มต้นไว้ เพราะ endpoint นั้นรายงาน position ที่หักกลบแล้วหนึ่งรายการซึ่ง
  Kraken ระบุในเอกสารว่าราคาเป็นราคาเข้าเฉลี่ย
- ไม่มีการเก็บกวาดด้วย FLAT สังเคราะห์: `OpenPositions` รายงานเฉพาะ position ที่ใช้ leverage ดังนั้น
  holding ของ spot ที่ไม่ใช้ leverage จะไม่ปรากฏที่นั่น และการไม่ปรากฏไม่ใช่หลักฐานว่า
  position ปิดแล้ว การอ่านแบบกลุ่มจะรายงานเฉพาะสิ่งที่ venue คืนมา
- ยอดเงิน margin: `POST /0/private/TradeBalance` ถูกเรียกควบคู่กับการ refresh สถานะ account margin ที่ใช้ไป
  จะเติมลงใน `MarginBalance.initial` ในขณะที่ equity และ free margin เติมลงใน summary balance (ดู
  การเทรดแบบ Spot margin)

:::warning
position ที่ใช้ leverage ซึ่งถูกปิดระหว่างที่ node หยุดทำงาน จะไม่ถูกกู้คืนจาก fill ที่ปิดมัน เมื่อ
ตั้ง `reconciliation_lookback_mins` ไว้ lot ที่ปิดครบแล้วจะไม่อยู่ใน `OpenPositions` ดังนั้น instrument
จึงไม่มี position report engine จึง project fill ของ order นั้นแบบ order-only order จะไปถึง
`FILLED` แต่ position ใน cache จะคงทั้ง quantity และ realized PnL ไว้ PnL จากการปิดจะไม่ถูกบันทึกเลย

นี่คือพฤติกรรมที่ระบุไว้ในเอกสารของ engine ที่ใช้ร่วมกันสำหรับ instrument ที่ไม่มี position report ในขอบเขต ไม่ใช่
กฎของ Kraken ดู
[Order-only fill projection](../concepts/execution/reconciliation.md#order-only-fill-projection) การนำ
FLAT สังเคราะห์ออกคือสิ่งที่ทำให้ spot margin ของ Kraken ได้รับผลจากพฤติกรรมนี้ เพราะก่อนหน้านี้ sweep จะให้
FLAT อย่างชัดเจน

การตรวจสอบ position เป็นระยะก็ไม่ได้กู้คืนเช่นกัน เพราะโหมด margin ไม่ได้ประกาศการครอบคลุม position
แบบกลุ่ม และการข้ามนั้นจะ log ที่ระดับ debug เงื่อนไขนี้ยังคงอยู่แม้ restart: order
ที่ปิดจะถูก cache เป็น `FILLED` และตรงกับ venue ทุกประการ reconciliation จึงถือว่า
ซิงก์กันแล้ว

การไม่ตั้ง `reconciliation_lookback_mins` จะหลีกเลี่ยงการ project นี้ได้ order ที่ปิดซึ่งอยู่ใน cache แล้ว เช่น
การออกจาก position ของ strategy ที่ส่งก่อนระบบหยุด จะกู้คืนเข้าไปใน position ของมัน fill จะถูกนำไปใช้กับ
order ใน cache และปิด position ของมัน

วิธีนี้ไม่ใช่ทางแก้ทั่วไป เพราะ order ที่ปิดซึ่งไม่อยู่ใน cache จะถูกถือว่าเป็นของ strategy `EXTERNAL`
และ netting position ใช้ instrument และ strategy เป็น key เว้นแต่ position ใน cache จะ
เป็นของ `EXTERNAL` หรือ instrument ถูก claim ผ่าน `external_order_claim` fill ที่กู้คืนมาจะเปิด
position ที่สองซึ่งสวนทางกัน และไม่ได้ปิดตัวที่อยู่ใน cache ความเสี่ยงสุทธิ (net exposure) จะเป็นศูนย์ แต่ position
ที่ค้างอยู่และ realized PnL ของมันยังคงอยู่

จนกว่าเรื่องนี้จะได้รับการแก้ไข ให้ reconcile margin position ที่ปิดระหว่างที่ระบบหยุดทำงานด้วยตนเอง หรือใช้
`spot_account_type=AccountType.CASH` ร่วมกับ `use_spot_position_reports=True` ซึ่งการอ่าน wallet จะแจกแจง
ทุก holding ที่ครอบคลุม และการไม่มี report จึงเป็นหลักฐานจริงว่าเป็น flat
:::

### Reconciliation ของ Futures {#futures-reconciliation}

#### Client order ID {#client-order-ids}

order ของ Futures จะถูกส่งพร้อม client order ID แบบเต็ม: Kraken Futures รับได้
สูงสุด 100 ตัวอักษร และ order ที่ ID ยาวกว่านั้นจะถูกปฏิเสธ (denied) ก่อน
ส่ง พร้อมกับสมาชิกอื่นที่ยังเปิดอยู่ทั้งหมดใน order list ของมัน venue จะรายงาน client
order ID ที่มันถืออยู่แล้ว หรือที่ยาวเกินหรือไม่ถูกต้องด้วยเหตุอื่น เป็น send status
ที่ปฏิเสธ order

การยกเลิกหรือการแก้ไขจะระบุ order ด้วย identifier เดียว: venue order ID ของมันเมื่อ
ทราบ มิฉะนั้นใช้ client order ID แบบเต็ม

report ของ open order และ fill จะอ่าน client order ID ภายใต้ key ที่มีเอกสาร
`cliOrdId` และภายใต้ `cli_ord_id` ด้วย ID ที่ echo กลับมาซึ่งว่างหรือไม่ใช่
client order ID ที่ถูกต้อง เช่น ID ที่เครื่องมืออื่นบน account ส่งมา จะไม่ระบุ
order ใด

order ของ futures ที่ส่งไปด้วย ID ที่ถูกย่อเป็น `O` ตามด้วย 17 ตัวอักษรสุดท้าย แบบเดียวกับ
ที่ spot ID ที่ยาวเกิน 18 ตัวอักษรถูกย่อเว้นแต่จะเป็น UUID จะถูกเก็บไว้ในรูปแบบ
นั้น ทุก REST report และ fill ของ order ที่ cache ทำ index ด้วย venue order ID
จะมี client order ID ที่อยู่ใน cache ไม่ว่า venue จะ echo อะไรกลับมา และ
orders-status window จะถูกสอบถามเกี่ยวกับ order ที่ถูกพักด้วยทั้งสองรูปแบบ การยกเลิกหรือ
แก้ไข order แบบนี้จะไปถึงมันได้ด้วย venue order ID เท่านั้น การยกเลิกหรือแก้ไขที่ส่งก่อน
report ที่มี ID นั้นจะไปถึง cache จะไม่ระบุ order ใดที่ venue และจะถูก
ปฏิเสธ

การ query order เดี่ยวแบบเจาะจงจะใช้ report ที่มี venue order ID ของ order
เมื่อทราบ มิฉะนั้น แถวหนึ่งจะเป็นของ order นั้นเมื่อมัน echo client order ID
ในรูปแบบใดรูปแบบหนึ่ง อยู่บน instrument และฝั่งของ order และวางไม่เกินห้า
วินาทีก่อนเวลาสร้าง order แถวที่เก่ากว่านั้นเป็น order อื่นที่ ID
ถูกย่อให้เหมือนกันหรือนำ ID มาใช้ซ้ำ fill ไม่มีเวลาวาง order ดังนั้นจึงใช้
เวลา fill แทน ซึ่งตัดออกได้เพียง fill ที่ execute ก่อนเวลาสร้าง
order แถวหรือ fill ของ venue order ที่ cache ทำ index ไว้ให้ order อื่น
จะไม่มีวันเข้าเงื่อนไข ไม่ว่าจะ echo อะไรมา เพราะ ID เต็มของ order นั้นอาจเป็น ID
ที่ถูกย่อของ order นี้ เมื่อไม่ทราบ venue order ID การ query จะล้มเหลวเมื่อการอ่านครั้งเดียวมี
แถวของ venue order สองรายการที่เข้าเงื่อนไข หรือมีแถวที่เข้าเงื่อนไขของ venue order ที่ cache
ไม่ได้ทำ index ให้ order ใด ซึ่ง echo ID เต็มของ order อื่นใน cache ที่ส่งไปโดยไม่มี
venue order ID บน instrument และฝั่งของ order นั้น และวางไม่เกินห้า
วินาทีก่อนเวลาสร้างของมัน หรือเมื่อไม่มีแถวใดเข้าเงื่อนไข และ fill ล่าสุดที่เข้าเงื่อนไข
เป็นของ venue order สองรายการ, มี fill แบบนั้นรวมอยู่ หรือมียอดน้อยกว่า quantity ของ order
เมื่อทราบ venue order ID
การ query จะล้มเหลวเมื่อไม่มีแถวหรือ fill ใดมี ID นั้น และพบแถวหรือ fill ที่เข้าเงื่อนไขของ
venue order อื่น ไม่ว่ากรณีใด reconciliation จะเลื่อน order ออกไปแทนที่จะ
ใช้แถวนั้นหรือรายงานว่า order หายไป

#### Order status report {#order-status-reports-1}

adapter คืน open order และเมื่อ `open_only=False` จะคืน order ย้อนหลังที่ปิดแล้วและ
fill แล้วด้วย การอ่านย้อนหลังมีข้อจำกัดดังด้านล่าง

#### ประวัติ order และการหาราคา {#order-history-and-pricing}

adapter อ่าน lifecycle event ของ order จาก
[`/api/history/v3/orders`](https://docs.kraken.com/api-reference/account-history/get-order-events) การอ่านแต่ละครั้ง
ใช้หนึ่งหน้า: endpoint `/history` ของ Kraken ใช้ pool ของ token ร่วมกัน 100 token ซึ่งเติมกลับ 100 ทุก 10 นาที
หน้าประวัติ order หนึ่งหน้าใช้หนึ่ง token ดังนั้นหน้าที่คืน continuation token มา ไม่ว่าจะใน body หรือ
header `Next-Continuation-Token` จะ log warning และปล่อยให้ชุดข้อมูลไม่สมบูรณ์แทนการอ่านต่อ
การ lookup สถานะของ order เดี่ยวยังอาจรายงานว่า order ใน cache ไม่มีอยู่ได้ เมื่อ order นั้นวางหลัง event ที่
เก่าที่สุดของหน้ามากกว่าค่าเผื่อ clock skew ดู[การ lookup order เดี่ยว](#single-order-lookups)

ประวัติแสดงทุก lifecycle event การอ่านคืนหนึ่ง report ต่อ order: snapshot ของ open order หาก
venue ยังแสดงอยู่ มิฉะนั้นจะคืนสถานะล่าสุดในประวัติ closed order จะไม่ถูกเปิดใหม่ด้วย
สถานะเปิดที่ประทับเวลาทีหลัง เช่น การ edit ที่ถูกปฏิเสธซึ่ง log หลังการยกเลิก

ชื่อสัญญาจะ resolve ตามที่ venue สะกด โดยเทียบแบบตรงทุกตัวก่อนแล้วจึงแบบไม่สนตัวพิมพ์ ประวัติ
ไม่มีราคา trigger ดังนั้น stop order จะถูกรายงานเป็น limit หรือ market order ที่มันจะ execute เป็นเมื่อ
ถูก trigger order ที่ venue เป็นผู้เริ่ม (liquidation, assignment, hedge assignment, unwind, block, RFQ) จะถูกรายงาน
เป็น market order

adapter จะข้ามแถวที่แทนค่าไม่ได้พร้อม warning และระบุว่าชุดข้อมูลไม่สมบูรณ์:

- ชนิดของ event ที่ไม่รู้จัก
- ประเภทของ order ที่รายงานเป็น `Unknown` หรือไม่ได้ระบุ
- ทิศทางที่ไม่รู้จัก
- timestamp ก่อน epoch

`OrderNotFound` ไม่มีสถานะของ order และจะถูกข้ามโดยไม่กระทบความสมบูรณ์
error ของ venue ที่คืนมาภายใต้สถานะ HTTP ที่สำเร็จจะทำให้การอ่านล้มเหลว

แถวประวัติขั้นสุดท้ายที่มีการ execute จะไม่มีราคาเฉลี่ย adapter จะหาราคา
จาก fill ในหน้าที่ครอบคลุม quantity ที่ fill ไปพอดี หรือจาก fill ในหน้ารวมกับ
fill ที่บันทึกไว้บน order ใน cache ซึ่งก็ต้องครอบคลุมพอดีเช่นกัน การอ่าน fill
ที่ล้มเหลวนับเป็นหน้าว่าง หากไม่ครอบคลุมพอดี การ query order เดี่ยวจะล้มเหลว
และการอ่านแบบกลุ่มจะกัน order ไว้ เพื่อให้ reconciliation เลื่อนออกไปแทนการอนุมาน
การ execute ที่ราคา limit

#### การสำรองด้วยประวัติตอน startup {#startup-history-fallback}

mass status ตอน startup อ่านหน้าประวัติหนึ่งหน้าควบคู่กับ open order ซึ่งช่วยกู้คืน
order ที่ไปถึงสถานะสุดท้ายระหว่างที่ระบบหยุดทำงานเมื่อหน้านั้นมี order นั้นอยู่
body ที่เป็น error ของ venue ภายใต้สถานะ HTTP ที่สำเร็จ ไม่ว่าจะเป็น code ใด หรือ HTTP
429 จะทำให้เกิด warning และถอยกลับไปใช้เพียง open order โดยระบุว่าชุดข้อมูลไม่สมบูรณ์
error ของ HTTP อื่น, ความล้มเหลวของ transport และ body ที่ parse ไม่ได้จะทำให้ mass status ล้มเหลว
เช่นเดียวกับการอ่าน open order ที่ล้มเหลว

#### การป้องกันการหาราคาตอน startup {#startup-pricing-safeguard}

mass status ใช้ข้อกำหนดเรื่องการครอบคลุม fill แบบพอดีเดียวกันกับ order ย้อนหลังขั้นสุดท้ายที่มีการ execute
โดยจะกัน order ที่ไม่ครอบคลุมไว้พร้อม warning ที่ระบุ order นั้น และระบุว่าชุดข้อมูลไม่สมบูรณ์ การอ่านครั้งต่อไปยังคงขอ
เฉพาะหน้า fill ล่าสุด จึงไม่สามารถย้อนหน้ากลับไปกู้คืนการ execute ที่เก่ากว่าซึ่งหายไปได้

fill ในหน้าของ order ที่ถูกกันไว้จะยังคงอยู่เมื่อ cache ถือ order นั้นไว้ เพราะ
engine สามารถ reconcile fill เหล่านั้นกับ order ได้โดยไม่ต้องมี report ส่วน fill ของ order
ที่ไม่อยู่ใน cache จะถูกกันไว้พร้อมกับ order

#### Instrument ที่เป็น flat {#flat-instruments}

การอ่าน position คืนเฉพาะ open position ดังนั้น instrument ที่ไม่มี position report จะเป็น flat ที่ venue
การอ่าน fill เป็นหน้าเดียว การซื้อขายไปกลับที่ fill ตอนเปิดเก่ากว่าหน้านั้นจะเหลือเพียง
ฝั่งปิด หากไม่มี `reconciliation_lookback_mins` engine จะนำทุก fill ที่เก็บไว้ไปใช้กับ position ซึ่งอาจเปิด
position ที่ venue ไม่ได้ถืออยู่

เพื่อชดเชย การอ่านตอน startup แบบไม่มีขอบเขตจะหักกลบ fill ของทุก order บนหน้าที่อ่านซึ่ง cache
ไม่ได้ถือไว้บน instrument ที่เป็น flat รวมถึง open order ด้วย เมื่อหักกลบแล้วไม่เป็นศูนย์และมี order ย้อนหลัง
ขั้นสุดท้ายอยู่ในนั้น order ขั้นสุดท้ายจะถูกกันไว้พร้อม fill ของมัน พร้อม warning และชุดข้อมูลจะถูกระบุว่า
ไม่สมบูรณ์

open order ยังคงอยู่ ดังนั้น fill ของ open order ยังอาจเปิด position ได้ instrument
ที่มี position เปิดอยู่ที่ venue จะปล่อยให้ position report จัดการ lookback แบบมีขอบเขตจะปล่อย
order ขั้นสุดท้ายให้ engine ซึ่งจะ project fill ที่ไม่มี position report ลงบนสถานะของ order เท่านั้น

#### Fill report {#fill-reports-1}

- ประวัติ fill: อ่าน
  [หน้า `/fills`](https://docs.kraken.com/api-reference/historical-data/get-your-fills) ล่าสุด คือ 100 fill ล่าสุด
  ข้ามทุกสัญญา futures การ execute ที่เก่ากว่านั้นจะไม่ถูกคืนมา
- การกรองตามเวลา: กรองที่ฝั่ง client ด้วย timestamp เริ่มต้น/สิ้นสุด (parse
  timestamp แบบ RFC3339)
- fill ทุกประเภท: fill แบบ maker และ taker พร้อมข้อมูลค่าธรรมเนียม

#### Position status report {#position-status-reports}

- Open position: ดึง futures position ที่ active ทั้งหมด
- ฟิลด์ของ report: ฝั่ง, ราคาเข้าเฉลี่ย และขนาด position จาก snapshot ของ REST
  Kraken ยังคืน unrealized funding มาด้วย แต่ `PositionStatusReport` ไม่ได้เปิดเผยค่านี้

#### สถานะ account {#account-state}

- ยอดเงิน: หนึ่งรายการต่อ asset ข้ามทุก wallet ตามที่อธิบายไว้ในส่วนรหัสสกุลเงิน
- Margin: หนึ่งรายการต่อสกุลเงิน รวมข้าม wallet แบบเดียวกับยอดเงิน ที่ทศนิยมแปดตำแหน่ง ข้อกำหนด
  margin ของ flex wallet เป็น USD จาก `initialMargin` และ `maintenanceMargin` ส่วนของ
  wallet แบบ single-collateral เป็น `currency` ของมัน ดังนั้นข้อกำหนดของ `fi_xbtusd` จะรายงานเป็น BTC
  และเงินที่ใช้ได้ของมันจะจำกัดเฉพาะยอดเงินของสกุลเงินนั้น wallet ที่ไม่มี
  `currency` ที่ใช้ได้จะไม่มีรายการ margin, ไม่รายงาน asset ใดของมันว่าถูกล็อก และ log warning

:::note
**การกรองตามเวลาของ Futures**: endpoint fill ของ Kraken Futures ไม่รองรับ
การกรองช่วงเวลาที่ฝั่ง server adapter จึง implement การกรองที่ฝั่ง client
โดย parse ฟิลด์ `fillTime` แล้วเปรียบเทียบกับ timestamp เริ่มต้น/สิ้นสุด
ที่ร้องขอ
:::

### Spot position report (โหมด cash) {#spot-position-reports-cash-mode}

ตั้ง `use_spot_position_reports=True` ในโหมด cash เพื่อรายงานยอดเงินใน wallet เป็น
Spot position ค่าเริ่มต้นคือ `False` account แบบ margin ใช้ `OpenPositions`
โดยไม่ขึ้นกับตัวเลือกนี้ (ดู[การเทรดแบบ Spot margin](#spot-margin-trading))

**การทำงาน**:

- เมื่อเปิดใช้ ยอดเงินใน wallet จะถูกแปลงเป็น object `PositionStatusReport`
- ยอดเงินที่เป็นบวกจะถูกรายงานเป็น position ฝั่ง `LONG`
- รายงานเฉพาะ instrument ที่ตรงกับ quote currency ที่ตั้งค่าไว้ (ค่าเริ่มต้น: `USDT`)
  ตัวกรองเดียวกันนี้ใช้ตัดสินว่า client ประกาศการครอบคลุม position แบบกลุ่มสำหรับ instrument ใด
  ดังนั้น instrument ที่ quote เป็นสกุลอื่นจะไม่ถูก reconcile เป็น flat จาก report ที่หายไป
- วิธีนี้ป้องกัน report ซ้ำเมื่อ asset เดียวกันมีให้ใช้กับหลาย
  quote currency (เช่น BTC/USD, BTC/USDT, BTC/EUR)

**การตั้งค่า**: ใน `KrakenExecutionClientConfig` ให้ระบุ `account_id`, `api_key`, `api_secret`
แล้วตั้ง `use_spot_position_reports=True` และ `spot_positions_quote_currency` (ค่าเริ่มต้น `"USDT"`)

:::warning
strategy ที่ปิด Spot position ที่ถูกรายงานอาจขาย holding ใน wallet ของคุณ
เปิดใช้ position report ที่ derive จาก wallet เฉพาะเมื่อ strategy ของคุณจัดการ holding เหล่านั้นได้
:::

## การเทรดแบบ Spot margin {#spot-margin-trading}

Kraken Spot รองรับการเทรดแบบใช้ leverage บนคู่ที่เลือกไว้ `AssetPairs` ประกาศ
tier ต่อคู่ไว้ใน `leverage_buy` และ `leverage_sell` adapter จะ cache ค่าเหล่านี้ตอน
โหลด instrument และตรวจสอบ order ผ่าน REST กับค่าเหล่านี้ก่อนส่ง
เปิดการเทรดแบบ margin ด้วย `spot_account_type=AccountType.MARGIN` และกำหนด leverage
ผ่าน `default_leverage` หรือพารามิเตอร์การส่งราย order

:::warning
ใช้ `use_ws_trade=False` สำหรับ Spot margin order เส้นทาง WebSocket ของ adapter
จะส่งต่อ `leverage` ที่เป็นตัวเลขโดยไม่ตรวจสอบกับ tier ใน cache ของคู่นั้น ส่วน
[schema `add_order` ของ WebSocket](https://docs.kraken.com/exchange/api-reference/spot-websocket-v2/add_order) ของ Kraken
กลับกำหนดฟิลด์ `margin` แบบ boolean ที่เลือก leverage สูงสุดของคู่นั้น
REST รองรับ[พารามิเตอร์ leverage](https://docs.kraken.com/api-reference/trading/add-order)แบบระบุชัดเจน
และการตรวจสอบ tier ในเครื่องของ adapter
:::

### การตั้งค่า {#configuration}

ใน `KrakenExecutionClientConfig` ให้ระบุ `account_id`, `api_key`, `api_secret` แล้วตั้ง
`spot_account_type=AccountType.MARGIN` และ `use_ws_trade=False` สามารถตั้ง `default_leverage`
(ค่าเริ่มต้นระดับ config แบบ optional) และ `margin_balance_asset` (asset สำหรับแสดงผล summary แบบ optional เช่น `"ZGBP"`) ได้ด้วย

`margin_balance_asset` ควบคุมเพียงสกุลเงินที่ใช้แสดง metric สรุปของ account
ที่ endpoint `TradeBalance` ของ Kraken คืนมา (equity, free margin,
used margin เป็นต้น) ตัวเลขราย position จาก `OpenPositions` จะเป็น
quote currency ของคู่ที่ซื้อขายเสมอ

### Leverage ราย order {#per-order-leverage}

override ค่าเริ่มต้นที่ตั้งไว้สำหรับ order เดียวผ่าน `params` เช่น ส่ง order ด้วย
`params={"leverage": 5}`

บน REST adapter จะตรวจสอบ tier กับ `leverage_buy` หรือ
`leverage_sell` ของคู่นั้นก่อนส่ง order tier ที่ไม่รองรับจะทำให้เกิด event
`OrderRejected` สำหรับการส่ง order เดี่ยวโดยไม่ไปถึง Kraken
พารามิเตอร์ leverage ราย order ที่ผิดรูปแบบ เช่น string หรือค่าที่อยู่นอก
`[0, 65535]` จะทำให้เกิด `OrderDenied` ก่อนการส่งไม่ว่าจะเป็นเส้นทางใด

### Reduce-only {#reduce-only}

margin order สามารถระบุ `reduce_only=True` เพื่อลด position ที่มีอยู่
โดยไม่เปิด position ฝั่งตรงข้ามที่ใหญ่กว่า ให้ตั้ง `spot_account_type=AccountType.MARGIN` และระบุ
`default_leverage` หรือ `params={"leverage": N}` ราย order adapter จะปฏิเสธ (deny)
order แบบ cash ที่มี `reduce_only` ก่อนส่งไปยัง Kraken

### สถานะ account {#account-state-1}

เมื่อใช้ `spot_account_type=AccountType.MARGIN` execution client จะเรียก endpoint
`TradeBalance` ของ Kraken ระหว่างการ refresh account สถานะ account แบบ live ใช้:

- equity (`e`) และ free margin (`mf`) สำหรับยอดเงินในสกุลที่กำหนดโดย
  `margin_balance_asset`
- used margin (`m`) สำหรับ `MarginBalance.initial` maintenance margin เป็นศูนย์
  เพราะ Kraken ไม่ได้คืนจำนวน maintenance margin แยกต่างหาก

method ระดับล่างของ `KrakenSpotHttpClient` คือ `request_margin_metrics()` และ
`request_account_state_with_metrics()` จะคืน metric ทั้งหมดของ `TradeBalance`
สำหรับผู้ใช้โดยตรง live execution client ไม่ได้แนบ
ข้อมูลนั้นไว้ใน `AccountState.info`

### Reconciliation ของ position {#position-reconciliation}

spot margin position ที่เปิดอยู่จะถูกดึงขึ้นมาผ่าน `POST /0/private/OpenPositions`
ในทุกรอบของ `position_check_interval_secs` เส้นทางนี้ไม่ขึ้นกับ
`use_spot_position_reports` (ซึ่ง derive จาก wallet และใช้ได้เฉพาะโหมด cash)

spot client ประกาศการครอบคลุม position แบบกลุ่มต่อ instrument และเฉพาะ instrument ที่
การอ่านจะแจกแจงได้จริง: โหมด cash ที่ตั้ง `use_spot_position_reports=True` และ
instrument ที่ quote เป็น `spot_positions_quote_currency` (ดู Spot position report ซึ่งใช้
ตัวกรองเดียวกัน) ภายใต้ `spot_account_type=AccountType.MARGIN` แหล่งข้อมูลคือ `OpenPositions` ซึ่งไม่รวม
lot ที่ไม่ใช้ leverage และโหมด cash ที่ไม่มี report จาก wallet จะไม่คืนอะไรเลย ในจุดใดก็ตามที่
ไม่ได้ประกาศการครอบคลุม การไม่มี report จะปล่อยให้ position ใน cache ไม่ถูกแตะต้อง แทนที่จะ
ปิดมัน

## Funding rate {#funding-rates}

adapter รับข้อมูล funding rate จาก WebSocket feed
[Futures ticker](https://docs.kraken.com/exchange/api-reference/futures-websocket/ticker)
ซึ่งให้ `relative_funding_rate` และ
`next_funding_rate_time` สำหรับ perpetual futures

adapter ปล่อย `FundingRateUpdate.interval` เป็น `None` เพราะ ticker feed ไม่มีฟิลด์
interval ของ funding ซึ่งไม่ได้หมายความว่าสัญญาไม่มีรอบ funding ที่กำหนดไว้
[ข้อกำหนดของสัญญา linear multi-collateral](https://support.kraken.com/articles/4844359082772-linear-multi-collateral-derivatives-contract-specifications) ของ Kraken
กำหนดรอบ funding เป็นรายชั่วโมง funding ที่สะสมจะชำระเมื่อสิ้นชั่วโมงหรือเมื่อ position สุทธิ
เปลี่ยนแปลง

## Rate limiting {#rate-limiting}

HTTP client ของ Kraken แต่ละตัวใช้การจำกัดอัตรา request ฝั่ง adapter ค่าเริ่มต้นคือ
ห้า request ต่อวินาที และ `max_requests_per_second` สามารถ override ได้ นี่เป็น
การจำกัดตามจำนวน request ไม่ใช่โมเดลที่สมบูรณ์ของต้นทุน endpoint หรือ
งบประมาณตาม tier ของ account ของ Kraken

Kraken ใช้ขีดจำกัดของ venue ที่ต่างกันสำหรับ Spot และ Futures:

- [rate limit ของ Spot REST](https://docs.kraken.com/exchange/guides/rest/ratelimits)
  ใช้ตัวนับการเรียกที่ขึ้นกับ tier การเรียก ledger และประวัติ trade จะเพิ่ม `2`
  การเรียก REST อื่นส่วนใหญ่เพิ่ม `1` และการจัดการ order ใช้ตัวจำกัดการเทรด
  แยกต่างหาก
- [rate limit ของ Derivatives](https://docs.kraken.com/exchange/guides/futures/ratelimits)
  ใช้ต้นทุนต่อ endpoint และงบประมาณแยกสำหรับ path `/derivatives` และ `/history`

Kraken ระบุขีดจำกัดของตัวนับการเรียก Spot REST ไว้ดังนี้:

| Spot tier    | ตัวนับสูงสุด | อัตราการลดของตัวนับ |
| ------------ | ---------- | ---------------- |
| Starter      | 15         | 0.33/วินาที       |
| Intermediate | 20         | 0.5/วินาที        |
| Pro          | 20         | 1/วินาที          |

หากอัตรา request คงที่ของ adapter สูงเกินไปสำหรับสัดส่วนการใช้ endpoint และ tier ของ
account Kraken ยังคงอาจปฏิเสธหรือจำกัดอัตรา request ได้

### คำแนะนำเรื่อง interval ของ reconciliation {#reconciliation-interval-guidance}

การตั้งค่า `open_check_interval_secs` และ
`position_check_interval_secs` ของ execution engine สร้างภาระต่อเนื่องให้กับ private REST API
interval ที่สั้นอาจใช้งบประมาณของ venue ของ Kraken หมด แม้ว่า adapter จะอยู่
ต่ำกว่าการจำกัด request ต่อวินาทีที่ตั้งไว้ก็ตาม

ให้ใช้ interval แบบระมัดระวังเป็นจุดเริ่มต้น โดยเฉพาะสำหรับ account ระดับ Spot Starter
เช่น ใน `LiveExecutionEngineConfig` ตั้ง `reconciliation=True`, `open_check_interval_secs=30.0`
(จุดเริ่มต้นแบบระมัดระวังสำหรับ tier Spot Starter) และ `position_check_interval_secs=120.0`

ปรับค่าเหล่านี้ตาม tier ของ account, การตรวจสอบ reconciliation ที่เปิดไว้ และ
client อื่นที่ใช้ API key เดียวกัน หาก Kraken คืน `EAPI:Rate limit exceeded`
ให้เพิ่ม interval หรือลด `max_requests_per_second`

## การตั้งค่า {#configuration-1}

### ตัวเลือก config ของ data client {#data-client-configuration-options}

| ตัวเลือก                    | ค่าเริ่มต้น  | คำอธิบาย                                                          |
| ------------------------- | --------- | --------------------------------------------------------------- |
| `product_type`            | `SPOT`    | ประเภทผลิตภัณฑ์ของ client นี้ (`SPOT` หรือ `FUTURES`)               |
| `environment`             | `LIVE`    | environment การเทรด (`LIVE` หรือ `DEMO`); demo ใช้ได้เฉพาะ Futures |
| `api_key`                 | `None`    | API key สำหรับข้อมูล L3 ของ Spot                                  |
| `api_secret`              | `None`    | API secret สำหรับข้อมูล L3 ของ Spot                               |
| `base_url`                | `None`    | override base URL ของ REST ของ Kraken                            |
| `ws_public_url`           | `None`    | override URL ของ WebSocket สาธารณะ                               |
| `ws_private_url`          | `None`    | override URL ของ WebSocket แบบ private                           |
| `ws_l3_url`               | `None`    | override URL ของ WebSocket L3 ของ Spot                           |
| `validate_l3_checksum`    | `True`    | ตรวจสอบ checksum ของ L3 ของ Kraken Spot และ resync เมื่อไม่ตรงกัน     |
| `validate_l2_checksum`    | `True`    | ตรวจสอบ checksum ของ `book` L2 ของ Kraken Spot และ resync เมื่อไม่ตรงกัน |
| `proxy_url`               | `None`    | proxy URL แบบ optional สำหรับ transport ของ HTTP และ WebSocket      |
| `timeout_secs`            | `30`      | timeout ของ HTTP request หน่วยวินาที                               |
| `heartbeat_interval_secs` | `30`      | interval ของ heartbeat ของ WebSocket หน่วยวินาที                    |
| `ws_idle_timeout_ms`      | `10,000`  | timeout เมื่อไม่มีข้อมูลสำหรับ WebSocket v2 ของ Spot; `0` คือปิด          |
| `max_requests_per_second` | `None`    | การจำกัด request ต่อ client; ค่าเริ่มต้นคือ 5 req/s                  |
| `transport_backend`       | `Sockudo` | transport backend ของ WebSocket                                   |

### ตัวเลือก config ของ execution client {#execution-client-configuration-options}

| ตัวเลือก                          | ค่าเริ่มต้น  | คำอธิบาย                                                                |
| ------------------------------- | --------- | --------------------------------------------------------------------- |
| `account_id`                    | จำเป็น     | account ID ของ account Kraken                                         |
| `api_key`                       | จำเป็น     | API key ของ Kraken                                                    |
| `api_secret`                    | จำเป็น     | API secret ของ Kraken                                                 |
| `product_type`                  | `SPOT`    | ประเภทผลิตภัณฑ์ของ client นี้ (`SPOT` หรือ `FUTURES`)                     |
| `environment`                   | `LIVE`    | environment การเทรด (`LIVE` หรือ `DEMO`); demo ใช้ได้เฉพาะ Futures       |
| `base_url`                      | `None`    | override base URL ของ REST ของ Kraken                                  |
| `ws_url`                        | `None`    | override URL ของ WebSocket ของ Kraken                                  |
| `proxy_url`                     | `None`    | proxy URL แบบ optional สำหรับ transport ของ HTTP และ WebSocket            |
| `timeout_secs`                  | `30`      | timeout ของ HTTP request หน่วยวินาที                                     |
| `heartbeat_interval_secs`       | `30`      | interval ของ heartbeat ของ WebSocket หน่วยวินาที                          |
| `auth_timeout_secs`             | `None`    | timeout ของ auth ของ WebSocket ฝั่ง Futures; `None` ใช้ค่าเริ่มต้นของ client |
| `max_requests_per_second`       | `None`    | การจำกัด request ต่อ client; ค่าเริ่มต้นคือ 5 req/s                        |
| `max_retries`                   | `3`       | จำนวนครั้งสูงสุดที่ retry สำหรับ REST request ที่ retry ได้                    |
| `spot_account_type`             | `CASH`    | ประเภท account สำหรับการเทรด spot; `MARGIN` เปิด leverage และ report     |
| `default_leverage`              | `None`    | leverage เริ่มต้นของ Spot margin; REST ส่ง `"N:1"` เมื่อตั้งค่าไว้           |
| `use_spot_position_reports`     | `False`   | รายงานยอดเงินใน wallet เป็น position; เฉพาะโหมด cash                    |
| `spot_positions_quote_currency` | `"USDT"`  | ตัวกรอง quote สำหรับ position report จาก wallet ของ spot และการครอบคลุม   |
| `margin_balance_asset`          | `None`    | asset สรุปสำหรับ `TradeBalance`; `None` จะใช้ `ZUSD` เป็นค่าเริ่มต้น          |
| `use_ws_trade`                  | `True`    | ใช้ Spot WebSocket v2 สำหรับการดำเนินการกับ order เมื่อ active              |
| `ws_request_timeout_secs`       | `5`       | timeout ของ response ของ order ผ่าน Spot WebSocket                       |
| `transport_backend`             | `Sockudo` | transport backend ของ WebSocket                                         |

สำหรับ Spot margin `default_leverage` จะมีผลเมื่อ order ไม่มีพารามิเตอร์ leverage
ราย order ดูรายละเอียดการส่ง order และสถานะ account ได้ที่[การเทรดแบบ Spot margin](#spot-margin-trading)

### การตั้งค่า demo environment {#demo-environment-setup}

หากต้องการทดสอบกับ Kraken Futures demo (paper trading):

1. สมัครที่ [Kraken Futures demo](https://demo-futures.kraken.com)
   และสร้าง API credential
1. ตั้ง environment variable ด้วย credential ของ demo:
   - `KRAKEN_FUTURES_DEMO_API_KEY`
   - `KRAKEN_FUTURES_DEMO_API_SECRET`
1. อ่าน credential แล้วส่งไปยัง `KrakenExecutionClientConfig` จากนั้นตั้ง
   `environment=KrakenEnvironment.DEMO` และ
   `product_type=KrakenProductType.FUTURES`

ดูการตั้งค่าฝั่ง Rust ได้ที่[ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/kraken/examples/)
และ [Run Rust live trading](../how_to/run_rust_live_trading.md)

### การตั้งค่าสำหรับ production {#production-configuration}

ใช้ `KrakenDataClientConfig` คู่กับ `KrakenDataClientFactory` และใช้
`KrakenExecutionClientConfig` คู่กับ `KrakenExecutionClientFactory` แล้วลงทะเบียน data client และ
execution client ผ่าน `LiveNode.builder(...)`

### API credential {#api-credentials}

config object ของ live node ไม่ได้อ่าน environment variable ของ credential
โดยอัตโนมัติ ให้ส่ง `api_key` และ `api_secret` อย่างชัดเจนไปยัง
`KrakenExecutionClientConfig` และสำหรับข้อมูล L3 ของ Spot ให้ส่งไปยัง
`KrakenDataClientConfig` ด้วย market data อื่นนอกจาก Spot L3 ไม่ต้องใช้ credential

แอปพลิเคชัน Rust สามารถใช้
`KrakenCredential::from_env_spot()` หรือ
`KrakenCredential::from_env_futures(demo)` เพื่อโหลดตัวแปรต่อไปนี้ก่อนสร้าง
config ของ live node

| Environment variable             | คำอธิบาย                                       |
| -------------------------------- | -------------------------------------------- |
| `KRAKEN_SPOT_API_KEY`            | API key สำหรับ live trading ของ Kraken Spot     |
| `KRAKEN_SPOT_API_SECRET`         | API secret สำหรับ live trading ของ Kraken Spot  |
| `KRAKEN_FUTURES_API_KEY`         | API key แบบ live ของ Kraken Futures            |
| `KRAKEN_FUTURES_API_SECRET`      | API secret แบบ live ของ Kraken Futures         |
| `KRAKEN_FUTURES_DEMO_API_KEY`    | API key สำหรับ Kraken Futures (demo)           |
| `KRAKEN_FUTURES_DEMO_API_SECRET` | API secret สำหรับ Kraken Futures (demo)        |

:::note
**Demo environment**: มีเพียง Kraken Futures ที่มี demo environment
(`https://demo-futures.kraken.com`) สำหรับทดสอบโดยไม่ใช้เงินจริง Kraken Spot
ไม่มี demo หรือ testnet environment
:::

:::tip
ใช้ environment variable เพื่อเก็บ credential แล้วส่งค่าของมันเข้าไปใน
config ของ live node ที่ขอบเขตของแอปพลิเคชัน
:::

error ของการยืนยันตัวตนจะถูกรายงานเมื่อ private client เชื่อมต่อหรือดำเนินการ
แบบ private สิทธิ์ที่ต้องใช้ขึ้นอยู่กับข้อมูลหรือการดำเนินการเทรด
ที่ร้องขอ

## การมีส่วนร่วม {#contributing}

:::info
หากต้องการมีส่วนร่วมพัฒนา adapter ของ Kraken ดู
[คู่มือการมีส่วนร่วม](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
