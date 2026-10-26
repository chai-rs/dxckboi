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
