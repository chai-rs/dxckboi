# Bybit

Bybit ก่อตั้งในปี 2018 และเป็นหนึ่งใน cryptocurrency exchange ที่ใหญ่ที่สุดเมื่อวัดจาก volume การซื้อขายรายวัน
และ open interest ของ crypto asset และผลิตภัณฑ์ crypto derivative

NautilusTrader มี integration กับ Bybit สำหรับ live market data และ execution โดย adapter นี้ implement ด้วย Rust

## ตัวอย่าง {#examples}

- [ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/bybit/examples/)

## ภาพรวม {#overview}

คู่มือนี้สมมติว่า trader กำลังตั้งค่าทั้ง live market data feed และ trade execution Bybit adapter ประกอบด้วยหลาย
component ซึ่งใช้ร่วมกันหรือแยกกันก็ได้ตาม use case

- `BybitDataClientConfig` และ `BybitExecutionClientConfig`: config ของ live client
- `BybitDataClientFactory` และ `BybitExecutionClientFactory`: client factory ของ trading node
- `BybitDataClient`: ตัวจัดการ market data feed ที่สร้างโดย data client factory
- `BybitExecutionClient`: gateway สำหรับจัดการ account และ trade execution ที่สร้างโดย execution client factory
- `BybitHttpClient`: การเชื่อมต่อ HTTP API ระดับล่าง
- `BybitWebSocketClient`: การเชื่อมต่อ WebSocket API ระดับล่างสำหรับผู้เรียกฝั่ง Rust
- `BybitLiquidation`: custom data สำหรับ liquidation สาธารณะของสัญญา linear และ inverse ดู
  [Liquidations](#liquidations)
- `BYBIT`, `BYBIT_CLIENT_ID`, `BYBIT_VENUE`: identifier สาธารณะ
- `BybitEnvironment`, `BybitProductType`, `BybitMarginMode`, `BybitPositionIdx`,
  `BybitPositionMode`: enum สาธารณะที่ใช้ใน config และ order params

:::note
ผู้ใช้ส่วนใหญ่จะตั้งค่า live trading node ตามที่แสดงด้านล่าง และไม่จำเป็นต้องใช้ component ระดับล่างโดยตรง
:::

## เอกสารของ Bybit {#bybit-documentation}

อ่าน [Bybit help center](https://www.bybit.com/en/help-center) ควบคู่กับคู่มือ integration ของ NautilusTrader นี้

## ผลิตภัณฑ์ {#products}

product เป็นคำรวมที่หมายถึงกลุ่มของ instrument type ที่เกี่ยวข้องกัน

:::note
ใน Bybit v5 API จะเรียก product ว่า `category`
:::

Bybit รองรับ product type ต่อไปนี้:

| Product Type                | รองรับ    | หมายเหตุ                                  |
| --------------------------- | --------- | ----------------------------------------- |
| Spot cryptocurrencies       | ✓         | ตลาด spot ดั้งเดิมที่รองรับ margin        |
| Linear perpetual contracts  | ✓         | perpetual swap ที่ใช้ USDT/USDC เป็น margin |
| Linear futures contracts    | ✓         | linear futures ที่ settle เมื่อส่งมอบ      |
| Inverse perpetual contracts | ✓         | perpetual swap ที่ใช้เหรียญเป็น margin     |
| Inverse futures contracts   | ✓         | delivery futures ที่ใช้เหรียญเป็น margin   |
| Option contracts            | ✓         | European option ที่ settle เป็น USDT หรือ USDC |

## Symbology {#symbology}

เพื่อแยก product type ต่าง ๆ บน Bybit นั้น Nautilus ใช้ suffix ตามหมวดของ product ต่อท้าย symbol:

- `-SPOT`: Spot cryptocurrencies
- `-LINEAR`: สัญญา perpetual และ futures
- `-INVERSE`: สัญญา inverse perpetual และ inverse futures
- `-OPTION`: สัญญา option

suffix เหล่านี้ต้องต่อท้าย raw symbol string ของ Bybit เพื่อระบุ product type ของ instrument ID ตัวอย่างเช่น:

- คู่เงิน spot Ether/Tether ระบุด้วย `-SPOT` เช่น `ETHUSDT-SPOT`
- สัญญา perpetual futures BTCUSDT ระบุด้วย `-LINEAR` เช่น `BTCUSDT-LINEAR`
- สัญญา inverse perpetual futures BTCUSD ระบุด้วย `-INVERSE` เช่น `BTCUSD-INVERSE`
- put option ของ BTC ที่ settle เป็น USDT: `BTC-27MAR26-70000-P-USDT-OPTION`
- call option ของ ETH ที่ settle เป็น USDC: `ETH-28FEB25-2800-C-OPTION`

symbol ของ option ใน Bybit จะมีสกุลเงิน settlement สำหรับสัญญาที่ settle เป็น USDT (เช่น
`BTC-27MAR26-70000-P-USDT`) แต่ไม่มีสำหรับสัญญาที่ settle เป็น USDC (เช่น `ETH-28FEB25-2800-C`) adapter จะต่อ
`-OPTION` ท้าย symbol ใดก็ตามที่ API คืนมา

## การโหลด instrument {#instrument-loading}

data client และ execution client จะโหลด instrument ทั้งหมดของ `product_types` ที่ตั้งค่าไว้เมื่อเชื่อมต่อ ค่า default
คือ `LINEAR` ให้ใส่ทุก product type ที่ subscription หรือ order ของคุณต้องใช้

## Environment {#environments}

Bybit มี trading environment สามแบบ ตั้งค่า environment ที่เหมาะสมด้วย enum `environment` ใน config ของ client

| Environment | Config                     | คำอธิบาย                                                         |
| ----------- | -------------------------- | ---------------------------------------------------------------- |
| **Mainnet** | `BybitEnvironment.MAINNET` | เทรดจริงด้วยเงินจริง                                             |
| **Demo**    | `BybitEnvironment.DEMO`    | ฝึกเทรดด้วยเงินจำลองบนโครงสร้างพื้นฐานของ mainnet                 |
| **Testnet** | `BybitEnvironment.TESTNET` | test network แยกต่างหากสำหรับพัฒนาและทดสอบ integration            |

### Mainnet (Production) {#mainnet-production}

environment default สำหรับ live trading ด้วยเงินจริง

Environment variable: `BYBIT_API_KEY`, `BYBIT_API_SECRET`

### Demo trading {#demo-trading}

demo trading ใช้โครงสร้างพื้นฐาน mainnet ของ Bybit ด้วยเงินจำลอง สร้าง demo API key ได้จาก
[หน้า demo trading ของ Bybit](https://www.bybit.com/en/demo-trading)

Environment variable: `BYBIT_DEMO_API_KEY`, `BYBIT_DEMO_API_SECRET`

:::warning
**ข้อจำกัดของ demo environment**:

- WebSocket Trade API **ไม่รองรับ** ใน demo trading NautilusTrader จะใช้ HTTP REST API สำหรับการดำเนินการกับ order ในโหมด demo โดยอัตโนมัติ รวมถึง order list และ batch cancel ซึ่งจะถูกส่งเป็น request แยกทีละตัว
- native TP/SL และ option params (`order_iv`, `mmp`) บน order ใหม่ใช้งานได้ใน demo ผ่าน HTTP create-order endpoint
- custom TP/SL trigger price `tp_trigger_price` และ `sl_trigger_price` ไม่รองรับใน demo (order ที่ตั้งค่าเหล่านี้จะถูก
  deny) เพราะ create-order endpoint ส่งค่าเหล่านี้ไม่ได้
- private stream ของ demo ใช้ `wss://stream-demo.bybit.com` แต่ public market data ใช้ public stream ของ mainnet คือ `wss://stream.bybit.com`

:::

### Testnet {#testnet}

test network แยกต่างหากสำหรับพัฒนาและทดสอบ integration

Environment variable: `BYBIT_TESTNET_API_KEY`, `BYBIT_TESTNET_API_SECRET`

:::note
Testnet รองรับฟีเจอร์การเทรดทั้งหมด รวมถึง WebSocket Trade API แต่ใช้โครงสร้างพื้นฐานที่แยกจาก mainnet ทั้งหมด
ดังนั้น market data และ liquidity จึงต่างจาก production มาก
:::

> หมายเหตุ: ต้นฉบับมีตัวอย่าง config ของแต่ละ environment เฉพาะในภาษา Python จึงตัดออก

เมื่อ `environment=BybitEnvironment.TESTNET` adapter จะ resolve endpoint ของ testnet ตามที่ Bybit ระบุไว้ให้โดยอัตโนมัติ:

- REST API: `https://api-testnet.bybit.com`
- Public WebSocket: `wss://stream-testnet.bybit.com/v5/public/{spot|linear|inverse|option}`
- Private WebSocket: `wss://stream-testnet.bybit.com/v5/private`
- Trade WebSocket: `wss://stream-testnet.bybit.com/v5/trade`

### การตั้งค่า testnet {#testnet-setup}

การตั้งค่า account และ credential ของ Bybit testnet:

1. เปิด [testnet.bybit.com](https://testnet.bybit.com) ใน browser บน desktop
2. สร้าง testnet account แยก หรือ sign in เข้า testnet account ที่มีอยู่
3. ขอ test coin จาก **Assets -> Assets Overview -> Request Test Coins** เพื่อให้ account มี balance สำหรับทดสอบ
4. เปิด **API Management** ที่
   [testnet.bybit.com/app/user/api-management](https://testnet.bybit.com/app/user/api-management)
5. คลิก **Create New Key**
6. เลือก permission ที่ use case ของคุณต้องการ
7. ทำ 2FA ให้เสร็จ แล้วคัดลอก API key และ secret
8. export credential ใน shell:

   ```bash
   export BYBIT_TESTNET_API_KEY="YOUR_TESTNET_API_KEY"
   export BYBIT_TESTNET_API_SECRET="YOUR_TESTNET_API_SECRET"
   ```

คำแนะนำเรื่อง testnet ปัจจุบันของ Bybit ยังระบุว่า:

- API key ต้องสร้างบนเว็บไซต์ ไม่ใช่ใน mobile app
- ผู้ใช้ใหม่อาจสร้าง API key ไม่ได้ในช่วง 48 ชั่วโมงแรกหลังสมัคร
- Testnet แยกจาก mainnet อย่าฝากเงินจริงเข้า testnet account
- ปัจจุบัน Bybit อธิบายการตั้งค่า testnet account ผ่าน browser บน desktop

## Quote และ order book {#quotes-and-order-books}

สำหรับ product แบบ SPOT, LINEAR และ INVERSE นั้น quote subscription ใช้ order book snapshot แบบ depth-1 ของ Bybit
feed นี้ให้ราคาและขนาดของ best bid/ask ด้วยความถี่การ push ตามเอกสารที่ 10 ms ส่วน quote ของ OPTION ใช้ ticker feed
ดู [ข้อกำหนด order book ของ Bybit](https://bybit-exchange.github.io/docs/v5/websocket/public/orderbook)

quote และ book subscription แบบ depth-1 ใช้ WebSocket topic เดียวกัน การ unsubscribe ตัวหนึ่งจะยังให้ topic active
ตราบที่อีกตัวยัง subscribe อยู่ book subscription ที่ลึกกว่าจะใช้ topic ของตัวเองคู่ไปกับ quote feed แบบ depth-1
ดังนั้นการเพิ่ม book delta จะไม่เปลี่ยนแหล่งของ quote book delta มาจากความลึกที่ request เท่านั้น ให้ subscribe
book เพียงความลึกเดียวต่อ instrument และ unsubscribe book เดิมก่อนเลือกความลึกอื่น

## การกู้คืน order book {#order-book-recovery}

### การตรวจสอบลำดับ {#sequence-validation}

delta แต่ละตัวต้องมี update ID `u` ที่ต่อจาก frame ล่าสุดที่ book รับไว้ delta อื่นใดจะระงับ output ของ book และ
ขอการกู้คืน delta ที่มาถึง book ก่อน snapshot แรกก็เช่นกัน ตัวอย่างเช่น update ที่ส่งมาก่อนการ resubscribe อย่าง
รวดเร็วอาจไปถึง subscription ใหม่ได้ request จะเริ่มการกู้คืนก็ต่อเมื่อยังไม่มีการกู้คืนที่กำลังทำงานหรือ snapshot
deadline ใดเป็นเจ้าของ book อยู่

snapshot ทุกตัวจะแทนที่ book ตามที่
[สัญญา order book ของ Bybit](https://bybit-exchange.github.io/docs/v5/websocket/public/orderbook) กำหนด venue
จะส่ง snapshot หลังการ subscribe แต่ละครั้งและหลังการ restart ของ service ซึ่งอาจ reset `u` เป็น 1 และที่ depth 1
จะ push แต่ snapshot เท่านั้น snapshot ที่มาแทนที่อาจมี update ID ต่ำกว่า delta ล่าสุดของ book เพราะ venue ให้มา
จาก state ก่อนหน้า แล้ว delta จะต่อจาก snapshot นั้น snapshot ที่มาถึงขณะที่การเขียน subscription หรือการแทนที่ของ
book ยังดำเนินอยู่จะถูกทิ้ง

### Snapshot deadline {#snapshot-deadlines}

subscription เริ่มต้นและ subscription เพื่อกู้คืนจะรอ snapshot ได้สูงสุด `book_snapshot_timeout_secs`
(default **10 วินาที**) หลังการเขียน subscription เสร็จ book บน socket ที่ reconnect แล้วจะรอเท่ากันนับจากการ
reconnect ถ้าไม่ได้ snapshot จะเริ่มหรือลองการกู้คืนใหม่

ตั้ง `book_snapshot_timeout_secs` เป็น `0` เพื่อปิด snapshot deadline gap ยังคงเริ่มการกู้คืน แต่ความพยายามกู้คืน
ที่ไม่เคยได้ snapshot จะรอจนกว่า retry budget 180 วินาทีจะหมด

book แบบ depth-1 ใช้ topic ร่วมกับ quote เมื่อ quote ถือ topic อยู่แล้ว การ subscribe book จะไม่ส่งอะไรออกไป และ
snapshot depth-1 ถัดไปจะ sync book นั้น

### ขีดจำกัดการ retry และการ reconnect {#retry-limits-and-reconnects}

การกู้คืนจะ unsubscribe และ resubscribe topic ของ book บน connection เดิม และ venue จะตอบด้วย snapshot ใหม่
การกู้คืนแต่ละรอบพยายาม **ได้สูงสุดแปดครั้งภายใน 180 วินาที** ด้วย exponential backoff และ jitter จากนั้นจะ
retry ต่อจนกว่าจะรับ snapshot ได้ ช่วง retry จะเพิ่มเป็นสองเท่าจากหนึ่งนาทีไปจนถึงสิบห้านาที การกู้คืนไม่จบด้วยสถานะ
ล้มเหลว การ unsubscribe และการปิดระบบจะยกเลิกการกู้คืน

การกู้คืน book แบบ depth-1 จะหยุด quote บน topic ที่ใช้ร่วมกันชั่วคราวจนกว่า snapshot ที่มาแทนที่จะมาถึง ถ้า venue
ปฏิเสธ subscription ที่มาแทนที่ จะ log ไว้ และความพยายามนั้นจะ retry เมื่อ snapshot deadline ผ่านไป

การ reconnect จะ resync ทุก book บน socket ของ product type ที่ reconnect จาก snapshot ของ subscription ที่
replay การกู้คืนที่กำลังทำงานจะทำต่อด้วย budget ที่เหลือ การกู้คืนที่กำลังรอระหว่างความพยายามหลังหมด budget จะ
retry ทันทีบน connection ใหม่

ถ้า client เข้าคิว subscription เริ่มต้นไม่ได้ จะ log error และปล่อย book แทนที่จะกู้คืน ตัวอย่างเช่น subscription
อาจเกินขีดจำกัด topic ของ connection ของ option subscription เริ่มต้นที่เข้าคิวแล้วแต่การเขียนล้มเหลวจะเริ่มการกู้คืน
ทันที

ดู [ownership ของการกู้คืน order book](../developer_guide/adapters.md#order-book-recovery-ownership)
สำหรับกลไกการกู้คืนที่ใช้ร่วมกันและความรับผิดชอบของ adapter

### การตรวจสอบการกู้คืนแบบ live {#live-recovery-validation}

harness `bybit-book-stress` เป็นเครื่องมือสำหรับนักพัฒนาเมื่อแก้ไขการ sync และการกู้คืน book มันเชื่อมต่อกับ public
market data ของ Bybit mainnet และไม่ส่ง order ใด ๆ โดยตรวจ linear book หกตัวที่ความลึก default 50 เทียบกับสัญญา
ของ book stream การตรวจเหล่านี้รวมถึงลำดับที่เพิ่มขึ้นภายในแต่ละรอบของ snapshot และยังตรวจเทียบกับการสร้าง 50
ระดับที่ดีที่สุดของ feed ของ venue ขึ้นมาใหม่อย่างอิสระด้วย

รันจาก root ของ repository:

```bash
CARGO_BUILD_JOBS=16 bash scripts/strip-adapter-env.bash \
  cargo test -p nautilus-bybit --features examples --test bybit-book-stress -- --timeout 10 --rounds 21
```

`--scenario` เลือกการรัน:

- `churn` (default): ตรวจการกู้คืนโดยไม่มีการ reconnect แล้วหมุนเวียนสถานการณ์ update ID gap, snapshot ที่หายหรือ
  ล่าช้า, การ reconnect ที่ถูกตัดก่อนได้ snapshot, การ unsubscribe ระหว่างกู้คืน, การ restart ระหว่างกู้คืน และการหยุด
  traffic 40 วินาที
- `initial`: ทิ้ง snapshot แรกของแต่ละ book หนึ่งครั้งต่อรอบใน session ใหม่
- `turnover`: unsubscribe และ resubscribe book ระหว่างกู้คืน
- `boundaries`: ทดสอบการตัด snapshot ที่มาแทนที่, การใช้ retry จนหมดไปถึงเพดานของ retry และการปิดระบบระหว่าง
  reconnect เมื่อ timeout ไม่เป็นศูนย์ จะปฏิเสธ subscription ที่มาแทนที่และ subscription ที่ replay ด้วย

`--timeout` กำหนด snapshot timeout หน่วยวินาที โดย `0` ปิด snapshot deadline `--rounds` กำหนดจำนวนรอบ
(default 21) สำหรับ `churn`, `initial` และ `turnover` ส่วน `boundaries` รันลำดับของตัวเองครั้งเดียว

harness นี้ต้องใช้ linear WebSocket stream ของ mainnet และ REST API สาธารณะ `instruments-info` ดู flag ที่ใช้ร่วมกัน
และรูปแบบ output ได้ที่ [Stress harnesses](../developer_guide/spec_data_testing.md#stress-harnesses)

## ความสามารถด้าน order {#orders-capability}

Bybit มี trigger type หลายแบบที่ผสมกันได้อย่างยืดหยุ่น ทำให้ใช้ order ของ Nautilus ได้หลากหลายขึ้น order type
ทั้งหมดด้านล่างใช้ได้ *ทั้ง* สำหรับเข้าและออก position

### Order type {#order-types}

| Order Type             | Spot | Linear | Inverse | Option | หมายเหตุ                               |
| ---------------------- | ---- | ------ | ------- | ------ | -------------------------------------- |
| `MARKET`               | ✓    | ✓      | ✓       | ✓      | quote quantity: เฉพาะ Spot              |
| `LIMIT`                | ✓    | ✓      | ✓       | ✓      |                                        |
| `STOP_MARKET`          | ✓    | ✓      | ✓       | -      | *ไม่รองรับสำหรับ Options*               |
| `STOP_LIMIT`           | ✓    | ✓      | ✓       | -      | *ไม่รองรับสำหรับ Options*               |
| `MARKET_IF_TOUCHED`    | ✓    | ✓      | ✓       | -      | *ไม่รองรับสำหรับ Options*               |
| `LIMIT_IF_TOUCHED`     | ✓    | ✓      | ✓       | -      | *ไม่รองรับสำหรับ Options*               |
| `TRAILING_STOP_MARKET` | -    | -      | -       | -      | ดู [Trailing stop](#trailing-stops)     |

ตอน submit adapter จะ deny order type ที่ไม่รองรับภายในเครื่องด้วยเหตุผล `UNSUPPORTED_ORDER_TYPE` และไม่ส่ง
order ไปยัง venue

### Execution instruction {#execution-instructions}

| Instruction   | Spot | Linear | Inverse | Option | หมายเหตุ                                                          |
| ------------- | ---- | ------ | ------- | ------ | ----------------------------------------------------------------- |
| `post_only`   | ✓    | ✓      | ✓       | ✓      | เฉพาะ limit order type ส่งเป็น time in force `PostOnly` ของ Bybit |
| `reduce_only` | -    | ✓      | ✓       | ✓      | *ไม่รองรับสำหรับ Spot*                                             |

### Time in force {#time-in-force}

| Time in force | Spot | Linear | Inverse | Option | หมายเหตุ                         |
| ------------- | ---- | ------ | ------- | ------ | -------------------------------- |
| `GTC`         | ✓    | ✓      | ✓       | ✓      | Good Till Canceled               |
| `GTD`         | -    | -      | -       | -      | ไม่มีวันหมดอายุแบบ native ส่งเป็น `GTC` |
| `FOK`         | ✓    | ✓      | ✓       | ✓      | Fill or Kill                     |
| `IOC`         | ✓    | ✓      | ✓       | ✓      | Immediate or Cancel              |

ค่า time in force อื่น (`DAY`, `AT_THE_OPEN` และ `AT_THE_CLOSE`) จะทำให้เกิด `OrderDenied` ด้วยเหตุผล
`UNSUPPORTED_TIME_IN_FORCE` ก่อนส่งผ่าน HTTP หรือ WebSocket ซึ่งใช้กับ market order และ post-only order ด้วย
TIF ที่ไม่รองรับใน order list จะ deny ทุก leg ก่อนส่ง

### ฟีเจอร์ order ขั้นสูง {#advanced-order-features}

| Feature            | Spot | Linear | Inverse | Option | หมายเหตุ                               |
| ------------------ | ---- | ------ | ------- | ------ | -------------------------------------- |
| Order Modification | ✓    | ✓      | ✓       | ✓      | แก้ไขราคาและปริมาณ                     |
| Bracket/OCO Orders | -    | -      | -       | -      | ไม่ได้ implement ให้ส่งแต่ละ leg เอง    |
| Iceberg Orders     | -    | -      | -       | -      | ไม่ได้ implement                        |

### Batch operation {#batch-operations}

| Operation    | Spot | Linear | Inverse | Option | หมายเหตุ                                  |
| ------------ | ---- | ------ | ------- | ------ | ----------------------------------------- |
| Batch Submit | ✓    | ✓      | ✓       | ✓      | ส่งหลาย order ใน request เดียว            |
| Batch Modify | -    | -      | -       | -      | ไม่ได้เชื่อมเข้ากับ execution client       |
| Batch Cancel | ✓    | ✓      | ✓       | ✓      | cancel หลาย order ใน request เดียว        |

batch submit และ batch cancel ใช้ trade WebSocket บน mainnet และ testnet ในโหมด demo adapter จะถอยไปใช้ HTTP
request แยกทีละตัว เพราะ demo environment ไม่มี trade WebSocket

Bybit รับได้ไม่เกิน 10 Spot order, 20 Linear หรือ Inverse order หรือ 5 Option order ใน batch request เดียว batch
ของ Linear, Inverse และ Spot ใช้ UID quota ต่อ order ขณะที่ batch ของ Option ใช้หนึ่ง request adapter แบ่ง batch
ของ Spot, Linear และ Inverse เป็นกลุ่มละ 10 โดย default เพื่อไม่ให้ request เดียวเกิน UID allowance แบบ rolling
มาตรฐาน และแบ่ง batch ของ Option เป็นกลุ่มละห้า method batch-cancel แบบ HTTP รับ Option operation ได้สูงสุด 20
รายการในการเรียกครั้งเดียว

### Cancel-all order {#cancel-all-orders}

สำหรับ cancel-all ทั้ง venue (`strategy_only=False`) ที่ไม่มีตัวกรอง `order_side` adapter จะอัปเดต order ใน
cache ก็ต่อเมื่อ venue ยืนยันการ cancel แล้วเท่านั้น fill ใช้ execution stream และ reconciliation cancel request,
การค้นประวัติ และ query ยืนยัน ใช้ deadline `http_timeout_secs` ร่วมกัน adapter จะรอ `retry_delay_initial_ms`
(อย่างน้อย 1 ms) ระหว่างรอบการ poll ถ้า request ล้มเหลวหรือหมด deadline order ที่ยังไม่ได้ยืนยันจะยังเปิดอยู่ภายใน
เครื่อง จนกว่า order update หรือ reconciliation ภายหลังจะยืนยันการปิด ถ้ามีตัวกรอง `order_side` adapter จะใช้
batch cancel กับ order ที่ตรงกันแทน ดูการ route command ได้ที่
[Cancel-all routing](../concepts/execution/index.md#cancel-all-routing)

### Reconciliation report {#reconciliation-reports}

เมื่อใช้ `LiveNode` การตรวจ open order และ order แบบเจาะจงจะรวบรวม order report บน runtime worker node จะรับ
report ก่อนที่ขั้นตอน finalization จะอัปเดต identity ของ order บน core thread การรวบรวมที่หมดเวลาหรือถูกยกเลิกยัง
เก็บ identity ที่จำเป็นสำหรับการ reject ของ pending cancel และ amend ไว้ ส่วน mass status ตอน startup และ method
report แบบ asynchronous โดยตรงยังคงรวบรวมและ finalize ในการเรียกครั้งเดียว

### การจัดการ position {#position-management}

| Feature          | Spot | Linear | Inverse | Option | หมายเหตุ                                                    |
| ---------------- | ---- | ------ | ------- | ------ | ----------------------------------------------------------- |
| Query positions  | -    | ✓      | ✓       | ✓      | อัปเดต position แบบ real-time                               |
| Position mode    | -    | ✓      | ✓       | -      | Options รองรับเฉพาะ One-Way                                 |
| Leverage control | -    | ✓      | ✓       | -      | ไม่ใช้กับ Options                                           |
| Margin mode      | -    | ✓      | ✓       | ✓      | `ISOLATED_MARGIN`, `REGULAR_MARGIN` หรือ `PORTFOLIO_MARGIN` |

ตั้ง `margin_mode` ใน config ของ execution client เพื่อใช้ `BybitMarginMode` กับ account เมื่อ client เชื่อมต่อ

#### Hedge mode (BothSides) {#hedge-mode-bothsides}

Bybit รับโหมด Both Sides เฉพาะบน USDT linear perpetual ให้ตั้งค่า position mode ที่ Bybit แล้วส่ง `position_idx`
ผ่าน `params` ของ order ใช้ `1` สำหรับฝั่ง long หรือ `2` สำหรับฝั่ง short ใช้ `0` หรือไม่ส่งพารามิเตอร์นี้สำหรับโหมด
one-way

Bybit อธิบายค่าเหล่านี้ไว้ใน V5 API [switch position mode](https://bybit-exchange.github.io/docs/v5/position/position-mode)
และ [place order](https://bybit-exchange.github.io/docs/v5/order/create-order#request-parameters)

order และ report ที่มี `positionIdx=0` (one-way / Merged Single mode) ไม่มี venue position ID สำหรับ index
`1` และ `2` ของ hedge mode adapter จะ map report ไปยัง venue position ID ที่ลงท้ายด้วย `-LONG` และ `-SHORT`
ถ้าข้อความ execution ของ Bybit ไม่มี `positionIdx` fill จะใช้ ID เดียวกัน

ใน hedge mode `positionIdx` ระบุ position ที่ได้รับผลกระทบ ไม่ใช่ทิศทางของ trade ดังนั้น reduce-only sell จะ
resolve ไปที่ index ของ long และ reduce-only buy จะ resolve ไปที่ index ของ short

ถ้าต้องการ override ให้ส่ง `position_idx` ผ่าน `params` (`0` one-way, `1` long, `2` short)

### Risk event {#risk-events}

| Feature              | Spot | Linear | Inverse | Option | หมายเหตุ                                              |
| -------------------- | ---- | ------ | ------- | ------ | ----------------------------------------------------- |
| Liquidation handling | -    | ✓      | ✓       | ✓      | fill จากการ takeover ถูก flag ว่า exchange สร้างขึ้น  |
| ADL handling         | -    | ✓      | ✓       | ✓      | fill จาก auto-deleveraging ถูก flag และ log ไว้        |
| ADL rank warnings    | -    | ✓      | ✓       | ✓      | log position report เมื่อ `adlRankIndicator >= 4`     |

Bybit ส่ง fill ที่ venue เป็นผู้เริ่ม โดยตั้ง `execType` เป็น:

- `AdlTrade`: execution จาก auto-deleveraging position ฝั่งตรงข้ามที่มีกำไรถูกเลือกให้ปิดคู่สัญญาที่ collateral ไม่พอ
  หลังจาก insurance fund ไม่สามารถชดเชยผลขาดทุนได้
- `BustTrade`: liquidation takeover liquidation engine ยึด position หลังจาก margin หมด
- `Delivery`: การส่งมอบ USDC futures
- `Settle`: การ settle inverse futures
- `ForwardSplitSettle`: การ settle เศษหุ้นจาก forward stock split
- `ReverseSplitSettle`: การ settle เศษหุ้นจาก reverse stock split
- `Dividend`: การจ่ายเงินปันผล

ก่อนหน้านี้ venue ใช้ `CorporateAction` สำหรับ stock split และ reverse split adapter ยังรับค่านี้ในประวัติ execution
ที่บันทึกไว้ก่อนจะถูกแทนที่ด้วย type ของ settle และ dividend ข้างต้น

adapter จะ flag แต่ละรายการว่า exchange สร้างขึ้น และ log warning ที่มี execution ID, symbol, side, quantity และ
price fill จะไหลผ่านเส้นทาง `FillReport` ปกติ เนื่องจาก order เหล่านี้มี `orderLinkId` ว่าง execution engine จึง
ถือว่าเป็น external order และ assign ผ่าน external order claim ที่ active ของ instrument ซึ่งตั้งค่าเริ่มต้นด้วย
`external_order_instrument_ids` ถ้าไม่มี claim จะ assign ให้ strategy `EXTERNAL`

execution type ที่ adapter ไม่รู้จักจะถูกจัดการเป็น `UNKNOWN` แทนที่จะถูก reject ดังนั้นค่าที่ venue เพิ่มในอนาคต
จะยังไหลผ่านเส้นทาง fill แทนที่จะถูกทิ้ง

funding settlement ใช้ `execType=Funding` แต่เป็นการปรับ balance ไม่ใช่ fill adapter จะข้ามรายการเหล่านี้ใน
historical fill report และข้อความ private `execution` มาตรฐาน จึงไม่ส่งทั้ง `FillReport` และ event `OrderFilled`
และไม่เปลี่ยนปริมาณ position ภายในเครื่อง ระหว่าง reconciliation รายการ funding ไม่นับรวมในขีดจำกัดของ fill report
ที่ request

Bybit ยัง publish ADL ranking ในการอัปเดต position ผ่านฟิลด์ `adlRankIndicator` ช่วงค่าคือ 0 (flat / ไม่มี
position) ถึง 5 (ถัดไปที่จะถูก deleverage) adapter จะ log warning ทุกครั้งที่ open position มี rank 4 ขึ้นไป เพื่อให้
คุณตอบสนองได้ก่อนที่ venue จะบังคับปิด

เอกสารอ้างอิงต้นทาง:

- [ค่า `execType` ของ V5](https://bybit-exchange.github.io/docs/v5/enum#exectype)
- [ค่า `createType` ของ V5](https://bybit-exchange.github.io/docs/v5/enum#createtype)
- [กลไก liquidation](https://www.bybit.com/en/help-center/article/Liquidation-Process-Derivatives-Trading)
- [กลไก Auto-Deleveraging](https://www.bybit.com/en/help-center/article/Auto-Deleveraging-ADL-Derivatives-Trading)

### การ query order {#order-querying}

| Feature              | Spot | Linear | Inverse | Option | หมายเหตุ                       |
| -------------------- | ---- | ------ | ------- | ------ | ------------------------------ |
| Query open orders    | ✓    | ✓      | ✓       | ✓      | แสดง order ที่ active ทั้งหมด  |
| Query order history  | ✓    | ✓      | ✓       | ✓      | ข้อมูล order ย้อนหลัง          |
| Order status updates | ✓    | ✓      | ✓       | ✓      | การเปลี่ยนสถานะ order แบบ real-time |
| Trade history        | ✓    | ✓      | ✓       | ✓      | execution report และ fill report |

### Contingent order {#contingent-orders}

| Feature            | Spot | Linear | Inverse | Option | หมายเหตุ                               |
| ------------------ | ---- | ------ | ------- | ------ | -------------------------------------- |
| Order lists        | ✓    | ✓      | ✓       | ✓      | ส่งเป็น batch ผ่าน WebSocket           |
| OCO orders         | -    | -      | -       | -      | ไม่ได้ implement ให้ส่งแต่ละ leg เอง    |
| Bracket orders     | -    | -      | -       | -      | ไม่ได้ implement ให้ส่งแต่ละ leg เอง    |
| Conditional orders | ✓    | ✓      | ✓       | -      | order แบบ stop และ limit-if-touched     |

order list จะถูกตรวจสอบเป็นหน่วยเดียวก่อนส่ง leg ใด ๆ เมื่อ leg หนึ่งไม่ผ่านการตรวจสอบ leg นั้นจะถูก deny ด้วยเหตุผล
เฉพาะของมัน ส่วน leg ที่เหลือจะถูก deny ด้วย `ORDER_LIST_DENIED` ดังนั้น list ที่ส่งไปเพียงบางส่วนจะไปไม่ถึง venue

### Order parameter {#order-parameters}

ปรับแต่ง order แต่ละตัวได้ด้วย `params` ตอน submit order:

| Parameter          | Type             | คำอธิบาย                                                            |
| ------------------ | ---------------- | ------------------------------------------------------------------- |
| `is_leverage`      | `bool`           | เฉพาะ Spot เปิด margin trading (การกู้ยืม) Default: `False`        |
| `take_profit`      | `str` or `float` | trigger price ของ TP แนบ native TP ไปกับ order                      |
| `stop_loss`        | `str` or `float` | trigger price ของ SL แนบ native SL ไปกับ order                      |
| `tp_trigger_by`    | `str`            | trigger type ของ TP: `"LastPrice"`, `"IndexPrice"` หรือ `"MarkPrice"` |
| `sl_trigger_by`    | `str`            | trigger type ของ SL: `"LastPrice"`, `"IndexPrice"` หรือ `"MarkPrice"` |
| `tp_order_type`    | `str`            | execution type ของ TP: `"Market"` หรือ `"Limit"`                    |
| `sl_order_type`    | `str`            | execution type ของ SL: `"Market"` หรือ `"Limit"`                    |
| `tp_limit_price`   | `str` or `float` | limit price ของ TP เมื่อ `tp_order_type` เป็น `"Limit"`             |
| `sl_limit_price`   | `str` or `float` | limit price ของ SL เมื่อ `sl_order_type` เป็น `"Limit"`             |
| `tp_trigger_price` | `str` or `float` | trigger price ของ TP แบบชัดเจนที่ส่งไปพร้อมกับ `take_profit`       |
| `sl_trigger_price` | `str` or `float` | trigger price ของ SL แบบชัดเจนที่ส่งไปพร้อมกับ `stop_loss`         |
| `tpsl_mode`        | `str`            | โหมด TP/SL: `"Full"` หรือ `"Partial"`                               |
| `close_on_trigger` | `bool`           | ปิด position เมื่อ TP/SL trigger                                    |
| `position_idx`     | `int`            | position index ของ hedge mode ดู [Hedge mode](#hedge-mode-bothsides) |
| `bbo_side_type`    | `str`            | BBO side ของ linear/inverse: `"Queue"` หรือ `"Counterparty"`        |
| `bbo_level`        | `str` or `int`   | ระดับ book ของ BBO สำหรับ linear/inverse: `"1"` ถึง `"5"`           |
| `smp_type`         | `str`            | self-match prevention ดู [SMP](#self-match-prevention)              |

พารามิเตอร์ที่ไม่ได้ตั้งค่าจะไม่ถูกใส่ใน request ดังนั้นจะใช้ค่า default ของ Bybit เอง

:::warning
พารามิเตอร์ `close_on_trigger` ของ Bybit ไม่ใช่สัญญาการออกจาก position ทั้งหมดแบบ `close_position` ทั่วไปที่
risk engine ใช้ adapter จะส่งปริมาณของ order และข้ามพารามิเตอร์ `close_position` ที่ไม่รู้จัก อย่าเพิ่ม `BYBIT` ลงใน
`full_position_exit_venues` โดยอิงจาก `close_on_trigger` ให้ปล่อย venue นี้ไว้นอกรายการ เพื่อให้การตรวจปริมาณและ
notional ตามปกติยังทำงาน
:::

adapter จะตรวจสอบ params เหล่านี้ก่อนส่ง `OrderSubmitted` และ deny order ด้วยเหตุผล `VALIDATION_FAILED` เมื่อ
ผิดกฎ:

- ฟิลด์ override ของ TP ทุกตัว (`tp_trigger_by`, `tp_order_type`, `tp_limit_price`, `tp_trigger_price`)
  ต้องมี `take_profit` และฟิลด์ override ของ SL ทุกตัวก็ต้องมี `stop_loss` เช่นกัน
- `tp_order_type="Limit"` ต้องมี `tp_limit_price` และ `tp_limit_price` ต้องมี `tp_order_type="Limit"` การจับคู่
  แบบเดียวกันใช้กับ `sl_order_type` และ `sl_limit_price`
- `bbo_side_type` และ `bbo_level` ต้องระบุพร้อมกัน
- `smp_type` ต้องเป็น `"None"`, `"CancelMaker"`, `"CancelTaker"` หรือ `"CancelBoth"` โดยเทียบแบบไม่สนตัวพิมพ์
  เล็กใหญ่

เมื่อตั้ง `take_profit` หรือ `stop_loss` โดยไม่มี `tpsl_mode` adapter จะส่ง `Full` เมื่อตั้งราคา TP หรือ SL โดยไม่มี
`tp_trigger_by` หรือ `sl_trigger_by` ของตัวเอง adapter จะอนุมาน trigger type จาก trigger type ของ order

:::note
บน demo params ของ native TP/SL จะ route ผ่าน HTTP create-order endpoint โดยมีข้อยกเว้นหนึ่งข้อ: endpoint ไม่สามารถ
ส่ง custom trigger price `tp_trigger_price` และ `sl_trigger_price` ได้ adapter จึง deny order ที่ตั้งราคาใดราคาหนึ่ง
ในนี้ param `is_leverage` ใช้กับ product แบบ Spot เท่านั้น ดู
[เอกสาร isLeverage ของ Bybit](https://bybit-exchange.github.io/docs/v5/order/create-order#request-parameters)
:::

เมื่อตั้ง `bbo_side_type` และ `bbo_level` แล้ว Nautilus จะส่งฟิลด์ `bboSideType` และ `bboLevel` ของ Bybit และไม่ใส่
ราคาของ order ใน API request BBO order รองรับสำหรับ limit, stop-limit และ limit-if-touched order ของ linear และ
inverse

#### Self-match prevention {#self-match-prevention}

self-match prevention (SMP) บอก Bybit ว่าต้องทำอย่างไรเมื่อ order ของคุณตัวหนึ่งจะเทรดกับ order อื่นของคุณเอง Bybit
รับค่าบน order ได้สี่ค่า ซึ่งแสดงด้านล่างในรูปการสะกดมาตรฐานบน wire adapter เทียบค่าแบบไม่สนตัวพิมพ์เล็กใหญ่ และส่ง
การสะกดมาตรฐานเสมอ

| ค่า           | พฤติกรรม                         |
| ------------- | -------------------------------- |
| `None`        | ไม่มี self-match prevention      |
| `CancelMaker` | cancel maker order ที่รออยู่      |
| `CancelTaker` | cancel taker order ที่เข้ามาใหม่  |
| `CancelBoth`  | cancel ทั้งสอง order             |

ตั้ง `smp_type` ใน config ของ execution client เพื่อส่งค่านั้นในทุก order ที่ client ส่ง และส่ง `smp_type` ผ่าน
`params` ของ order เพื่อ override ค่า default ที่ตั้งไว้สำหรับ order เดียว

adapter จะ deny order ที่ `smp_type` ไม่ใช่หนึ่งในสี่ค่าข้างต้น และปฏิเสธ config ของ client ที่มีค่าอื่น ถ้าไม่ได้ตั้งทั้ง
config และ param adapter จะไม่ส่ง `smpType` ดังนั้นจะใช้ค่า default ของ Bybit เอง Bybit override การตั้งค่านี้ในบาง
ภูมิภาค account derivatives ในคาซัคสถานถูกบังคับเปิด SMP และเลือกไม่ใช้ไม่ได้ สำหรับ spot account ในตุรกี คาซัคสถาน
และจอร์เจีย Bybit จะใช้ `CancelMaker` เมื่อ `smpType` ไม่ได้ส่งมา เป็น `None` หรือไม่ถูกต้อง

Bybit อธิบายค่าและกฎตามภูมิภาคไว้ใน [SMP guide](https://bybit-exchange.github.io/docs/v5/smp) ของ V5

> หมายเหตุ: ต้นฉบับมีตัวอย่าง order ที่ใช้ native TP/SL, BBO order และ Spot margin trading เฉพาะในภาษา Python
> จึงตัดออก

:::note
ถ้าไม่ได้ตั้ง `is_leverage=True` ใน params Spot order จะใช้ balance ที่มีอยู่และไม่กู้ยืม แม้จะเปิด auto-borrow
ไว้ใน account ของ Bybit ก็ตาม
:::

### ข้อจำกัดของการเทรด Spot {#spot-trading-limitations}

ข้อจำกัดต่อไปนี้ใช้กับ product แบบ Spot เพราะ venue ไม่ได้ติดตาม position ให้:

- *ไม่รองรับ* order แบบ `reduce_only`
- *ไม่รองรับ* trailing stop order

### การเทรด option {#options-trading}

Bybit มี option แบบ European บนสินทรัพย์อ้างอิงที่เป็น crypto และหุ้นสหรัฐ เช่น BTC, ETH, SOL และ NVDA ซึ่ง settle
เป็น USDT หรือ USDC
adapter ใช้ instrument type `CryptoOption` และ symbol suffix `-OPTION` ดูรูปแบบ symbol ทั้งหมดได้ที่
[หัวข้อ symbology](#symbology)

#### ข้อมูล option {#options-data}

adapter รองรับ market data ของ option แบบ real-time ผ่าน WebSocket ticker channel:

| ชนิดข้อมูล                 | คำอธิบาย                                                                 |
| -------------------------- | ------------------------------------------------------------------------ |
| Quotes (bid/ask)           | ราคาและขนาดระดับ top-of-book ของสัญญา option แต่ละตัว                    |
| Greeks                     | delta, gamma, vega, theta และ IV ของ bid/ask/mark Bybit ไม่ publish rho  |
| Mark price                 | mark price ของ exchange สำหรับสัญญา option แต่ละตัว                      |
| Index price                | index price ของสินทรัพย์อ้างอิง                                          |
| Underlying reference price | ราคาอ้างอิงของ venue ต่อ expiry ที่ใช้หา ATM                             |
| Open interest              | open interest ต่อสัญญา                                                   |
| Order book deltas          | การอัปเดตแบบ L2 MBP จาก option orderbook stream                          |

subscribe Greeks ต่อ instrument หรือรวมเป็น option chain snapshot พร้อมการกรอง strike แบบอิงตำแหน่งเทียบกับ ATM
ได้ ดูรูปแบบการ subscribe ได้ที่[คู่มือแนวคิด options](../concepts/options.md) และดูการเดินผ่านทีละขั้นได้ที่
[tutorial ข้อมูล options](../tutorials/options_data_bybit.md) NautilusTrader สร้างมุมมอง option chain ภายในเครื่อง
จาก market data ของ option ต่อสัญญาของ Bybit

ข้อมูล bar (kline) ไม่มีสำหรับ option เพราะ Bybit ไม่มี kline stream สำหรับ product type นี้

#### Order parameter ของ option {#options-order-parameters}

นอกจาก order parameter มาตรฐานแล้ว option order ยังรับ:

| Parameter  | Type             | คำอธิบาย                                                          |
| ---------- | ---------------- | ---------------------------------------------------------------- |
| `order_iv` | `str` or `float` | ส่งหรือแก้ไข order ด้วย implied volatility แทนราคา              |
| `mmp`      | `bool`           | เปิด Market Maker Protection สำหรับ order                         |

พารามิเตอร์เหล่านี้ส่งผ่าน `params` บน `SubmitOrder` บน mainnet จะไหลผ่าน WebSocket trade channel บน demo จะ
route ผ่าน HTTP create-order endpoint การแก้ไข order ที่มีอยู่ด้วย `order_iv` ไม่รองรับในโหมด demo

#### ข้อจำกัดของการเทรด option {#options-trading-limitations}

- การแก้ไข order ด้วย implied volatility (`order_iv`) และฟีเจอร์อื่นที่มีเฉพาะใน WS trade ไม่รองรับในโหมด demo
- ตั้งค่า leverage ไม่ได้ ผู้ซื้อ option จ่าย premium ผู้ขายวาง margin
- position mode เป็น one-way เท่านั้น ไม่รองรับ hedge mode
- ไม่รองรับ conditional order type (`STOP_MARKET`, `STOP_LIMIT`, `MARKET_IF_TOUCHED`, `LIMIT_IF_TOUCHED`)
- ไม่รองรับ trading stop (TP/SL บน position)
- funding rate ไม่ใช้กับ option
- option ต้องใช้ Unified Trading Account (UTA)

### Trailing stop {#trailing-stops}

adapter ไม่ส่ง order `TRAILING_STOP_MARKET` ของ Nautilus ไปยัง Bybit การส่ง order แบบนี้จะถูก deny ภายในเครื่อง
ด้วยเหตุผล `UNSUPPORTED_ORDER_TYPE`

Bybit model trailing stop เป็น attribute ของ position แบบ netted แทนที่จะเป็น order ดังนั้น trailing stop จึงไม่มี
client order ID ฝั่ง venue และ query ไม่ได้จนกว่าจะเปิดแล้ว ถ้าต้องการ trailing stop ให้แนบผ่าน interface ของ Bybit
และจัดการการออกจาก position ที่เกิดขึ้นนอก Nautilus

## การกู้ยืมและชำระคืน spot margin {#spot-margin-borrowing-and-repayment}

NautilusTrader มีฟังก์ชันชำระคืนการกู้ยืม spot margin อัตโนมัติ เพื่อป้องกันดอกเบี้ยสะสมหลังปิด short position บน Bybit

### ความเป็นมา {#background}

เมื่อเทรด Spot โดยเปิด margin (`is_leverage=True`) Bybit จะกู้เหรียญให้อัตโนมัติเมื่อคุณเปิด short position
แต่หลังจากปิด short position แล้ว (BUY order ถูก fill) เหรียญที่กู้มา **จะไม่ถูกชำระคืนโดยอัตโนมัติ** และจะยังมีดอกเบี้ย
รายชั่วโมงสะสมต่อไปจนกว่าจะชำระคืนเอง
ซึ่งอาจทำให้เสียดอกเบี้ยจำนวนมากถ้าปล่อยทิ้งไว้

### การชำระคืนอัตโนมัติ (แนะนำ) {#automatic-repayment-recommended}

execution client สามารถชำระคืนการกู้ยืม spot margin โดยอัตโนมัติหลังจาก BUY order ถูก fill ทั้งหมดบน Spot
instrument ฟีเจอร์นี้ปิดไว้โดย default ให้ตั้ง `auto_repay_spot_borrows=True` เพื่อเปิดใช้

**การทำงาน**:

1. เมื่อ Spot BUY order ถูก fill ทั้งหมดบน channel `execution` มาตรฐาน execution client จะพยายามชำระคืนการกู้
   ยืม base coin
1. จำนวนที่ชำระคืนจะไม่เกินค่าที่น้อยกว่าระหว่างยอดกู้ค้างอยู่ และปริมาณ base ที่ได้มาจาก execution ทั้งหมดของ order
1. execution client ใช้ converting repay endpoint ของ Bybit เพื่อครอบคลุมค่าธรรมเนียมการเทรดที่คิดเป็น base สำหรับ
   MNT ซึ่ง Bybit ไม่รวมใน converting repayment จะใช้ no-convert repay และหักค่าธรรมเนียมที่คิดเป็น MNT ออกจาก
   จำนวนเงิน
1. request ที่ล้มเหลวหรือผลลัพธ์สถานะ `FA` จะถูก log โดยไม่ทำให้ execution client crash ผลลัพธ์สถานะ `P` จะถูก
   log ว่ากำลังดำเนินการ ไม่ใช่เสร็จแล้ว
1. execution client จะเลื่อนการชำระคืนที่เข้าคิวไว้ในช่วง UTC blackout window ของ Bybit

> หมายเหตุ: ต้นฉบับมีตัวอย่าง config เฉพาะในภาษา Python (ตั้ง `product_types` เป็น Spot และ
> `auto_repay_spot_borrows=True`) จึงตัดออก

### UTC blackout window {#utc-blackout-window}

Bybit block repayment endpoint ทั้งสองตัวตั้งแต่ **นาทีที่ 4 ถึง 5 นาที 30 วินาทีหลังต้นชั่วโมง UTC ทุกชั่วโมง** เพื่อ
คำนวณดอกเบี้ย การชำระคืนอัตโนมัติจะคง request ไว้ในคิวและพยายามส่งที่ 5 นาที 31 วินาทีหลังต้นชั่วโมง

### การตั้งค่าการชำระคืนอัตโนมัติ {#auto-repayment-configuration}

| ตัวเลือก                  | Type   | Default | คำอธิบาย                                                                                                                   |
| ------------------------- | ------ | ------- | -------------------------------------------------------------------------------------------------------------------------- |
| `auto_repay_spot_borrows` | `bool` | `False` | ถ้าเป็น `True` จะชำระคืนการกู้ยืม Spot margin โดยอัตโนมัติหลัง BUY order ถูก fill ทั้งหมด การชำระคืนจะถูกเลื่อนในช่วง blackout |

### หมายเหตุเกี่ยวกับการชำระคืนอัตโนมัติ {#auto-repayment-notes}

- การชำระคืนอัตโนมัติทำงานเฉพาะกับ **Spot BUY order** ไม่ใช่ derivatives
- การชำระคืนใช้ converting repayment ยกเว้น MNT ซึ่งใช้ no-convert repayment
- Bybit อธิบายข้อจำกัดของ endpoint และสถานะผลลัพธ์ไว้ใน
  [Manual Repay](https://bybit-exchange.github.io/docs/v5/account/repay) และ
  [Manual Repay Without Asset Conversion](https://bybit-exchange.github.io/docs/v5/account/no-convert-repay)
- ยังต้องกู้ยืมเองก่อนเปิด short position เว้นแต่จะเปิด auto-borrow ไว้ใน account ของ Bybit

## Funding rate {#funding-rates}

adapter รับข้อมูล funding rate จาก WebSocket stream
[Linear Ticker](https://bybit-exchange.github.io/docs/v5/websocket/public/ticker#linear-inverse-perpetual-response)
Bybit ให้ฟิลด์ `fundingIntervalHour` ในการอัปเดต ticker ซึ่ง adapter ใช้เติมฟิลด์ `interval` ของ
`FundingRateUpdate`

adapter cache ค่า `fundingIntervalHour` ล่าสุดที่รู้ไว้ต่อ symbol เพื่อให้การอัปเดต ticker แบบบางส่วน (ซึ่งอาจไม่มี
ฟิลด์นี้) ยังมี interval ที่ถูกต้อง

สำหรับ request funding rate ย้อนหลัง adapter จะคำนวณ interval จาก funding timestamp ที่ต่อเนื่องกัน record ที่เก่าที่สุด
ใน response ไม่มี timestamp ก่อนหน้าให้จับคู่ จึงไม่มีค่า interval

## ข้อมูลเฉพาะของ Bybit {#bybit-specific-data}

### Liquidation {#liquidations}

adapter ส่ง custom data `BybitLiquidation` จาก WebSocket stream สาธารณะ
[all liquidation](https://bybit-exchange.github.io/docs/v5/websocket/public/all-liquidation) ของ Bybit
(`allLiquidation.{symbol}`) Bybit publish stream นี้สำหรับสัญญา linear และ inverse เท่านั้น adapter จึงปฏิเสธ
subscription ของ Spot และ Option Bybit push อัปเดตทุก 500 ms และอัปเดตหนึ่งครั้งอาจมีหลาย liquidation adapter จะส่ง
`BybitLiquidation` หนึ่งตัวต่อแต่ละรายการ

| Field              | Type           | คำอธิบาย                                                          |
| ------------------ | -------------- | ----------------------------------------------------------------- |
| `instrument_id`    | `InstrumentId` | instrument ของ position ที่ถูก liquidate                          |
| `position_side`    | `PositionSide` | ฝั่งของ position ที่ถูก liquidate เป็น `LONG` สำหรับ `Buy` ของ Bybit |
| `bankruptcy_price` | `Price`        | bankruptcy price ของ position ที่ถูก liquidate                    |
| `quantity`         | `Quantity`     | ขนาดของ liquidation ที่ execute                                   |
| `ts_event`         | `int`          | UNIX timestamp หน่วย nanoseconds ตอนที่ Bybit อัปเดต liquidation  |
| `ts_init`          | `int`          | UNIX timestamp หน่วย nanoseconds ตอนที่สร้าง object               |

`position_side` คือฝั่งของ position ที่ถูก liquidate ไม่ใช่ฝั่งของ liquidation order Bybit รายงาน `Buy` สำหรับ long
position ที่ถูก liquidate และ `Sell` สำหรับ short position ที่ถูก liquidate `bankruptcy_price` คือราคาที่ margin ของ
position หมด ไม่ใช่ราคาที่ liquidation execute

subscribe จาก actor หรือ strategy ด้วย `DataType` ที่ใช้ชื่อ `BybitLiquidation` โดยต้องมี metadata key
`instrument_id` (เช่น `BTCUSDT-LINEAR.BYBIT`) และระบุ client ID `BYBIT`

> หมายเหตุ: ต้นฉบับมีตัวอย่างการ subscribe และ `on_data` เฉพาะในภาษา Python จึงตัดออก

`CustomData` แต่ละตัวที่ส่งออกมาจะมี `DataType` ที่ subscription ใช้ ดังนั้น event จะไปถึง `on_data` ของ actor
ที่ subscribe adapter ถือ data type หนึ่งตัวต่อ instrument และจะปฏิเสธ subscription ของ instrument เดียวกันด้วย data
type อื่น จนกว่าตัวแรกจะถูก unsubscribe

`BybitLiquidation` รองรับการ persist ลง Arrow/Parquet catalog ภายใต้ `data/custom/BybitLiquidation/{identifier}`
build ของ Rust ต้องเปิด feature flag `arrow` ของ `nautilus-bybit` สำหรับการ persist นี้

:::warning
feed นี้มี liquidation event ที่ Bybit publish ให้ถือว่าเป็นสัญญาณที่ venue รายงาน ไม่ใช่ข้อมูลจริงทั้ง exchange ของทุก
liquidation ที่ผ่านการตรวจสอบอย่างอิสระ
:::

## Rate limiting {#rate-limiting}

adapter เข้าคิว request ตาม rolling window ที่แม่นยำก่อนสร้าง authentication timestamp หรือ signature HTTP client
และ trade WebSocket ใช้ UID state ร่วมกันสำหรับ API key และ environment เดียวกัน data client และ execution client
ยังใช้ IP state ร่วมกันเมื่อใช้ origin และ proxy เดียวกัน

| ขอบเขต                        | ขีดจำกัดของ Bybit                     | พฤติกรรมของ adapter                          |
| ----------------------------- | ------------------------------------- | --------------------------------------------- |
| HTTP IP                       | 600 request ต่อ 5 วินาที              | ใช้ร่วมกันตาม origin และ proxy                |
| HTTP and trade WebSocket UID  | แตกต่างตาม endpoint และ product       | ใช้ร่วมกันตาม API key และ environment         |
| Trade WebSocket IP            | 3,000 request ต่อวินาที               | ใช้ร่วมกันตาม WebSocket origin และ proxy      |
| WebSocket connection attempts | 500 ครั้งต่อ 5 นาทีต่อ domain         | ใช้ร่วมกันทั้งการเชื่อมต่อครั้งแรกและการ reconnect |
| Option subscriptions          | 2,000 argument ต่อ connection         | ปฏิเสธก่อนที่ state ของ subscription จะเปลี่ยน |

UID limiter ครอบคลุม route ของ account และ user ที่มีอัตราต่ำกว่าตามเอกสาร, read route ที่ 50 request, order route
เฉพาะ product, ขีดจำกัดของ cancel-all และ batch operation แบบถ่วงน้ำหนัก response ของ HTTP และ trade WebSocket จะ
อัปเดต UID limit ที่ตั้งไว้จาก `X-Bapi-Limit` ติดตาม `X-Bapi-Limit-Status` และเคารพ `X-Bapi-Limit-Reset-Timestamp`
ในอนาคตเมื่อจำนวนคงเหลือถึงศูนย์

`recv_window_ms` ของ execution client ใช้กับ signed REST request และ order command ผ่าน trade WebSocket order บน
WebSocket ที่เข้าคิวจะได้ timestamp และ receive-window header ก็ต่อเมื่อ quota ที่เกี่ยวข้องทั้งหมดอนุญาตให้ส่งแล้ว
การ retry หลัง reconnect จะสร้าง command ใหม่พร้อม header ใหม่ และใช้การเขียนที่ผูกกับ connection ดังนั้น transport
จะไม่ replay order payload ที่ล้าสมัย

:::warning
Bybit คืน `retCode` `10006` ("Too many visits") เมื่อเกิน API rate limit
การเกินเพดาน IP ที่ 600 request ต่อ 5 วินาทีจะได้ HTTP 403 และ IP จะถูก ban อย่างน้อย 10 นาที 403 ที่ตรงกันจะทิ้ง
HTTP session ใน pool ที่ได้รับผลกระทบ และเริ่ม cooldown 10 นาทีที่ใช้ร่วมกัน response 403 อื่นจะไม่เปิด cooldown
:::

:::warning
การประสานงานเกิดขึ้นภายใน process เท่านั้น process หรือ host อื่นที่ใช้ API key หรือ public IP เดียวกันอาจใช้ quota
ของ venue ที่ adapter นี้จองล่วงหน้าไม่ได้ response header ช่วยลดช่องว่างนี้สำหรับ UID limit แต่ process ที่แยกกันยัง
ต้องประสานงานกันในระดับการปฏิบัติงาน
:::

response rate-limit ที่ venue ส่งมาอย่างชัดเจนเป็นการ reject ขั้นสุดท้ายของการดำเนินการกับ order ที่ได้รับผลกระทบ
transport timeout, การ restart ของ service และ request identifier ที่ซ้ำกันยังต้องผ่าน order reconciliation เพราะ
ไม่ได้พิสูจน์ว่า venue รับ order แล้วหรือไม่

:::info
รายละเอียดเพิ่มเติมเกี่ยวกับ rate limiting ดูได้ที่เอกสารทางการ: <https://bybit-exchange.github.io/docs/v5/rate-limit>
:::

## Account type {#account-types}

execution client factory กำหนด account type และ OMS type จาก product type ที่ตั้งค่าไว้:

- **Spot อย่างเดียว**: account type แบบ `CASH` และ OMS type แบบ `HEDGING`
- **Derivatives หรือผสมหลาย product**: account type แบบ `MARGIN` (UTA - Unified Trading Account) และ OMS
  type แบบ `NETTING`

ทำให้เทรด Spot ควบคู่กับ derivatives ได้ใน Unified Trading Account เดียว ซึ่งเป็น account type มาตรฐานของผู้ใช้ Bybit
ส่วนใหญ่

:::info
**Unified Trading Account (UTA) และ Spot margin trading**

ผู้ใช้ Bybit ส่วนใหญ่ตอนนี้มี Unified Trading Account (UTA) เพราะ Bybit ชักนำผู้ใช้ใหม่ไปยัง account type นี้
classic account ถือเป็นแบบเก่า

สำหรับ Spot margin trading บน UTA account:

- การกู้ยืม **ไม่ได้เปิดโดยอัตโนมัติ** ต้องตั้งค่าผ่าน API อย่างชัดเจน
- การใช้ Spot margin ผ่าน API ต้องส่ง order โดยมี `is_leverage=True` ในพารามิเตอร์ (ดู [เอกสาร Bybit](https://bybit-exchange.github.io/docs/v5/order/create-order#request-parameters))
- ถ้าเปิด auto-borrow/auto-repay ไว้ใน account ของ Bybit venue จะกู้/ชำระคืนให้อัตโนมัติสำหรับ margin order เหล่านั้น
- ถ้าไม่เปิด auto-borrow ต้องจัดการการกู้ยืมเองผ่าน interface ของ Bybit

**สำคัญ**: Bybit adapter ของ Nautilus ใช้ค่า default `is_leverage=False` สำหรับ Spot order ดังนั้นจะไม่ใช้ margin
เว้นแต่จะเปิดอย่างชัดเจน
:::

## ตรรกะของสกุลเงินค่าธรรมเนียม {#fee-currency-logic}

การเข้าใจว่า Bybit กำหนดสกุลเงินของค่าธรรมเนียมการเทรดอย่างไรสำคัญต่อการทำบัญชีและการติดตาม position ที่แม่นยำ
กฎของสกุลเงินค่าธรรมเนียมแตกต่างกันระหว่าง product แบบ Spot และ derivatives

adapter ใช้จำนวนและสกุลเงินของ commission โดยตรงจากฟิลด์ `execFee` และ `feeCurrency` ของ venue ดังนั้นกฎด้านล่าง
อธิบายสิ่งที่ Bybit รายงาน ไม่ใช่การคำนวณภายในเครื่อง

### ค่าธรรมเนียมการเทรด Spot {#spot-trading-fees}

สำหรับการเทรด Spot สกุลเงินของค่าธรรมเนียมขึ้นกับฝั่งของ order และค่าธรรมเนียมเป็น rebate หรือไม่ (ค่าธรรมเนียมติดลบ
สำหรับ maker order):

#### ค่าธรรมเนียมปกติ (บวก) {#normal-fees-positive}

- **BUY order**: คิดค่าธรรมเนียมเป็น **base currency** (เช่น BTC สำหรับ BTCUSDT)
- **SELL order**: คิดค่าธรรมเนียมเป็น **quote currency** (เช่น USDT สำหรับ BTCUSDT)

#### Maker rebate (ค่าธรรมเนียมติดลบ) {#maker-rebates-negative-fees}

เมื่อ maker fee ติดลบ (rebate) ตรรกะของสกุลเงินจะ **กลับด้าน**:

- **BUY order ที่ได้ maker rebate**: ได้รับ rebate เป็น **quote currency** (เช่น USDT สำหรับ BTCUSDT)
- **SELL order ที่ได้ maker rebate**: ได้รับ rebate เป็น **base currency** (เช่น BTC สำหรับ BTCUSDT)

:::note
**taker order ไม่มีการกลับด้านของตรรกะ** แม้ maker fee rate จะติดลบ taker fee เป็นไปตามกฎสกุลเงินค่าธรรมเนียมปกติเสมอ
:::

#### ตัวอย่าง: BTCUSDT Spot {#example-btcusdt-spot}

- **ซื้อ 1 BTC แบบ taker (ค่าธรรมเนียม 0.1%)**: จ่ายค่าธรรมเนียม 0.001 BTC
- **ขาย 1 BTC แบบ taker (ค่าธรรมเนียม 0.1%)**: จ่ายค่าธรรมเนียมเป็น USDT ในมูลค่าเทียบเท่า
- **ซื้อ 1 BTC แบบ maker (rebate -0.01%)**: ได้รับ rebate เป็น USDT (กลับด้าน)
- **ขาย 1 BTC แบบ maker (rebate -0.01%)**: ได้รับ rebate เป็น BTC (กลับด้าน)

### ค่าธรรมเนียมการเทรด derivatives {#derivatives-trading-fees}

สำหรับ product แบบ derivatives ทั้งหมด (LINEAR, INVERSE, OPTION) จะคิดค่าธรรมเนียมเป็น **settlement currency** เสมอ:

| Product Type | Settlement Currency              | Fee Currency |
| ------------ | -------------------------------- | ------------ |
| LINEAR       | USDT (โดยทั่วไป)                 | USDT         |
| INVERSE      | Base coin (เช่น BTC สำหรับ BTCUSD) | Base coin  |
| OPTION       | USDT หรือ USDC                   | Settle coin  |

### ข้อมูลค่าธรรมเนียมที่ขาดหาย {#missing-fee-data}

private channel `execution.fast` ของ Bybit ไม่มีฟิลด์ค่าธรรมเนียมและ execution type ดังนั้น fill report ที่ parse
จาก channel นั้นจะมี commission เป็นศูนย์ ให้ subscribe channel `execution` มาตรฐานเมื่อต้องการข้อมูลค่าธรรมเนียม
ที่แม่นยำ

### เอกสารทางการ {#official-documentation}

รายละเอียดทั้งหมดเกี่ยวกับโครงสร้างค่าธรรมเนียมและกฎสกุลเงินของ Bybit ดูได้ที่:

- [Bybit WebSocket Private Execution](https://bybit-exchange.github.io/docs/v5/websocket/private/execution)
- [Bybit Spot Fee Currency Instruction](https://bybit-exchange.github.io/docs/v5/enum#spot-fee-currency-instruction)

## การตั้งค่า {#configuration}

ต้องระบุ product type ของแต่ละ client ใน config

### ตัวเลือกการตั้งค่าของ data client {#data-client-configuration-options}

| ตัวเลือก                           | Default    | คำอธิบาย                                                                                                        |
| ---------------------------------- | ---------- | --------------------------------------------------------------------------------------------------------------- |
| `product_types`                    | `[LINEAR]` | ลำดับของค่า `BybitProductType` ที่จะเปิดใช้                                                                     |
| `environment`                      | `MAINNET`  | enum ของ Bybit environment ใช้ `BybitEnvironment.MAINNET`, `BybitEnvironment.DEMO` หรือ `BybitEnvironment.TESTNET` |
| `api_key`                          | `None`     | API key โหลดจาก environment variable ที่ตรงกันเมื่อไม่ได้ระบุ                                                   |
| `api_secret`                       | `None`     | API secret โหลดจาก environment variable ที่ตรงกันเมื่อไม่ได้ระบุ                                                |
| `base_url_http`                    | `None`     | override REST base URL                                                                                          |
| `base_url_ws_public`               | `None`     | override public WebSocket URL                                                                                   |
| `base_url_ws_private`              | `None`     | override private WebSocket URL                                                                                  |
| `proxy_url`                        | `None`     | proxy URL (ไม่บังคับ) สำหรับ transport แบบ HTTP และ WebSocket                                                    |
| `http_timeout_secs`                | `60`       | timeout (วินาที) ของ REST request                                                                               |
| `max_retries`                      | `3`        | จำนวนครั้งสูงสุดที่ retry REST request                                                                          |
| `retry_delay_initial_ms`           | `1,000`    | delay เริ่มต้นของการ retry (มิลลิวินาที)                                                                        |
| `retry_delay_max_ms`               | `10,000`   | delay สูงสุดของการ retry (มิลลิวินาที)                                                                          |
| `heartbeat_interval_secs`          | `20`       | ช่วง heartbeat (วินาที) ของ WebSocket client                                                                    |
| `recv_window_ms`                   | `5,000`    | receive window (มิลลิวินาที) ของ signed REST request                                                            |
| `update_instruments_interval_mins` | `60`       | ช่วงเวลา (นาที) ระหว่างการ refresh instrument catalog                                                           |
| `instrument_status_poll_secs`      | `60`       | ช่วงเวลา (วินาที) ระหว่างการ poll instrument และ status ค่า `0` ปิดการ poll                                      |
| `book_snapshot_timeout_secs`       | `10`       | เวลารอ snapshot ตอนเริ่มต้น, reconnect และกู้คืน ค่า `0` ปิดการรอ                                                |
| `transport_backend`                | `Sockudo`  | transport backend ของ WebSocket                                                                                 |
| `bars_timestamp_on_close`          | `True`     | ใส่ timestamp ของ bar ที่ตอนปิดช่วงเวลา ค่า `False` ใส่ timestamp ที่ตอนเปิดช่วงเวลา                            |

### ตัวเลือกการตั้งค่าของ execution client {#execution-client-configuration-options}

| ตัวเลือก                    | Default    | คำอธิบาย                                                                                                        |
| --------------------------- | ---------- | --------------------------------------------------------------------------------------------------------------- |
| `product_types`             | `[LINEAR]` | ลำดับของค่า `BybitProductType` ที่จะเปิดใช้                                                                     |
| `environment`               | `MAINNET`  | enum ของ Bybit environment ใช้ `BybitEnvironment.MAINNET`, `BybitEnvironment.DEMO` หรือ `BybitEnvironment.TESTNET` |
| `api_key`                   | `None`     | API key โหลดจาก environment variable ที่ตรงกันเมื่อไม่ได้ระบุ                                                   |
| `api_secret`                | `None`     | API secret โหลดจาก environment variable ที่ตรงกันเมื่อไม่ได้ระบุ                                                |
| `base_url_http`             | `None`     | override REST base URL                                                                                          |
| `base_url_ws_private`       | `None`     | override private WebSocket base URL                                                                             |
| `base_url_ws_trade`         | `None`     | override trade WebSocket base URL                                                                               |
| `proxy_url`                 | `None`     | proxy URL (ไม่บังคับ) สำหรับ transport แบบ HTTP และ WebSocket                                                    |
| `http_timeout_secs`         | `60`       | timeout ของ REST request และ deadline รวมของการยืนยัน cancel-all (วินาที)                                       |
| `max_retries`               | `3`        | จำนวนครั้งสูงสุดที่ retry REST request                                                                          |
| `retry_delay_initial_ms`    | `1,000`    | delay เริ่มต้นของการ retry REST และช่วงการ poll ของ cancel-all (มิลลิวินาที)                                     |
| `retry_delay_max_ms`        | `10,000`   | delay สูงสุดของการ retry (มิลลิวินาที)                                                                          |
| `heartbeat_interval_secs`   | `5`        | ช่วง heartbeat (วินาที) ของ WebSocket client                                                                    |
| `auth_timeout_secs`         | `None`     | timeout ของการ authenticate WebSocket (วินาที) ไม่บังคับ                                                        |
| `recv_window_ms`            | `5,000`    | receive window (มิลลิวินาที) ของ signed REST และ trade WebSocket request                                        |
| `account_id`                | `None`     | account ID ที่ผูกกับ client นี้ (ไม่บังคับ)                                                                     |
| `use_spot_position_reports` | `False`    | รายงาน balance ของ Spot wallet เป็น position สำหรับ request แบบเจาะจง bulk report จะไม่รวม Spot (ระบุคู่ไม่ได้)  |
| `auto_repay_spot_borrows`   | `False`    | ชำระคืนการกู้ยืม Spot margin ที่ติดตามไว้โดยอัตโนมัติหลัง BUY order ถูก fill ทั้งหมด                              |
| `margin_mode`               | `None`     | การตั้งค่า unified margin mode ของ account                                                                      |
| `smp_type`                  | `None`     | self-match prevention ที่ส่งไปในทุก order ดู [SMP](#self-match-prevention)                                      |
| `transport_backend`         | `Sockudo`  | transport backend ของ WebSocket                                                                                 |

ค่า default ที่ compile ไว้คือ Sockudo เมื่อเปิด Cargo feature `transport-sockudo` มิฉะนั้นจะเป็น Tungstenite

ใช้ `BybitDataClientConfig` คู่กับ `BybitDataClientFactory` และใช้ `BybitExecutionClientConfig` คู่กับ
`BybitExecutionClientFactory` ดูตัวอย่างการตั้งค่า node พร้อม data client และ execution client แบบเต็มได้ที่
[ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/bybit/examples/)
และ [รัน live trading (Rust)](../how_to/run_rust_live_trading.md)

### API credential {#api-credentials}

มีสองวิธีในการให้ credential แก่ Bybit client คือส่งค่า `api_key` และ `api_secret` ไปใน config object หรือตั้ง
environment variable ต่อไปนี้:

สำหรับ Bybit live client ตั้งได้ดังนี้:

- `BYBIT_API_KEY`
- `BYBIT_API_SECRET`

สำหรับ Bybit demo client ตั้งได้ดังนี้:

- `BYBIT_DEMO_API_KEY`
- `BYBIT_DEMO_API_SECRET`

สำหรับ Bybit testnet client ตั้งได้ดังนี้:

- `BYBIT_TESTNET_API_KEY`
- `BYBIT_TESTNET_API_SECRET`

:::tip
แนะนำให้ใช้ environment variable ในการจัดการ credential
:::

เมื่อเริ่ม trading node จะได้รับการยืนยันทันทีว่า credential ถูกต้องและมี permission สำหรับการเทรดหรือไม่

## การมีส่วนร่วม {#contributing}

:::info
หากต้องการฟีเจอร์เพิ่มเติมหรือต้องการร่วมพัฒนา Bybit adapter ดูได้ที่
[contributing guide](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
