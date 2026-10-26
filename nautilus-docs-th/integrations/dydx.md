# dYdX

dYdX เป็นหนึ่งใน exchange cryptocurrency แบบกระจายศูนย์ (decentralized) ที่ใหญ่ที่สุดสำหรับผลิตภัณฑ์ crypto derivatives
integration นี้รองรับการรับ market data แบบ live และการส่ง order กับ dYdX v4 ซึ่งรันบน blockchain เฉพาะแอปพลิเคชัน
ของตัวเองที่สร้างด้วย Cosmos SDK (dYdX Chain) และใช้ consensus แบบ CometBFT order book และ matching engine
ทำงานบน chain เป็นส่วนหนึ่งของ process ของ validator order ถูกส่งเป็น Cosmos transaction ผ่าน gRPC และ settle
ในทุก block ส่วน service ชื่อ Indexer เปิด REST และ WebSocket API สำหรับ market data และสถานะของ account

## ตัวอย่าง {#examples}

- [ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/dydx/examples/)

## ภาพรวม {#overview}

adapter นี้ implement ด้วย Rust และเชื่อมต่อโดยตรงกับ Indexer API ของ dYdX (REST/WebSocket) สำหรับ market data
และใช้ gRPC สำหรับส่ง transaction ของ Cosmos SDK โดยไม่ต้องใช้ client library ภายนอก

### ผลิตภัณฑ์ที่รองรับ {#product-support}

| Product Type      | Data Feed | Trading | หมายเหตุ                                                     |
| ----------------- | --------- | ------- | ---------------------------------------------------------- |
| Perpetual Futures | ✓         | ✓       | perpetual ทั้งหมด settle ด้วย USDC                            |
| Spot              | -         | -       | dYdX มี spot บน Solana แต่ adapter นี้ไม่รองรับ                   |
| Options           | -         | -       | *ไม่มีบน dYdX*                                               |

:::note
adapter นี้รองรับเฉพาะ perpetual futures ทุกตลาด quote เป็น USD และ settle เป็น USDC
:::

## สถาปัตยกรรมของ chain {#chain-architecture}

ต่างจาก exchange แบบรวมศูนย์ (CEX) ที่เปิด REST/WebSocket API เพียงชุดเดียว dYdX v4 รันบน
**blockchain เฉพาะแอปพลิเคชันที่สร้างด้วย Cosmos SDK** ของตัวเอง หมายความว่าทุก trade เป็น Cosmos transaction
ที่ต้องผ่าน consensus และ adapter ต้องจัดการ sequence, gas และการหมดอายุที่อิงตาม block height

### ชั้นของ transport {#transport-layers}

adapter สื่อสารผ่าน transport สามชั้นที่แยกจากกัน:

```
                         ┌─────────────────────────────────────────────┐
                         │              dYdX v4 Chain                  │
                         │                                             │
 ┌──────────┐  HTTP      │   ┌──────────────────────┐                  │
 │          │───────────►│   │  Indexer (read-only) │                  │
 │          │  WebSocket │   │  - REST API          │                  │
 │ Nautilus │───────────►│   │  - Streaming API     │                  │
 │ Adapter  │            │   └──────────────────────┘                  │
 │          │  gRPC      │   ┌──────────────────────┐                  │
 │          │───────────►│   │  Validator (write)   │                  │
 └──────────┘            │   │  - Cosmos Tx submit  │                  │
                         │   │  - Sequence mgmt     │                  │
                         │   └──────────────────────┘                  │
                         └─────────────────────────────────────────────┘
```

| Layer     | Target    | Direction | จุดประสงค์                                              |
| --------- | --------- | --------- | ---------------------------------------------------- |
| HTTP      | Indexer   | Read-only | metadata ของ instrument, ข้อมูลในอดีต, สถานะ account      |
| WebSocket | Indexer   | Read-only | market data แบบ real-time, update ของ order/fill/position |
| gRPC      | Validator | Write     | การวาง order, การยกเลิก และการทำงานแบบ batch              |

### การ settle ตาม block {#block-based-settlement}

trade จะ settle เมื่อ block ถูก commit และ short-term order หมดอายุตาม block height ไม่ใช่ตามเวลาจริง
adapter ติดตาม block height และ timestamp จาก WebSocket ในหน้าต่างแบบเลื่อนขนาด 100 block แล้วประมาณค่า
`seconds_per_block` จากหน้าต่างนี้ จากนั้นใช้ค่าประมาณนี้แปลงเวลาหมดอายุของ order ให้เป็น offset ของ block height

จนกว่าจะเก็บตัวอย่าง block ได้ครบห้าตัว ค่าประมาณจะ fallback ไปใช้ **500 ms** ต่อ block ขณะที่เวลา block
ที่สังเกตได้บน mainnet ใกล้เคียงหนึ่งวินาที ดังนั้นค่า fallback จะประเมินหน้าต่าง short-term ต่ำกว่าความจริง
ทำให้ order ที่อยู่ก้ำกึ่งถูก route ไปทาง long-term แทนที่จะเป็นในทางกลับกัน

## สถาปัตยกรรม {#architecture}

adapter ของ dYdX v4 ประกอบด้วยหลาย component ที่ใช้ร่วมกันหรือแยกกันได้:

- `DydxHttpClient`: HTTP client สำหรับ query ผ่าน Indexer REST API
- `DydxWebSocketClient`: WebSocket client สำหรับผู้เรียกฝั่ง Rust
- `DydxGrpcClient`: gRPC client สำหรับส่ง transaction ของ Cosmos SDK
- `InstrumentCache`: การ parse และโหลด instrument ใช้ร่วมกันโดย HTTP, WebSocket และ execution client
- `DydxDataClient`: ตัวจัดการ market data feed
- `DydxExecutionClient`: gateway สำหรับจัดการ account และ execute การเทรด
- `DydxDataClientFactory`: factory สำหรับ data client ของ dYdX v4 (ใช้โดย builder ของ trading node)
- `DydxExecutionClientFactory`: factory สำหรับ execution client ของ dYdX v4 (ใช้โดย builder ของ trading node)

:::note
ผู้ใช้ส่วนใหญ่จะตั้งค่า live trading node ตามที่แสดงด้านล่าง และไม่จำเป็นต้องใช้ component ระดับล่างเหล่านี้โดยตรง
:::

:::warning[การ activate account ครั้งแรก]
account สำหรับเทรดของ dYdX v4 (sub-account 0) จะถูกสร้างขึ้นหลังจากการฝากเงินหรือการเทรดครั้งแรกของ wallet เท่านั้น
ก่อนหน้านั้น การ query account ผ่าน gRPC และ Indexer จะได้ผลเป็น not-found ทำให้ `DydxExecutionClient.connect()`
ล้มเหลวระหว่าง initialize transaction sequence

