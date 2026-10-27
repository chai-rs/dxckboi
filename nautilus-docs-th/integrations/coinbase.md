---
title: "Coinbase"
---

Coinbase ก่อตั้งในปี 2012 และเป็นหนึ่งใน cryptocurrency exchange ที่ใหญ่ที่สุดซึ่งอยู่ภายใต้การกำกับดูแลของสหรัฐฯ
Advanced Trade API ของ Coinbase รองรับการเทรด spot, perpetual swap และ dated futures adapter นี้รองรับการรับ
ข้อมูลตลาดแบบ live และการ execute order ทั้งบน account แบบ spot (Cash) และ CFM derivatives (Margin) ผ่าน execution
client ตัวเดียวกัน โดย factory เป็นผู้เลือกประเภท account (ดู [ขอบเขตของ execution](#execution-scope))

## ภาพรวม [#overview]

Adapter ของ Coinbase implement ด้วย Rust

Component:

- `CoinbaseRawHttpClient`: REST client ระดับล่างที่ดูแล transport, การ sign JWT และ rate limit
- `CoinbaseHttpClient`: REST client ระดับ domain ที่ parse response ของ venue ให้เป็น type ของ Nautilus
- `CoinbaseWebSocketClient`: การเชื่อมต่อ WebSocket ระดับล่างพร้อม JWT สำหรับ auth ตอน subscribe
- `CoinbaseInstrumentProvider`: parse และโหลด instrument
- `CoinbaseDataClient`: ตัวจัดการ market data feed
- `CoinbaseDataClientFactory`: factory ของ data client
- `CoinbaseExecutionClient`: execution client (spot หรือ CFM derivatives; order ผ่าน REST + stream ผ่าน WS)
- `CoinbaseExecutionClientFactory`: factory ของ execution client โดยเลือก spot หรือ CFM derivatives ด้วย
  `account_type` ใน config

## ตัวอย่าง [#examples]

- [ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/coinbase/examples/)

## เอกสารของ Coinbase [#coinbase-documentation]

Coinbase มีเอกสารสำหรับ Advanced Trade API:

- [REST API reference](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction)
- [คู่มือการจัดการ order](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/orders)
- [WebSocket channels](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-channels)
- [การยืนยันตัวตนด้วย API key](https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/api-key-authentication)
- [Rate limiting](https://docs.cdp.coinbase.com/coinbase-app/api-architecture/rate-limiting)

ควรอ่านเอกสารของ Coinbase ควบคู่กับคู่มือ integration ของ NautilusTrader นี้

:::info
Adapter นี้มุ่งไปที่ Coinbase Advanced Trade API ไม่ครอบคลุม venue แยกต่างหากอย่าง
[Coinbase International Exchange (INTX)](https://international.coinbase.com) ซึ่ง adapter ของ INTX
ถูกนำออกไปแล้วใน NautilusTrader 1.224.0
:::

## Products [#products]

Product เป็นคำเรียกรวมสำหรับกลุ่มของ instrument type ที่เกี่ยวข้องกัน

Product type ที่รองรับมีดังนี้:

| Product Type        | รองรับ | หมายเหตุ                                              |
| ------------------- | ------ | ---------------------------------------------------- |
| Spot                | ✓      | คู่ spot ที่ quote เป็น USD, USDC และ USDT               |
| Perpetual contracts | ✓      | Perpetual swap แบบ USD-margined บน FCM venue          |
| Futures contracts   | ✓      | Dated delivery futures (nano BTC, nano ETH ฯลฯ)       |

## Symbology [#symbology]

Coinbase ใช้ field `product_id` ดั้งเดิมของ venue เป็น symbol ของ Nautilus โดยตรง และ instrument ID คือ
`{product_id}.COINBASE`

| Product      | รูปแบบ                            | ตัวอย่าง                             |
| ------------ | ------------------------------- | ---------------------------------- |
| Spot         | `{base}-{quote}`                | `BTC-USD`, `ETH-USDC`, `SOL-USDT`  |
| Perpetual    | `{contract_code}-{ddMMMyy}-CDE` | `BIP-20DEC30-CDE` (BTC PERP)       |
| Dated future | `{contract_code}-{ddMMMyy}-CDE` | `BIT-24APR26-CDE` (BTC Apr 2026)   |

Suffix `-CDE` หมายถึง Coinbase Derivatives Exchange (FCM venue) perpetual มีวันหมดอายุในอนาคตอันไกลที่ exchange
กำหนดให้ (เช่น `20DEC30`) แต่ถูกจัดเป็น `CryptoPerpetual` เพราะมี funding rate ต่อเนื่อง ส่วน dated futures
ถูกจัดเป็น `CryptoFuture`

Adapter ระบุ product type เชิงโครงสร้างจาก metadata ของ API (`future_product_details.contract_expiry_type`
และเมื่อค่านั้นเป็น `EXPIRING` จะดูว่ามี `future_product_details.funding_rate` ที่ไม่ว่างหรือไม่ ซึ่งเป็นสัญญาณเชิง
โครงสร้างที่มีเฉพาะ perpetual) heuristic สำรองจะตรวจว่า `display_name` มี substring `PERP` หรือ `Perpetual`

ตัวอย่าง instrument ID แบบเต็มของ Nautilus:

- `BTC-USD.COINBASE` (spot Bitcoin/USD)
- `ETH-USDC.COINBASE` (spot Ether/USDC)
- `BIP-20DEC30-CDE.COINBASE` (BTC perpetual swap)
- `BIT-24APR26-CDE.COINBASE` (BTC dated future, Apr 2026)

### Product ที่เป็น alias (USDC และ USD) [#aliased-products-usdc-and-usd]

Coinbase รวมคู่เดียวกันที่ quote เป็น USDC และ USD ไว้ใน book เดียวของ matching engine `GET /products` แสดง
ความสัมพันธ์นี้ผ่าน `alias` และ `alias_to`:

```text
BTC-USD :  alias=""        alias_to=["BTC-USDC"]   # canonical
BTC-USDC:  alias="BTC-USD" alias_to=[]             # alias of BTC-USD
```

เมื่อผู้เรียก subscribe หรือส่ง order ด้วยฝั่ง alias venue จะเขียน request ใหม่เป็น canonical id บน wire adapter
จะบันทึก map `product_id -> alias` ไว้ตอน bootstrap instrument และจัดการการเขียนใหม่นี้ให้โปร่งใสทั้งสองฝั่ง:

- Data subscription จะส่งออกไปด้วย canonical id โดย data WebSocket client ถือ mapping ย้อนกลับไว้ และเปลี่ยน
  key ของ message ขาเข้ากลับเป็น id ที่ผู้เรียกระบุก่อน parse
- Order ถูกส่งด้วย `product_id` ของผู้เรียก execution client บันทึก ID นั้นไว้ใต้ `client_order_id` และเปลี่ยน key
  ของ canonical ID ที่ user channel ส่งกลับมา ดังนั้น order ฝั่ง alias จะไม่ถูกรายงานกับ canonical instrument

Strategy ที่ถือแค่ USDC จึงเทรด `BTC-USDC.COINBASE` ได้ตั้งแต่ต้นจนจบโดยไม่ต้องอ้างถึง canonical `BTC-USD`
settlement currency กำหนดจาก `product_id` ที่ส่ง ดังนั้น order ที่วางบน `BTC-USDC.COINBASE` จะหักหรือเติม
wallet USDC เสมอ

## Environment [#environments]

Coinbase มี environment สำหรับเทรดสองแบบ ตั้งค่า environment ที่ต้องการด้วย field `environment` ใน config ของ client

| Environment | ค่า `environment`              | REST base URL                      |
| ----------- | ----------------------------- | ---------------------------------- |
| Live        | `CoinbaseEnvironment.LIVE`    | `https://api.coinbase.com`         |
| Sandbox     | `CoinbaseEnvironment.SANDBOX` | `https://api-sandbox.coinbase.com` |

### Live (production) [#live-production]

Environment เริ่มต้นสำหรับ live trading ด้วยเงินจริง ระบุ `api_key` และ `api_secret` ใน
`CoinbaseExecutionClientConfig` (ค่าเริ่มต้นของ environment คือ Live)

Environment variable: `COINBASE_API_KEY`, `COINBASE_API_SECRET`

### Sandbox [#sandbox]

Environment ทดสอบแบบ static mock สำหรับเชื่อมต่อระบบให้ครบ ตาม
[เอกสาร Sandbox](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/sandbox) ตั้ง `environment`
เป็น Sandbox และใส่ `api_key`/`api_secret` เป็น string ใดก็ได้ที่ไม่ว่าง

Venue แบบ sandbox ไม่ได้บังคับการยืนยันตัวตน แต่ `CoinbaseExecutionClient::new` ต้องการทั้งสอง field (หรือ
environment variable ที่ตรงกัน) ในการสร้าง

:::warning
**Sandbox ไม่ใช่ trading venue คู่ขนาน**:

- Response ทั้งหมดเป็นแบบ static และกำหนดไว้ล่วงหน้า ไม่มีตลาดจริงหรือราคาที่เปลี่ยนแปลง
- มีเฉพาะ endpoint ของ Accounts และ Orders ไม่มี resource อื่น
- ไม่ต้องยืนยันตัวตน (และไม่ได้บังคับ)
- Header `X-Sandbox` แบบกำหนดเองสามารถ trigger สถานการณ์ error ที่กำหนดไว้ล่วงหน้าได้

ใช้ sandbox เพื่อเชื่อมต่อ client และตรวจรูปแบบของ request/response ส่วนการทดสอบพฤติกรรมที่สมจริงให้ใช้
production (ด้วยเงินจริงและความระมัดระวัง)
:::

## การยืนยันตัวตน [#authentication]

Coinbase Advanced Trade ใช้การยืนยันตัวตนด้วย JWT แบบ ES256 ทุก REST request และทุก WebSocket subscription
จะสร้าง JWT อายุสั้นที่ sign ด้วย EC private key ของคุณ adapter จะ resolve credential จาก environment variable
หรือจาก field ใน config

### การสร้าง API key [#creating-an-api-key]

Coinbase มี key หลายประเภท adapter ต้องการ **Coinbase App Secret API key** ที่ใช้ signature algorithm แบบ
**ECDSA** (ไม่ใช่ Ed25519)

<Steps>
<Step>
ไปที่หน้า API keys ของ CDP portal:
[portal.cdp.coinbase.com/projects/api-keys](https://portal.cdp.coinbase.com/projects/api-keys)
</Step>
<Step>
เลือกแท็บ **Secret API Keys** แล้วคลิก **Create API key**
</Step>
<Step>
ตั้งชื่อเล่น (เช่น `nautilus-trading`)
</Step>
<Step>
ขยาย **API restrictions** แล้วตั้ง permission เป็น **View** และ **Trade**
</Step>
<Step>
ขยาย **Advanced Settings** แล้วเปลี่ยน signature algorithm จาก Ed25519 เป็น **ECDSA** ขั้นตอนนี้จำเป็น
เพราะ key แบบ Ed25519 ใช้กับ Advanced Trade API ไม่ได้
</Step>
<Step>
คลิก **Create API key** แล้วบันทึก key name และ private key จาก modal
key name มีรูปแบบเป็น `organizations/{org_id}/apiKeys/{key_id}` ส่วน private key เป็น EC key ที่ encode
แบบ PEM (รูปแบบ SEC1)
</Step>
</Steps>

:::warning
Coinbase ไม่ดาวน์โหลดไฟล์ key ให้อัตโนมัติอีกต่อไป ให้คัดลอกค่าจาก modal ตอนสร้าง หรือคลิกปุ่มดาวน์โหลดก่อนปิด
เพราะคุณจะดึง private key กลับมาภายหลังไม่ได้
:::

:::info
อย่าใช้ legacy Coinbase App API key ที่หมดอายุแล้ว ให้สร้าง CDP API key และเลือก algorithm แบบ ECDSA adapter
sign request ด้วย ES256 ดู
[คู่มือการย้าย legacy key](https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/legacy-keys)
ของ Coinbase
:::

ดูรายละเอียดทั้งหมดได้ที่
[คู่มือการยืนยันตัวตนด้วย API key](https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/api-key-authentication)
ของ Coinbase

### Environment variable [#environment-variables]

| Variable              | คำอธิบาย                                                  |
| --------------------- | ------------------------------------------------------- |
| `COINBASE_API_KEY`    | Key name (`organizations/{org_id}/apiKeys/{key_id}`)    |
| `COINBASE_API_SECRET` | EC private key ที่ encode แบบ PEM (string หลายบรรทัดแบบเต็ม) |

ตัวอย่าง:

```bash
export COINBASE_API_KEY="organizations/abc-123/apiKeys/def-456"
export COINBASE_API_SECRET="$(cat ~/path/to/cdp_api_key.pem)"
```

:::tip
แนะนำให้ใช้ environment variable ในการจัดการ credential
:::

### อายุของ JWT [#jwt-lifetime]

JWT ของ Coinbase หมดอายุหลัง 120 วินาที ตาม
[WebSocket overview](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-overview)
ต้องสร้าง JWT ใหม่สำหรับ WebSocket message ที่ต้องยืนยันตัวตนทุกครั้ง (คือทุกครั้งที่ subscribe) adapter จะสร้าง
JWT ใหม่ให้ทุก REST request ที่ sign และทุก subscribe message ที่ต้องยืนยันตัวตน ไม่ต้อง rotate เอง

## Portfolio [#portfolios]

Account ของ Coinbase มี **portfolio** ตั้งแต่หนึ่งตัวขึ้นไป แต่ละ portfolio มี wallet ของตัวเอง (USD, USDC, BTC ฯลฯ)
ยอดเงิน และขอบเขตของ order ทุก account มี portfolio `DEFAULT` และผู้ใช้สร้าง portfolio แบบ `CONSUMER` เพิ่มได้
เพื่อแยก strategy ความเสี่ยง หรือ tax lot

CDP API key จะ **ผูกกับ portfolio เดียวตั้งแต่ตอนสร้าง** ทุก request ที่ยืนยันตัวตน (การค้นหา account, การส่ง order,
การยกเลิก) จะทำงานกับ portfolio นั้น เว้นแต่จะระบุ portfolio อื่นอย่างชัดเจน

### การหา UUID ของ portfolio [#finding-your-portfolio-uuids]

รัน probe binary ที่ยืนยันตัวตนของ adapter:

```bash
cargo run --bin coinbase-http-private --package nautilus-coinbase
```

Binary นี้จะพิมพ์:

- Portfolio ที่ CDP key ของคุณมองเห็น
- ยอดเงินของ account ใน portfolio ที่ผูกไว้
- REST call อ้างอิงจำนวนหนึ่ง

ตัวอย่าง output:

```text
Found 1 portfolio(s)
  name=Default type=DEFAULT uuid=ca7244bc-21d1-5e4c-bfe5-80f208ac5723 deleted=false
Account has 3 balance(s)
  USDC total=100.00000000 USDC free=100.00000000 USDC locked=0.00000000 USDC
  AUD total=0.00 AUD free=0.00 AUD locked=0.00 AUD
  BTC total=0.00000000 BTC free=0.00000000 BTC locked=0.00000000 BTC
```

curl ที่เทียบเท่ากัน (ต้อง sign JWT แบบ ES256 ด้วย PEM key ของ CDP เองก่อน):

```bash
curl -H "Authorization: Bearer $JWT" \
  https://api.coinbase.com/api/v3/brokerage/portfolios
```

### เมื่อใดต้องใช้ `retail_portfolio_id` [#when-retail_portfolio_id-is-required]

Endpoint `POST /orders` ของ Coinbase จะ route ไปยัง portfolio ที่ key ผูกไว้โดยค่าเริ่มต้น ดังนั้น account ที่มี
portfolio เดียวไม่จำเป็นต้องตั้ง field นี้ ให้ตั้งใน
[`CoinbaseExecutionClientConfig`](#execution-client-configuration-options) เมื่อเข้ากรณีใดกรณีหนึ่ง:

- Account มีหลาย portfolio และต้องการเทรดกับ portfolio ที่ไม่ใช่ค่าเริ่มต้นของ key
- Venue reject order ด้วย `account is not available` และตัดสาเหตุเรื่อง wallet ด้านล่างออกไปแล้ว

:::note
[Create Order reference](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/create-order)
ของ Coinbase ระบุว่า `retail_portfolio_id` เลิกใช้แล้ว (deprecated) และใช้ได้กับ legacy key เท่านั้น โดยระบุว่า
CDP key จะใช้ portfolio ที่ key มี permission เป็นค่าเริ่มต้น adapter ยังคงส่ง field นี้เมื่อตั้งค่าไว้ จึงยังใช้ได้
หากการ routing เริ่มต้นของ venue ไม่ตรงกับโครงสร้าง account ของคุณ
:::

### การสร้าง portfolio ใหม่ [#creating-a-new-portfolio]

ผู้ใช้ส่วนใหญ่ไม่จำเป็นต้องสร้าง portfolio เพราะ portfolio เริ่มต้นของ account ใช้งานได้โดยไม่ต้องตั้งค่าเพิ่ม ให้สร้าง
ที่ [coinbase.com/portfolios](https://www.coinbase.com/portfolios) เฉพาะเมื่อต้องการ:

- แยกการเทรดผ่าน API ออกจากการเทรดแบบ retail ด้วยมือ
- แยกความเสี่ยงหรือ P&L ระหว่าง strategy
- เลี่ยงข้อจำกัดของ portfolio เริ่มต้น (เช่น Vault)

หลังสร้าง portfolio แล้ว ให้เติมเงิน (โอนจาก wallet ของ portfolio เริ่มต้นบน coinbase.com) ก่อนส่ง order ใด ๆ
มิฉะนั้น venue จะคืน `account is not available` สำหรับ quote currency

### การแก้ปัญหา `account is not available` [#troubleshooting-account-is-not-available]

Venue คืน error นี้ได้จากหลายสาเหตุ วินิจฉัยได้ด้วยการรัน probe binary ข้างต้นแล้วตรวจรายการ wallet ของ portfolio

| อาการ                                                                  | สาเหตุที่เป็นไปได้                                                                                                                                                                              | วิธีแก้                                                                                                                                                                                                                                                          |
| --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ถูก reject เฉพาะบาง product (เช่น `BTC-USD` ขณะที่มีแค่ USDC)              | Portfolio ไม่มี wallet สำหรับ quote currency ของ product นั้น USD และ USDC แยกกันบน Coinbase และ venue route order ตาม `product_id` ที่ส่ง ไม่ใช่ตาม canonical alias                                        | ส่ง order กับ product ที่ quote currency ตรงกับที่ถืออยู่ (เช่น `BTC-USDC` สำหรับ wallet USDC) adapter resolve alias ฝั่ง data ให้ภายในอยู่แล้ว ไม่ต้องแก้ config การเติมเงินเข้า wallet ที่ขาดผ่าน coinbase.com ก็เป็นทางเลือกได้ แต่ไม่จำเป็นเมื่อถือสกุลเงินเดียว |
| ทุก order ถูก reject ในทุก product                                       | Key ผูกกับ portfolio ที่ไม่ใช่ค่าเริ่มต้น และไม่ได้ตั้ง `retail_portfolio_id`                                                                                                                       | ตั้ง `retail_portfolio_id` ใน `CoinbaseExecutionClientConfig` เป็น UUID ของ portfolio เป้าหมาย                                                                                                                                                                  |
| ถูก reject สำหรับ product `*-USD` บน account ที่ไม่ใช่ของสหรัฐฯ             | ข้อจำกัดตามเขตอำนาจ (เช่น account ของ AU เทรดคู่ที่ quote เป็น USD ไม่ได้)                                                                                                                     | ใช้ quote ที่มีให้ในพื้นที่ (USDC, AUD, EUR ฯลฯ) แทน USD                                                                                                                                                                                                      |
| ถูก reject ทันทีหลัง rotate key                                         | Key ใหม่ถูกสร้างใน portfolio ต่างจาก key เดิม                                                                                                                                                  | อัปเดต `retail_portfolio_id` ให้ตรงกับ portfolio ของ key ใหม่ หรือย้ายเงิน                                                                                                                                                                                        |

## ข้อมูลตลาด [#market-data]

Data client ให้บริการทุกอย่างจาก WebSocket channel ยกเว้นข้อมูล index และ funding ของ derivatives เนื่องจาก
Coinbase Advanced Trade ไม่ publish index price หรือ funding rate บน WebSocket channel ใดเลย สอง stream นี้จึง
ดึงจากการ poll REST แทน

| Nautilus subscription | แหล่งข้อมูล                     | หมายเหตุ                                                                       |
| --------------------- | --------------------------- | ----------------------------------------------------------------------------- |
| Book deltas           | Channel `level2`            | `L2_MBP` เท่านั้น book type อื่นจะถูกปฏิเสธ                                       |
| Quotes                | Channel `ticker`            | Top-of-book จาก payload ของ ticker ของ venue                                  |
| Trades                | Channel `market_trades`     | Request ผ่าน REST ได้ด้วย                                                      |
| Bars                  | Channel `candles`           | Bucket 5 นาทีคงที่ venue ไม่รับ parameter ของ granularity                        |
| Instrument status     | Channel `status`            | ดู [Instrument status](#instrument-status)                                    |
| Index prices          | Poll REST `/products/{id}`  | Derivatives เท่านั้น ตามรอบ `derivatives_poll_interval_secs`                     |
| Funding rates         | Poll REST `/products/{id}`  | Perpetual เท่านั้น ดู [Funding rates](#funding-rates)                            |
| Mark prices           | *Coinbase ไม่รองรับ*          | Subscription จะถูกปฏิเสธ แทนที่จะสังเคราะห์จาก settlement price                    |

Subscription `heartbeats` จะถูกส่งเสมอตอนเชื่อมต่อและส่งซ้ำทุกครั้งที่ reconnect เพื่อให้ทันกำหนด subscribe
ภายในห้าวินาทีของ venue และรักษาการเชื่อมต่อไว้เมื่อ topic ของ product ที่ subscribe ไม่มีความเคลื่อนไหว

### Bars [#bars]

Request bar ย้อนหลังรับ aggregation แบบ EXTERNAL ที่ granularity ที่ adapter map ไว้ ได้แก่ 1m, 5m, 15m, 30m, 1h,
2h, 6h และ 1d step หรือ aggregation อื่นจะถูกปฏิเสธ endpoint `/products/{id}/candles` ของ Coinbase ยังรับ
`FOUR_HOUR` ด้วย ซึ่ง adapter ยังไม่ได้ map

Bar subscription แบบ live ต่างออกไป channel `candles` ของ WebSocket ไม่รับ parameter ของ granularity และ publish
เฉพาะ bucket 5 นาที adapter จะประทับ `BarType` ที่ลงทะเบียนไว้สำหรับ product นั้นให้กับ candle ที่ได้รับแต่ละอัน
ดังนั้น specification ของ subscription แบบอื่นจะได้ bar 5 นาทีที่ติดป้ายเป็น type ที่ request ไว้ ให้ request bar type
`5-MINUTE-LAST-EXTERNAL` สำหรับ subscription แบบ live และใช้ request ย้อนหลังสำหรับ granularity อื่น

### Funding rates [#funding-rates]

Adapter poll endpoint REST `/products/{id}` ตามรอบ `derivatives_poll_interval_secs` (ค่าเริ่มต้น 15 วินาที) และปล่อย
`FundingRateUpdate` จาก payload `future_product_details` ของ FCM เมื่อมี `funding_rate` funding interval
parse จาก field `funding_interval` (โดยทั่วไปเป็น `"3600s"` คือ funding รายชั่วโมง) และ timestamp ของ funding
ครั้งถัดไปจาก `funding_time` Coinbase Advanced Trade ไม่ publish `funding_rate` บน channel `ticker` ของ
WebSocket ดังนั้นการ poll REST จึงเป็นแหล่งข้อมูลแบบ live เพียงแหล่งเดียว

ยังไม่ได้ implement request funding rate ย้อนหลัง

### Instrument status [#instrument-status]

`subscribe_instrument_status` จะเข้าร่วม channel `status` ของ WebSocket ของ Coinbase เมื่อ subscribe ครั้งแรก
(venue publish status feed เดียวสำหรับทุก product) และกรอง event ขาเข้าให้เหลือเฉพาะ instrument ที่ subscribe ไว้
โดยปล่อย event `InstrumentStatus` ด้วย action ต่อไปนี้:

- `MarketStatusAction::Trading` สำหรับ `online`
- `Halt` สำหรับ `offline`
- `Close` สำหรับ `delisted`

Product ที่รายงาน status ว่าง (futures) หรือ status ที่ adapter ไม่ได้ model ไว้ ไม่มีข้อมูลที่เป็นประโยชน์ต่อ data engine
และจะถูกข้าม subscription ของ channel จะถูกยกเลิกเมื่อ instrument ตัวสุดท้าย unsubscribe

## ความสามารถด้าน order [#orders-capability]

ตารางด้านล่างอธิบายความสามารถด้าน order ของ **venue** Coinbase โดย
[`CoinbaseExecutionClient`](#execution-scope) ที่มีให้จัดการ spot หรือ CFM derivatives ตาม `account_type`
ที่ตั้งค่าไว้ ความสามารถด้าน order ของ Coinbase ต่างกันระหว่าง Spot และ Derivatives (perpetual และ dated futures
ใช้ชุดความสามารถ order ของ FCM ร่วมกัน)

### ขอบเขตของ execution [#execution-scope]

`CoinbaseExecutionClientFactory` สร้าง `CoinbaseExecutionClient` เพียง type เดียว โดยเลือกกลุ่ม product ด้วย
field `account_type` ของ `CoinbaseExecutionClientConfig`:

| `account_type`        | Instrument ที่ bootstrap                         | แหล่งข้อมูลสถานะ account                                                                                    |
| --------------------- | ---------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `AccountType::Cash`   | `CoinbaseProductType::Spot` เท่านั้น              | Endpoint REST `/accounts`                                                                                |
| `AccountType::Margin` | `CoinbaseProductType::Future` (perp + dated)   | REST `balance_summary` ของ CFM + WS `futures_balance_summary` พร้อม position report จาก `cfm/positions`     |

Account type อื่นจะถูกปฏิเสธตอนสร้าง factory OMS เป็น `Netting` เสมอ เพราะ venue ไม่มี hedge mode

เพื่อป้องกันข้อมูลรั่วข้าม account:

1. การ bootstrap instrument ตอนเชื่อมต่อจำกัดอยู่ที่กลุ่ม product ที่ตั้งค่าไว้ product ของอีกกลุ่มจะไม่เข้าสู่ cache
   ภายใน process เลย
2. `submit_order` จะปฏิเสธ order ใดก็ตามที่ instrument อยู่นอก cache นั้น
3. `generate_order_status_report(s)` และ `generate_fill_reports` กรอง output ผ่าน cache เดียวกันอีกครั้ง ดังนั้น
   client ตัวเดียวจะไม่เปิดเผย report ของอีกขอบเขตหนึ่ง แม้ account จะมีกิจกรรมทั้ง spot และ derivative

รัน execution client หนึ่งตัวต่อหนึ่งขอบเขต หากต้องการทั้งกิจกรรม spot และ CFM บน trader เดียวกัน ให้สร้าง client
สองตัวที่มีค่า `account_type` และ `account_id` ต่างกัน แล้วตั้ง venue route `COINBASE` ที่ชัดเจนหรือ client
เริ่มต้น (ดู [Execution client routing](../concepts/live.md#execution-client-routing)) และเมื่อส่ง order ให้ส่ง
`client_id` ที่ต้องการไปด้วย

### ประเภท order [#order-types]

ตารางนี้แสดงประเภท order ตามที่ model ของ Nautilus เปิดให้ใช้ คอลัมน์ขวาแสดง key ของ `order_configuration`
ที่ adapter ส่งออกไป ประเภท order ของ Coinbase ที่ไม่อยู่ในตารางนี้ (TWAP, Bracket, Scaled, SOR LIMIT IOC)
อธิบายไว้ใน[ฟีเจอร์ order ขั้นสูง](#advanced-order-features) และระบุว่า adapter *ยังไม่ได้ implement*

| Order Type             | Spot | Perpetual | Future | รูปแบบบน wire                                                    |
| ---------------------- | ---- | --------- | ------ | --------------------------------------------------------------- |
| `MARKET`               | ✓    | ✓         | ✓      | `market_market_ioc` (ทุก product); `market_market_fok` (perps)  |
| `LIMIT`                | ✓    | ✓         | ✓      | `limit_limit_gtc` / `limit_limit_gtd` / `limit_limit_fok`       |
| `STOP_LIMIT`           | ✓    | ✓         | ✓      | `stop_limit_stop_limit_gtc` / `stop_limit_stop_limit_gtd`       |
| `STOP_MARKET`          | -    | -         | -      | *Coinbase ไม่รองรับ*                                              |
| `MARKET_IF_TOUCHED`    | -    | -         | -      | *Coinbase ไม่รองรับ*                                              |
| `LIMIT_IF_TOUCHED`     | -    | -         | -      | *Coinbase ไม่รองรับ*                                              |
| `TRAILING_STOP_MARKET` | -    | -         | -      | *Coinbase ไม่รองรับ*                                              |

### Execution instruction [#execution-instructions]

| Instruction   | Spot | Perpetual | Future | หมายเหตุ                                                                       |
| ------------- | ---- | --------- | ------ | ----------------------------------------------------------------------------- |
| `post_only`   | ✓    | ✓         | ✓      | LIMIT GTC และ LIMIT GTD เท่านั้น                                                |
| `reduce_only` | -    | -         | -      | *Coinbase ไม่รองรับ* ดู [การเทรด derivatives](#derivatives-trading)              |

### Time in force [#time-in-force]

Adapter รับค่าตามตารางนี้ ชุดค่าผสมที่ไม่ได้ระบุไว้จะถูกปฏิเสธตอนส่งด้วย `"Unsupported TIF {tif} for {order_type}"`

| Order type   | GTC | GTD | IOC | FOK | หมายเหตุ                                                                                                                                               |
| ------------ | --- | --- | --- | --- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MARKET`     | ✓   | -   | ✓   | (✓) | GTC ถูก map เป็น IOC ส่วน IOC ที่ระบุชัดเจนจะใช้ตามนั้น FOK จะสร้าง `market_market_fok` ซึ่ง Coinbase ระบุว่าใช้ได้กับ perpetual เท่านั้นและจะ reject บน spot |
| `LIMIT`      | ✓   | ✓   | -   | ✓   | GTD ต้องมี `expire_time` ส่วน LIMIT IOC *ยังไม่ได้ implement* (ดู [SOR LIMIT IOC](#advanced-order-features))                                            |
| `STOP_LIMIT` | ✓   | ✓   | -   | -   | ต้องมี `trigger_price`                                                                                                                                  |

### ฟีเจอร์ order ขั้นสูง [#advanced-order-features]

| ฟีเจอร์              | Spot | Perpetual | Future | หมายเหตุ                                                                                           |
| ------------------ | ---- | --------- | ------ | ------------------------------------------------------------------------------------------------- |
| Order Modification | ✓    | -         | -      | เฉพาะ GTC ที่เปิดอยู่ Coinbase reject การแก้ไข order บน futures venue ด้วย `CANNOT_EDIT_FUTURES_ORDER` |
| Bracket Orders     | -    | -         | -      | *ยังไม่ได้ implement* venue มี `trigger_bracket_gtc` / `trigger_bracket_gtd`                          |
| OCO Orders         | -    | -         | -      | *Coinbase ไม่รองรับ* ในฐานะ order type แยกต่างหาก                                                   |
| Iceberg Orders     | -    | -         | -      | *Coinbase ไม่รองรับ*                                                                                |
| TWAP Orders        | -    | -         | -      | *ยังไม่ได้ implement* venue มี `twap_limit_gtd`                                                     |
| Scaled Orders      | -    | -         | -      | *ยังไม่ได้ implement* venue มี `scaled_limit_gtc`                                                   |
| SOR LIMIT IOC      | -    | -         | -      | *ยังไม่ได้ implement* venue มี `sor_limit_ioc` สำหรับ LIMIT IOC แบบ smart-order-routed               |

ดูสเปกของ venue ได้ที่
[Create Order reference](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/create-order)
และ [Edit Order reference](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/edit-order)

### การควบคุม position (derivatives) [#position-controls-derivatives]

| การควบคุม      | หมายเหตุ                                         |
| ------------- | ----------------------------------------------- |
| Leverage      | ตั้งต่อ order ค่าเริ่มต้น `1.0`                     |
| Margin type   | ตั้งต่อ order: cross (ค่าเริ่มต้น) หรือ isolated       |
| Position mode | One-way เท่านั้น ไม่มี hedge mode                   |

### Batch operation [#batch-operations]

| การดำเนินการ    | หมายเหตุ                                                                                                                                       |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------- |
| Batch Submit | ไม่รองรับ แต่ละ order คือ request `Create Order` หนึ่งครั้ง                                                                                       |
| Batch Modify | ไม่รองรับ แต่ละการแก้ไขคือ request `Edit Order` หนึ่งครั้ง                                                                                         |
| Batch Cancel | `POST /api/v3/brokerage/orders/batch_cancel` รับ array `order_ids` ไม่มีขนาดสูงสุดที่ระบุไว้ในเอกสาร และ response แจ้งผลสำเร็จ/ล้มเหลวราย order |

### การ query order [#order-querying]

| ฟีเจอร์                | Spot | Perpetual | Future | หมายเหตุ                                          |
| -------------------- | ---- | --------- | ------ | ------------------------------------------------ |
| Query open orders    | ✓    | ✓         | ✓      | แสดงรายการ order ที่ active ทั้งหมด                 |
| Query order history  | ✓    | ✓         | ✓      | ข้อมูล order ย้อนหลังพร้อม cursor paging             |
| Order status updates | ✓    | ✓         | ✓      | การเปลี่ยนสถานะแบบ real-time ผ่าน channel `user`    |
| Trade history        | ✓    | ✓         | ✓      | Execution report และ fill report                 |

### ข้อจำกัดของการเทรด spot [#spot-trading-limitations]

- ไม่รับ MARKET FOK บน spot Coinbase ระบุว่า `market_market_fok` ใช้ได้กับ perpetual เท่านั้น และ reject ด้วย
  `UNSUPPORTED_ORDER_CONFIGURATION`
- รองรับ MARKET order ที่ระบุขนาดเป็น quote ส่วน LIMIT order ระบุขนาดเป็นหน่วย base

### การเทรด derivatives [#derivatives-trading]

Derivatives ของ Coinbase เทรดผ่าน FCM (Futures Commission Merchant) venue exec client ส่ง order ผ่าน endpoint
`POST /orders` เดียวกับที่ใช้สำหรับ spot ค่าเริ่มต้นของ `leverage` และ `margin_type` (`CROSS` หรือ `ISOLATED`)
ต่อ order มาจาก `CoinbaseExecutionClientConfig.default_leverage` และ `default_margin_type` margin balance อัปเดต
จากทั้ง endpoint REST `cfm/balance_summary` (snapshot ตอนเชื่อมต่อ, `query_account` และตอน reconnect WebSocket)
และ channel WebSocket `futures_balance_summary` ที่ยืนยันตัวตน position report มาจาก endpoint REST `cfm/positions`

Advanced Trade API ของ Coinbase ไม่มี field `reduce_only` ในเอกสาร schema ของการสร้าง order execution client
จึง reject order แบบ reduce-only ก่อนส่ง แทนที่จะส่งออกไปโดยไม่มี instruction นั้น

Adapter จะ log warning ในกรณีต่อไปนี้:

- Order status report จาก REST อธิบาย order ที่ถูกบังคับปิด (forced-close)
- Balance summary ของ CFM รายงาน liquidation buffer ต่ำกว่า 20% ของ liquidation threshold

Coinbase ไม่ได้แยก flag ของ auto-deleveraging ออกจาก liquidation ทั้งสองกรณีจึงแสดงผ่าน warning เดียวกัน user
channel ไม่มี warning ที่เทียบเท่า ดังนั้นการบังคับปิดจะเห็นได้จาก reconciliation แทนที่จะเห็นจาก live stream

## พฤติกรรมของ execution client [#execution-client-behavior]

ส่วนนี้อธิบายว่า `CoinbaseExecutionClient` แปลงคำสั่ง order ของ Nautilus และ event จาก venue ของ Coinbase
ให้เป็น execution event ของ Nautilus อย่างไร

### การส่ง order [#order-submission]

`submit_order` สร้างรูปแบบ `order_configuration` ของ Coinbase โดยตรงจาก field ของ order ใน Nautilus:

- `MARKET` IOC และ GTC (ค่าเริ่มต้นของ Nautilus) -> `market_market_ioc` ส่วน FOK -> `market_market_fok`
  `Day` และ `Gtd` จะถูกปฏิเสธก่อนเรียก HTTP เพื่อไม่ให้ผู้เรียกได้ semantics แบบ IOC โดยไม่รู้ตัว order `MARKET`
  ที่สร้างด้วย `Gtc` จะ execute เป็น IOC ที่ venue strategy ที่ต้องการให้ backtest และ live ตรงกันอย่างเคร่งครัด
  ควรสร้าง `MarketOrder` ด้วย `Ioc` อย่างชัดเจน
- `LIMIT` GTC -> `limit_limit_gtc`, GTD -> `limit_limit_gtd` (ต้องมี `expire_time`), FOK -> `limit_limit_fok`
- `STOP_LIMIT` GTC -> `stop_limit_stop_limit_gtc`, GTD -> `stop_limit_stop_limit_gtd` ทิศทางของ stop
  derive จากฝั่งของ order (`Buy` -> `STOP_DIRECTION_STOP_UP`, `Sell` -> `STOP_DIRECTION_STOP_DOWN`)
- `STOP_MARKET`, `MARKET_IF_TOUCHED`, `LIMIT_IF_TOUCHED` และ trailing stop แบบต่าง ๆ Coinbase ไม่รองรับ
  จะแสดงเป็น `OrderRejected` พร้อม error จาก `build_order_configuration` ของ submit task ที่ spawn ไว้
  (order จะถูกปล่อยเป็น `OrderSubmitted` ก่อน)

เมื่อสร้างผ่าน HTTP สำเร็จ จะปล่อย `OrderAccepted` พร้อม venue order ID ที่คืนมาใน `success_response.order_id`
หาก response เป็น `success=false` จะปล่อย `OrderRejected` พร้อมเหตุผลความล้มเหลวจาก venue ที่จัดรูปแบบแล้ว
เนื่องจากการพยายามส่งทุกครั้งอาจไปถึง Coinbase แล้ว error ของ transport, timeout, response แบบ rate-limit, การ
decode ล้มเหลว หรือ HTTP 5xx จึงไม่ได้พิสูจน์ว่าถูก reject adapter จะปล่อย order ไว้ในสถานะ in flight และเก็บ
metadata ของการส่งไว้จนกว่า user channel หรือ reconciliation จะ resolve ได้

### การแก้ไข order [#order-modification]

`modify_order` post ไปที่ `/orders/edit` ด้วย `EditOrderRequest` ที่มี type Coinbase รองรับการแก้ไขเฉพาะ GTC
ที่เปิดอยู่ และ reject การแก้ไข order บน futures venue ด้วย `CANNOT_EDIT_FUTURES_ORDER` ดังนั้นการแก้ไขจึงใช้ได้
เฉพาะ spot ในทางปฏิบัติ order ประเภทอื่นต้องใช้ cancel-replace

`/orders/edit` ของ Coinbase ต้องการทั้ง `price` และ `size` แม้จะเปลี่ยนเพียงค่าเดียว หากไม่ส่ง `size` จะถูกอ่านเป็น 0
และถูก reject ด้วย `INVALID_EDITED_SIZE` หรือ `CANNOT_EDIT_TO_BELOW_FILLED_SIZE` exec client จะเติม field
ที่ขาดจาก order ใน cache ให้อัตโนมัติ strategy จึงเรียก `modify_order(price=X)` ได้โดยไม่ต้องระบุปริมาณปัจจุบันซ้ำ
ค่าจากคำสั่ง `ModifyOrder` มีความสำคัญกว่า มิฉะนั้นจะใช้ `price` และ `quantity` ปัจจุบันของ order ใน cache

ความล้มเหลวในการแก้ไขจาก venue จะปล่อย `OrderModifyRejected` พร้อมเหตุผลจาก `EditOrderResponse` ที่มี type
(ใช้ `edit_failure_reason` ก่อน หากไม่มีจึงใช้ `preview_failure_reason`) ความล้มเหลวของ HTTP ที่ไม่รู้ผลลัพธ์ที่ venue
จะปล่อย order ไว้ใน `PENDING_UPDATE` จนกว่า update, ผลการ query หรือ reconciliation จะ resolve ได้

### การยกเลิก [#cancellation]

- `cancel_order` post `batch_cancel` ที่มี id เดียว ความล้มเหลวราย order ที่ venue ระบุชัดจะแสดงเป็น
  `OrderCancelRejected` ส่วนความล้มเหลวของ transport ทั้ง request ที่ไม่รู้ผลลัพธ์ที่ venue จะปล่อย order ไว้ใน
  `PENDING_CANCEL` เพื่อรอ reconciliation
- `cancel_all_orders` ดึงรายการ order ที่เปิดอยู่ผ่าน REST โดยไม่ใช้ filter `OPEN` อย่างเดียว (เพราะ filter `OPEN`
  ของ Coinbase ไม่รวม order แบบ `PENDING` และ `QUEUED` ซึ่งยังยกเลิกได้) กรองในเครื่องให้เหลือ
  `{Accepted, Triggered, PendingUpdate, PartiallyFilled}` และฝั่งที่ request แล้วแบ่งการเรียก `batch_cancel`
  เป็นกลุ่มละ 100 ความล้มเหลวราย order จาก venue จะปล่อย `OrderCancelRejected` ส่วนความล้มเหลวทั้ง request
  ที่ไม่รู้ผลลัพธ์ที่ venue จะปล่อย order ที่ได้รับผลกระทบไว้รอ reconciliation
- `batch_cancel_orders` แบ่งกลุ่มแบบเดียวกัน และแสดงความล้มเหลวราย order ที่ venue ระบุชัดเป็น
  `OrderCancelRejected` ความล้มเหลวของ transport ที่ไม่รู้ผลลัพธ์ที่ venue จะปล่อย order ที่ได้รับผลกระทบไว้รอ reconciliation

### User WebSocket channel [#user-websocket-channel]

`CoinbaseExecutionClient` subscribe channel `user` โดยไม่กรอง `product_ids` และใช้ JWT ใหม่ แล้ว parse แต่ละ
event เป็น `OrderStatusReport` และส่งเข้า execution event stream Coinbase รายงานสถานะสะสมต่อ order แทน fill
ราย trade ดังนั้น exec client จะสังเคราะห์ `FillReport` จากส่วนต่างของค่าสะสม ราคาต่อ fill คำนวณจาก
`(avg_now * qty_now - avg_prev * qty_prev) / delta_qty` เพื่อให้ order ที่มีหลาย fill มีราคา trade ที่ถูกต้อง
ไม่ใช่ค่าเฉลี่ยถ่วงน้ำหนักสะสม ปริมาณเดิมจะถูกกู้คืนในการอัปเดตที่เป็นสถานะสิ้นสุด (`CANCELLED`, `EXPIRED`,
`FAILED`) ซึ่ง venue ตั้ง `leaves_quantity` เป็นศูนย์

User channel ไม่ส่ง `price`, `stop_price`, `trigger_type` หรือการจำแนก maker/taker กลับมา exec client จึง cache
ค่าเหล่านี้ไว้ตอนส่งภายใต้ `client_order_id` และ patch report ก่อนปล่อยออกไป เพื่อให้ reconciler ไม่เห็นความต่าง
แบบ `Some(price) -> None` และ fill ของ `post_only` ถูกประทับ `liquidity_side = Maker` อย่างถูกต้อง สถานะ order
`PENDING`, `QUEUED` และ `OPEN` ทั้งหมด map เป็น `OrderStatus::Accepted` เพื่อเลี่ยง warning เรื่องการเปลี่ยนสถานะ
ย้อนกลับที่ไม่จริง เมื่อการอัปเดตจาก user channel แข่งกับ event `OrderAccepted` จาก REST

การ reject ของ `submit_order` ที่มี `INVALID_LIMIT_PRICE_POST_ONLY` (หรือค่าที่เทียบเท่าของ preview/new-order)
จะถูกปล่อยออกมาพร้อม `due_post_only = true` เพื่อให้ strategy ตอบสนองต่อการที่ post-only ข้ามราคาได้ (โดยทั่วไป
คือ quote ใหม่เทียบกับ TOB ใหม่)

เมื่อ reconnect สถานะ account จะถูกดึงใหม่ผ่าน REST เพื่อกู้การเปลี่ยนแปลงยอดเงินระหว่างช่วงที่หลุดการเชื่อมต่อ
การติดตามค่าสะสมต่อ order ยังคงอยู่ข้ามการ reconnect เพื่อให้ส่วนต่างของ fill ที่สังเคราะห์ยังถูกต้อง

### การตัด fill ซ้ำ [#fill-deduplication]

User-channel WebSocket อาจ replay event เมื่อ reconnect execution client มี FIFO สำหรับตัดรายการซ้ำขนาด 10,000
entry ที่ใช้ key เป็น `(venue_order_id, trade_id)` และทิ้ง fill ที่ trade ID ที่สังเคราะห์ตรงกับที่เพิ่งเห็นไป map ของ
สถานะสะสมก็จำกัดขนาดเท่ากัน เพื่อป้องกัน order ที่ไม่เคยได้รับ event สถานะสิ้นสุดตลอดอายุของ client นี้ หลังหลุดการ
เชื่อมต่อนานมาก (เกินช่วงการตัดซ้ำใน memory) fill ที่ replay อาจปล่อย `FillReport` ซ้ำได้ ในกรณีนั้น strategy ควร
พึ่ง reconciliation ผ่าน REST เพื่อกู้สถานะที่ถูกต้อง

### Position reconciliation [#position-reconciliation]

สำหรับ account แบบ Cash (spot) client จะไม่คืน position report เพราะ spot ของ Coinbase ไม่มี position สำหรับ
account แบบ Margin position report มาจาก endpoint REST `cfm/positions` (รายการ) และ `cfm/positions/{product_id}`
(รายตัว) และถูกกรองอีกครั้งด้วย instrument cache ที่ bootstrap ไว้ order ที่เปิดอยู่และ fill ย้อนหลังถูก reconcile
จาก REST ผ่าน `generate_order_status_report(s)` และ `generate_fill_reports` ตอนเชื่อมต่อ และตามรอบ reconciliation
มาตรฐานที่ตั้งไว้ใน `LiveExecutionEngineConfig`

## Rate limiting [#rate-limiting]

Coinbase เผยแพร่ limit ต่อไปนี้สำหรับ Advanced Trade API:

| ส่วน                            | Limit                                                                                 | แหล่งอ้างอิง                            |
| ------------------------------ | ------------------------------------------------------------------------------------- | ------------------------------------ |
| WebSocket connections          | 8 ครั้งต่อวินาทีต่อ IP address                                                           | Advanced Trade WebSocket Rate Limits |
| WebSocket unauthenticated msgs | 8 ครั้งต่อวินาทีต่อ IP address                                                           | Advanced Trade WebSocket Rate Limits |
| WebSocket subscribe deadline   | Subscribe message แรกต้องมาถึงภายใน 5 วินาทีหลังเชื่อมต่อ มิฉะนั้น server จะตัดการเชื่อมต่อ       | Advanced Trade WebSocket Overview    |
| Authenticated WebSocket JWT    | 120 วินาที ต้องสร้าง JWT ใหม่สำหรับ subscribe message ที่ยืนยันตัวตนทุกครั้ง                 | Advanced Trade WebSocket Overview    |
| REST per-key quota             | 10,000 request ต่อชั่วโมงต่อ API key (นโยบายทั่วไปของ Coinbase App)                       | Coinbase App Rate Limiting           |

เมื่อเกิน REST limit Coinbase จะคืน HTTP `429` พร้อม body นี้:

```json
{
  "errors": [
    {
      "id": "rate_limit_exceeded",
      "message": "Too many requests"
    }
  ]
}
```

Adapter ยัง throttle REST traffic ทั้งหมดฝั่ง client ที่ 30 request ต่อวินาที และจำกัด subscribe และ unsubscribe
message ไว้ที่ 8 ครั้งต่อวินาที เพื่อปรับ burst ก่อนไปถึง venue

:::info
เอกสาร Advanced Trade ปัจจุบันของ Coinbase เผยแพร่ limit ของ WebSocket แต่ไม่มี REST quota เฉพาะของ Advanced
Trade (เพดานต่อวินาที, limit ต่อ portfolio) ดังนั้น quota รายชั่วโมงของ Coinbase App ข้างต้นจึงเป็นค่าที่มีเอกสาร
ระบุเจาะจงที่สุด อ้างอิง:
[WebSocket rate limits](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-rate-limits),
[WebSocket overview](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-overview),
[Coinbase App rate limiting](https://docs.cdp.coinbase.com/coinbase-app/api-architecture/rate-limiting)
:::

## Reconnect และ resubscribe [#reconnect-and-resubscribe]

WebSocket client ใช้ exponential backoff เมื่อ reconnect โดยเริ่มที่ 250ms และสูงสุด 30s subscription ที่เก็บไว้
จะถูก replay อัตโนมัติหลัง handshake เสร็จ Coinbase จะตัดการเชื่อมต่อ client ที่ไม่ส่ง subscribe message ภายใน
5 วินาทีหลังเชื่อมต่อ ดังนั้นการ replay จึงรวม `heartbeats` เสมอ client จะทำเครื่องหมาย topic นี้ก่อน replay ครั้งแรก
และคงไว้ตลอดอายุของ client

สำหรับ channel ที่ต้องยืนยันตัวตน (`user` และ `futures_balance_summary` บน client แบบ Margin) adapter จะสร้าง
JWT ใหม่ทุก subscribe message เพราะ Coinbase ต้องการ JWT ที่ต่างกันสำหรับ WebSocket message ที่ยืนยันตัวตน
แต่ละครั้ง หากไม่มี credential หรือสร้าง JWT ไม่สำเร็จ adapter จะข้าม topic ที่ต้องยืนยันตัวตนแทนการส่งโดยไม่ sign
และความล้มเหลวจะแสดงเป็น error บน message stream ของ client เมื่อ subscription ได้รับการยอมรับแล้ว ข้อมูลจะไหล
ต่อเนื่องตลอดอายุของการเชื่อมต่อ WebSocket โดยไม่ต้องยืนยันตัวตนอีก

หาก user WebSocket ก่อนหน้ายัง active หรือกำลัง reconnect อยู่ตอนที่ execution client เชื่อมต่อใหม่ client จะปิด
การเชื่อมต่อนั้น และสร้าง inner client ใหม่แทนการใช้ state machine เดิม เพื่อรับประกันว่าจะได้ command channel,
output channel และ shutdown signal ใหม่ แม้คำสั่ง `Disconnect` ของ session ก่อนหน้าจะแพ้ race กับ shutdown signal

## การตั้งค่า [#configuration]

### ตัวเลือก config ของ data client [#data-client-configuration-options]

| ตัวเลือก                             | ค่าเริ่มต้น  | คำอธิบาย                                                                            |
| ---------------------------------- | --------- | --------------------------------------------------------------------------------- |
| `api_key`                          | `None`    | หากไม่ระบุจะใช้ env var `COINBASE_API_KEY`                                           |
| `api_secret`                       | `None`    | หากไม่ระบุจะใช้ env var `COINBASE_API_SECRET`                                        |
| `base_url_rest`                    | `None`    | Override REST base URL                                                            |
| `base_url_ws`                      | `None`    | Override URL ของ WebSocket สำหรับ market data                                      |
| `proxy_url`                        | `None`    | Proxy URL (ไม่บังคับ) สำหรับ transport แบบ HTTP และ WebSocket                          |
| `environment`                      | `Live`    | `Live` หรือ `Sandbox`                                                             |
| `http_timeout_secs`                | `10`      | Timeout ของ HTTP request (วินาที)                                                  |
| `ws_timeout_secs`                  | `30`      | Timeout ของ WebSocket (วินาที)                                                     |
| `update_instruments_interval_mins` | `60`      | ช่วงเวลาระหว่างการ refresh instrument catalog                                         |
| `derivatives_poll_interval_secs`   | `15`      | ช่วงเวลาระหว่างการ poll REST ที่ปล่อย `IndexPriceUpdate` และ `FundingRateUpdate`         |
| `transport_backend`                | `Sockudo` | Backend ของ transport แบบ WebSocket                                               |

### ตัวเลือก config ของ execution client [#execution-client-configuration-options]

| ตัวเลือก                   | ค่าเริ่มต้น  | คำอธิบาย                                                                                                                           |
| ------------------------ | --------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `account_id`             | `Venue`   | Identifier ของ account ใน Nautilus ค่าเริ่มต้นคือ `COINBASE-001`                                                                     |
| `api_key`                | `None`    | หากไม่ระบุจะใช้ env var `COINBASE_API_KEY`                                                                                          |
| `api_secret`             | `None`    | หากไม่ระบุจะใช้ env var `COINBASE_API_SECRET`                                                                                       |
| `base_url_rest`          | `None`    | Override REST base URL                                                                                                           |
| `base_url_ws`            | `None`    | Override URL ของ WebSocket สำหรับ user data                                                                                       |
| `proxy_url`              | `None`    | Proxy URL (ไม่บังคับ) สำหรับ transport แบบ HTTP และ WebSocket                                                                         |
| `environment`            | `Live`    | `Live` หรือ `Sandbox`                                                                                                            |
| `http_timeout_secs`      | `10`      | Timeout ของ HTTP request (วินาที)                                                                                                 |
| `max_retries`            | `3`       | จำนวนครั้งสูงสุดที่ retry HTTP request                                                                                                |
| `retry_delay_initial_ms` | `100`     | Delay เริ่มต้นของการ retry (มิลลิวินาที)                                                                                              |
| `retry_delay_max_ms`     | `5,000`   | Delay สูงสุดของการ retry (มิลลิวินาที)                                                                                                |
| `account_type`           | `Cash`    | `Cash` สำหรับ spot หรือ `Margin` สำหรับ CFM derivatives ดู [ขอบเขตของ execution](#execution-scope)                                   |
| `default_margin_type`    | `None`    | `CoinbaseMarginType` เริ่มต้น (`Cross` หรือ `Isolated`) สำหรับ order ของ derivatives ไม่มีผลกับ Cash                                   |
| `default_leverage`       | `None`    | Leverage เริ่มต้นสำหรับ order ของ derivatives ไม่มีผลกับ Cash                                                                         |
| `retail_portfolio_id`    | `None`    | UUID ของ CDP retail portfolio ส่งไปตอนสร้าง order เมื่อตั้งค่าไว้ Coinbase ระบุว่า field นี้ deprecated สำหรับ CDP key ดู [Portfolio](#portfolios) |
| `transport_backend`      | `Sockudo` | Backend ของ transport แบบ WebSocket                                                                                              |

> หมายเหตุ: ต้นฉบับแสดงการสร้าง config เหล่านี้ด้วย Python จึงตัดออก ดูการใช้งานฝั่ง Rust ได้ที่
> [ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/coinbase/examples/)
> และ[การรัน live trading ด้วย Rust](../how_to/run_rust_live_trading.md)

## ข้อจำกัดที่ทราบ [#known-limitations]

### ฝั่ง venue [#venue-side]

- การแก้ไข order จำกัดเฉพาะ GTC ที่เปิดอยู่ และถูก reject บน order ของ futures venue ด้วย
  `CANNOT_EDIT_FUTURES_ORDER` กรณีอื่นทั้งหมดต้องใช้ cancel-replace
- OCO order ไม่มีให้ใช้ในฐานะ order type แยกต่างหาก
- Coinbase ไม่รองรับ trailing stop, MARKET_IF_TOUCHED, LIMIT_IF_TOUCHED และ iceberg order
- ไม่มีการ publish mark price ทั้งบน REST และ WebSocket ดังนั้น subscription ของ mark price จะถูกปฏิเสธ
- ไม่มี batch submit และ batch modify มีเพียง batch cancel
- Sandbox เป็น environment แบบ static mock (มีเฉพาะ endpoint ของ Accounts และ Orders, response ที่กำหนดไว้
  ล่วงหน้า, ไม่มี market data จริง)
- User-channel WebSocket รายงานสถานะสะสมต่อ order ไม่ใช่ fill ราย trade exec client derive ปริมาณ ราคา และ
  commission ต่อ fill จากส่วนต่างของค่าสะสม `trade_id` ราย trade สังเคราะห์จาก `(venue_order_id, cumulative_quantity)`

### ฝั่ง adapter [#adapter-side]

- **Identity ของ fill ที่คงที่ต่างกันระหว่างเส้นทาง live และ REST** user channel ไม่มี `trade_id` ต่อ fill ของ
  Coinbase ดังนั้น `FillReport` แบบ live จึงใช้ ID ที่สังเคราะห์จาก venue order ID และปริมาณสะสม ส่วน REST
  reconciliation ใช้ `trade_id` ของ venue identifier จึงอาจต่างกันระหว่างการประมวลผลแบบ live และ reconciliation
- **หนึ่งกลุ่ม product ต่อหนึ่ง client** การส่ง การแก้ไข การยกเลิก และการสร้าง report ถูกกรองตามกลุ่ม product ที่
  ตั้งค่าไว้ (spot ภายใต้ `AccountType::Cash`; perp + dated futures ภายใต้ `AccountType::Margin`) order ที่
  instrument อยู่นอก cache ที่ bootstrap ไว้จะถูกปฏิเสธ ดู [ขอบเขตของ execution](#execution-scope)
- **Position report ว่างเสมอสำหรับ account แบบ Cash** spot ของ Coinbase ไม่มี position ส่วน position report ของ
  derivatives (CFM) มาจาก `cfm/positions` และปรากฏเฉพาะบน client แบบ Margin
- **การอัปเดตจาก user channel ไม่มี `price`, `stop_price` และ `trigger_type`** สำหรับ order ที่ client นี้ส่งเอง
  field ที่ขาดจะถูก patch จาก cache ที่เก็บไว้ตอน `submit_order` สำหรับ order ภายนอก (ส่งจาก process อื่นหรือผ่าน
  UI ของ Coinbase) handler ของ user channel จะเสริม report เมื่อพบครั้งแรกโดยดึง `/orders/historical/{venue_order_id}`
  แล้ว cache ผลไว้ REST call นี้เพิ่ม latency ให้กับการอัปเดตแรกจาก user channel ของ order ภายนอก การอัปเดต
  ครั้งต่อไปจะใช้ข้อมูลที่ cache ไว้
- **ความล้มเหลวในการดึงรายการผ่าน REST ของ cancel-all และ batch-cancel ถูก log เท่านั้น** หาก REST call สำหรับ
  ดึง order ที่เปิดอยู่ล้มเหลว จะไม่มีการปล่อย `OrderCancelRejected` ราย order และ order จะค้างอยู่ใน `PendingCancel`
  จนกว่า reconciliation ครั้งถัดไปจะกู้คืน ซึ่งเป็นรูปแบบเดียวกับ adapter ของ Bybit
- **Product ที่ list ใหม่ต้อง reconnect ก่อนจึงจะเทรดได้** instrument cache ถูกเติมตอนเชื่อมต่อ product ที่ list
  หลังจากนั้นจะไม่อยู่ใน cache และ `submit_order` จะปฏิเสธ
- **MARKET order ใช้ IOC เป็นค่าเริ่มต้น** `MarketOrder` ที่สร้างด้วยค่าเริ่มต้นของ Nautilus `TimeInForce::Gtc` จะถูก
  map เป็น `market_market_ioc` ที่ venue `TimeInForce::Ioc` ที่ระบุชัดเจนจะใช้ตามนั้น `TimeInForce::Fok` จะ route
  ไปที่ `market_market_fok` ซึ่ง Coinbase ระบุว่าใช้ได้กับ perpetual เท่านั้น และจะ reject ตอน runtime บน spot ด้วย
  `UNSUPPORTED_ORDER_CONFIGURATION` ส่วน `Day` และ `Gtd` จะถูกปฏิเสธตอนส่ง
- **ยังไม่ได้ implement request funding rate ย้อนหลัง** funding rate มีให้เฉพาะในรูปการอัปเดตแบบ live จากการ poll
  REST ของ derivatives
- **Bar subscription แบบ live ไม่สนใจ granularity ที่ request** channel `candles` ของ venue publish bucket 5 นาที
  และไม่รับ parameter ของ granularity และ `subscribe_bars` ไม่ได้ปฏิเสธ bar specification แบบอื่น subscription ที่
  step อื่นจะได้ bar 5 นาทีที่ประทับด้วย `BarType` ที่ request ไว้ ดู [Bars](#bars)

## Diagnostic binary [#diagnostic-binaries]

มี binary สามตัวที่ช่วยตรวจการเชื่อมต่อ ยืนยันการทำงานแบบ live และดูแลความเรียบร้อยของ account:

- `coinbase-http-public` request instrument แบบ spot, product book และ trade ล่าสุดโดยไม่ต้องใช้ credential
  ใช้เพื่อยืนยันการเชื่อมต่อก่อนตั้งค่า API key
- `coinbase-http-private` แสดงรายการ portfolio, พิมพ์ยอดเงินใน wallet, รัน `/orders/preview` สำหรับ `BTC-USD`
  และ `BTC-USDC` และแสดง flag การจำกัดราย product แนะนำให้ใช้เป็นจุดแรกเมื่อเริ่มใช้ account ใหม่
- `coinbase-cancel-all-open` ยกเลิก order ที่เปิดอยู่ทั้งหมดบน CDP key ที่ยืนยันตัวตน มีประโยชน์ระหว่างรอบการทดสอบ
  เพื่อเคลียร์ order ที่ค้างอยู่

Binary ที่ต้องยืนยันตัวตนทั้งสองตัวอ่าน `COINBASE_API_KEY` และ `COINBASE_API_SECRET` จาก environment

## การมีส่วนร่วม [#contributing]

:::info
หากต้องการฟีเจอร์เพิ่มเติมหรือต้องการมีส่วนร่วมพัฒนา adapter ของ Coinbase โปรดดู
[contributing guide](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
