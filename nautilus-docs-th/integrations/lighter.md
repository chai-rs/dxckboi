# Lighter

[Lighter](https://lighter.xyz) เป็น exchange แบบ central-limit-order-book ที่กระจายศูนย์ สำหรับ spot และ
perpetual futures โดย settle ผ่าน zero-knowledge rollup บน Ethereum และทำ matching และ sequencing
แบบ off-chain adapter รองรับทั้ง Lighter และ deployment บน Robinhood Chain

crate `nautilus-lighter` มี data client และ execution client ฝั่ง Rust, model ของ REST และ WebSocket
แบบมี type และ L2 signer ในตัวที่ใช้ Schnorr signature บน ECgFp5

สำหรับ benchmark ของ L2 signing และการเปรียบเทียบกับ Go SDK อย่างเป็นทางการ ดู
[`crates/adapters/lighter/benches/BENCHMARKS.md`](../../crates/adapters/lighter/benches/BENCHMARKS.md)
ให้เปรียบเทียบส่วนต่างของ benchmark บนเครื่องเดียวกันเท่านั้น เพราะเวลาสัมบูรณ์จะแตกต่างกันไป

## ภาพรวม {#overview}

- `LighterRawHttpClient`: REST client ระดับล่างสำหรับ endpoint สาธารณะและ endpoint ของ account
- `LighterHttpClient`: REST client ระดับ domain ที่ parse instrument, trade, bar, funding และ book
  เป็น model type ของ Nautilus และคืน payload ของ venue order และ account แบบมี type
- `LighterWebSocketClient`: stream สาธารณะของตลาดและ stream ส่วนตัวของ account ที่ reconnect ได้เอง
- `LighterDataClient`: instrument, trade, quote และ book แบบ L2 market-by-price (MBP)
- `LighterExecutionClient`: execution client ของ Nautilus สำหรับ account stream, การ submit order,
  การแก้ไข, การ cancel และ reconciliation report
- `LighterDataClientFactory` และ `LighterExecutionClientFactory`: การเชื่อม factory เข้ากับ live node

live node ใช้งาน data client และ execution client ผ่าน Rust trait

## ตัวอย่าง {#examples}

ตั้งค่าคงที่ระดับ module ของ tester แต่ละตัว ได้แก่ `LIGHTER_DEPLOYMENT`, `LIGHTER_ENVIRONMENT` และค่าคงที่อื่น
ก่อนรัน tester ของ Rust จะเชื่อมต่อทันที และตัวเลือกเหล่านี้ไม่ได้อ่านจาก environment variable

:::warning
execution tester จะ submit order โดยค่าเริ่มต้น: Rust ใช้ `DRY_RUN = false` บน Lighter Mainnet
account ที่มีเงินสามารถวาง order จริงได้บน mainnet deployment ทั้งสองแบบ ให้ตรวจ instrument, quantity,
deployment และ environment หรือเปิด `DRY_RUN` เพื่อเชื่อมต่อโดยไม่ submit order
:::

ตัวอย่าง Rust อยู่ใน `crates/adapters/lighter/examples/`:

```bash
cargo run --example lighter-data-tester --package nautilus-lighter --features examples
cargo run --example lighter-exec-tester --package nautilus-lighter --features examples
```

### การล้าง account ฉุกเฉิน {#emergency-account-cleanup}

`cargo run --bin lighter-flatten -p nautilus-lighter` จะส่งการ cancel ทั้ง account ทันที จากนั้นอ่าน
snapshot ของ position หนึ่งครั้งและส่งคำสั่งปิดแบบ reduce-only immediate-or-cancel (IOC) สำหรับ position เหล่านั้น

:::warning
หยุด writer อื่นของ account ก่อนรันคำสั่งนี้ การล้างเป็นแบบทั้ง account ไม่ได้จำกัดเฉพาะ strategy
ดังนั้นให้ตรวจ account และ position ที่ active อยู่ก่อน
:::

คำสั่งนี้ไม่ได้ยืนยันหรือ retry request ความสำเร็จหมายถึงการ submit โดยไม่มี error ที่ทราบ ไม่ได้หมายถึง
account ที่ flat แล้ว ให้ตรวจ account state หลังจากนั้นและรันซ้ำหากยังมีอะไรเหลืออยู่

การรันแต่ละครั้งอนุญาตให้ปิดได้สูงสุด 15 รายการ เพราะการ cancel ใช้หนึ่งช่องใน nonce window ขนาด 16 transaction
snapshot ที่ไม่สมบูรณ์และความล้มเหลวของ request หรือการ submit จะคืน error รอบการปิดจะ timeout หลัง 15 นาที

ตั้ง `LIGHTER_DEPLOYMENT` เป็น `lighter` หรือ `robinhood` และ `LIGHTER_ENVIRONMENT` เป็น `mainnet` หรือ `testnet`
หากไม่ระบุ ค่าเริ่มต้นจะเป็น Lighter Mainnet และเลือก namespace ของ credential ที่ตรงกัน

## ผลิตภัณฑ์ที่รองรับ {#product-support}

ตาราง capability อธิบายสิ่งที่ adapter รองรับ *Lighter ไม่รองรับ* หมายถึงข้อจำกัดของ API ของ venue
ส่วน *ยังไม่ได้ implement* หมายถึงช่องว่างของ adapter

| ประเภทผลิตภัณฑ์     | Data feed | Trading | หมายเหตุ                                                          |
| ----------------- | --------- | ------- | ----------------------------------------------------------------- |
| Spot              | ✓         | ✓       | ตลาด spot ตลาดใหม่ใช้ id แบบ 64-bit เริ่มจาก 4095                 |
| Perpetual futures | ✓         | ✓       | linear perpetual ตลาดใหม่ใช้ id แบบ 64-bit เริ่มจาก 4095          |
| Dated futures     | -         | -       | *Lighter ไม่รองรับ*                                               |
| Options           | -         | -       | *Lighter ไม่รองรับ*                                               |

## ข้อจำกัด {#limitations}

- กลุ่ม OTO, OCO และ OTOCO แบบ native ต้องเป็นไปตาม[รูปแบบที่รองรับ](#contingent-orders) กลุ่มแบบ native
  ไม่รองรับความสัมพันธ์แบบ OUO ของ Nautilus
- list ที่ไม่ใช่ contingent, การแก้ไขแบบ batch และการ cancel แบบ batch ที่ระบุชัดเจน จะถูกแบ่งเป็น
  WebSocket batch ละไม่เกิน 15 transaction ส่วน contingent list แบบ native ใช้ grouped transaction เดียว
- `CancelAllOrders` จะ cancel open order ใน cache ข้ามทุก strategy บน account ของ client โดยคง strategy
  ของแต่ละ order ไว้ และกรองตาม instrument และ `order_side` (ถ้าระบุ) มันใช้การ cancel ทีละ order เป็น batch
  ละไม่เกิน 15 schema ของ cancel-all แบบ native ที่ adapter ใช้เป็นแบบทั้ง account
  [signer รุ่นใหม่](https://apidocs.lighter.xyz/docs/wasm-signer#signcancelallorders) ของ Lighter เพิ่ม filter
  ของตลาดแต่ไม่มี filter ของ side
- `query_account` และ position report จะ replay state ล่าสุดของ private WebSocket ที่อยู่ใน cache
- order reconciliation ที่ไม่ได้ระบุขอบเขตจะครอบคลุมเฉพาะตลาดที่ active ซึ่งตั้งค่าไว้หรือสังเกตพบ
  เพื่อหลีกเลี่ยงการกระจาย request ไปทั้ง venue ภายใต้โควตา REST ระดับ Standard

## Symbology {#symbology}

Lighter ใช้ค่า `market_index` แบบ 64-bit ตลาดเดิมจะคง ID ที่แบ่งตามช่วงไว้ หลังจาก
[การอัปเกรดของ Lighter Mainnet ในเดือนกันยายน 2026](https://t.me/lighter_api_updates/174) และ
[การอัปเกรดของ Robinhood ที่ตามมา](https://t.me/lighter_api_updates/184) ตลาด spot และ perpetual ใหม่
จะได้ index ว่างถัดไปที่มีค่าตั้งแต่ `4095` ขึ้นไป

ประเภทผลิตภัณฑ์มาจาก `market_type` เท่านั้น adapter โหลด `GET /api/v1/orderBookDetails` และ map
symbol ของ venue ไปเป็นค่า `InstrumentId` ของ Nautilus

| ผลิตภัณฑ์ของ deployment | รูปแบบ symbol ของ Nautilus                | ตัวอย่าง                             | หมายเหตุ                    |
| ------------------- | --------------------------------------- | ---------------------------------- | --------------------------- |
| Lighter perpetual   | `{BASE}-PERP.LIGHTER`                   | `BTC-PERP.LIGHTER`                 | symbol ดิบของ venue `BTC`   |
| Lighter spot        | `{BASE}/{QUOTE}-SPOT.LIGHTER`           | `ETH/USDC-SPOT.LIGHTER`            | symbol ดิบ `ETH/USDC`       |
| Robinhood perpetual | `{BASE}-PERP.LIGHTER_ROBINHOOD`         | `SNDK-PERP.LIGHTER_ROBINHOOD`      | symbol ดิบของ venue `SNDK`  |
| Robinhood spot      | `{BASE}/{QUOTE}-SPOT.LIGHTER_ROBINHOOD` | `SNDK/USDG-SPOT.LIGHTER_ROBINHOOD` | symbol ดิบ `SNDK/USDG`      |

suffix ใช้แยก listing ของ spot และ perpetual ออกจากกัน request ขาออกจะตัด suffix ออกและใช้
`market_index` ที่อยู่ใน cache symbol ของ spot จะคงคู่เงินของ venue ไว้

## Deployment และ environment {#deployments-and-environments}

| Deployment | Environment | REST URL                              | WebSocket URL                              | L2 signing chain ID | Settlement | Venue เริ่มต้น       |
| ---------- | ----------- | ------------------------------------- | ------------------------------------------ | ------------------- | ---------- | ------------------- |
| Lighter    | Mainnet     | `https://mainnet.zklighter.elliot.ai` | `wss://mainnet.zklighter.elliot.ai/stream` | 304                 | USDC       | `LIGHTER`           |
| Lighter    | Testnet     | `https://testnet.zklighter.elliot.ai` | `wss://testnet.zklighter.elliot.ai/stream` | 300                 | USDC       | `LIGHTER`           |
| Robinhood  | Mainnet     | `https://api.rh.lighter.xyz`          | `wss://api.rh.lighter.xyz/stream`          | 466324              | USDG       | `LIGHTER_ROBINHOOD` |
| Robinhood  | Testnet     | `https://api.rh-testnet.lighter.xyz`  | `wss://api.rh-testnet.lighter.xyz/stream`  | 300                 | USDG       | `LIGHTER_ROBINHOOD` |

chain ID เหล่านี้เป็นค่า signing domain ของ Lighter L2 ไม่ใช่ chain ID ของเครือข่าย EVM

`LighterDeployment::Lighter` หรือ `Robinhood` และ `LighterEnvironment::Mainnet` หรือ `Testnet` เป็นตัวกำหนด
URL เริ่มต้น, chain ID, สกุลเงิน settlement, venue และ policy ของ attribution testnet ทั้งสองใช้ chain ID 300
และ adapter จะไม่อนุมานพฤติกรรมของ deployment จาก ID นั้น

การ override URL สำหรับ private gateway หรือ test fixture ในเครื่องมีผลเฉพาะกับ transport โดยยังคงการ signing,
สกุลเงิน settlement และ attribution ของ deployment ไว้

data client จะต่อท้าย `readonly=true` เข้ากับ WebSocket URL ส่วน execution client ไม่เปลี่ยน URL

### Venue identity แบบกำหนดเอง {#custom-venue-identity}

ตั้งค่า `venue` ใน config ของ client ทั้งสองตัวเพื่อให้มี identity ของ Nautilus ที่แยกกันสำหรับ endpoint
ที่ใช้ protocol ของ Lighter มันกำหนดขอบเขตของ instrument, entry ใน cache, message topic, state ของ socket
และการ route execution โดยไม่เปลี่ยนพฤติกรรมของ protocol ของ deployment

ชื่อ factory ยังคงเป็น `LIGHTER` เพื่อความเข้ากันได้ `ClientId` เป็นชื่อการลงทะเบียน client และรับชื่อที่
กำหนดเองได้ สำหรับการ route ไปยัง Robinhood ด้วย `ClientId` ให้ลงทะเบียนเป็น `LIGHTER_ROBINHOOD`
ตัวอย่างจะได้ค่านี้มาจาก `LIGHTER_DEPLOYMENT`

Nautilus route command ของ account ตาม issuer ดังนั้น issuer ของ `account_id` ต้องตรงกับ venue ที่ resolve ได้:
`LIGHTER_RH_ALT` ต้องใช้ ID เช่น `LIGHTER_RH_ALT-001` venue แบบกำหนดเองไม่ได้เปิดให้ใช้ chain ID หรือ
attribution แบบกำหนดเอง

## การตั้งค่า account และ API key {#account-and-api-key-setup}

ข้อมูลสาธารณะไม่ต้องใช้ account ส่วน private stream และ execution ต้องใช้ account index, API key index
และ private key จาก deployment และ environment เดียวกัน:

| Deployment | Environment | หน้า account และ API key                                         | Account issuer      | Credential prefix             |
| ---------- | ----------- | --------------------------------------------------------------- | ------------------- | ----------------------------- |
| Lighter    | Mainnet     | [Lighter Mainnet](https://app.lighter.xyz/apikeys)              | `LIGHTER`           | `LIGHTER_*`                   |
| Lighter    | Testnet     | [Lighter Testnet](https://testnet.app.lighter.xyz/apikeys)      | `LIGHTER`           | `LIGHTER_TESTNET_*`           |
| Robinhood  | Mainnet     | [Robinhood Mainnet](https://robinhoodchain.lighter.xyz/apikeys) | `LIGHTER_ROBINHOOD` | `LIGHTER_ROBINHOOD_*`         |
| Robinhood  | Testnet     | [Robinhood Testnet](https://rhctestnet.lighter.xyz/apikeys)     | `LIGHTER_ROBINHOOD` | `LIGHTER_ROBINHOOD_TESTNET_*` |

อย่าใช้ account หรือ key ปนกันข้ามแถว แม้แต่ระหว่าง testnet ที่ใช้ signing chain ID 300 เหมือนกัน

1. sign in เข้า deployment เป้าหมาย เลือกหรือสร้าง trading account และ sub-account ที่ต้องการ
   ก่อนสร้าง API key
1. ทำตาม
   [การค้นหา account index](https://apidocs.lighter.xyz/docs/get-started#find-your-account-index)
   ของ Lighter กับ REST URL ของ deployment แทนที่ URL ของ Robinhood Mainnet ด้านล่างสำหรับ
   [deployment อื่น](#deployments-and-environments):

   ```bash
   LIGHTER_SETUP_API_URL="https://api.rh.lighter.xyz"
   LIGHTER_SETUP_L1_ADDRESS="0xYOUR_ETHEREUM_ADDRESS"

   curl -sS --get \
     "${LIGHTER_SETUP_API_URL}/api/v1/accountsByL1Address" \
     --data-urlencode "l1_address=${LIGHTER_SETUP_L1_ADDRESS}"
   ```

   อ่านค่า `index` ของ entry ใน `sub_accounts` ที่เลือก wallet หนึ่งอาจเป็นเจ้าของ main account และ
   sub-account หลายตัว ซึ่งแต่ละตัวมี index และ key แยกกัน
1. ในหน้า API key ของ account ที่เลือก ให้กด **Generate API Key** ใช้ index ที่ยังไม่ถูกใช้ในช่วง `[4, 254]`
   [เอกสาร API key](https://apidocs.lighter.xyz/docs/api-keys) ของ Lighter สงวน `[0, 3]` ไว้สำหรับ
   interface ของตัวเอง และ `255` เป็นค่า sentinel สำหรับ query API Robinhood ยังสงวน `157` ไว้ด้วย ดู
   [Robinhood API keys](https://apidocs.lighter.xyz/docs/lighter-rh#api-keys)
1. บันทึก private key ที่สร้างขึ้นก่อนปิดหน้าต่าง Lighter จะไม่แสดงมันอีก
1. ตั้งค่า `account_index`, `api_key_index` และ `private_key` โดยตรง หรือผ่าน
   [environment variable](#api-credentials) `account_id` ของ Nautilus แยกจาก index ของ venue:
   ให้ใช้ issuer ตามตาราง เช่น `LIGHTER-001` หรือ `LIGHTER_ROBINHOOD-001`
1. ยืนยันว่า deployment เป้าหมายรู้จัก account index และ key index ที่เลือก:

   ```bash
   LIGHTER_SETUP_ACCOUNT_INDEX="123456"
   LIGHTER_SETUP_API_KEY_INDEX="4"

   curl -sS --get \
     "${LIGHTER_SETUP_API_URL}/api/v1/apikeys" \
     --data-urlencode "account_index=${LIGHTER_SETUP_ACCOUNT_INDEX}" \
     --data-urlencode "api_key_index=${LIGHTER_SETUP_API_KEY_INDEX}"
   ```

หากสำเร็จจะคืน `"code": 200` และ key ที่เลือก ซึ่งยืนยัน index ได้โดยไม่เปิดเผยหรือตรวจสอบ private key

:::warning
API key ของ Lighter ให้สิทธิ์ในการเทรด, เข้าถึง account ส่วนตัว และการถอนเงินบางประเภท ให้เก็บ
private key ไว้ใน secret manager หรือการตั้งค่า environment ที่มีการป้องกัน อย่า commit ลง repository
หรือแสดงใน log
:::

## Integrator attribution {#integrator-attribution}

execution client จะระบุ attribution ของ transaction ประเภท create และ modify ผ่าน `L2TxAttributes` ไปยัง
[integrator account บน Lighter Mainnet](https://apidocs.lighter.xyz/docs/partner-integration) (`723813`) หรือ
[integrator account บน Robinhood Mainnet](https://apidocs.rh.lighter.xyz/docs/partner-attribution) (`17725`)
ของ NautilusTrader ซึ่งครอบคลุมทุก tier รวมถึง Standard และทุก order ใน list ค่าธรรมเนียม integrator
ทั้งฝั่ง maker และ taker เป็นศูนย์

บน mainnet ทั้งสอง key ที่ไม่ใช่แบบ maker-only จะ submit `ApproveIntegrator` ตอน startup ด้วย L2 signature
และค่าธรรมเนียมสูงสุดเป็นศูนย์ ไม่ต้องใช้ L1 signature

testnet ทั้งสองไม่ส่ง integrator attribute และข้าม `ApproveIntegrator` ตอน startup

ระหว่าง startup บน mainnet ใดก็ตาม เมื่อมีรายละเอียดของ account execution client จะพยายามใช้ referral code
`NAUTILUS` กับ L1 address สาธารณะของ account โดยใช้ L2 API key ที่ตั้งค่าไว้ การเรียกนี้เป็นแบบ best effort
โดยมี timeout 10 วินาที และความล้มเหลวจะไม่ขัดขวางการเทรด การเลือก mainnet ใดก็ตามถือว่า account
ยินยอมเข้าร่วม attribution นี้เพื่อสนับสนุนการพัฒนา NautilusTrader testnet ทั้งสองข้าม referral attribution

[referral program](https://docs.lighter.xyz/programs/referral-program) ของ Lighter Mainnet ให้ส่วนลด
ค่าธรรมเนียมการเทรด 10% แก่ผู้ใช้ที่ถูกแนะนำและมีสิทธิ์ โดยจ่ายเป็น rebate venue เป็นผู้ตัดสินการยอมรับ
referral และสิทธิ์

attribution เป็นไปตาม deployment และ environment ที่มี type กำกับ ไม่ว่าจะตั้งชื่อ venue เองอย่างไร

key แบบ maker-only บน mainnet ทั้งสองไม่สามารถ submit `ApproveIntegrator` ได้และจะข้ามการ approve ตอน startup
ก่อนเทรด ให้ approve integrator ด้วย key ที่ไม่ใช่ maker-only บน account เดียวกัน เพราะการ approve มีขอบเขตระดับ account

### การเพิกถอนการ approve {#revoking-the-approval}

หากต้องการเพิกถอนการ approve บน mainnet ใดก็ตามเมื่อเลิกใช้ adapter ให้ส่ง `ApproveIntegrator` ด้วย
`approval_expiry = 0` และค่าธรรมเนียมสูงสุดเป็นศูนย์ การ startup ของ execution client ครั้งถัดไปด้วย key
ที่ไม่ใช่ maker-only จะต่ออายุการ approve แบบไม่มีค่าธรรมเนียมในทุก tier ของ mainnet นั้น คำสั่งด้านล่าง
รองรับเฉพาะ Lighter Mainnet

```bash
export LIGHTER_API_KEY_INDEX=5
export LIGHTER_API_SECRET=REPLACE_ME
export LIGHTER_ACCOUNT_INDEX=123456
cargo run -p nautilus-lighter --bin lighter-integrator-revoke           # Lighter Mainnet
```

source ของ script:
[`crates/adapters/lighter/bin/integrator_revoke.rs`](https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/adapters/lighter/bin/integrator_revoke.rs)

คำสั่ง Rust จะแสดง action และรอให้กด Enter ก่อน sign หรือส่ง หากไม่ถูกต้องให้ยกเลิกด้วย `Ctrl+C`

## Data subscription {#data-subscriptions}

| ประเภทข้อมูล           | Sub.         | Snapshot | Hist. | Nautilus type       | หมายเหตุ                                                     |
| -------------------- | ------------ | -------- | ----- | ------------------- | ------------------------------------------------------------ |
| Instrument metadata  | Cache replay | ✓        | -     | `InstrumentAny`     | โหลดจาก `orderBookDetails`                                   |
| Order book deltas    | ✓            | ✓        | -     | `OrderBookDeltas`   | `L2_MBP` ข้อมูลย้อนหลัง *Lighter ไม่รองรับ*                   |
| Order book depth     | ✓            | -        | -     | `OrderBookDepth`    | top-10 แบบ live ส่วน REST *ยังไม่ได้ implement*              |
| Order book snapshots | -            | ✓        | -     | `OrderBook`         | REST snapshot รวมได้สูงสุด 250 order ต่อฝั่ง                  |
| Quote ticks          | ✓            | -        | -     | `QuoteTick`         | BBO แบบ live ข้อมูลย้อนหลัง *Lighter ไม่รองรับ*              |
| Trade ticks          | ✓            | -        | ✓     | `TradeTick`         | trade จาก WebSocket และประวัติจาก REST `recentTrades` สาธารณะ |
| Bars                 | ✓            | -        | ✓     | `Bar`               | candle stream จาก WebSocket และประวัติจาก REST สำหรับ backfill |
| Mark prices          | ✓            | -        | -     | `MarkPriceUpdate`   | stream สถิติตลาดของ perp                                     |
| Index prices         | ✓            | -        | -     | `IndexPriceUpdate`  | stream สถิติของตลาดและ spot                                  |
| Funding rates        | ✓            | -        | ✓     | `FundingRateUpdate` | ค่าประมาณปัจจุบันและประวัติรายชั่วโมงจาก REST                 |
| Instrument status    | REST         | ✓        | -     | `InstrumentStatus`  | snapshot `active` / `inactive`                               |

### ข้อมูล order book {#order-book-data}

subscription ของ book delta และ depth จะปฏิเสธ type อื่นนอกจาก `BookType::L2_MBP` ก่อนการ subscribe

มีเพียง `subscribed/order_book` เท่านั้นที่ initialize book ของ WebSocket จนกว่าจะได้รับมัน adapter จะทิ้ง
frame `update/order_book` และไม่ emit ข้อมูล book เพราะ incremental ไม่มีระดับราคาที่ไม่เปลี่ยนแปลง

depth และ delta ใช้ stream `order_book` ของ WebSocket ร่วมกัน snapshot หรือ incremental update แต่ละตัว
ที่ยอมรับจะสร้าง depth view แบบ top-10 ที่อัปเดตแล้ว

### Bar {#bars}

bar subscription ใช้ `candle/{market_id}/{resolution}` โดยมี update ของ candle ที่ยังเปิดอยู่ประมาณทุก 500 ms
adapter จะ emit `Bar` ของช่วงเวลาที่ปิดแล้วหนึ่งตัวเฉพาะเมื่อ timestamp เริ่มต้นของ candle เลื่อนไปข้างหน้า
การ reconnect และ unsubscribe จะล้าง cache ของ candle ที่กำลังดำเนินอยู่

REST และ WebSocket รองรับ `1m`, `5m`, `15m`, `30m`, `1h`, `4h`, `12h` และ `1d` สำหรับ bar รายสัปดาห์
ให้รวมจากช่วงที่สั้นกว่า: subscription จะปฏิเสธ `1w` ภายในเครื่อง request REST รับค่านี้ แต่ Lighter Mainnet
คืน `20001 invalid param` และ [REST schema](https://apidocs.lighter.xyz/reference/candles) ไม่มีค่านี้

ประวัติจาก REST จะข้ามแถวที่เป็นช่องว่างซึ่งมีค่า OHLC หายไป, เป็น null หรือไม่เป็นบวก เพราะไม่สามารถ
สร้างเป็น bar ของ Nautilus ที่ถูกต้องได้ แถวที่ถูกต้องที่ตามมายังคงโหลดได้

### Instrument status และ trade {#instrument-status-and-trades}

subscription ของ instrument status จะ replay สถานะจาก `orderBookDetails` ใน cache หรือดึง snapshot จาก REST
Lighter ไม่มี WebSocket stream สำหรับการเปลี่ยนสถานะ

trade subscription ใช้ WebSocket stream สาธารณะ ประวัติใช้ `/api/v1/recentTrades` สาธารณะโดยไม่ต้องใช้
credential adapter จะกรอง trade สูงสุด 100 รายการให้อยู่ในช่วงเวลาที่ร้องขอโดยไม่มี pagination

ดู [Funding rates](#funding-rates) สำหรับความหมายของ funding ทั้งแบบ live และย้อนหลัง

### Data request ที่ไม่รองรับ {#unsupported-data-requests}

`request_quotes` ยังไม่ได้ implement: REST endpoint ที่ adapter ใช้ไม่มี snapshot หรือประวัติของ quote
ที่มี timestamp ซึ่งเหมาะกับ `QuoteTick` ให้ subscribe `ticker` ของ WebSocket เพื่อรับข้อมูล best bid และ
offer แบบ live

`request_book_depth` ยังไม่ได้ implement: book จาก REST ไม่มี timestamp ของ venue สำหรับ `OrderBookDepth.ts_event`
ให้ใช้ `subscribe_book_depth` สำหรับ depth แบบ live หรือ `request_book_snapshot` สำหรับ snapshot `OrderBook` จาก REST

## การกู้คืน order book {#order-book-recovery}

### การตรวจสอบลำดับ {#sequence-validation}

`begin_nonce` ของ incremental update แต่ละตัวต้องตรงกับ `nonce` ของ book ก่อนหน้า หากไม่ตรงจะระงับ
output ของ book และเริ่มการแทนที่ด้วย unsubscribe/subscribe

`offset` ของ venue ไม่ใช่ตัวนับความต่อเนื่อง: มันอาจข้ามค่าและเปลี่ยนไปข้าม server เมื่อ reconnect
ดู [Lighter order book contract](https://apidocs.lighter.xyz/docs/websocket-reference#order-book)

### ข้อกำหนดของ snapshot {#snapshot-requirements}

หลังจากการเขียน subscription ครั้งแรกหรือการแทนที่เสร็จสิ้น adapter จะรอนานสุด
`book_snapshot_timeout_secs` (ค่าเริ่มต้น 10 วินาที) สำหรับ snapshot `subscribed/order_book` แบบมี type
ตั้งเป็น `0` เพื่อปิด deadline

snapshot ที่หายไปจะเริ่มหรือ retry การกู้คืน แม้หลังจากได้ `Already Subscribed` หรือ control acknowledgement
control acknowledgement จะคืนช่อง subscription แต่ไม่ได้ทำให้การกู้คืนเสร็จ

output จะกลับมาเฉพาะเมื่อ snapshot ที่ตรงกันมาแทนที่ระดับราคาที่อยู่ใน cache snapshot ที่ว่างจะล้าง book

### ขีดจำกัดของการ retry และการ reconnect {#retry-limits-and-reconnects}

แต่ละรอบของการกู้คืนจะพยายามแทนที่ได้สูงสุดแปดครั้งภายใน 180 วินาที ด้วย exponential backoff และ jitter
จากนั้นจะ retry ด้วย exponential backoff ในช่วง [1 นาที, 15 นาที] จนกว่าจะยอมรับ snapshot ได้
การเขียน unsubscribe และ subscribe สำหรับการแทนที่ใช้ connection เดียวกัน

การ reconnect จะปลด generation ของ subscription ที่ล้าสมัย แต่คงการกู้คืนและ budget ที่เหลือไว้
หากการกู้คืนใช้ budget หมดแล้วและกำลังรออยู่ มันจะ retry ทันทีบน connection ใหม่

### Consumer และความล้มเหลวแบบต่อเนื่อง {#consumers-and-persistent-failures}

delta และ depth ใช้รอบการกู้คืนร่วมกันสำหรับแต่ละตลาด:

- การนำ consumer ตัวหนึ่งออกจะคงอีกตัวไว้
- การนำ consumer ตัวสุดท้ายออกจะยกเลิกการเขียนที่ค้างอยู่และการรอ snapshot
- การ shutdown จะยกเลิกงานทั้งหมดที่เป็นเจ้าของ

การกู้คืนไม่เคยจบลงในสถานะล้มเหลว การแทนที่ที่ถูกปฏิเสธและ subscription ที่ถูก replay หลัง reconnect
จะ retry ต่อไปด้วยช่วงเวลาที่เพิ่มขึ้น snapshot ที่มาช้ายังคงคืน book ได้

ความล้มเหลวของ venue subscription ที่ไม่ใช่ rate limiting จะทำให้การเรียก subscribe ครั้งแรกที่รออยู่ล้มเหลว
แม้ในระหว่างการกู้คืน การกู้คืนจะดำเนินต่อเฉพาะเมื่อยังมี consumer อื่นเหลืออยู่ subscription ที่ตามมาจะเริ่มใหม่
ส่วนตลาดอื่นจะกู้คืนแยกกันอย่างอิสระ

ดู [Order book recovery ownership](../developer_guide/adapters.md#order-book-recovery-ownership)
สำหรับกลไกกู้คืนที่ใช้ร่วมกันและหน้าที่ของ adapter

### การตรวจสอบการกู้คืนแบบ live {#live-recovery-validation}

`lighter-book-stress` ตรวจการ synchronize และการกู้คืนบน book ของ perpetual บน Lighter mainnet หกตัว
โดยไม่ submit order มันตรวจ contract ของ stream รวมถึงการเพิ่มขึ้นของ nonce ภายในแต่ละรอบของ snapshot
และเปรียบเทียบ 20 ระดับที่ดีที่สุดกับการสร้าง feed ขึ้นใหม่แบบอิสระ

จาก root ของ repository ให้รัน:

```bash
CARGO_BUILD_JOBS=16 bash scripts/strip-adapter-env.bash \
  cargo test -p nautilus-lighter --features examples --test lighter-book-stress -- --timeout 10 --rounds 12
```

`--scenario` ใช้เลือกรูปแบบการรัน:

- `churn` (ค่าเริ่มต้น): ตรวจการกู้คืนโดยไม่มีการ reconnect จากนั้นหมุนเวียนไปตาม nonce gap, snapshot
  ที่หายไปและล่าช้า, การแทนที่ที่ถูกปฏิเสธ, การ reconnect และการ restart ระหว่างการกู้คืน
- `initial`: ทิ้ง snapshot แรกของแต่ละ book โดยใช้ session ใหม่ในแต่ละรอบ
- `boundaries`: ปฏิเสธทุกความพยายามใน retry budget จากนั้นตรวจเพดานของการ retry, การ reconnect
  ที่ยุติการรอเพดาน, การ unsubscribe ระหว่างการกู้คืน และการ shutdown ระหว่างการ reconnect

`--timeout` กำหนด snapshot timeout เป็นวินาที (`0` คือปิด deadline) และ
`--rounds` กำหนดจำนวนรอบ (ค่าเริ่มต้น 12)

harness ต้องใช้ WebSocket stream ของ mainnet และ API สาธารณะ `orderBooks` และ `orderBookDetails`
ดู [Stress harnesses](../developer_guide/spec_data_testing.md#stress-harnesses) สำหรับ flag ที่ใช้ร่วมกัน
และรูปแบบ output

## ความสามารถด้าน order {#order-capabilities}

### การระบุตัวตนของ order {#order-identification}

Lighter ใช้ order index ของ venue แบบตัวเลขและ `client_order_index` ที่ผู้เรียกกำหนด adapter จะสร้าง
index ขนาด 31-bit จาก `ClientOrderId` ของ Nautilus และเลื่อนหาค่าถัดไปเมื่อชนกัน

การ query ใช้ venue order ID สำหรับประวัติของ order ที่ active และที่จบแล้ว ก่อนที่จะทราบ ID นั้น client index
ที่สร้างขึ้นจะ query ได้เฉพาะ order ที่ active การกำหนด client order ID ของผู้เรียกต้องมี binding ของ index
ที่เลือกซึ่งลงทะเบียนไว้ มิฉะนั้น query report จะใช้ client order ID จาก binding ของ venue ที่ตรงกันพอดี
หรือใช้ venue order ID ที่ไม่ซ้ำเป็น external client order ID หากมี active match ซ้ำกันจะล้มเหลวเพราะกำกวม

index ที่ได้จากการเลื่อนหาไม่สามารถสร้างซ้ำได้หลัง restart ก่อนแปล order report และ fill report
reconciliation จะคืน client index จริงผ่าน mapping ของ venue order ID ใน core cache ซึ่งต้องมี order นั้นอยู่
และ cache ต้องยังเก็บ mapping นั้นไว้ open order ใน cache จะกลับมาถูกติดตามแบบ active ส่วน order ที่จบแล้ว
จะใช้การติดตามแบบ replay ที่มีขอบเขต

การกู้คืนจะไม่อนุมาน client order ID จากตัวเลขจำนวนเต็มเพียงอย่างเดียว หากไม่มี binding ที่กู้คืนได้ report
จะใช้ venue order ID ที่ไม่ซ้ำเป็น external client order ID

### Order type {#order-types}

| Order type             | Perpetuals | Spot | หมายเหตุ                                                        |
| ---------------------- | ---------- | ---- | -------------------------------------------------------------- |
| `MARKET`               | ✓          | ✓    | ราคาเพดานคำนวณจาก quote ฝั่งตรงข้ามใน cache + slippage          |
| `LIMIT`                | ✓          | ✓    | ต้องระบุราคา limit                                             |
| `STOP_MARKET`          | ✓          | -    | เฉพาะ perp ราคาเพดานคำนวณจาก `trigger_price` + slippage        |
| `STOP_LIMIT`           | ✓          | -    | เฉพาะ perp map ไปเป็น stop-loss limit order ของ Lighter        |
| `MARKET_IF_TOUCHED`    | ✓          | -    | เฉพาะ perp ราคาเพดานคำนวณจาก `trigger_price` + slippage        |
| `LIMIT_IF_TOUCHED`     | ✓          | -    | เฉพาะ perp map ไปเป็น take-profit limit order ของ Lighter      |
| `MARKET_TO_LIMIT`      | -          | -    | *Lighter ไม่รองรับ*                                            |
| `TRAILING_STOP_MARKET` | -          | -    | *Lighter ไม่รองรับ*                                            |
| `TRAILING_STOP_LIMIT`  | -          | -    | *Lighter ไม่รองรับ*                                            |
| `TWAP`                 | -          | -    | *ยังไม่ได้ implement* ไม่มี mapping ใน Nautilus                |

conditional order ต้องมี `trigger_price` adapter จะปฏิเสธ order ที่ไม่มี trigger, trigger ที่ถูกตัดเหลือ
`0` tick ตาม price precision ของ instrument และ conditional order ทุกตัวบน spot

Lighter ต้องการ `price` ที่แย่ที่สุดที่ยอมรับได้สำหรับ order แบบ market ราคาเพดานเริ่มต้นจาก:

- `MARKET`: ask ใน cache สำหรับฝั่งซื้อ และ bid สำหรับฝั่งขาย หากไม่มี `QuoteTick` ใน cache order จะถูก deny
- `STOP_MARKET` และ `MARKET_IF_TOUCHED`: `trigger_price`

`market_order_slippage_bps` (ค่าเริ่มต้น 50 bps) จะปรับราคาเพดานก่อนปัดให้ตรงกับ price precision ของ instrument:
ปรับขึ้นสำหรับฝั่งซื้อ และปรับลงสำหรับฝั่งขาย override ได้ด้วย `SubmitOrder.params["market_order_slippage_bps"]`

### Contingent order {#contingent-orders}

| ฟีเจอร์                    | Perpetuals | Spot | หมายเหตุ                                                           |
| ------------------------ | ---------- | ---- | ----------------------------------------------------------------- |
| Stop-loss market         | ✓          | -    | `STOP_MARKET` map ไปเป็น `STOP_LOSS` ของ Lighter                  |
| Stop-loss limit          | ✓          | -    | `STOP_LIMIT` map ไปเป็น `STOP_LOSS_LIMIT` ของ Lighter             |
| Take-profit market       | ✓          | -    | `MARKET_IF_TOUCHED` map ไปเป็น `TAKE_PROFIT` ของ Lighter          |
| Take-profit limit        | ✓          | -    | `LIMIT_IF_TOUCHED` map ไปเป็น `TAKE_PROFIT_LIMIT`                 |
| Trigger price            | ✓          | -    | จำเป็นสำหรับ conditional order ทุกตัวที่รองรับ                     |
| Trigger source selection | -          | -    | *Lighter ไม่รองรับ* trigger ด้วย mark price เท่านั้น               |
| Grouped order lists      | ✓          | -    | กลุ่ม OTO, OCO และ OTOCO แบบ native                               |
| OCO / OTO orders         | ✓          | -    | กลุ่มในตลาดเดียวกันที่มี order ลูกแบบ SL/TP                        |
| Bracket orders           | ✓          | -    | parent แบบ OTO พร้อม conditional exit ที่ผูกกันแบบ OCO             |
| `CreateGroupedOrders`    | ✓          | -    | signed transaction และ nonce เดียวสำหรับทุก leg ในกลุ่ม            |

ใช้ `submit_order_list` สำหรับกลุ่มแบบ native ความสัมพันธ์ของ
[`CreateGroupedOrders`](https://apidocs.lighter.xyz/docs/trading#stop-loss-and-take-profit) ของ Lighter
map ไปยัง Nautilus ดังนี้:

| การจัดกลุ่มของ Lighter | Venue code | Parent ใน Nautilus | Order ลูกหรือ order พี่น้องใน Nautilus                              |
| -------------------- | ---------- | ----------------- | ------------------------------------------------------------------ |
| OTO                  | 1          | `OTO`             | order ลูกหนึ่งตัวที่อ้างอิง parent โดยไม่มี contingency กับพี่น้อง     |
| OCO                  | 2          | ไม่มี              | order พี่น้องแบบ `OCO` สองตัวที่ผูกกันแบบสองทาง                      |
| OTOCO                | 3          | `OTO`             | order ลูกสองตัวที่อ้างอิง parent และผูกกันแบบสองทางเป็น `OCO`         |

Nautilus ไม่มี type OTOCO แยกต่างหาก ความสัมพันธ์ OTO และ OCO ต้อง map อย่างชัดเจนเพราะรหัสตัวเลข
ของ venue และของ Nautilus ต่างกัน

#### ข้อกำหนดของกลุ่ม {#group-requirements}

- ทุก leg ใช้ perpetual instrument, strategy และ order-list ID เดียวกัน โดยมี client order ID ไม่ซ้ำกัน
- parent แบบ OTO ใช้ `MARKET` หรือ `LIMIT` order ลูกต้องอยู่ฝั่งตรงข้ามกับ parent, ใช้ `reduce_only` และ
  ขอ quantity เท่ากับ parent
- order พี่น้องแบบ OCO ใช้ `reduce_only` และมี side และ quantity เดียวกัน คู่นี้ประกอบด้วย stop-loss หนึ่งตัว
  (`STOP_MARKET` หรือ `STOP_LIMIT`) และ take-profit หนึ่งตัว (`MARKET_IF_TOUCHED` หรือ `LIMIT_IF_TOUCHED`)

วันหมดอายุของ venue ขึ้นอยู่กับ leg:

- order ลูกใช้วันหมดอายุที่ไม่เป็นศูนย์ร่วมกัน
- parent แบบ market และ IOC limit ใช้วันหมดอายุเป็นศูนย์
- parent แบบ limit อื่นใช้วันหมดอายุเดียวกับ order ลูก

ค่าเริ่มต้นใช้ timestamp เดียว วันหมดอายุ GTD ที่ระบุชัดเจนต้องตรงกับวันหมดอายุที่ไม่เป็นศูนย์อื่นทั้งหมด
ในกลุ่มก่อน dispatch

รูปแบบ wire เป็นไปตาม[การ validate ของ Go SDK](https://github.com/elliottech/lighter-go/blob/9d38261d1a4cc5c7211b383ba07a4d6e41604708/types/txtypes/create_grouped_orders.go)
adapter ยังกำหนดให้ order ลูกของ OTO เป็น reduce-only และขอ quantity ที่ตรงกัน เพื่อคงความหมายของ exit ใน Nautilus

สำหรับ bracket แบบ native ให้เลือก exit แบบ `OCO` และ take-profit แบบ conditional ค่าเริ่มต้นของ
`OrderFactory.bracket` ใช้ exit แบบ `OUO` และ take-profit แบบ `LIMIT` ธรรมดา ดังนั้น adapter จะปฏิเสธ
adapter ไม่ได้แทนการอัปเดต quantity ด้วยการ cancel หรือแปลง resting limit เป็น take-profit แบบ trigger

#### การเปิดใช้งานและการกำหนดขนาดของ order ลูก {#child-activation-and-sizing}

การ execute ของ parent จะเปิดใช้งานและกำหนดขนาดให้ order ลูกของ OTO/OTOCO จนกว่าจะถึงตอนนั้น base amount
บน wire ของ order ลูกจะเป็นศูนย์ แต่ report ของ order ลูกที่ถูกถือไว้ซึ่ง Nautilus submit จะคง quantity ที่ร้องขอไว้
การเปลี่ยน quantity ที่ venue จะ emit `OrderUpdated`

order ลูกที่ถูกถือไว้ไม่สามารถแก้ไขได้ขณะที่ quantity ที่ venue เป็นศูนย์ adapter ไม่มีทั้งการควบคุมการ
release เมื่อ fill เต็ม และการกำหนดขนาดตามสัดส่วนแบบอิสระ

#### พฤติกรรมการ cancel {#cancellation-behavior}

OCO ของ Lighter จะ cancel order พี่น้องเมื่อ order SL/TP **ถูก trigger** แม้จะยังไม่ fill ส่วน OCO ของ Nautilus
จะขอ cancel order คู่เมื่อ fill หรือถูก cancel adapter คงจังหวะเวลาของ venue ไว้โดยไม่สร้าง order พี่น้องขึ้นใหม่
ลำดับของ event อาจทำให้การส่ง cancel ล่าช้า ดู
[Native child fills and position reconciliation](#native-child-fills-and-position-reconciliation)

`cancel_order` บน leg OCO ของ Nautilus ที่ live อยู่จะขอ cancel order คู่ที่ live อยู่ด้วย ทั้งสองเป็น transaction
อิสระที่อาจสำเร็จหรือล้มเหลวแยกกัน: การ cancel order เดียวของ Lighter จะปล่อยให้ order คู่ยังเปิดอยู่

ให้รอ `OrderAccepted` ของ leg เป้าหมาย order คู่ของมันต้องมี binding ของ venue ที่ทราบแล้ว หรือมี grouped
submission index ที่เก็บไว้ `batch_cancel_orders` ก็รวม order คู่ของ OCO ที่ live อยู่ด้วย โดย submit
order ที่เลือกแต่ละตัวเพียงครั้งเดียว

:::warning
ให้คง `StrategyConfig.manage_contingent_orders = false` (ค่าเริ่มต้น) สำหรับกลุ่มแบบ native การเปิด
การจัดการ contingency ภายในเครื่องอาจปรับขนาด order ลูกโดยไม่ขึ้นกับ venue หรือ cancel exit ที่ถูก trigger
แต่ยังไม่ fill เมื่อ venue cancel order คู่ใน OCO ของมัน
:::

#### Trigger event และการกู้คืนความสัมพันธ์ {#trigger-events-and-relationship-recovery}

conditional order ใช้
[trigger ด้วย mark price](https://docs.lighter.xyz/trading/order-types-and-matching#trigger-conditions)
conditional limit จะ emit `OrderTriggered` ส่วน conditional แบบ market จะ fill โดยไม่ผ่านสถานะ `TRIGGERED`
ระหว่างทาง ตามความหมายของ Nautilus

context ของ order ภายในเครื่องหรือ binding ของ venue ใน cache จะคง client ID, parent ID และ linked ID ของ
Nautilus ไว้หลังจากที่ Lighter ล้าง parent link ตอนเปิดใช้งาน client ที่เริ่มใหม่จะคืนความเป็นเจ้าของจาก cache
ก่อน order หรือ fill แรกจาก private stream

conditional limit ที่ถูก trigger แล้วจะคง type ของ Nautilus และราคา trigger ล่าสุดที่ทราบไว้ ส่วนที่มาจาก
ภายนอกและไม่มีราคา trigger ที่กู้คืนได้จะรายงานเป็น order `LIMIT` ที่กำลังทำงาน

หลังถูก trigger การแก้ไขราคาและ quantity จะส่งราคา trigger เป็นศูนย์ ไม่รองรับการเปลี่ยนค่านี้อีก
transaction batch ที่เป็นอิสระจะไม่ได้รับความสัมพันธ์แบบ contingency

### Order option {#order-options}

| Option           | Perpetuals | Spot | หมายเหตุ                                                                  |
| ---------------- | ---------- | ---- | ------------------------------------------------------------------------- |
| `post_only`      | ✓          | ✓    | map ไปเป็น time-in-force แบบ post-only ของ Lighter                        |
| `reduce_only`    | ✓          | -    | ส่งผ่านไปยัง `CreateOrder` ใช้เฉพาะเพื่อลด position ที่มีอยู่               |
| `quote_quantity` | -          | -    | *Lighter ไม่รองรับ* ให้ submit เป็น base quantity แทน                      |
| `display_qty`    | -          | -    | *Lighter ไม่รองรับ* ไม่มี field สำหรับ display quantity แบบ iceberg       |

### Order param ของ adapter {#adapter-order-params}

| Param                                      | Perpetuals | Spot | หมายเหตุ                                                  |
| ------------------------------------------ | ---------- | ---- | -------------------------------------------------------- |
| `market_order_slippage_bps`                | ✓          | ✓    | override ค่าเริ่มต้นใน config สำหรับราคาเพดานของ order แบบ market |
| `self_trade_behavior_mode`                 | ✓          | ✓    | เลือก action เมื่อเกิด self-trade                         |
| `self_trade_equality_mode`                 | ✓          | ✓    | เลือกขอบเขต account สำหรับการป้องกัน self-trade          |
| `post_only` ผ่าน `SubmitOrder.params`       | -          | -    | *ยังไม่ได้ implement* ให้ใช้ flag ของ order               |
| `reduce_only` ผ่าน `SubmitOrder.params`     | -          | -    | *ยังไม่ได้ implement* ให้ใช้ flag ของ order               |

param ของ self-trade รับ string แบบไม่สนตัวพิมพ์เล็ก/ใหญ่บน `SubmitOrder` และ `SubmitOrderList`:

- `self_trade_behavior_mode`: `expire_maker` (ค่าเริ่มต้น), `expire_taker`, `cancel_both` หรือ `reduce`
- `self_trade_equality_mode`: `account_index` (ค่าเริ่มต้น) หรือ `master_account_index` การเทียบระดับ
  master account จะป้องกันการ match ระหว่าง subaccount ของ master account เดียวกันด้วย

adapter ส่งค่าเริ่มต้นทั้งสองอย่างชัดเจน กลุ่มแบบ native แต่ละกลุ่มใช้คู่ค่าเดียวสำหรับทุก leg ชื่อที่ไม่รู้จัก,
ค่าที่ไม่ใช่ string และ `reduce` คู่กับ `master_account_index` จะทำให้ order ถูก deny ก่อน submit
mode ที่ไม่ใช่ค่าเริ่มต้นต้องใช้ค่าธรรมเนียม integrator เป็นศูนย์ ซึ่ง adapter ใช้อยู่แล้ว

`reduce` จะหักลบ quantity ที่ทับซ้อนกันโดยไม่มี fill quantity ของ working order และ report จะแสดงส่วนที่เหลือ
บวกกับ fill จริง การแก้ไขเฉพาะราคาจะคง quantity นี้ไว้

request แก้ไขจะไม่ส่ง mode ของ self-trade สำหรับ perpetual limit order ที่ถูกแก้ไขจนไป take resting order ของ
account เดียวกัน การเปลี่ยนเฉพาะราคา และการเปลี่ยนทั้งราคาและ quantity จะใช้ `expire_maker` ไม่ว่าจะ submit
ด้วยพฤติกรรมใด การเปลี่ยนเฉพาะ quantity, พฤติกรรมของ maker หลังแก้ไข, การคงขอบเขตของการเทียบ และการแก้ไข
order บน spot หรือ conditional order ยังไม่ได้ตรวจสอบ

### Time in force {#time-in-force}

| Time in force  | Perpetuals | Spot | หมายเหตุ                                                                            |
| -------------- | ---------- | ---- | ---------------------------------------------------------------------------------- |
| `GTC`          | ✓          | ✓    | แบบ limit ใช้ `GoodTillTime` แบบ market ใช้ `IOC`                                  |
| `DAY`          | ✓          | ✓    | order แบบ limit และ conditional ใช้วันหมดอายุของ order ที่เป็นบวก                  |
| `GTD`          | ✓          | ✓    | วันหมดอายุแบบ native อยู่ในช่วง 5 นาทีถึง 30 วัน ดู policy managed-GTD ด้านล่าง     |
| `IOC`          | ✓          | ✓    | `MARKET`/`LIMIT` ธรรมดาใช้วันหมดอายุ `0` ส่วน conditional limit ใช้วันหมดอายุของ trigger |
| `FOK`          | -          | -    | *Lighter ไม่รองรับ*                                                                |
| `AT_THE_OPEN`  | -          | -    | *Lighter ไม่รองรับ*                                                                |
| `AT_THE_CLOSE` | -          | -    | *Lighter ไม่รองรับ*                                                                |

`MARKET`, `STOP_MARKET` และ `MARKET_IF_TOUCHED` ใช้ `ImmediateOrCancel` เพราะ Lighter ปฏิเสธ
`GoodTillTime` สำหรับ order แบบ market วันหมดอายุและการรองรับ `IOC` ของ Nautilus ขึ้นอยู่กับ order type:

- `MARKET` ธรรมดาใช้ `OrderExpiry = 0`
- conditional market order จะคงวันหมดอายุที่เป็นบวกไว้จนกว่าจะถูก trigger adapter จะปฏิเสธ `IOC` ของ
  Nautilus สำหรับ order เหล่านี้ เพราะ Lighter สงวน IOC ไว้สำหรับการ execute หลัง trigger
- conditional limit order รับ `IOC` ของ Nautilus ได้: trigger จะรออยู่ด้วยวันหมดอายุที่เป็นบวก จากนั้น
  order ลูกจะ execute เป็น `ImmediateOrCancel`

หากไม่ได้ระบุวันหมดอายุ GTD อย่างชัดเจน `GTC`, `DAY` และ `GTD` แบบ limit รวมถึง `GTC`, `DAY` และ
`IOC` แบบ limit ของ conditional จะใช้เวลาปัจจุบันบวก 28 วัน Lighter ปฏิเสธค่า `-1` ในเส้นทางเหล่านี้ด้วย
`21711 invalid expiry`

#### Policy ของ GTD {#gtd-policy}

เมื่อ `use_gtd = true` (ค่าเริ่มต้น) `expire_time` ของ strategy จะกำหนดวันหมดอายุ `GoodTillTime` ของ venue
อายุที่ยอมรับได้คือ `[5 นาที + 1 วินาที, 30 วัน]` โดยเผื่อหนึ่งวินาทีสำหรับการ sign และ transport

สำหรับอายุที่สั้นกว่านั้น ให้ตั้ง `use_gtd = false` คู่กับ `manage_gtd_expiry = true` บน strategy ที่ submit เท่านั้น
ตัวจัดการ GTD ภายในเครื่องจะขอ cancel เมื่อถึงเวลาหมดอายุของ strategy หากไม่มีตัวจัดการนี้ order อาจค้างอยู่
จนถึงค่าสำรอง 28 วันของ venue

time-in-force บน wire ยังคงเป็น `GoodTillTime`: Lighter ไม่มี `GoodTillCancel` ดังนั้น `use_gtd = false`
จึงไม่สามารถสลับมันได้แบบที่ทำบน Binance

การจัดการภายในเครื่องจะข้ามขั้นต่ำ 5 นาทีของ native แต่จะปฏิเสธวันหมดอายุที่เกิน 28 วัน ซึ่ง venue
จะทำให้หมดอายุก่อน ให้ใช้ GTD แบบ native สำหรับอายุที่ยาวกว่า latency ของการ cancel ที่ venue ยังคงทำให้
การนำ order ออกล่าช้า

### Execution instruction {#execution-instructions}

| Instruction           | Perpetuals | Spot | หมายเหตุ                                                       |
| --------------------- | ---------- | ---- | ------------------------------------------------------------- |
| `post_only`           | ✓          | ✓    | override TIF และส่ง `PostOnly` ของ Lighter                     |
| `reduce_only`         | ✓          | -    | flag สำหรับลด position derivative ที่มีอยู่                    |
| Self-trade prevention | ✓          | ✓    | ตั้งพฤติกรรมและขอบเขต account ผ่าน order param                |

ใช้ `post_only` กับ order แบบ limit adapter ไม่ได้สังเคราะห์ market order แบบ maker-only

การทดสอบบน Lighter Mainnet ยืนยันว่า `reduce_only=true` ปิด perpetual position ได้ การเปิดแบบ reduce-only
ที่ไม่ถูกต้องอาจถูกทิ้งโดยไม่มี venue order report และ reconcile เป็น `INFLIGHT_TIMEOUT` โดยไม่มีเหตุผล
การ reject จาก venue

### ฟีเจอร์ order ขั้นสูง {#advanced-order-features}

| ฟีเจอร์              | Perpetuals | Spot | หมายเหตุ                                                          |
| ------------------ | ---------- | ---- | ---------------------------------------------------------------- |
| Order modification | ✓          | ✓    | แก้ไข quantity, ราคา และราคา trigger บน order ที่ live อยู่       |
| Bracket orders     | ✓          | -    | parent แบบ OTO native พร้อม exit SL/TP ที่ผูกกันแบบ OCO           |
| Iceberg orders     | -          | -    | *Lighter ไม่รองรับ*                                              |
| Trailing stops     | -          | -    | *Lighter ไม่รองรับ*                                              |
| Pegged orders      | -          | -    | *Lighter ไม่รองรับ* ไม่มี type ของ pegged API แบบ native         |
| TWAP orders        | -          | -    | *ยังไม่ได้ implement* ไม่มี mapping ใน Nautilus                  |
| Chase limit orders | -          | -    | *ยังไม่ได้ implement*                                            |
| Atomic baskets     | -          | -    | *ยังไม่ได้ implement*                                            |
| Leverage update    | ✓          | -    | เฉพาะ perp submit tx `UpdateLeverage` ที่ sign แล้ว               |
| Native cancel-all  | ✓          | ✓    | การ cancel ทั้ง account อย่างชัดเจนผ่าน Rust client              |
| Dead man's switch  | ✓          | ✓    | ต่ออายุอัตโนมัติหรือตั้งเวลาเอง ดูข้อควรระวังด้านล่าง              |

#### การ cancel ทั้ง account {#account-wide-cancellation}

[`CancelAllOrders`](https://apidocs.lighter.xyz/docs/wasm-signer#signcancelallorders) ของ Lighter ซึ่งเป็น
transaction type 16 รองรับการ cancel ทั้ง account แบบทันทีและแบบตั้งเวลา Rust execution client มี
`LighterExecutionClient::cancel_all_orders_account()` สำหรับการ cancel ทันที และ
`LighterExecutionClient::schedule_cancel(time)` สำหรับตั้ง, แทนที่ หรือเพิกถอน schedule
method เหล่านี้ต้องเข้าถึง Rust execution client ตัวจริงโดยตรง command ของ `ExecutionClient` ไม่ได้เปิดให้ใช้
`lighter-flatten` ก็ใช้การ cancel ทั้ง account แบบ native ทันทีเช่นกัน

ส่ง `Some(time)` เพื่อตั้งเวลา cancel หรือแทนที่ deadline ที่มีอยู่ และส่ง `None` เพื่อเพิกถอน
deadline เป็น Unix timestamp หน่วยมิลลิวินาที อยู่ระหว่างห้านาทีถึงสิบห้าวันหลังเวลาของนาฬิกาในเครื่อง
ให้เผื่อเวลาสำหรับ rate limiting, latency และ clock skew เพราะ venue อาจปฏิเสธ request ที่ล่าช้า
การเรียกเองจะไม่เริ่มการต่ออายุอัตโนมัติ การหลุดการเชื่อมต่อและการจบ process จะคง deadline ล่าสุดไว้

ตั้งค่า `scheduled_cancel_timeout_secs` ใน config ของ execution client (เช่น `600`) เพื่อเปิดการตั้งและ
ต่ออายุอัตโนมัติบน deployment ใดก็ได้

ใช้ config นี้กับ `LighterExecutionClientFactory` และ [API credential](#api-credentials)
ค่าเริ่มต้น `None` จะไม่แตะ schedule ของ venue รวมถึง deadline จาก session ก่อนหน้า การเพิกถอนด้วยมือ
ทำได้ผ่าน Rust client เมื่อเปิดการต่ออายุ startup จะรอนานสุด 30 วินาทีให้ transaction การตั้งเวลาถูก execute
หลังจากที่ execution WebSocket, nonce และ [account stream](#account-stream-readiness) พร้อมแล้ว
หากการตั้งเวลาล้มเหลว `connect()` จะล้มเหลวและปิด connection ภายในเครื่อง แต่ request ที่อยู่ระหว่างทาง
ยังอาจมีผล การตั้งเวลาและการต่ออายุอัตโนมัติยืนยันการ execute โดยไม่อ่าน schedule ของ account กลับมา

timeout ต้องอยู่ในช่วง `[420, 1_295_940]` วินาที ขอบเขตเหล่านี้เผื่อเวลาสำหรับ clock skew และความล่าช้า
ของ request ภายในช่วง deadline ของ Lighter ให้นาฬิกาของ host sync อยู่เสมอ
การต่ออายุจะทำงานทุก `min((timeout - 300) // 3, 60)` วินาทีขณะที่เชื่อมต่ออยู่และ [nonce พร้อม](#nonce-recovery)
ความล้มเหลวจะถูก log และ retry การหยุดทำงานหรือการแย่งโควตาต่อเนื่องอาจทำให้ deadline ล่าสุดหมดอายุได้
การต่ออายุที่ timeout ยังอาจถูก submit เพื่อรักษาลำดับของ transaction
การ reconnect จะกลับมาต่ออายุ การ stop, reset และ disconnect จะหยุดการต่ออายุโดยไม่เพิกถอน deadline
หลีกเลี่ยงการเปลี่ยน schedule ด้วยมือขณะเปิดการต่ออายุอัตโนมัติ เพราะมันจะแทนที่สิ่งที่เปลี่ยนไว้

:::warning
การ cancel แบบ native มีผลกับ open order ทั้งหมดบน account ที่ตั้งค่าไว้ ข้ามทุกตลาดและทุก strategy
รวมถึง order ที่ไม่มีใน cache ภายในเครื่อง มันไม่ได้ปิด position ให้ใช้ account เฉพาะเมื่อมีแอปพลิเคชันอื่น
เทรดบน Lighter ด้วย
:::

การอ่านจาก venue อาจยังแสดง resting order หลังถึง deadline ที่ตั้งไว้ ให้ถือว่าปิดแล้วก็ต่อเมื่อ venue order update
หรือการอ่านจาก venue ครั้งถัดไปรายงานการ cancel แล้วเท่านั้น
การกวาด resting order แบบอัตโนมัติและความปลอดภัยของการ matching หลังหมดอายุยังไม่ได้รับการพิสูจน์

method ที่เรียกเองใช้ execution WebSocket แบบมีลำดับ และคืน transaction hash หลังจากที่การค้นหา transaction
ยืนยันการยอมรับ event การ cancel มาจาก venue order update การยอมรับเพียงอย่างเดียวไม่ได้พิสูจน์การ cancel
หรือว่า schedule ถูกตั้ง, แทนที่ หรือเพิกถอนแล้ว ให้ยืนยันการ execute ของ transaction และ schedule ของ account
ที่ venue ก่อนจะพึ่งพาการเรียกด้วยมือ
transport error, การค้นหา transaction ล้มเหลว หรือ confirmation timeout อาจทำให้ไม่ทราบผลลัพธ์ ดังนั้น
ให้ตรวจ transaction และ state ของ venue order ก่อน retry
การทิ้งหรือ timeout ของ future จากการเรียกด้วยมือขณะรอผลจะไม่หยุดการ submit แม้ในขณะที่รอโควตา rate limit
สิ่งนี้ใช้กับการเพิกถอนด้วย: การเรียกที่ timeout ยังอาจเพิกถอน schedule ได้ การ shutdown session อาจทิ้ง
request ที่ยังไม่ได้ส่ง ส่วน request ที่อยู่ระหว่างทางแล้วอาจไม่ทราบผลลัพธ์
command `CancelAllOrders` ทั่วไปยังคง filter ของ instrument และ side และใช้การ cancel ทีละ order อย่างชัดเจนต่อไป

[chase limit order และ atomic basket](https://docs.lighter.xyz/trading/order-types-and-matching#advanced-orders)
ของ venue ไม่ได้เปิดให้ใช้ transaction batch ธรรมดาจะ execute order ลูกแต่ละตัวแยกกันอย่างอิสระ

### การทำงานกับ order {#order-operations}

| การทำงาน             | Perpetuals | Spot | หมายเหตุ                                                             |
| ------------------- | ---------- | ---- | ------------------------------------------------------------------- |
| Submit order        | ✓          | ✓    | ส่ง transaction `L2CreateOrder` ที่ sign แล้วผ่าน WebSocket           |
| Submit order list   | ✓          | ✓    | batch อิสระ กลุ่ม contingent แบบ native เฉพาะ perp                   |
| Modify order        | ✓          | ✓    | ส่ง `ModifyOrder` ที่ sign แล้ว report อาจแสดงการ accept ซ้ำ          |
| Batch modify orders | ✓          | ✓    | WebSocket batch ละไม่เกิน 15 modify transaction ที่ sign แล้ว         |
| Cancel order        | ✓          | ✓    | ส่ง transaction `L2CancelOrder` ที่ sign แล้ว                        |
| Cancel all orders   | ✓          | ✓    | cancel order ใน cache ตาม instrument และ side (ถ้าระบุ)              |
| Set leverage        | ✓          | -    | เฉพาะ perp submit tx `UpdateLeverage` ที่ sign แล้ว                   |
| Batch cancel orders | ✓          | ✓    | WebSocket batch ละไม่เกิน 15 cancel transaction ที่ sign แล้ว         |
| Query order         | ✓          | ✓    | ต้องมี credential และค้นหาผ่าน REST                                 |
| Query account       | ✓          | ✓    | replay account state ล่าสุดจาก private WebSocket                    |
| Mass status         | ✓          | ✓    | จำกัดเฉพาะตลาดที่ account active จาก report ของ WS และ REST          |

#### Order list และการทำงานแบบ batch {#order-lists-and-batch-operations}

`SubmitOrderList`, `BatchModifyOrders` และ `BatchCancelOrders` ที่ไม่ใช่ contingent ใช้ WebSocket
`jsonapi/sendtxbatch` order ลูกแต่ละตัวคง signed transaction และการระบุ order event ของตัวเองไว้ โดยไม่มี
การ execute แบบ atomic หรือความสัมพันธ์แบบ contingency

การ validate ภายในเครื่องจะ deny order ลูกของ submit ที่ไม่ถูกต้อง หรือ reject order ลูกของ modify ที่ไม่ถูกต้อง
ส่วนตัวอื่นที่รองรับจะดำเนินต่อ batch หนึ่งมีได้สูงสุด 15 transaction: order ลูกที่รองรับ 31 ตัวโดยปกติจะได้
15, 15 และ 1 การจัดสรร nonce หรือการกู้คืนที่เกิดพร้อมกันอาจทำให้ batch เล็กลง

แต่ละ batch ใช้ token ของ rate limiter และ acknowledgement แยกกัน การ reject มีผลเฉพาะกับ order ลูกของ batch นั้น

adapter จะรอ acknowledgement ของแต่ละ chunk นานสุด 10 วินาทีก่อน sign chunk ถัดไป
หาก timeout การ dispatch จะดำเนินต่อ โดยเก็บ entry ที่ยังไม่ได้ acknowledge ไว้สำหรับ reconciliation

`CancelAllOrders` ใช้เส้นทางนี้และคง side filter ไว้ การแบ่ง command ไม่ได้เพิ่ม
[ขีดจำกัด active order](#active-and-pending-order-limits) ของ venue

contingent list ต้องมีกลุ่มที่รองรับเพียงกลุ่มเดียวซึ่งมี perpetual order สองหรือสามตัว โดยใช้ transaction
`CreateGroupedOrders` และ nonce เดียวกัน leg ใดที่ไม่ถูกต้องจะทำให้ทั้งกลุ่มถูก deny

[trading guide](https://apidocs.lighter.xyz/docs/trading#batch) ของ Lighter ระบุค่าสูงสุดเป็น 50 ขณะที่
[WebSocket reference](https://apidocs.lighter.xyz/docs/websocket-reference#send-batch-tx) ระบุ 15
adapter ใช้ขีดจำกัดที่ต่ำกว่าตามเอกสาร

#### การจัดสรร nonce และการรับ transaction {#nonce-allocation-and-transaction-admission}

แต่ละ batch ใช้ account เดียว, API key เดียว และ nonce ที่ต่อเนื่องกัน
[nonce contract](https://apidocs.lighter.xyz/docs/core-concepts#nonce) ของ Lighter กำหนดให้ nonce เรียงลำดับ
เมื่อ `skip_nonce=0`

window ภายในเครื่องอนุญาตการจัดสรรที่ยังไม่ยืนยันได้ 16 ตัวต่อ key และจะไม่ส่ง transaction ไปยัง venue
แบบไม่เรียงลำดับ order ลูกที่ยังไม่ได้ sign จะรอ capacity หรือการกู้คืน nonce โดยไม่ถูกทิ้ง การ shutdown
จะหยุดการ dispatch และทิ้งงานที่อยู่ในคิว

response ของ batch จะจับคู่ด้วย request ID และ signed transaction hash

acknowledgement ยืนยันการรับ transaction (admission) ไม่ใช่การ accept, การแก้ไข หรือการ cancel ของ order:

- การ reject ก่อน admission ทำให้ batch ล้มเหลวโดยไม่ใช้ nonce response แบบ invalid-nonce จะ refresh
  ค่าฐานของ nonce ด้วย
- หลัง admission transaction อาจสำเร็จหรือล้มเหลวแยกกัน ความล้มเหลวระดับ application จะใช้ nonce ไป
  ส่วนการ reject ของ sequencer ก่อน application อาจทำให้ nonce ไม่ถูกใช้ ซึ่งทำให้ nonce ที่ตามมาบน key นั้นใช้ไม่ได้

ดู [nonce consumption](https://apidocs.lighter.xyz/docs/core-concepts#when-a-rejected-transaction-consumes-its-nonce)

เมื่อการส่งกำกวม adapter จะเก็บ state ที่ pending ไว้สำหรับ reconciliation และไม่ส่ง batch ซ้ำ

#### การยืนยันการแก้ไข {#modification-confirmation}

ให้รอ `OrderUpdated` หรือ `OrderModifyRejected` ก่อนแก้ไข order อีกครั้ง แต่ละ order มีการแก้ไขที่ยังไม่ได้
ข้อสรุปได้เพียงครั้งเดียว ครั้งที่สองจะได้ `OrderModifyRejected` โดยไม่ทำให้ครั้งแรกได้ข้อสรุป

การ cancel ที่ pending อยู่จะปฏิเสธการแก้ไข ระหว่างที่การแก้ไข pending อยู่ การ cancel ยังทำได้ แต่ผลลัพธ์
จะรอจนกว่าการแก้ไขจะได้ข้อสรุป

การยืนยันต้องอาศัยการค้นหา transaction และรูปแบบ venue order ปัจจุบันที่ตรงกัน:

- frame ของ WebSocket order และ fill ที่ได้รับผลกระทบจะรอการ publish `OrderUpdated` หรือ `OrderModifyRejected`
- HTTP report ที่อาจนำการแก้ไขหรือ fill ไปใช้จะรอให้ cache นำ event นั้นไปใช้ก่อน

| Request                              | พฤติกรรมขณะที่การแก้ไข pending อยู่                               |
| ------------------------------------ | --------------------------------------------------------------- |
| การค้นหาสถานะ order ตัวเดียว           | อาจคืน working report ที่ไม่เปลี่ยนแปลงเป็น `PendingUpdate`     |
| order report แบบ bulk และ mass status | กัน order ไว้และทำเครื่องหมายว่าผลลัพธ์ไม่สมบูรณ์                  |
| `query_order`                        | ไม่ emit report                                                 |

report อาจล่าช้าได้หลายวินาที position reconciliation จะกัน instrument ที่ได้รับผลกระทบไว้จนกว่า
execution event ของมันจะไปถึง cache

#### Fill ของ order ลูกแบบ native และ position reconciliation {#native-child-fills-and-position-reconciliation}

fill ของ order ลูกแบบ native ใน OTO/OTOCO จะรอจนกว่า terminal venue order และการครอบคลุม trade จริงที่สมบูรณ์
จะกำหนด quantity ที่มีผลได้ จากนั้น adapter จะ:

1. publish quantity ที่มีผล
1. release fill จริง
1. reconcile สถานะ terminal

order ลูกที่ยังรออยู่จะ publish trigger event เมื่อการแก้ไขของมันได้ข้อสรุป

fill หรือการแก้ไขของ order ลูกที่ยังไม่ได้ข้อสรุปจะกัน fill และ terminal event ที่ตามมาสำหรับ strategy และ
instrument เดียวกันไว้ fill จะถูก release ตามเวลา event ของ venue และใช้ลำดับการมาถึงเมื่อเวลาเท่ากัน
HTTP report ที่อาจนำการเปลี่ยนแปลงเหล่านี้ไปใช้จะรอให้ cache ตามทัน

fill ใน cache และการบัญชีของ position อาจล่าช้าจนกว่า order ลูกจะปิดและการครอบคลุม trade จะสมบูรณ์
ระหว่างความล่าช้านี้:

- query position จะล้มเหลวสำหรับขอบเขตที่มี instrument ที่ได้รับผลกระทบ
- mass status จะละ position report และ synthetic flat report ที่ได้รับผลกระทบ และคงสถานะไม่สมบูรณ์
- snapshot ของ position จาก venue ยังคงมีให้บน private stream

#### การกู้คืนผลลัพธ์ {#outcome-recovery}

order update และการค้นหา transaction ให้ข้อสรุปของผลลัพธ์การ create และ cancel การกู้คืนจะค้นหาสูงสุดสามครั้ง
ห่างกันสองวินาทีขณะที่ action ที่ pending ยังเป็นปัจจุบัน และตรวจ transaction hash, type, account, API key
และ nonce ของการ submit ครั้งแรก

cancel ที่ได้ข้อสรุปแล้วหรือถูกแทนที่จะหยุดการ probe ของมัน รวมถึง request ที่รอโควตา REST อยู่
เมื่อ transaction ของ cancel ถูก execute การกู้คืนจะ retry สถานะของ order ที่หายไปหรือยังไม่ terminal
ภายในการค้นหาสามครั้งเดียวกัน

การแก้ไขที่ถูกรับแล้วแต่ล้มเหลวอย่างชัดเจน ไม่ว่าจะเป็นแบบเดี่ยวหรือแบบ batch จะ emit `OrderModifyRejected`
หนึ่งครั้งและล้าง action ที่ pending ที่ตรงกัน nonce ที่ถูกใช้ไปแล้วจะไม่ถูกย้อนคืน

ผลลัพธ์ที่มาช้าไม่สามารถล้างการแก้ไขที่ใหม่กว่า หรือ reject การแก้ไขที่การกู้คืนยืนยันไปแล้วได้

:::warning
การแก้ไขที่ยังไม่ได้ยืนยันอาจค้างโดยไม่ได้ข้อสรุปไปตลอด session หลังจากค้นหาครบตามขอบเขตแล้ว
มันจะกัน order event และ fill ที่ตามมา และปฏิเสธการแก้ไขเพิ่มเติม adapter จะไม่สมมติว่าสำเร็จและไม่ส่งซ้ำ
ให้ restart และ reconcile state ของ venue ก่อนจัดการ order นั้นอีกครั้ง
:::

probe ของการแก้ไขจะคง identity ของการ submit ครั้งแรกไว้ข้ามการ reconnect ส่วน probe ของการ cancel จะหยุด
เมื่อ connection เปลี่ยน โดย cancel ที่ pending จะคง identity ของการ submit ไว้สำหรับ reconciliation

การกู้คืนการ cancel รวมถึง order ที่ไม่ได้อยู่ใน `PendingCancel` เช่น เป้าหมายของ `CancelAllOrders`
มีเพียง order ใน cache ที่อยู่ใน `PendingCancel` ตอน dispatch เท่านั้นที่จะได้รับ `OrderCancelRejected`
ความล้มเหลวอื่นจะถูก log ไว้

#### การอัปเดต leverage และการตรวจสอบการ sign {#leverage-updates-and-signing-validation}

เรียก `LighterExecutionClient::update_leverage(instrument_id, initial_margin_fraction, margin_mode)`
สัดส่วน margin ใช้ tick ของ venue (สัดส่วน 1e-4): `500` คือ initial margin 5% (leverage 20x)
และ `1000` คือ 10% (10x)

`UpdateLeverage`, `CancelAllOrders`, modify order ที่มี integrator attribute, conditional create order
และ `CreateGroupedOrders` ถูกตรึงระดับ byte เทียบกับ signer `lighter-go` v1.0.12 ที่ compile แล้ว
create, cancel, modify, grouped order และการอัปเดต leverage ครอบคลุม market ID ที่เกิน 16 และ 32 bit

### การ query order และ reconciliation {#order-querying-and-reconciliation}

| ฟีเจอร์                | Perpetuals | Spot | หมายเหตุ                                                          |
| -------------------- | ---------- | ---- | ---------------------------------------------------------------- |
| Query open orders    | ✓          | ✓    | REST `accountActiveOrders` กำหนดขอบเขตตามตลาด                    |
| Query order history  | ✓          | ✓    | REST `accountInactiveOrders` พร้อม pagination แบบ cursor          |
| Order status updates | ✓          | ✓    | order stream จาก private WebSocket บวกกับ status report           |
| Trade history        | ✓          | ✓    | REST `trades` ต้องมี credential สำหรับประวัติของ account          |
| Fill reports         | ✓          | ✓    | payload ของ trade จาก REST และ private WebSocket                  |
| Position reports     | ✓          | -    | เฉพาะ perp replay position stream ที่อยู่ใน cache                |
| Account state        | ✓          | ✓    | replay snapshot ของ account state ที่รวมแล้วใน cache              |
| Mass status          | ✓          | ✓    | รวม order, fill และ position ที่อยู่ใน cache                      |

pagination ของ inactive order และ fill ที่ยืนยันตัวตนแล้วจะปฏิเสธ cursor ที่ซ้ำ และหยุดหลังครบ 1,000 หน้า
fill reconciliation ทำซ้ำได้ข้ามการเรียกแต่ละครั้ง และจะระงับ fill ที่ WebSocket emit ไปแล้ว

report ย้อนหลังจะผูก client index ที่ map แล้วเฉพาะกับ venue order ID ที่ตรงกัน เพื่อป้องกันไม่ให้ index
ที่ถูกนำกลับมาใช้รวม lifecycle ที่ไม่เกี่ยวข้องกันเข้าด้วยกัน

#### ความสมบูรณ์ของ report {#report-completeness}

mass status แบบมีขอบเขตใช้จุดตัดเดียวกันสำหรับ inactive order และ fill ผลลัพธ์ที่สมบูรณ์ต้องมี:

- แหล่งข้อมูล order, fill และ position ที่จำเป็นทั้งหมดสำเร็จ
- fill ย้อนหลังทุกตัว map ไปยัง order ของมันได้

event ที่ถูกกักไว้, การแก้ไขที่ยังไม่ได้ข้อสรุป หรือ update ที่รอให้ cache นำไปใช้ จะทำให้แหล่งข้อมูล order, fill
และ position ที่ถูกกักไว้ไม่สมบูรณ์

ความล้มเหลวของประวัติเพียงอย่างเดียวไม่ได้ยกเลิก active order หรือ position report ที่ระบุชัดเจนที่มีอยู่
fill ย้อนหลังแบบมีขอบเขตที่ไม่มี position report ในขอบเขต จะเป็นไปตามกฎ
[order-only projection](../concepts/execution/reconciliation.md#order-only-fill-projection) ของ engine

[`trades` reference](https://apidocs.lighter.xyz/reference/trades) ไม่ได้รับประกันความลึกของประวัติ
pagination จะไล่ย้อนจาก trade ล่าสุด มีเพียง trade ที่เก่ากว่าจุดเริ่มต้นของ lookback เท่านั้นที่พิสูจน์การครอบคลุม:

- มี trade ที่เก่ากว่าจุดเริ่มต้นของ lookback: ชุด report ยังคงสมบูรณ์
- cursor หมดก่อน: adapter จะ log ช่วงที่ไม่ได้ครอบคลุมและทำเครื่องหมายว่าชุด report ไม่สมบูรณ์
- ไม่มี trade เก็บไว้เลย: ไม่มีอะไรถูกตัดได้ ชุด report จึงยังคงสมบูรณ์

การที่ cursor หมดไม่สามารถแยกแยะระหว่างการถูกตัดกับการไม่มี trade ที่เก่ากว่าได้ account ที่เพิ่งสร้าง
อาจรายงานว่าไม่สมบูรณ์ทั้งที่ไม่มีอะไรหายไป ให้เลือก lookback ที่ venue ให้บริการได้

สำหรับ fill ที่เก่ากว่านั้น [historical data export](https://apidocs.lighter.xyz/docs/historical-data) ของ venue
ให้ข้อมูล trade ของ account ได้ถึง 12 เดือน adapter ไม่ได้อ่าน endpoint `export`

#### การตรวจ position ตอน startup {#startup-position-checks}

การเปิด position ตอน strategy เริ่มต้นอาจทำให้เกิด warning `cached=0, venue=N` ชั่วครู่ หาก
`account_all_positions` มาถึงก่อนที่ fill ของมันจะถูกประมวลผลไม่กี่มิลลิวินาที การนำ fill ไปใช้จะแก้ความ
ไม่ตรงกันนี้โดยไม่สร้าง reconciliation order

## การจัดการ account และ position {#account-and-position-management}

execution client จะยืนยันตัวตนและ subscribe private stream ห้าตัว:

- `account_all_orders`: order status report
- `account_all_trades`: fill report
- `account_all_positions`: snapshot ของ position เริ่มต้นและ update แบบ live
- `account_all_assets`: snapshot ของ balance ราย asset (spot balance บวก collateral ของ perp)
- `user_stats`: สรุป margin ของ account perp (collateral และ balance ที่ใช้ได้)

adapter จะ emit state ที่รวม `account_all_assets` และ `user_stats` แล้ว หลังจากได้รับ frame แรกของทั้งสองแล้วเท่านั้น

การสร้าง client อนุญาตให้ไม่มี credential ได้ แต่ `connect()` ต้องมี `private_key`, `account_index` และ
`api_key_index` private stream และการ refresh nonce เป็นสิ่งที่บังคับ

### Snapshot และ update ของ position {#position-snapshots-and-updates}

perpetual ใช้ net position เดียวต่อตลาด spot balance ใช้ asset state ของ account snapshot
`subscribed/account_all_positions` ที่เชื่อถือได้จะ:

- ทำให้ตลาดที่ไม่มีอยู่ใน snapshot และแถวที่ `position` เป็นศูนย์กลายเป็น flat map `positions` ที่ว่าง
  จะทำให้ทั้ง cache เป็น flat
- คง position ใน cache ไว้สำหรับแถวที่ map ไม่ได้หรือ parse ไม่ได้ เพื่อป้องกัน flat report ที่ผิด

การ reconnect จะทำให้การครอบคลุมตลาดของ snapshot ใช้ไม่ได้ reconciliation แบบมีขอบเขตจะรายงานตลาดที่เคย
แตะต้องแต่ไม่มีใน snapshot ว่า flat ก็ต่อเมื่อมีการครอบคลุมปัจจุบันเท่านั้น แถวที่ map ไม่ได้หรือมีรูปแบบผิด
จะทำให้ mass status ไม่สมบูรณ์แทนที่จะพิสูจน์ว่า flat

frame `update/account_all_positions` แบบ incremental จะแทนที่ position ที่ไม่เป็นศูนย์, ทำให้ค่าศูนย์ที่ระบุชัดเจน
เป็น flat และคงตลาดที่ไม่ได้ระบุไว้ update ที่ว่างจะคง position ทั้งหมดไว้

| ฟีเจอร์                   | Perpetuals | Spot | หมายเหตุ                                                           |
| ----------------------- | ---------- | ---- | ----------------------------------------------------------------- |
| Account balances        | ✓          | ✓    | asset ที่รวมแล้ว + `user_stats` replay จาก cache เมื่อ query       |
| Position state          | ✓          | -    | เฉพาะ perp snapshot เริ่มต้นบวก update แบบ live                   |
| Netting positions       | ✓          | -    | position ของ Nautilus หนึ่งตัวต่อตลาด perpetual                   |
| Cross margin            | ✓          | -    | ส่งผ่าน `LighterPositionMarginMode::Cross`                         |
| Isolated margin         | ✓          | -    | ส่งผ่าน `LighterPositionMarginMode::Isolated`                      |
| Leverage updates        | ✓          | -    | transaction `UpdateLeverage` ที่ sign แล้ว                         |
| Spot margin / borrowing | -          | -    | *ยังไม่ได้ implement* ไม่มี workflow การกู้ยืม                      |
| Collateral-mode changes | -          | -    | *ยังไม่ได้ implement* ให้ตั้งค่าด้วย tool ของ venue                 |
| Deposits / withdrawals  | -          | -    | *ยังไม่ได้ implement* ให้ใช้ tool หรือ API ของ venue                |

[multi-asset margin](https://docs.lighter.xyz/trading/multi-asset-margin) ของ Lighter รองรับ collateral ของ
perpetual ที่ไม่ใช่ USDC เอกสารของมันอธิบาย use case ของ spot แต่ไม่รวม spot ตอนเปิดตัว adapter ไม่ได้เปิดให้ใช้
ทั้งการกู้ยืมบน spot และการเปลี่ยน collateral mode แถว spot-margin จึงอธิบายช่องว่างของ adapter ไม่ใช่ข้อจำกัดของ venue

## การจัดการ liquidation และ ADL {#liquidation-and-adl-handling}

| Event หรือ field              | รองรับ | หมายเหตุ                                                            |
| --------------------------- | ------ | ------------------------------------------------------------------ |
| Liquidation trades          | ✓      | แถว trade ของ account parse เป็น fill ได้ โดยไม่มี event พิเศษ      |
| Deleverage trades           | ✓      | แถว trade ของ account parse เป็น fill ได้ โดยไม่มี event พิเศษ      |
| Liquidation price reporting | ✓      | position report มีค่าที่ venue รายงาน                              |
| ADL event stream            | -      | *ยังไม่ได้ implement* ไม่มี subscription การแจ้งเตือน              |

`PositionStatusReport.liquidation_price` คงค่าทศนิยมของ venue ไว้ รวมถึงศูนย์ synthetic flat report
จะไม่ตั้งค่านี้

adapter ไม่ได้ subscribe
[`notification` stream](https://apidocs.lighter.xyz/docs/websocket-reference#notification) ที่ต้องยืนยันตัวตน
สำหรับการแจ้งเตือน liquidation และ deleverage trade เหล่านี้ใช้เส้นทาง fill report ตามปกติ

## Funding rate {#funding-rates}

`market_stats` ของ perpetual จะ emit `MarkPriceUpdate`, `IndexPriceUpdate` และ `FundingRateUpdate`
field ของ funding แตกต่างกัน:

- `current_funding_rate`: ค่าประมาณที่กำลังจะมาถึงซึ่งใช้ใน update แบบ live
- `funding_rate` และ `funding_timestamp`: การจ่ายครั้งล่าสุดที่เสร็จสิ้นแล้ว

funding update แบบ live ใช้ `interval=60` เพราะ Lighter คิด funding รายชั่วโมง market stats ไม่มีเวลา
settlement ในอนาคต จึงไม่ได้ตั้ง `next_funding_ns` ส่วน `spot_market_stats` ของ spot จะ emit `IndexPriceUpdate`

adapter แปลง funding rate ทั้งแบบ live และย้อนหลังจากเปอร์เซ็นต์ของ Lighter เป็นสัดส่วนของ Nautilus

request ขอ funding ย้อนหลังใช้ `/api/v1/fundings` สาธารณะ ไม่ใช่ `positionFunding` ที่เฉพาะ account:

- resolution `1h` จะตั้ง `interval=60`
- `direction=long` คงเป็นค่าบวก ส่วน `short` จะเป็นค่าลบ
- pagination ครอบคลุมช่วงที่ร้องขอจนถึง[เพดานจำนวนหน้า](#historical-request-limits)ของ adapter
  โดยขึ้นกับ `limit` ที่ระบุชัดเจน

## Account tier {#account-tiers}

account tier เป็นตัวกำหนด latency, rate limit และค่าธรรมเนียม client จะ log tier จาก `GET /api/v1/account`
รวมถึงค่า `account_type` ที่ไม่รู้จัก

ทุก tier บน mainnet ทั้งสองใช้ [integrator attribution แบบไม่มีค่าธรรมเนียม](#integrator-attribution)
โควตาจะไม่เพิ่มขึ้นเองโดยอัตโนมัติ และการ override ภายในเครื่องไม่ได้ให้ขีดจำกัดของ venue ที่สูงขึ้น

สำหรับ [Lighter Mainnet](https://apidocs.lighter.xyz/docs/account-types):

| Tier     | Latency (maker / taker) | REST weighted limit | `sendTx` limit          | ค่าธรรมเนียม (maker / taker)       | หมายเหตุ                                    |
| -------- | ----------------------- | ------------------- | ----------------------- | ------------------------------- | ----------------------------------------- |
| Standard | 0 ms / 300 ms           | 60 req/min          | 60 req/min              | 0 / 0                           | tier เริ่มต้นแบบไม่มีค่าธรรมเนียม             |
| Premium  | 0 ms / 140 ms           | 24,000 req/min      | [4,000, 57,600] req/min | [0.28, 0.40] / [1.96, 2.80] bps | latency ต่ำสุด ปรับตามจำนวน LIT ที่ stake    |
| Plus     | 0 ms / 300 ms           | 24,000 req/min      | 4,000 req/min           | 0.5 / 0.5 bps                   | ขีดจำกัดสูงขึ้น latency มาตรฐาน              |
| Builder  | -                       | 240,000 req/min     | -                       | -                               | throughput ของ REST สูงสุด                 |

ขีดจำกัดและค่าธรรมเนียมของ Premium ปรับตามจำนวน LIT ที่ stake Robinhood มีค่าธรรมเนียมและขีดจำกัดต่างออกไป
โดยมี [Premium tier](https://apidocs.lighter.xyz/docs/lighter-rh#account-tiers) ที่อิงจากปริมาณการเทรด 14 วัน
ให้ยืนยันขีดจำกัดของ deployment ก่อนเพิ่มโควตาภายในเครื่อง ดู [Rate limiting](#rate-limiting)

## Rate limiting {#rate-limiting}

Lighter จำกัดตาม IP และ L1 address data client และ execution client มี REST limiter แยกกันโดยมีค่าเริ่มต้น
ระดับ Standard ให้วางงบ traffic รวมให้อยู่ภายในขีดจำกัดของ venue

[tier](#account-tiers) ที่สูงกว่าต้องตั้งโควตาภายในเครื่องอย่างชัดเจน:

- `rest_quota_per_min`: จำนวนการอ่านต่อนาทีบน client ทั้งสอง ค่าเริ่มต้น 60
- `sendtx_quota_per_min`: จำนวน transaction request ต่อนาทีบน execution client ค่าเริ่มต้น 60 ใน bucket แยก
  ไม่ขึ้นกับ `rest_quota_per_min`

การ override เปลี่ยนเฉพาะจังหวะภายในเครื่อง ข้อมูลสาธารณะยังไม่ต้องยืนยันตัวตนและไม่ได้รับขีดจำกัดระดับ account
ของ venue จากการเพิ่มโควตาภายในเครื่อง

### Request bucket และน้ำหนักของ endpoint {#request-buckets-and-endpoint-weights}

HTTP และ WebSocket ใช้ transaction bucket ของ venue ร่วมกันหนึ่งตัวต่อ L1 address WebSocket `sendTxBatch`
นับเป็นหนึ่งครั้งสำหรับ transaction ของ adapter สูงสุด 15 ตัว

account ระดับ Standard ใช้ 60 request/นาทีร่วมกันระหว่างการอ่านและ transaction ส่วน Plus และ Premium
ใช้ bucket ของ venue แยกกัน bucket ภายในเครื่องแยกกันในทุก tier ดังนั้นผู้ใช้ Standard ต้องวางงบ traffic รวม
รวมถึงการอ่าน nonce และ reconciliation

REST limiter ใช้หนึ่ง token ต่อการเรียกหนึ่งครั้ง โดยไม่สนใจน้ำหนักของ venue ให้ตั้ง `rest_quota_per_min`
ตามสัดส่วนของ endpoint ที่ใช้ ก่อน request อื่น ขีดจำกัด Premium 24,000 weighted req/min ของ Lighter Mainnet
อนุญาตอย่างใดอย่างหนึ่ง:

- เรียก `/api/v1/recentTrades` 40 ครั้ง/นาที (น้ำหนัก 600)
- เรียก `/api/v1/trades` 120 ครั้ง/นาที (น้ำหนัก 200)

ให้ยืนยันน้ำหนักสำหรับ deployment ที่ใช้

### ขีดจำกัดตามประเภท transaction {#transaction-type-limits}

ขีดจำกัดตามประเภท transaction เริ่มต้นคือ 40 request/นาที โดยมีข้อยกเว้นตามประเภท และแยกจากขีดจำกัด
`sendTx`/`sendTxBatch` ของ tier

การแก้ไขทุกครั้งที่ quote ขยับทำให้ชน `code=23000` (`Too Many Requests`) ของ Lighter Mainnet หลังจาก
แก้ไขไปประมาณ 40 ครั้งในหนึ่งนาที การแก้ไขยังใช้[volume quota](#volume-quota-and-no-fill-quoting)ด้วย

สำหรับการ quote ที่ใช้ transaction หนัก ให้ตั้ง `sendtx_quota_per_min` เป็น 40 หรือต่ำกว่า ซึ่งจะควบคุมจังหวะ
ของการ create และ cancel ด้วย batch ใช้หนึ่ง token สำหรับ transaction สูงสุด 15 ตัว ดังนั้นค่านี้จึงไม่ได้จำกัด
จำนวน order ลูกใน order list หรือการแก้ไขแบบ batch ให้ยืนยันขีดจำกัดตามประเภท transaction ของ deployment
สำหรับ order ลูกใน batch

โควตาภายในเครื่องอนุญาตให้มี burst ไม่ใช่เพดานแบบ rolling-minute ที่เข้มงวด โควตา 30 อนุญาตให้ burst
30 request ในตอนแรกแล้วเติมกลับที่ 30/นาที ซึ่งอาจเกินขีดจำกัดของ venue ที่ 40 ใน 60 วินาทีได้ ให้เผื่อพื้นที่
สำหรับ burst และ client อื่นที่ใช้ L1 address ร่วมกัน

สำหรับการทดสอบแบบมีขอบเขตบน Standard การใช้ transaction request 15 ครั้งและการเรียก REST 5 ครั้งต่อนาที
จะเหลือพื้นที่สำหรับการตรวจ account แยกต่างหาก

response `23000` ของ WebSocket ที่จับคู่ไม่ได้จะถูก log โดยไม่ reject order หรือ batch เพราะอาจเกี่ยวกับ
traffic ที่ไม่ใช่ transaction ให้ reconcile ผลลัพธ์ที่ pending อยู่ อย่าส่งซ้ำแบบไม่ตรวจสอบ

### Limiter ของ adapter ที่ใช้ร่วมกัน {#shared-adapter-limiters}

execution limiter ตัวเดียวบังคับใช้ `sendtx_quota_per_min` ข้ามการเทรดผ่าน WebSocket `sendTx`/`sendTxBatch`
รวมถึงการ cancel ทั้ง account และการควบคุม schedule และการ approve integrator ผ่าน HTTP `sendTx` ตอน startup
การเรียกแบบ raw จะใช้ limiter นี้เมื่อตั้งค่าไว้ มิฉะนั้นจะถอยไปใช้ REST limiter ของ raw client

client ที่มี string ของ WebSocket URL เหมือนกันจะใช้ control limiter สำหรับ message ที่ไม่ใช่ transaction
ขนาด 200 message/นาทีร่วมกัน `readonly=true` ใน URL เริ่มต้นของ data ทำให้แยกออกจาก execution
ให้วางงบ control traffic รวมให้อยู่ภายในขีดจำกัดต่อ IP

subscription gate ของแต่ละ connection อนุญาต request ที่ยังไม่ได้ acknowledge ได้ 35 ตัว ซึ่งขึ้นกับ latency
ของ acknowledgement ไม่ใช่อัตราการส่ง มันไม่ได้บังคับใช้เพดาน 50 message ต่อ IP ของ venue ข้าม connection
`sendTx` และ `sendTxBatch` ไม่ใช้ทั้ง bucket ของ client message และเพดาน inflight ของ venue

ขีดจำกัดของ tier ด้านล่างใช้กับ [Lighter Mainnet](https://apidocs.lighter.xyz/docs/rate-limits)

| ขอบเขต                                | ขีดจำกัดของ venue          | พฤติกรรมของ adapter                                         |
| ------------------------------------ | ------------------------ | ---------------------------------------------------------- |
| REST, standard account               | 60 req/min               | ค่าเริ่มต้น ตั้ง `rest_quota_per_min` เพื่อ override          |
| REST, premium account                | 24,000 weighted req/min  | ต้อง override ภายในเครื่อง โดยใช้ attribution ของ venue      |
| REST, plus account                   | 24,000 weighted req/min  | ต้อง override ภายในเครื่อง โดยใช้ attribution ของ venue      |
| REST, builder account                | 240,000 weighted req/min | ต้อง override ภายในเครื่อง โดยใช้ attribution ของ venue      |
| `sendTx` / `sendTxBatch`, standard   | 60 req/min               | ใช้ร่วมกับการอ่าน REST รวมทั้ง HTTP และ WebSocket            |
| `sendTx` / `sendTxBatch`, premium    | [4,000, 57,600] req/min  | ตั้ง `sendtx_quota_per_min` (ปรับตามจำนวน LIT ที่ stake)    |
| `sendTx` / `sendTxBatch`, plus       | 4,000 req/min            | ตั้ง `sendtx_quota_per_min` เพื่อใช้ขีดจำกัดนี้              |
| Default transaction type limit       | 40 req/min               | ใช้กับ tx type ที่ไม่อยู่ภายใต้ volume quota                 |
| `L2UpdateLeverage` transaction limit | 40 req/min               | เกี่ยวข้องกับ `update_leverage`                             |

### ขีดจำกัด active order และ pending order {#active-and-pending-order-limits}

[ขีดจำกัดตาม tier](https://apidocs.lighter.xyz/docs/rate-limits) ของ Lighter ใช้ต่อ account และต่อตลาด
ภายใน account นั้น:

| Account tier | Active ต่อ account | Active ต่อตลาด | Pending ต่อ account | Pending ต่อตลาด |
| ------------ | ----------------- | ------------- | ------------------ | -------------- |
| Standard     | 250               | 30            | 50                 | 10             |
| Plus         | 750               | 250           | 500                | 100            |
| Premium      | 1,500             | 1,000         | 1,000              | 100            |

active order คือ order ที่วางอยู่ใน book order ที่ pending ที่ venue รวมถึง take-profit, stop-loss และ TWAP
ที่ยังไม่ถูก trigger ไม่ใช่ `PendingCancel` ของ Nautilus หรือ request ที่ยังไม่ได้ acknowledge adapter ไม่ได้
นับ order เหล่านี้ล่วงหน้า ให้เผื่อพื้นที่สำหรับ order ที่มีอยู่แล้ว โควตาภายในเครื่องไม่ได้เพิ่มเพดานของ venue

### น้ำหนักของ endpoint และขีดจำกัดของ transport {#endpoint-weights-and-transport-limits}

น้ำหนัก REST ที่พบบ่อยจาก [rate limit ของ Lighter Mainnet](https://apidocs.lighter.xyz/docs/rate-limits)
แสดงด้านล่าง adapter คิดหนึ่ง REST token ต่อการเรียก HTTP หนึ่งครั้ง `sendTx` และ `sendTxBatch` ใช้
transaction limiter ส่วน `nextNonce` ใช้ REST

| กลุ่ม endpoint                                                   | น้ำหนัก |
| --------------------------------------------------------------- | ------ |
| `sendTx`, `sendTxBatch`, `nextNonce`                            | 6      |
| `accountInactiveOrders`, `accountActiveOrders`, `accountOrders` | 100    |
| `apikeys`                                                       | 150    |
| `trades`                                                        | 200    |
| `recentTrades`                                                  | 600    |
| endpoint ที่ venue ไม่ได้ระบุน้ำหนักไว้                             | 300    |

ตรวจน้ำหนักอย่างเป็นทางการของ endpoint อื่นที่ระบุชื่อก่อนวางงบการเรียก
[น้ำหนักของ Robinhood](https://apidocs.lighter.xyz/docs/lighter-rh#rate-limits) อาจต่างออกไป

| Endpoint หรือ transport                 | ขีดจำกัด              | หมายเหตุ                                                              |
| -------------------------------------- | ------------------- | -------------------------------------------------------------------- |
| `/api/v1/trades`                       | 100 แถว             | adapter แบ่งหน้า reconciliation ตามเพดานนี้                          |
| `/api/v1/accountInactiveOrders`        | 100 แถว             | adapter ตาม `next_cursor` ตามเพดานนี้                                |
| `/api/v1/orderBookOrders`              | 250 order ต่อฝั่ง    | adapter รวม order เหล่านี้เป็นระดับราคา L2                            |
| `/api/v1/candles`                      | 500 แถว             | adapter จำกัดหน้า bar ของ REST ที่ค่าสูงสุดนี้ของ venue               |
| `/api/v1/fundings`                     | 750 แถว             | เพดานของ venue โดย adapter ขอ 100 แถวต่อหน้า                         |
| WebSocket connections                  | 255 / IP            | ขีดจำกัดของ venue                                                    |
| WebSocket subscriptions / connection   | 500                 | ขีดจำกัดของ venue                                                    |
| WebSocket subscriptions / IP           | 5,000               | ขีดจำกัดของ venue                                                    |
| WebSocket unique accounts / connection | 500                 | ขีดจำกัดของ venue                                                    |
| WebSocket unique accounts / IP         | 5,000               | ขีดจำกัดของ venue                                                    |
| WebSocket connections / minute         | 255                 | ขีดจำกัดของ venue                                                    |
| WebSocket client messages / minute     | 200                 | ควบคุมจังหวะ frame ที่ไม่ใช่ tx โดย heartbeat ping ไม่นับ             |
| WebSocket inflight messages            | 50 / IP             | subscription gate อนุญาต request ค้างได้ 35 ตัวต่อ connection        |
| WebSocket `sendTxBatch` batch size     | 15 txs              | เพดานของ adapter ต่อ request command ที่ใหญ่กว่าจะถูกแบ่งเป็น batch    |
| WebSocket keepalive                    | 2 นาที              | adapter ส่ง heartbeat ทุก 30 วินาที                                  |
| WebSocket outbound command queue       | ไม่จำกัด             | ควบคุมจังหวะก่อนเขียน ไม่มีเพดานความลึกของคิว                         |

### ขีดจำกัดของ request ข้อมูลย้อนหลัง {#historical-request-limits}

ประวัติของ bar และ funding จำกัดที่ 500 หน้า REST: สูงสุด 250,000 bar หรือ 49,500 ช่วง funding รายชั่วโมง
การเสร็จสิ้นในหน้าสุดท้ายถือว่าสำเร็จ เช่นเดียวกับ request ที่ระบุ `start` ชัดเจนและได้ครบตาม `limit` ที่ระบุ

ช่วงที่ไม่ได้ครอบคลุมจะคืน `LighterHttpError::HistoryIncomplete` โดยไม่มีประวัติบางส่วนหรือการ retry
data client จะ log error และไม่ emit response ให้ลดช่วงลงเพื่อดำเนินการต่อ

## Volume quota และการ quote ที่ไม่ได้ fill {#volume-quota-and-no-fill-quoting}

[volume quota](https://apidocs.lighter.xyz/docs/volume-quota-program) แยกจากขีดจำกัดของ transport
และใช้เฉพาะกับ account ระดับ Plus และ Premium โดยใช้ร่วมกันข้าม sub-account ภายใต้ L1 address เดียว:

- `L2CreateOrder`, `L2CancelAllOrders`, `L2ModifyOrder` และ `L2CreateGroupedOrders` ใช้โควตานี้
  transaction ใน batch ที่เข้าเกณฑ์แต่ละตัวใช้แยกกัน การ cancel order เดียวไม่ใช้
- ปริมาณการเทรดที่เสร็จสิ้นและโควตาฟรีจะเติมโควตากลับ

adapter ไม่ได้ตรวจโควตาที่เหลืออยู่ ให้ดูเอกสารของ venue สำหรับกฎและตัวเลขปัจจุบัน

การ refresh quote ที่ไม่ได้ fill อาจทำให้โควตาหมดได้แม้จะควบคุมจังหวะของ WebSocket และ `sendTx` แล้ว
สำหรับการทดสอบแบบ live ให้ใช้การ quote ฝั่งเดียวที่ช้ากว่า, threshold การ refresh ที่กว้างขึ้น, testnet หรือ
strategy แบบมีขอบเขตที่มี fill มากพอจะเติมโควตากลับ

## การจัดการ connection {#connection-management}

### Heartbeat และการ reconnect {#heartbeats-and-reconnects}

WebSocket client ส่ง heartbeat ทุก 30 วินาที และ reconnect ด้วย exponential backoff ในช่วง
[250 มิลลิวินาที, 30 วินาที] มันจะ reconnect หลังจากไม่มี frame ขาเข้า 90 วินาที แม้ socket จะยังเปิดอยู่
pong จะ refresh ช่วงเวลานี้แม้ไม่มี market data

ประกาศ `shutdown` ของ venue จะขอ reconnect ทันทีผ่าน WebSocket client ที่ใช้ร่วมกัน ส่วนประกาศของ connection
เก่าจะถูกเพิกเฉย การ replay subscription ตามปกติ, [การกู้คืน nonce](#nonce-recovery) และ
[ความพร้อมของ account stream](#account-stream-readiness) จะทำงานตามปกติ

stream เก่าจะหยุดก่อนที่ stream ทดแทนจะพร้อม snapshot ใหม่และ HTTP reconciliation จะกู้คืนช่วงที่ขาดไป
การปิดจาก server ที่ตามมา รวมถึง `1012` ใช้เส้นทาง reconnect เดียวกัน

### การยืนยันตัวตนแบบ private {#private-authentication}

auth token แบบ private มีอายุไม่เกิน 8 ชั่วโมง adapter สร้าง token อายุ 7 ชั่วโมง และหมุนเวียนพร้อม resubscribe
ทุก 6 ชั่วโมง การ reconnect แบบโปร่งใสจะสร้าง token ใหม่และ resubscribe account stream หลังจากที่
subscription ที่ติดตามอยู่เริ่ม replay แล้ว

### การกู้คืน nonce {#nonce-recovery}

การ reconnect ของ execution จะดึงค่าฐานของ nonce จาก `GET /api/v1/nextNonce` command submit, modify และ cancel
แบบเดี่ยวจะ sign ไม่ได้จนกว่าการ refresh หรือการ retry เบื้องหลังจะติดตั้งค่าฐานของ connection ใหม่
command แบบ batch จะรอจนพร้อม รวมถึง command ที่ได้รับระหว่างการ refresh

ภายใน session หนึ่ง:

- การยืนยันจาก venue จะเลื่อน nonce window ภายในเครื่องไปข้างหน้า
- การ reject ที่ชัดเจนหรือความล้มเหลวก่อนส่งต่ออาจย้อน nonce ล่าสุดกลับ
- state ที่เก่าจะ trigger การ resync ด้วย `GET /api/v1/nextNonce`

ผลลัพธ์ที่อาจไปถึง venue แล้วจะคง nonce ที่ pending และ identity ของ order ไว้สำหรับการกู้คืนผ่าน WebSocket
หรือ reconciliation

### ความพร้อมของ account stream {#account-stream-readiness}

`LighterExecutionClient::connect()` จะล้าง cache ของ position และ account จากนั้นรอนานสุด 30 วินาทีให้
[private stream](#account-and-position-management) ทั้งห้าพร้อม

มีเพียง `subscribed/account_all_positions` เท่านั้นที่ทำให้ position พร้อม update แบบ live ไม่นับ
payload ของ account จาก REST ไม่เคยถูกใช้เป็นทางสำรอง

การ reconnect แบบโปร่งใสและการหมุนเวียน auth จะไม่เข้า `connect()` อีกครั้ง position จะคงอยู่จนกว่า
`subscribed/account_all_positions` ครั้งถัดไปจะใช้กฎการแทนที่ snapshot update แบบ live จะ merge
โดยไม่ลบตลาดที่ไม่ได้ระบุไว้

## API credential {#api-credentials}

การ sign ต้องใช้ credential สามอย่าง:

- Account index: ตัวระบุ account ของ Lighter แบบตัวเลข
- API key index: slot ของ API key แบบตัวเลข ใช้ index ที่ไม่ได้สงวนไว้ในช่วง `[4, 254]` Robinhood ยังสงวน `157`
  ไว้ด้วย อย่าใช้ `[0, 3]` หรือ `255` เพราะ `255` เป็นค่า sentinel สำหรับ query `apikeys` ไม่ใช่ signing key
- API private key: private key แบบ hex ขนาด 40 byte จะมีหรือไม่มี prefix `0x` ก็ได้

ค่าใน config มีลำดับความสำคัญสูงกว่า field ที่หายไป หรือ private key ที่ว่างหรือมีแต่ช่องว่าง จะถอยไปใช้
environment variable ของ `deployment` และ `environment` ที่เลือก

| Deployment | Environment | API key index                             | API private key                        | Account index                             |
| ---------- | ----------- | ----------------------------------------- | -------------------------------------- | ----------------------------------------- |
| Lighter    | Mainnet     | `LIGHTER_API_KEY_INDEX`                   | `LIGHTER_API_SECRET`                   | `LIGHTER_ACCOUNT_INDEX`                   |
| Lighter    | Testnet     | `LIGHTER_TESTNET_API_KEY_INDEX`           | `LIGHTER_TESTNET_API_SECRET`           | `LIGHTER_TESTNET_ACCOUNT_INDEX`           |
| Robinhood  | Mainnet     | `LIGHTER_ROBINHOOD_API_KEY_INDEX`         | `LIGHTER_ROBINHOOD_API_SECRET`         | `LIGHTER_ROBINHOOD_ACCOUNT_INDEX`         |
| Robinhood  | Testnet     | `LIGHTER_ROBINHOOD_TESTNET_API_KEY_INDEX` | `LIGHTER_ROBINHOOD_TESTNET_API_SECRET` | `LIGHTER_ROBINHOOD_TESTNET_ACCOUNT_INDEX` |

namespace ที่แยกกันทำให้ process เดียวกำหนดเป้าหมายไปยังหลาย deployment ได้โดยไม่ใช้ credential ร่วมกัน

execution จะปฏิเสธ credential ที่ไม่ครบ data subscription ทั้งหมดและ REST request สำหรับ instrument,
book, trade, bar และ funding ใช้ endpoint สาธารณะโดยไม่ต้องใช้ credential

## Configuration {#configuration}

### Option ของ data client {#data-client-configuration-options}

| Option                             | ค่าเริ่มต้น | คำอธิบาย                                                        |
| ---------------------------------- | --------- | -------------------------------------------------------------- |
| `environment`                      | `Mainnet` | `LighterEnvironment::Mainnet` หรือ `Testnet`                   |
| `deployment`                       | `Lighter` | `LighterDeployment::Lighter` หรือ `Robinhood`                  |
| `venue`                            | `None`    | override venue ของ Nautilus (ตัวเลือกเสริม) ค่าเริ่มต้นมาจาก `deployment` |
| `account_index`                    | `None`    | field ของ factory (ตัวเลือกเสริม) การเรียกข้อมูลสาธารณะไม่ได้ใช้   |
| `api_key_index`                    | `None`    | field ของ factory (ตัวเลือกเสริม) การเรียกข้อมูลสาธารณะไม่ได้ใช้   |
| `private_key`                      | `None`    | field ของ factory (ตัวเลือกเสริม) การเรียกข้อมูลสาธารณะไม่ได้ใช้   |
| `base_url_http`                    | `None`    | override URL ของ REST (ตัวเลือกเสริม)                           |
| `base_url_ws`                      | `None`    | override URL ของ WebSocket (ตัวเลือกเสริม)                      |
| `proxy_url`                        | `None`    | URL ของ proxy สำหรับ HTTP และ WebSocket (ตัวเลือกเสริม)          |
| `http_timeout_secs`                | `60`      | timeout ของ HTTP request หน่วยวินาที                            |
| `ws_timeout_secs`                  | `30`      | timeout ของการเชื่อมต่อและการเชื่อมต่อใหม่ของ WebSocket          |
| `update_instruments_interval_mins` | `60`      | ช่วงเวลา refresh metadata ของ instrument หน่วยนาที              |
| `book_snapshot_timeout_secs`       | `10`      | เวลารอ snapshot ตอนเริ่ม, reconnect และกู้คืน                   |
| `rest_quota_per_min`               | `None`    | override โควตา REST หากไม่ตั้งจะคงไว้ที่ 60 req/min              |
| `transport_backend`                | Default   | backend ของ WebSocket transport                                |

### Option ของ execution client {#execution-client-configuration-options}

| Option                          | ค่าเริ่มต้น     | คำอธิบาย                                                                                                  |
| ------------------------------- | ------------- | -------------------------------------------------------------------------------------------------------- |
| `environment`                   | `Mainnet`     | `LighterEnvironment::Mainnet` หรือ `Testnet`                                                             |
| `deployment`                    | `Lighter`     | `LighterDeployment::Lighter` หรือ `Robinhood`                                                            |
| `venue`                         | `None`        | override venue ของ Nautilus (ตัวเลือกเสริม) ค่าเริ่มต้นมาจาก `deployment`                                  |
| `account_id`                    | `LIGHTER-001` | account ID ของ Nautilus โดย issuer ต้องตรงกับ venue ที่ resolve ได้                                       |
| `account_index`                 | `None`        | account index ของ Lighter                                                                                |
| `api_key_index`                 | `None`        | slot ของ API key ของ Lighter                                                                             |
| `private_key`                   | `None`        | private key แบบ hex สำหรับ auth และการ sign L2 transaction                                               |
| `base_url_http`                 | `None`        | override URL ของ REST (ตัวเลือกเสริม)                                                                     |
| `base_url_ws`                   | `None`        | override URL ของ WebSocket (ตัวเลือกเสริม)                                                                |
| `proxy_url`                     | `None`        | URL ของ proxy สำหรับ HTTP และ WebSocket (ตัวเลือกเสริม)                                                    |
| `http_timeout_secs`             | `60`          | timeout ของ HTTP request หน่วยวินาที                                                                      |
| `ws_timeout_secs`               | `30`          | timeout ของการเชื่อมต่อและการเชื่อมต่อใหม่ของ WebSocket                                                    |
| `market_order_slippage_bps`     | `50`          | เพดาน slippage (bps) สำหรับ `MARKET` / `STOP_MARKET` / `MIT`                                             |
| `rest_quota_per_min`            | `None`        | override โควตา REST หากไม่ตั้งจะคงไว้ที่ 60 req/min                                                        |
| `sendtx_quota_per_min`          | `None`        | override โควตา transaction หากไม่ตั้งจะคงไว้ที่ 60 req/min                                                 |
| `transport_backend`             | Default       | backend ของ WebSocket transport                                                                          |
| `use_gtd`                       | `true`        | ใช้ GTD แบบ native ของ venue ดู [GTD policy](#gtd-policy)                                                 |
| `scheduled_cancel_timeout_secs` | `None`        | timeout ของการ cancel ทั้ง account อัตโนมัติ หน่วยวินาที ดู [การ cancel](#account-wide-cancellation)       |

### ตัวอย่าง configuration {#configuration-example}

```rust
use nautilus_lighter::{
    common::enums::{LighterDeployment, LighterEnvironment},
    config::{LighterDataClientConfig, LighterExecutionClientConfig},
};
use nautilus_model::identifiers::AccountId;

let data_config = LighterDataClientConfig::builder()
    .environment(LighterEnvironment::Testnet)
    .deployment(LighterDeployment::Lighter)
    .build();

let exec_config = LighterExecutionClientConfig::builder()
    .environment(LighterEnvironment::Testnet)
    .deployment(LighterDeployment::Lighter)
    .account_id(AccountId::from("LIGHTER-001"))
    .build();

let robinhood_data_config = LighterDataClientConfig::builder()
    .environment(LighterEnvironment::Mainnet)
    .deployment(LighterDeployment::Robinhood)
    .build();

let robinhood_exec_config = LighterExecutionClientConfig::builder()
    .environment(LighterEnvironment::Mainnet)
    .deployment(LighterDeployment::Robinhood)
    .account_id(AccountId::from("LIGHTER_ROBINHOOD-001"))
    .build();
```

execution config เหล่านี้ใช้[ตัวแปร credential](#api-credentials)ของ deployment และ environment ของตัวเอง
field credential ที่ระบุชัดเจนจะ override ค่าเหล่านั้น

ตั้ง `LiveExecutionEngineConfig.reconciliation_instrument_ids` เพื่อกำหนดขอบเขตของ reconciliation และ
`reconciliation_lookback_mins` เพื่อจำกัดการ replay ของ inactive order และ fill

## เอกสารอย่างเป็นทางการ {#official-documentation}

- [Get started](https://apidocs.lighter.xyz/docs/get-started)
- [Trading and signing](https://apidocs.lighter.xyz/docs/trading)
- [Core concepts and nonces](https://apidocs.lighter.xyz/docs/core-concepts)
- [Environments and endpoints](https://apidocs.lighter.xyz/docs/environments)
- [Lighter on Robinhood Chain](https://apidocs.lighter.xyz/docs/lighter-rh)
- [API keys](https://apidocs.lighter.xyz/docs/api-keys)
- [Account types](https://apidocs.lighter.xyz/docs/account-types)
- [Rate limits](https://apidocs.lighter.xyz/docs/rate-limits)
- [Volume quota](https://apidocs.lighter.xyz/docs/volume-quota-program)
- [Historical data and exports](https://apidocs.lighter.xyz/docs/historical-data)
- [Data structures, constants, and errors](https://apidocs.lighter.xyz/docs/data-structures-constants-and-errors)
- [REST OpenAPI](https://raw.githubusercontent.com/elliottech/lighter-python/main/openapi.json)
- [WebSocket reference](https://apidocs.lighter.xyz/docs/websocket-reference)

## การมีส่วนร่วม {#contributing}

:::info
หากต้องการมีส่วนร่วมพัฒนา adapter ของ Lighter หรือขอฟีเจอร์ ดู
[contributing guide](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