ก่อนเริ่ม `LiveNode` ให้ส่ง USDC หรือ collateral อื่นที่รองรับจำนวนเท่าใดก็ได้ที่มากกว่าศูนย์ จาก wallet เดียวกัน
บน network เดียวกัน (mainnet/testnet) เมื่อ transaction finalize แล้ว (ไม่กี่ block) ให้ restart node
แล้ว client จะเชื่อมต่อได้ตามปกติ
:::

## การแก้ปัญหา {#troubleshooting}

### gRPC `NotFound` ตอน connect {#grpc-notfound-on-connect}

**สาเหตุ**: wallet/sub-account ยังไม่เคยได้รับเงินฝาก จึงยังไม่มีอยู่บน chain

**วิธีแก้**:

1. ฝาก USDC จำนวนใดก็ได้ที่มากกว่าศูนย์เข้า sub-account 0 บน network ที่ถูกต้อง
2. รอ finality (ประมาณ 30 วินาทีบน mainnet และนานกว่านั้นบน testnet)
3. restart `LiveNode` แล้วการเชื่อมต่อควรสำเร็จ

:::tip
สำหรับการ deploy ที่ไม่มีคนดูแล ให้ครอบการเรียก `connect()` ด้วย loop แบบ exponential backoff
เพื่อให้ client retry จนกว่าเงินฝากจะปรากฏ
:::

## Symbology {#symbology}

dYdX ใช้รูปแบบ symbol เฉพาะสำหรับสัญญา perpetual futures

### รูปแบบ symbol {#symbol-format}

รูปแบบ: `{Base}-USD-PERP`

perpetual ทั้งหมดบน dYdX:

- quote เป็น USD
- settle เป็น USDC
- ใช้ suffix ของ venue เป็น `.DYDX` ใน Nautilus

ตัวอย่าง:

- `BTC-USD-PERP.DYDX` - Bitcoin perpetual futures
- `ETH-USD-PERP.DYDX` - Ethereum perpetual futures
- `SOL-USD-PERP.DYDX` - Solana perpetual futures

:::info
ticker ของ perpetual ใน dYdX Indexer คือ `{Base}-USD` (เช่น `BTC-USD`) adapter เติม suffix `-PERP`
เพื่อให้สอดคล้องกับ adapter อื่น และเผื่อไว้สำหรับผลิตภัณฑ์ประเภทอื่นในอนาคต
:::

## ความสามารถด้าน order {#orders-capability}

dYdX รองรับการเทรด perpetual futures ด้วยประเภท order และฟีเจอร์การ execute ครบชุด adapter จะจัดประเภท
แต่ละ order เป็น short-term, long-term หรือ conditional ให้อัตโนมัติจากประเภท, time-in-force และวันหมดอายุ
จึงไม่ต้องติด tag เอง

### ประเภท order {#order-types}

| Order Type             | Perpetuals | หมายเหตุ                                             |
| ---------------------- | ---------- | -------------------------------------------------- |
| `MARKET`               | ✓          | execute ทันทีที่ราคาดีที่สุดที่มีอยู่                         |
| `LIMIT`                | ✓          |                                                    |
| `STOP_MARKET`          | ✓          | conditional order แบบ stop-loss เป็น stateful เสมอ   |
| `STOP_LIMIT`           | ✓          | conditional order เป็น stateful เสมอ                |
| `MARKET_IF_TOUCHED`    | ✓          | take-profit market order trigger เมื่อราคาแตะ         |
| `LIMIT_IF_TOUCHED`     | ✓          | take-profit limit order trigger เมื่อราคาแตะ          |
| `TRAILING_STOP_MARKET` | -          | *ไม่รองรับ*                                         |
| `TRAILING_STOP_LIMIT`  | -          | *ไม่รองรับ*                                         |

### Execution instructions {#execution-instructions}

| Instruction   | Perpetuals | หมายเหตุ                                                                                                                                                                                         |
| ------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `post_only`   | ✓          | รองรับกับ order แบบ LIMIT, STOP_LIMIT และ LIMIT_IF_TOUCHED post-only order ที่ตั้งราคาข้าม spread จะ **ถูก accept แล้วถูก cancel ทันที** โดย venue (ไม่ได้ถูก reject พร้อมเหตุผล) |
| `reduce_only` | ✓          | chain ยอมรับ **เฉพาะ order ที่ execute ทันที** (IOC) แบบอื่นจะถูก reject บน chain ด้วย `code=9003`, `Reduce-only is currently disabled for non-IOC orders`                          |

วิธีที่ adapter จัดการ flag นี้ขึ้นอยู่กับประเภท order:

| Order type                                | พฤติกรรมของ `reduce_only`                                                       |
| ----------------------------------------- | ------------------------------------------------------------------------------ |
| `MARKET`                                  | ส่งต่อเป็น IOC order                                                             |
| `LIMIT`, `STOP_LIMIT`, `LIMIT_IF_TOUCHED` | ส่งต่อพร้อม time in force ที่ระบุ ต้องใช้ `IOC` มิฉะนั้น chain จะ reject                |
| `STOP_MARKET`, `MARKET_IF_TOUCHED`        | ส่งต่อไป แต่ order เหล่านี้ไม่มี time in force จึงถูก chain reject เสมอ               |

ตั้ง `reduce_only` กับ market order หรือใช้ร่วมกับ `IOC` บน limit order ประเภทที่รองรับ

### ตัวเลือก time in force {#time-in-force-options}

| Time in force | Perpetuals | หมายเหตุ                                                                                                                                                                |
| ------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `GTC`         | ✓          | Good Till Canceled                                                                                                                                                   |
| `GTD`         | ✓          | Good Till Date venue รายงานการหมดอายุเป็น cancel event และ adapter จะ map เป็น `OrderExpired` (ไม่ใช่ `OrderCanceled`) เมื่อเลย `expire_time` ของ order แล้ว |
| `IOC`         | ✓          | Immediate or Cancel                                                                                                                                                  |
| `FOK`         | -          | *ถูกยกเลิกการใช้งานใน dYdX v4* chain จะ reject FOK order ด้วย `code=48` adapter จึงสร้าง `OrderDenied` ในเครื่องและไม่ broadcast                               |
| `DAY`         | -          | *ไม่รองรับ* adapter สร้าง `OrderDenied` ในเครื่องและไม่ broadcast                                                                                                   |

### ฟีเจอร์ order ขั้นสูง {#advanced-order-features}

| Feature            | Perpetuals | หมายเหตุ                                                                                                                                                                                    |
| ------------------ | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Order modification | -          | ไม่รองรับ dYdX รองรับการ [replacement](https://docs.dydx.xyz/concepts/trading/limit-orderbook#replacements) ของ short-term order (ID เดิม, GTB สูงขึ้น) แต่ยังไม่ได้เปิดใช้เป็น `ModifyOrder` |
| Bracket/OCO orders | -          | *ไม่รองรับ*                                                                                                                                                                                |
| Iceberg orders     | -          | *ไม่รองรับ*                                                                                                                                                                                |

### การทำงานแบบ batch {#batch-operations}

| Operation    | Perpetuals | หมายเหตุ                                                                                                              |
| ------------ | ---------- | ---------------------------------------------------------------------------------------------------------------------- |
| Batch submit | ✓          | รองรับสำหรับ `LIMIT` order แบบ long-term ส่วน short-term order จะส่งทีละตัว                                              |
| Batch modify | -          | *ไม่รองรับ*                                                                                                             |
| Batch cancel | ✓          | แบ่งกลุ่ม: short-term order ใช้ `MsgBatchCancel` (เรียก gRPC ครั้งเดียว) ส่วน long-term order ใช้ `MsgCancelOrder` แบบ batch |

### การจัดการ position {#position-management}

| Feature          | Perpetuals | หมายเหตุ                                                          |
| ---------------- | ---------- | --------------------------------------------------------------- |
| Query positions  | ✓          | update ของ position แบบ real-time                                |
| Position mode    | -          | netting เท่านั้น (ดูด้านล่าง)                                       |
| Leverage control | -          | กำหนดโดย margin fraction ของแต่ละตลาด override ราย account ไม่ได้   |
| Margin mode      | -          | cross margin เท่านั้น                                              |

:::note
dYdX ทำ netting ของ position (หนึ่ง position ต่อ instrument) ในระดับ venue ดังนั้น adapter จึงทำงานใน
โหมด `NETTING` เท่านั้น
:::

### การ query order {#order-querying}

| Feature              | Perpetuals | หมายเหตุ                          |
| -------------------- | ---------- | ------------------------------ |
| Query open orders    | ✓          | แสดงรายการ order ที่ active ทั้งหมด |
| Query order history  | ✓          | ข้อมูล order ในอดีต                |
| Order status updates | ✓          | การเปลี่ยนสถานะ order แบบ real-time |
| Trade history        | ✓          | report ของการ execute และ fill    |

### Contingent orders {#contingent-orders}

| Feature            | Perpetuals | หมายเหตุ                                            |
| ------------------ | ---------- | ------------------------------------------------ |
| Order lists        | -          | *ไม่รองรับ*                                       |
| OCO orders         | -          | *ไม่รองรับ*                                       |
| Bracket orders     | -          | *ไม่รองรับ*                                       |
| Conditional orders | ✓          | stop, take-profit market และ take-profit limit   |

### ขีดจำกัดตาม equity tier {#equity-tier-limit}

dYdX จำกัดจำนวน **stateful order** (long-term และ conditional) ที่ subaccount หนึ่งเปิดค้างไว้ได้พร้อมกัน
โดยอิงตาม net collateral ของ subaccount ส่วน short-term order ไม่ถูกนับรวมในขีดจำกัดนี้ การส่ง order เกินขีดจำกัด
จะถูก reject บน chain ด้วย `code=10001` และ log message ในรูปแบบ
`Opening order would exceed equity tier limit of N` ให้ cancel stateful order ที่มีอยู่ก่อนวางเพิ่ม
หรือแบ่ง strategy ไปไว้หลาย subaccount

| Net collateral      | จำนวน stateful order ที่เปิดได้สูงสุด |
| ------------------- | ---------------------------- |
| ต่ำกว่า $20           | 0                            |
| $20 ถึง $100         | 10                           |
| $100 ถึง $1,000      | 20                           |
| $1,000 ถึง $10,000   | 40                           |
| $10,000 ถึง $100,000 | 100                          |
| $100,000 ขึ้นไป       | 200                          |

tier เหล่านี้ปรับได้ผ่าน governance query ค่าปัจจุบันได้จาก endpoint `/dydxprotocol/clob/equity_tier`
ของ node หรือดู [equity tier limits](https://docs.dydx.xyz/concepts/trading/limits/equity-tier-limits)

### การแปลงไปกลับของ MIT และ LIT {#mit-and-lit-round-tripping}

protocol ของ dYdX ใช้ order type `TAKE_PROFIT` แบบเดียวที่มีราคา (`subticks`) และราคา trigger ส่วนจะทำงานแบบ
market-on-trigger หรือ limit-on-trigger นั้นแฝงอยู่ในราคา adapter ส่ง `MARKET_IF_TOUCHED` ของ Nautilus
เป็น take-profit ที่ตั้งราคาเป็นกรณีแย่ที่สุดแบบ pay-through 5% และส่ง `LIMIT_IF_TOUCHED` เป็น take-profit
ที่ราคา limit ของผู้ใช้ Indexer คืนทั้งสองแบบมาเป็น `"type":"TAKE_PROFIT"`

ตอน reconciliation adapter จะกู้ order type เดิมของ Nautilus จากระยะห่างระหว่างราคาที่รายงานกับราคา trigger
ความต่าง **2% ขึ้นไป** หมายความว่าราคามาจาก buffer แบบ pay-through 5% order จึงถูก reconcile เป็น
`MARKET_IF_TOUCHED` หากใกล้กว่านั้นจะถือว่าเป็นราคา limit ที่ผู้ใช้เลือกและ reconcile เป็น `LIMIT_IF_TOUCHED`
threshold 2% แยกช่วง pay-through ออกจาก offset ของ take-profit limit ทั่วไปซึ่งมักต่ำกว่า 1% มาก

### การจัดการ liquidation และ ADL (deleveraging) {#liquidation-and-adl-deleveraging-handling}

dYdX v4 ใช้กลไกความเสี่ยงสองขั้นตามลำดับ:

1. **Liquidation** ทำงานเมื่อ account ตกต่ำกว่า maintenance margin
   position จะถูกปิดกับ insurance fund ภายใน spread ที่จำกัดจากราคา oracle
2. **Deleveraging (ADL)** ทำงานเมื่อ liquidation ไม่สามารถฟื้นระดับ collateral ได้เต็มที่
   หรือเมื่อราคา oracle กระโดดมากจนทำให้ account ติดลบในขั้นเดียว deleveraging จะปิด position
   ที่ collateral ไม่พอกับ account ฝั่งตรงข้ามที่สุ่มเลือก

Indexer ระบุการจัดประเภทผ่าน field `type` ในแต่ละ record ของ `Fill`:

| `type`        | ความหมาย                                              |
| ------------- | -------------------------------------------------- |
| `LIMIT`       | fill ปกติ                                            |
| `LIQUIDATED`  | ฝั่ง taker ของ liquidation (collateral ไม่พอ)          |
| `LIQUIDATION` | ฝั่ง maker ของ liquidation (insurance fund)          |
| `DELEVERAGED` | ฝั่ง taker ของ deleveraging (การปิดแบบ ADL)             |
| `OFFSETTING`  | ฝั่ง maker ของ deleveraging (account ฝั่งตรงข้าม)        |

ค่าอื่นใดที่ venue เพิ่มเข้ามาจะถูก decode เป็น fill type ที่ไม่รู้จักและจัดการเหมือน fill ปกติ
ดังนั้นการจัดประเภทใหม่จะไม่ทำให้ fill หายไป

adapter จะ log warning พร้อม instrument, side, size และราคาสำหรับ fill แต่ละรายการที่เป็น liquidation /
deleveraging จากนั้นส่ง `FillReport` ออกไปตามเส้นทางปกติ position ที่ venue รายงานว่าเป็น `LIQUIDATED`
จะถือว่าปิดแล้ว ซึ่งจะปิด position report ที่สอดคล้องกันด้วย

เอกสารอ้างอิงจากต้นทาง:

- [Liquidations](https://docs.dydx.xyz/concepts/trading/liquidations)
- [Contract loss mechanisms (deleveraging)](https://help.dydx.trade/en/articles/166973-contract-loss-mechanisms-on-dydx-chain)

### การจัดประเภท order {#order-classification}

dYdX จัดทุก order ไว้ในหนึ่งในสามหมวดบน chain adapter จะกำหนดหมวดให้อัตโนมัติตาม time-in-force และวันหมดอายุ
จึงไม่ต้องตั้งค่าเอง

| หมวด        | ที่วาง      | การหมดอายุ        | การใช้งานทั่วไป                                         |
| ----------- | --------- | --------------- | ------------------------------------------------- |
| Short-term  | In-memory | Block height    | IOC หรือ order ที่หมดอายุภายใน 40 block              |
| Long-term   | On-chain  | Timestamp (UTC) | GTC/GTD ที่หมดอายุเกินหน้าต่าง short-term             |
| Conditional | On-chain  | Timestamp (UTC) | trigger แบบ stop-loss และ take-profit              |

ในระดับ protocol **order ทุกตัวของ dYdX เป็น limit order** order type `MARKET` เป็นความสะดวกที่ Nautilus
เพิ่มให้ โดย adapter implement เป็น IOC limit order แบบ aggressive ที่ตั้งราคาทะลุ book ไปมาก หมายความว่า
market order มี lifecycle `Submitted > Accepted > Filled` เหมือน limit order (คาดว่าจะได้ event `OrderAccepted`
ก่อน fill)

ดู [เอกสาร order ของ dYdX](https://docs.dydx.xyz/concepts/trading/orders) สำหรับรายละเอียดระดับ protocol
ทั้งหมดของกลไก short-term เทียบกับ stateful order

#### Short-term orders {#short-term-orders}

short-term order อยู่ **ในหน่วยความจำของ validator เท่านั้น** และหมดอายุตาม block height `ShortBlockWindow`
ของ protocol จำกัดอายุไว้ไม่เกิน **40 block** จาก height ปัจจุบัน เป็น order ที่เร็วที่สุดบน dYdX
เพราะไม่ต้องเก็บบน chain

**คุณสมบัติ**:

- **IOC (และ FOK ที่ถูกยกเลิกการใช้งานแล้ว) เป็น short-term เสมอ** ไม่ว่า parameter อื่นจะเป็นอย่างไร
- **GTD order** จะถูกจัดเป็น short-term อัตโนมัติเมื่อวันหมดอายุอยู่ภายในหน้าต่าง short-term แบบ dynamic
  (`40 blocks × seconds_per_block`)
- ใช้ Good-Til-Block (GTB) ป้องกัน replay แทน sequence ของ Cosmos SDK
- broadcast **พร้อมกันได้** (ไม่มี semaphore ใช้ sequence ที่ cache ไว้)
- หมดอายุได้โดยไม่ต้องมี cancel transaction indexer จะรายงานการหมดอายุแต่ละครั้งเป็น cancel ซึ่ง adapter
  ส่งออกเป็น `OrderCanceled`
- รวมเป็น batch ใน transaction เดียวไม่ได้ (`MsgPlaceOrder` หนึ่งตัวต่อหนึ่ง tx)

#### Long-term orders {#long-term-orders}

long-term order (แบบ stateful) **ถูกเก็บบน chain** และหมดอายุตาม timestamp แบบ UTC จะสร้าง cancel event
ที่ชัดเจนเมื่อหมดอายุหรือถูก cancel

**คุณสมบัติ**:

- order แบบ **GTC** มีวันหมดอายุเริ่มต้น 90 วัน (ขีดจำกัดของ protocol คือ 95 วัน)
- order แบบ **GTD** ใช้ timestamp วันหมดอายุที่ผู้ใช้ระบุ
- ต้องจัดการ sequence ของ Cosmos SDK อย่างถูกต้อง (เรียงลำดับผ่าน semaphore)
- ต้อง broadcast **ทีละตัวตามลำดับ** โดยเลข sequence เพิ่มขึ้นเรื่อย ๆ
- รวมเป็น batch ใน transaction เดียวได้

#### Conditional orders {#conditional-orders}

conditional order (stop-loss, take-profit) **ถูกเก็บบน chain เสมอ** และถูก trigger ด้วยเงื่อนไขราคาบน validator

**คุณสมบัติ**:

- ใช้การหมดอายุแบบ timestamp เสมอ (ค่าเริ่มต้น 90 วันสำหรับ GTC ขีดจำกัดของ protocol 95 วัน)
- ใช้เส้นทาง broadcast แบบ long-term เสมอ (เรียงลำดับด้วย semaphore)
- รวมถึง `StopMarket`, `StopLimit`, `TakeProfitMarket` และ `TakeProfitLimit`

#### การ route อัตโนมัติ {#automatic-routing}

adapter กำหนดอายุของ order อัตโนมัติจากเวลา block ที่ประมาณได้:

```
max_short_term_secs = 40 blocks (ShortBlockWindow) × seconds_per_block
```

หากเวลาที่เหลือก่อนหมดอายุของ order อยู่ภายใน `max_short_term_secs` จะถูก route เป็น short-term
มิฉะนั้นจะถูก route เป็น long-term โดยไม่ต้องตั้งค่าเอง

#### การ implement MARKET order {#market-order-implementation}

dYdX ไม่มี market order แบบ native adapter จึง implement `MARKET` order เป็น **IOC limit order**
แบบ aggressive ที่ตั้งราคาไว้ที่:

- **Buy**: `oracle_price × (1 + 0.05)` (สูงกว่า oracle 5%)
- **Sell**: `oracle_price × (1 - 0.05)` (ต่ำกว่า oracle 5%)

buffer slippage 5% นี้ (`DEFAULT_MARKET_ORDER_SLIPPAGE = 0.05`) กำหนดราคากรณีแย่ที่สุด (เรียกว่า "pay-through price")
เนื่องจาก order เป็น IOC slippage ส่วนที่ไม่ได้ fill จะไม่ถูกใช้ buffer ถูกตั้งให้กว้างโดยเจตนาเพื่อเพิ่มโอกาส fill
ให้มากที่สุดในสภาวะตลาดผันผวน

### การ encode client order ID {#client-order-id-encoding}

dYdX ต้องการ client ID แบบ `u32` บน chain แต่ Nautilus ใช้ค่า `ClientOrderId` แบบ string
(เช่น `O-20260220-031943-001-000-51`) adapter จะ encode ค่าเหล่านี้แบบสองทิศทาง เพื่อให้ reconcile order
ข้ามการ restart ได้โดยไม่ต้อง persist สถานะ

สำหรับรูปแบบ O มาตรฐาน (`O-YYYYMMDD-HHMMSS-TTT-SSS-CCC`) การ encode เป็นแบบ deterministic:

| dYdX field        | Bits | เนื้อหา                                              |
| ----------------- | ---- | -------------------------------------------------- |
| `client_id`       | 32   | `[trader:10][strategy:10][count:12]` (key ที่ไม่ซ้ำ) |
| `client_metadata` | 32   | จำนวนวินาทีนับจาก 2020-01-01 UTC (timestamp)         |

เพราะการ encode เป็นแบบ deterministic adapter จึง decode order ที่ reconcile แล้วกลับเป็น string `ClientOrderId`
เดิมได้โดยไม่ต้องใช้ database หรือไฟล์ mapping

`ClientOrderId` ที่เป็นตัวเลขล้วนก็เป็นแบบ deterministic เช่นกัน ตัวเลขนั้นจะกลายเป็น `client_id` และ
`client_metadata` ถูกตั้งเป็น marker คงที่ จึง decode ข้ามการ restart ได้เช่นกัน รูปแบบอื่นนอกจากนี้จะ fallback
ไปใช้การจัดสรรแบบเรียงลำดับพร้อม reverse map ในหน่วยความจำ และ ID เหล่านั้นจะ decode ได้ภายใน session เดียวกันเท่านั้น

#### การป้องกันการชนกันหลัง restart {#restart-collision-prevention}

เมื่อ restart Nautilus จะ reset ตัวนับ order ภายในตามจำนวน order ที่ reconcile ได้ ซึ่งอาจต่ำกว่าค่าตัวนับสูงสุด
ที่ใช้ใน session ก่อน (เช่น เมื่อ order บางตัวหมดอายุไปจาก response ของ API แล้ว) ทำให้ order ใหม่อาจได้
`client_id` เดียวกับ order จาก session ก่อน และเกิด venue order UUID ซ้ำกัน

adapter ป้องกันปัญหานี้โดยลงทะเบียน `client_id` ทุกตัวที่เห็นระหว่าง reconciliation หากการ encode รูปแบบ O
ใหม่ได้ `client_id` ที่เคยใช้แล้ว encoder จะ log warning แล้ว fallback ไปใช้การจัดสรรแบบเรียงลำดับ
ซึ่งก็จะข้ามค่าที่ลงทะเบียนไว้ทั้งหมดเช่นกัน

:::note
การป้องกันนี้ทำงานอัตโนมัติโดยไม่ต้องตั้งค่า log warning
`[ENCODER] client_id ... collides with reconciled order` เป็นเพียงข้อมูล order จะยังถูกส่งสำเร็จ
ด้วย ID ทางเลือก
:::

## กลยุทธ์การ broadcast และ retry {#broadcasting-and-retry-strategy}

### Short-term broadcast {#short-term-broadcast}

short-term order ใช้ Good-Til-Block (GTB) ป้องกัน replay ante handler `ClobDecorator` ของ chain
จะข้ามการตรวจสอบ sequence ของ Cosmos SDK สำหรับ message แบบ short-term ดังนั้น:

- **ไม่มี semaphore**: broadcast พร้อมกันได้เต็มที่
- **Cached sequence**: ไม่ต้องเพิ่มค่าหรือจัดสรร
- **ไม่มี retry**: หาก broadcast ล้มเหลวจะล้มเหลวทันที
- error ของการ cancel ที่ไม่เป็นอันตรายจะถือว่าสำเร็จ (ดูด้านล่าง)

### Long-term broadcast {#long-term-broadcast}

long-term และ conditional order ต้องจัดการ sequence ของ Cosmos SDK อย่างถูกต้อง:

- **Semaphore** ที่มี 1 permit ทำให้ long-term broadcast ทั้งหมดเรียงลำดับกัน
- **Exponential backoff**: 500ms -> 1s -> 2s -> 4s (retry สูงสุด 5 ครั้ง)
- **งบเวลารวม 10 วินาที** ป้องกัน loop การ retry ไม่รู้จบ
- เมื่อ sequence ไม่ตรงกัน จะ **resync sequence จาก chain** ก่อน retry
- ความล้มเหลวชั่วคราวของ gRPC (unavailable, deadline exceeded, resource exhausted) ก็จะ resync ก่อน retry
  เช่นกัน เพื่อไม่ให้ timeout ซ้ำ ๆ ทำให้ sequence ในเครื่องเลื่อนนำหน้า chain

### การตรวจจับ sequence ที่ไม่ตรงกัน {#sequence-mismatch-detection}

| Error code | แหล่งที่มา           | ความหมาย                                            |
| ---------- | ------------------ | ------------------------------------------------ |
| `code=32`  | Cosmos SDK         | account sequence ไม่ตรงกัน                        |
| `code=104` | dYdX authenticator | การตรวจสอบ signature ล้มเหลว (เกี่ยวกับ sequence)     |

ทั้งสองกรณีจะ trigger การ resync + retry อัตโนมัติผ่าน `RetryManager`

### Error ของการ cancel ที่ไม่เป็นอันตราย {#benign-cancel-errors}

error เหล่านี้ระหว่างการ cancel แบบ short-term จะถือว่า **สำเร็จ**:

| Error code  | ความหมาย                                                            |
| ----------- | ----------------------------------------------------------------- |
| `code=19`   | transaction อยู่ใน mempool cache แล้ว (tx ซ้ำ)                     |
| `code=9`    | มี cancel อยู่ใน memclob แล้วด้วย GoodTilBlock ที่ >=                 |
| `code=3006` | order ที่จะ cancel ไม่มีอยู่ (fill/หมดอายุ/cancel ไปแล้ว)               |

### การแบ่งกลุ่ม batch cancel {#batch-cancel-partitioning}

เมื่อ cancel หลาย order adapter จะแบ่งกลุ่มตามอายุของ order:

1. **Short-term orders**: `MsgBatchCancel` ตัวเดียวผ่าน `broadcast_short_term()`
2. **Long-term orders**: message `MsgCancelOrder` แบบ batch ผ่าน `broadcast_with_retry()`

วิธีนี้ทำให้แต่ละกลุ่มใช้กลยุทธ์การ broadcast ที่เหมาะสม

## Funding rates {#funding-rates}

perpetual futures ของ dYdX ใช้รอบ funding คงที่ 1 ชั่วโมง adapter ตั้ง `interval` เป็น `60` (นาที)
ใน `FundingRateUpdate` ทุกตัว ทั้งข้อมูล funding จาก WebSocket และข้อมูลในอดีต

## Rate limiting {#rate-limiting}

### gRPC rate limiting {#grpc-rate-limiting}

adapter จำกัด rate ของการเรียก gRPC `broadcast_tx` เพื่อป้องกัน error `ResourceExhausted` (429)
จาก validator node

| Setting                      | Default | คำอธิบาย                                                               |
| ---------------------------- | ------- | --------------------------------------------------------------------- |
| `grpc_rate_limit_per_second` | `4`     | จำนวน gRPC broadcast request สูงสุดต่อวินาที ตั้งเป็น `None` เพื่อปิด       |

ค่านี้เป็น field ของ config struct

### ขีดจำกัดของผู้ให้บริการ {#provider-limits}

rate limit ที่ทราบของผู้ให้บริการ gRPC สาธารณะ:

| Provider  | Limit                |
| --------- | -------------------- |
| Polkachu  | 300 req/min (~5/s)   |
| KingNodes | 250 req/min (~4.2/s) |
| AutoStake | 4 req/s              |

ค่าเริ่มต้น 4 req/s เป็นค่าที่ระมัดระวังและใช้ได้กับผู้ให้บริการสาธารณะทุกราย

### Fallback หลาย gRPC URL {#multiple-grpc-url-fallback}

adapter จะเชื่อมต่อกับ node แรกที่เข้าถึงได้ในรายการ gRPC URL และ fallback ไปตัวถัดไปเมื่อการเชื่อมต่อล้มเหลว
เรื่องนี้สำคัญบน DEX ซึ่ง node สาธารณะแต่ละตัวอาจล่มได้โดยไม่แจ้งล่วงหน้า execution config จะ resolve รายการนี้ตามลำดับ:

1. `grpc_urls` เมื่อไม่ว่าง
2. `grpc_endpoint` เป็นรายการที่มี URL เดียว การตั้งเฉพาะ field นี้จะไม่มี fallback
3. validator node สาธารณะเริ่มต้นของ network ที่เลือก

ทั้งสอง field เป็นส่วนหนึ่งของ config struct

## Quantization ของราคาและขนาด {#price-and-size-quantization}

dYdX ใช้ quantization แบบจำนวนเต็มสำหรับราคาและขนาด adapter จัดการการแปลงทั้งหมดให้อัตโนมัติผ่าน
`OrderMessageBuilder` แต่การเข้าใจ parameter เหล่านี้ช่วยในการ debug

### Parameter ของตลาด {#market-parameters}

| Parameter                     | คำอธิบาย                                                  |
| ----------------------------- | ------------------------------------------------------- |
| `atomic_resolution`           | exponent สำหรับแปลงขนาดแบบที่คนอ่านได้เป็น quantum            |
| `quantum_conversion_exponent` | exponent สำหรับแปลง quantum เป็น token                    |
| `step_base_quantums`          | ขั้นขนาด order ต่ำสุดในหน่วย quantum                         |
| `subticks_per_tick`           | ความละเอียดของราคาภายในแต่ละ tick                            |

### การตั้งราคาของ market order {#market-order-pricing}

order ที่ส่งโดยไม่ระบุราคาจะใช้ราคา oracle พร้อม buffer slippage 5% ("pay-through price") ซึ่งครอบคลุม
`MARKET`, `STOP_MARKET` และ `MARKET_IF_TOUCHED`:

- **Buy**: `oracle_price × 1.05`
- **Sell**: `oracle_price × 0.95`

การตั้งราคา order อ่านราคา oracle จาก instrument cache Indexer เติม cache นี้ตอน client เชื่อมต่อและไม่ refresh
หลังจากนั้น ดังนั้นช่วง pay-through จะยึดกับราคา oracle ที่เห็นตอน connect ส่วน execution client ติดตามราคา
oracle แบบ live จาก markets channel แยกต่างหาก และใช้ราคาเหล่านั้นประเมินมูลค่าสถานะ account และ position
ไม่ได้ใช้ตั้งราคา order

### การจัดการอัตโนมัติ {#automatic-handling}

quantization ของราคาและขนาดทั้งหมดถูกจัดการอัตโนมัติโดย `OrderMessageBuilder`
ไม่ต้องแปลงเองเมื่อส่ง order ผ่าน Nautilus

## Data subscriptions {#data-subscriptions}

adapter รองรับ data subscription ต่อไปนี้:

| Data type            | Subscription | Historical request | หมายเหตุ                                           |
| -------------------- | ------------ | ------------------ | ------------------------------------------------ |
| Trade ticks          | ✓            | ✓                  |                                                  |
| Quote ticks          | ✓            | -                  | สังเคราะห์จาก top-of-book ของ order book          |
| Order book deltas    | ✓            | -                  | depth แบบ L2 เท่านั้น                              |
| Order book snapshots | -            | ✓                  | snapshot ครั้งเดียวผ่าน HTTP request                |
| Bars                 | ✓            | ✓                  | ดู resolution ที่รองรับด้านล่าง                       |
| Mark prices          | ✓            | -                  | ผ่าน markets channel                              |
| Index prices         | ✓            | -                  | ผ่าน markets channel                              |
| Funding rates        | ✓            | ✓                  | real-time ผ่าน markets channel ข้อมูลในอดีตผ่าน HTTP |
| Instrument status    | ✓            | -                  | ผ่าน markets channel                              |

### Precision ของ mark และ index price {#mark-and-index-price-precision}

mark และ index price ที่ได้จาก oracle ใช้ precision คงที่ 16 ตำแหน่งทศนิยมใน build แบบ high-precision
และ 9 ตำแหน่งใน build แบบ standard-precision วิธีนี้รักษาหลักของ oracle ที่ละเอียดกว่า tick ของการเทรดไว้
ได้ถึง precision ที่ build รองรับ และทำให้ metadata ด้าน precision ของ streaming catalog คงที่

ข้อความราคาจะมีศูนย์ต่อท้ายจนถึง precision นี้ ให้อ่าน recording ที่ precision 16 ด้วย build แบบ high-precision
ส่วนตัวอ่านแบบ standard-precision จะ reject metadata ด้าน precision ของ recording เหล่านั้น

### Resolution ของ bar ที่รองรับ {#supported-bar-resolutions}

| Resolution | dYdX candle |
| ---------- | ----------- |
| 1-MINUTE   | `1MIN`      |
| 5-MINUTE   | `5MINS`     |
| 15-MINUTE  | `15MINS`    |
| 30-MINUTE  | `30MINS`    |
| 1-HOUR     | `1HOUR`     |
| 4-HOUR     | `4HOURS`    |
| 1-DAY      | `1DAY`      |

## Subaccounts {#subaccounts}

dYdX รองรับหลาย subaccount ต่อหนึ่ง wallet address ทำให้แยก strategy และการบริหารความเสี่ยงได้ภายใน wallet เดียว

### แนวคิด {#concepts}

- แต่ละ wallet address มี subaccount แบบมีหมายเลขได้หลายตัว (0, 1, 2, ..., 127) หมายเลข 128 ขึ้นไป
  เป็น child subaccount แบบ isolated-margin ของ venue ซึ่ง adapter นี้ไม่รองรับ
- subaccount 0 เป็น **ค่าเริ่มต้น** และถูกสร้างอัตโนมัติเมื่อฝากเงินครั้งแรก
- แต่ละ subaccount มีของตัวเองดังนี้:
  - Positions
  - Open orders
  - Collateral balance
  - Margin requirements

### การตั้งค่า {#configuration}

ระบุหมายเลข subaccount ด้วย option `subaccount_number` ใน config ของ execution client
(ร่วมกับ `account_id` เช่น `DYDX-001`)

:::note
ผู้ใช้ส่วนใหญ่จะใช้ subaccount `0` (ค่าเริ่มต้น) ผู้ใช้ขั้นสูงสามารถตั้งค่า execution client หลายตัว
สำหรับ subaccount ต่างกัน เพื่อแยก strategy หรือแยกความเสี่ยง
:::

## การตั้งค่า testnet {#testnet-setup}

testnet ของ dYdX (`dydx-testnet-4`) เป็นแบบจำลองเต็มรูปแบบของ mainnet สำหรับทดสอบ strategy
โดยไม่เสี่ยงเงินจริง endpoint เริ่มต้นของ testnet ทั้งหมดจะถูก resolve อัตโนมัติเมื่อตั้ง
`network=DydxNetwork.TESTNET`

### 1. สร้าง testnet wallet {#1-create-a-testnet-wallet}

**ทางเลือก A: ผ่านเว็บแอป testnet ของ dYdX (ง่ายที่สุด)**

1. ไปที่ [v4.testnet.dydx.exchange](https://v4.testnet.dydx.exchange)
2. เชื่อมต่อด้วย MetaMask, Keplr, Phantom หรือ WalletConnect
3. account ของ dYdX จะถูกสร้างให้อัตโนมัติ
4. export secret phrase: คลิกที่ address ของคุณ (มุมขวาบน) แล้วเลือก "Export secret phrase"

**ทางเลือก B: ใช้ private key แบบ secp256k1 ที่มีอยู่**

private key แบบ secp256k1 ขนาด 32 byte ที่ encode เป็น hex ใช้ได้ทั้งหมด adapter จะ derive address
`dydx1...` จาก key ให้อัตโนมัติด้วยการ encode แบบ bech32 ของ Cosmos

### 2. เติมเงินให้ testnet account {#2-fund-the-testnet-account}

ต้องเติมเงินให้ subaccount ก่อน adapter จึงจะเชื่อมต่อได้ (ดู [การ activate account ครั้งแรก](#architecture))

**ผ่านเว็บแอป testnet**:

คลิกปุ่ม deposit/recharge บน [v4.testnet.dydx.exchange](https://v4.testnet.dydx.exchange)
เพื่อรับ testnet USDC อัตโนมัติ

**ผ่าน faucet API โดยตรง**:

```bash
# Fund subaccount 0 with 2000 USDC
curl -X POST https://faucet.v4testnet.dydx.exchange/faucet/tokens \
  -H "Content-Type: application/json" \
  -d '{"address": "dydx1...", "subaccountNumber": 0, "amount": 2000}'

# Fund native tokens (for gas fees)
curl -X POST https://faucet.v4testnet.dydx.exchange/faucet/native-token \
  -H "Content-Type: application/json" \
  -d '{"address": "dydx1..."}'
```

### 3. ตั้งค่า environment variable {#3-set-environment-variables}

```bash
export DYDX_TESTNET_WALLET_ADDRESS="dydx1..."
export DYDX_TESTNET_PRIVATE_KEY="0x..."  # hex-encoded, 0x prefix optional
```

### 4. ตั้งค่า trading node {#4-configure-the-trading-node}

ตั้ง `network=DydxNetwork.TESTNET` ให้ทั้ง data client และ execution client หากไม่ระบุ `wallet_address`
และ `private_key` ใน execution config จะ fallback ไปใช้ `DYDX_TESTNET_WALLET_ADDRESS` และ
`DYDX_TESTNET_PRIVATE_KEY` ตามลำดับ

### Endpoint ของ testnet {#testnet-endpoints}

adapter เลือก endpoint เริ่มต้นของ testnet ให้อัตโนมัติ:

| Service   | Default URL                                          |
| --------- | ---------------------------------------------------- |
| HTTP      | `https://indexer.v4testnet.dydx.exchange`            |
| WebSocket | `wss://indexer.v4testnet.dydx.exchange/v4/ws`        |
| gRPC      | `https://test-dydx-grpc.kingnodes.com:443` (primary) |
| Faucet    | `https://faucet.v4testnet.dydx.exchange`             |
| Web app   | `https://v4.testnet.dydx.exchange`                   |

### Endpoint ของ mainnet {#mainnet-endpoints}

adapter เลือก endpoint เริ่มต้นของ mainnet ให้อัตโนมัติ:

| Service   | Default URL                                         |
| --------- | --------------------------------------------------- |
| HTTP      | `https://indexer.dydx.trade`                        |
| WebSocket | `wss://indexer.dydx.trade/v4/ws`                    |
| gRPC      | `https://dydx-ops-grpc.kingnodes.com:443` (primary) |

## Configuration {#configuration-1}

ตั้งค่า adapter ของ dYdX ผ่าน configuration ของ trading node execution client รองรับการ fallback ไปใช้
environment variable สำหรับ credential ส่วน data client ใช้ endpoint สาธารณะและไม่ต้องใช้ credential ของ wallet

### Option ของ data client {#data-client-configuration-options}

| Option      | Default   | คำอธิบาย                                         |
| ----------- | --------- | ----------------------------------------------- |
| `network`   | `MAINNET` | `DydxNetwork.MAINNET` หรือ `DydxNetwork.TESTNET` |
| `proxy_url` | `None`    | proxy URL (optional) สำหรับ HTTP และ WebSocket  |

### Option ของ execution client {#execution-client-configuration-options}

| Option              | Default   | คำอธิบาย                                                                           |
| ------------------- | --------- | --------------------------------------------------------------------------------- |
| `account_id`        | Required  | account ID ของ Nautilus สำหรับ client                                               |
| `network`           | `MAINNET` | `DydxNetwork.MAINNET` หรือ `DydxNetwork.TESTNET`                                   |
| `private_key`       | `None`    | signing key แบบ hex หากไม่ระบุจะ fallback ไปใช้ environment variable ของ network นั้น |
| `wallet_address`    | `None`    | wallet address ของ dYdX หากไม่ระบุจะ fallback ไปใช้ environment variable ของ network นั้น |
| `subaccount_number` | `0`       | หมายเลข subaccount ตั้งแต่ `0` ถึง `127`                                          |
| `proxy_url`         | `None`    | proxy URL (optional) สำหรับ HTTP และ WebSocket                                    |

### การตั้งค่าพื้นฐาน {#basic-setup}

ลงทะเบียน `DydxDataClientConfig` กับ `DydxDataClientFactory` และ `DydxExecutionClientConfig` กับ
`DydxExecutionClientFactory` บน builder ของ node ดูตัวอย่างการใช้งานได้ที่
[ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/dydx/examples/)

### API credentials {#api-credentials}

credential ส่งผ่าน config โดยตรงได้ (`wallet_address`, `private_key`) หรือ resolve อัตโนมัติจาก
environment variable ตาม `network` ที่ตั้งค่าไว้

#### Environment variables {#environment-variables}

| Variable                      | Network | คำอธิบาย                                         |
| ----------------------------- | ------- | ---------------------------------------------- |
| `DYDX_WALLET_ADDRESS`         | Mainnet | wallet address แบบ bech32 (`dydx1...`)         |
| `DYDX_PRIVATE_KEY`            | Mainnet | private key แบบ secp256k1 ที่ encode เป็น hex สำหรับ sign |
| `DYDX_TESTNET_WALLET_ADDRESS` | Testnet | wallet address ของ testnet (`dydx1...`)        |
| `DYDX_TESTNET_PRIVATE_KEY`    | Testnet | private key ของ testnet                        |

#### ลำดับการ resolve {#resolution-priority}

1. ค่าที่ส่งมาใน config (ถ้าไม่ว่าง)
2. environment variable ที่เลือกตาม `network`

### การเทรดด้วย permissioned key {#permissioned-key-trading}

#### API Trading Keys คืออะไร {#what-are-api-trading-keys}

API Trading Keys ช่วยให้มอบหมายการเทรดให้ signing key แยกต่างหากได้ โดยไม่ต้องเปิดเผย seed phrase ของ
wallet หลัก API key สามารถวาง trade โดยใช้ margin ทั้งหมดที่มีใน cross-margin account ของเจ้าของ
แต่ถอนเงินหรือโอน asset ไม่ได้

#### การสร้าง API key {#creating-an-api-key}

1. ในเว็บแอปของ dYdX ไปที่ **More > API Trading Keys**
2. คลิก **Generate New API Key**
3. บันทึก **API Wallet Address** และ **Private Key** (แสดงเพียงครั้งเดียว dYdX ไม่ได้เก็บไว้)
4. คลิก **Authorize API Key** (เป็นการลงทะเบียน key บน chain ในฐานะ authenticator)
5. key จะ active และใช้เทรดได้ทันที

ดู [เอกสาร permissioned keys ของ dYdX](https://docs.dydx.xyz/interaction/permissioned-keys)
สำหรับโมเดลของ authenticator และ
[คู่มือการใช้งานผ่านหน้าเว็บ](https://help.dydx.trade/en/articles/267486-api-trading-keys-creating-a-new-key-on-the-front-end)
สำหรับการสร้างและจัดการ key ในเว็บแอป

#### การตั้งค่า adapter {#adapter-configuration}

ตั้ง private key ของ API key เป็น `DYDX_PRIVATE_KEY` และ wallet address ของเจ้าของเป็น `DYDX_WALLET_ADDRESS`
(หรือส่งผ่าน `wallet_address` ซึ่งเป็น account ของเจ้าของที่ถือ margin และ `private_key` ซึ่งเป็น private key
ของ API Trading Key ใน config) adapter จะตรวจพบว่าไม่ตรงกันระหว่าง connect และ query chain
หา authenticator ID ที่ตรงกันให้อัตโนมัติ

:::note
API Trading Keys ใช้ได้เฉพาะกับ account แบบ **cross-margin** และตลาดแบบ cross เท่านั้น ไม่รองรับ isolated margin
:::

## Order books {#order-books}

order book สามารถดูแลได้แบบ full depth หรือแบบ quote ระดับ top-of-book ขึ้นอยู่กับ subscription
venue ไม่ได้ให้ quote โดยตรง adapter จึง subscribe order book delta แล้วสังเคราะห์ quote ให้ `DataEngine`
เมื่อราคาหรือขนาดที่ top-of-book เปลี่ยน รองรับเฉพาะ book type แบบ L2 (MBP)

## การมีส่วนร่วม {#contributing}

:::info
หากต้องการฟีเจอร์เพิ่มเติมหรือมีส่วนร่วมพัฒนา adapter ของ dYdX โปรดดู
[contributing guide](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
