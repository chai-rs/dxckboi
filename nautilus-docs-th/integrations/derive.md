# Derive

Derive (ชื่อเดิม Lyra) เป็น venue derivatives แบบกระจายศูนย์ที่ให้บริการ option แบบ European-style และ perpetual
swap แบบ cash-settled และเป็นหนึ่งในตลาด option แบบ on-chain ที่ใหญ่ที่สุด การเทรดทำผ่าน subaccount ที่ EOA หรือ
multisig ของผู้ใช้เป็นเจ้าของ ดังนั้น collateral ยังอยู่ในความดูแลของผู้ใช้ ขณะที่ order จับคู่ผ่าน orderbook ของ venue

API v3 รวมการดำเนินการเป็น batch เพื่อ settle บน Ethereum order จับคู่ off chain และ settle on chain ทำให้ได้ทั้ง
การ execute แบบ orderbook และ self-custody order ใช้ signature แบบ EIP-712 typed-data จาก session key ที่จำกัด
ขอบเขตไว้กับ subaccount ซึ่งแยก signing key ออกจากเจ้าของ wallet ผู้ใช้สามารถ rotate หรือ revoke สิทธิ์ได้โดย
ไม่ต้องย้ายเงิน

## ภาพรวม {#overview}

Adapter ของ Derive implement ด้วย Rust อยู่ใน `crates/adapters/derive` โดยมี:

- `DeriveHttpClient`: การเชื่อมต่อ REST ระดับล่างไปยัง `api.derive.xyz/v3` (mainnet) หรือ
  `testnet.api.derive.xyz/v3` (testnet)
- `DeriveWebSocketClient`: transport แบบ JSON-RPC WebSocket พร้อมการติดตาม subscription, การ reconnect และการส่ง
  order ที่ sign แล้ว
- `DeriveInstrumentProvider`: ดึงและ cache instrument แยกตามสกุลเงิน
- `DeriveDataClient`: market data client แบบ live
- `DeriveDataClientFactory`: factory ของ data client สำหรับ live node builder
- `DeriveExecutionClient`: execution client แบบ live สำหรับ flow ของ order ที่ sign, การยกเลิก, การ query และ report
- `DeriveExecutionClientFactory`: factory ของ execution client สำหรับ live node builder

Flow ของ execution ใช้การ sign แบบ EIP-712 typed-data กับ module contract ราย action ของ venue

## ตัวอย่าง {#examples}

- [ตัวอย่าง Rust](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/adapters/derive/examples/)

## เอกสารของ Derive {#derive-documentation}

Derive เผยแพร่เอกสาร API ที่ [docs.derive.xyz](https://docs.derive.xyz) ใช้อ้างอิงควบคู่กับคู่มือนี้สำหรับรายละเอียดเพิ่มเติม

### การ deploy และการย้ายไป V3 {#v3-deployment-and-migration}

[ประกาศการย้ายระบบเสร็จสมบูรณ์](https://x.com/DeriveXYZ/status/2107605164131852434) ของ Derive แจ้งว่าการย้ายไป
v3 เสร็จสิ้นเมื่อวันที่ 6 ตุลาคม 2026 ส่วน[ข้อเสนอการย้ายระบบ](https://forums.derive.xyz/t/dip-launch-derive-v3/322)
อธิบายการทยอยปิด Derive Chain หลังจากนั้น แหล่งข้อมูลเหล่านี้ไม่ได้ระบุวันปลดระวาง API v2 ที่แน่นอน และไม่ได้ยืนยันว่า
v2 ยังใช้เป็นทางสำรองได้

ตรวจสอบ owner ที่ย้ายแล้ว subaccount ที่มีเงิน และ permission ของ session key ก่อนเริ่ม execute V3 ใช้ EOA ที่เป็น
เจ้าของสูงสุดหรือ Ethereum multisig เป็น `wallet_address` แทน Derive Wallet ตัวกลางแบบเดิม ข้อเสนอการย้ายระบบ
แยก subaccount แบบ standard-margin ตาม risk universe ดังนั้นให้ยืนยัน `subaccount_id` ของ v3 อย่าเหมาว่า ID ของ v2
จะเลือกยอดเงินและ position ที่ต้องการ ส่วน subaccount แบบ portfolio-margin ย้ายแบบหนึ่งต่อหนึ่ง

Admin key ของ v2 ที่ย้ายมาจะได้ scope สำหรับการเทรด การโอนภายใน owner เดียวกัน การถอน และการ liquidation โดยวัน
หมดอายุจำกัดไม่เกิน 28 วันหลัง genesis ซึ่งเป็นสถานะตอนเปิดตัว v3 key เหล่านี้ไม่ได้รับสิทธิ์ `admin`,
`set_session_key` หรือการโอนไปยัง owner อื่น ส่วน read-only key และ account key ของ v2 จะย้ายมาโดยไม่มี protocol
scope และเทรดไม่ได้
ตรวจวันหมดอายุ scope และ subaccount ที่อนุญาตของ key ปัจจุบันผ่าน `private/session_keys`
เจ้าของต้องจัดเตรียม key ของ v3 ที่มี scope เหมาะสมเมื่อ key ที่ย้ายมาไม่ตรงตามความต้องการของการ execute:
EOA จะเรียก `private/set_session_key` ส่วน owner ที่เป็น contract จะส่ง action Set Session Key แบบ on-chain ดู
[Testnet onboarding](#testnet-onboarding) และ [Mainnet onboarding](#mainnet-onboarding)

## Products {#products}

| Product type           | รองรับ | หมายเหตุ                                                                                         |
| ---------------------- | ------ | ----------------------------------------------------------------------------------------------- |
| ERC-20 spot            | ✓      | `CurrencyPair` ที่ quote เป็น USDC ดู[พฤติกรรมของ spot](#spot-behavior)สำหรับข้อจำกัดบน v3 แบบ live     |
| Perpetual swaps        | ✓      | Cash-settled เป็น USDC มีการ list แยกตามสกุลเงิน เช่น `ETH-PERP`                                     |
| Options (calls / puts) | ✓      | Option แบบ European-style ใช้รูปแบบ `{CURRENCY}-{EXPIRY}-{STRIKE}-{C\|P}`                          |

## Symbology {#symbology}

Instrument ของ Derive ใช้ symbol ดั้งเดิมของ venue ต่อด้วย suffix ของ venue `.DERIVE`:

- Spot: `ETH-USDC.DERIVE` (base currency, quote currency)
- Perpetual: `ETH-PERP.DERIVE`, `BTC-PERP.DERIVE`
- Option: `ETH-20260626-3000-C.DERIVE` (สกุลเงิน, วันหมดอายุ, strike, ชนิด)

ส่วนแรกของ symbol ที่คั่นด้วยขีดคือสกุลเงินอ้างอิง provider จะดึงทุกหน้าของ `public/get_all_instruments` สำหรับ
แต่ละ product type และสกุลเงินที่ตั้งค่าไว้ การ subscribe สกุลเงินใหม่จะ trigger การดึงผ่าน REST แบบ lazy เมื่อเปิด
`auto_load_missing_instruments` (ค่าเริ่มต้น)

เมื่อเปิด `include_expired` provider จะรวมรายการ option ที่ยัง live และที่หมดอายุแล้วตามชื่อ instrument โดยใช้นิยาม
ที่ยัง live เมื่อทั้งสองรายการมี option ตัวเดียวกันที่ใกล้หมดอายุ รายการ perpetual และ spot ไม่สนใจค่าตั้งนี้

Adapter route ตาม `instrument_type` ของ venue (`perp`, `option`, `erc20`) ไม่ใช่ตาม suffix ของ symbol ดังนั้นคู่ spot
จึงไม่ต้อง parse symbology เป็นพิเศษ spot ใช้เส้นทางการ sign ผ่าน Trade module เดียวกับ perp และ option fixture ใน
`crates/adapters/derive/test_data/spot/` เก็บรูปแบบ field ของ instrument, order book, ticker และ trade ของ spot
และตรึงเส้นทางของ parser และ execution ไว้กับรูปแบบเหล่านั้น

### พฤติกรรมของ spot {#spot-behavior}

:::warning
Testnet ของ V3 รับและยกเลิก limit order แบบ passive ของ `ETH-USDC` ได้ ส่วน public spot trade frame, commission
ของ fill ที่ติดตาม และ reconciliation ของ spot ตอน restart ยังไม่ได้รับการยืนยันบน v3 การวาง/ยกเลิกบน mainnet ทดสอบ
ด้วยมือก่อนการย้ายไป v3 จำนวนขั้นต่ำของ `ETH-USDC` ในปัจจุบันคือ `0.1 ETH` book ของ spot ที่ว่างไม่มีวิธีที่จำกัด
ความเสี่ยงได้ในการปิด fill ขนาดขั้นต่ำ
:::

- **Trade ที่เบาบาง**: public spot trade channel (`trades.erc20.ETH`, `trades.ETH-USDC`) subscribe ได้สำเร็จ
  แต่อาจมี volume ต่ำ จึงควรคาดว่าจะได้ trade frame ไม่บ่อย
- **Book ที่ว่าง**: venue ยังคง broadcast frame ของ ticker และ order book ที่ top of book เป็นศูนย์ adapter จะไม่ปล่อย
  quote เมื่อฝั่งใดฝั่งหนึ่งมีราคาหรือขนาดเป็นศูนย์ ดู [Quotes](#quotes) trade frame ขับเคลื่อนด้วยการจับคู่และไม่ขึ้นกับ
  สถานะของ book ดังนั้น trade ที่ทำให้ book ว่างก็ยังปล่อย event ออกมา
- **Standard Margin**: spot order คิด margin ตามสัดส่วน initial margin ไม่ใช่ notional เต็มจำนวน

## Environment {#environments}

ตั้งค่า environment ด้วย enum `DeriveEnvironment` ใน config ของ client ตัวใดก็ได้

| Environment | Config                       | REST                                | WebSocket                            |
| ----------- | ---------------------------- | ----------------------------------- | ------------------------------------ |
| Mainnet     | `DeriveEnvironment::Mainnet` | `https://api.derive.xyz/v3`         | `wss://api.derive.xyz/v3/ws`         |
| Testnet     | `DeriveEnvironment::Testnet` | `https://testnet.api.derive.xyz/v3` | `wss://testnet.api.derive.xyz/v3/ws` |

:::important
Testnet ใช้ Ethereum Sepolia โดยมี session key และยอดเงินแยกของตัวเอง **API key ของ mainnet และ testnet ใช้แทนกันไม่ได้**
:::

Public market data (book, ticker, trade) ไม่ต้องใช้ credential

เมื่ออัปเกรดจาก v2 ให้แทนที่ address ของ Derive Wallet เดิมใน `wallet_address` หรือใน environment variable ของ wallet
ด้วย address ของ EOA หรือ multisig ของเจ้าของ ดู [การ deploy และการย้ายไป V3](#v3-deployment-and-migration)

EIP-712 domain separator, Action typehash และ address ของ Trade module สำหรับทั้งสองเครือข่ายมีมาให้ใน
`crates/adapters/derive/src/common/consts.rs` และติดตามให้ตรงกับ
[v3 action-signing reference](https://docs.derive.xyz/authentication/action-signing) ของ Derive mainnet ใช้
Ethereum chain 1 และ testnet ใช้ Sepolia chain 11155111 ทั้งสองใช้ address ของ Trade module เดียวกัน owner คือ EOA
หรือ multisig ส่วน session key เป็นผู้ให้ signer address `DeriveExecutionClientConfig::domain_separator`,
`action_typehash` และ `trade_module_address` รับค่า override ราย instance ซึ่งมีลำดับความสำคัญเหนือค่าที่มีมาให้

Request ของ order, trigger order และ replace จะ sign Trade ABI ขนาดเจ็ด word ด้วยการ scale แบบ fixed-point 1e18
input ทางการเงินต้องมีทศนิยมไม่เกิน 12 หลัก รวมเลขศูนย์ท้าย precision ที่เกินจะเกิด error แทนการตัดค่าที่ sign
request จะส่งค่าทศนิยมเดียวกับที่ใช้สร้าง signature

Action nonce ใช้ UNIX nanosecond และส่งเป็น string ทศนิยมทั้งใน request และ response
ตัวจัดสรรใช้ stream ที่เพิ่มขึ้นทางเดียว (monotonic) ร่วมกันต่อ wallet และ subaccount ข้าม client instance ใน process
เดียวกัน ไม่ว่าตัวพิมพ์ของ wallet จะเป็นแบบใด การย้อนของนาฬิกาจะเลื่อน nonce ล่าสุดไปข้างหน้าเฉพาะเมื่อยังอยู่ภายใน
หนึ่งชั่วโมงจากเวลาในเครื่อง adapter ใช้หน้าต่าง order แบบรวมขอบ [เวลาในเครื่องลบ 90 วัน, เวลาในเครื่องบวก 1 ชั่วโมง]
ซึ่งเป็นการมองย้อนหลังแบบระมัดระวังภายในหน้าต่าง 120 วันที่ venue ระบุไว้ เวลาในเครื่องเป็นค่าประมาณของนาฬิกา server
ดังนั้น clock skew ยังอาจทำให้ venue reject ได้ timestamp ของการ login ใช้มิลลิวินาที และวันหมดอายุของ signature ใช้วินาที

### การสลับ deployment ที่มีอยู่ไปใช้ v3 {#switch-an-existing-deployment-to-v3}

1. หยุดส่ง order และ reconcile order, fill, ยอดเงิน และ position ที่มีอยู่
1. ตรวจ address ของ owner บน v3, subaccount ที่มีเงิน, วันหมดอายุและ scope ของ session key และ environment
1. เชื่อมต่อด้วยค่าเริ่มต้นของ v3 แล้วยืนยันว่า public market data และการอ่าน account ที่ยืนยันตัวตนทำงานได้
1. ตรวจการ execute ด้วยขนาดขั้นต่ำ โดยกำหนดขอบเขตของราคา fee และ exposure อย่างชัดเจน ทั้งบน testnet และ
   mainnet ก่อนเปิดใช้ strategy

หากการอ่าน account หรือการ sign ล้มเหลว ให้หยุดการ execute แล้วตรวจ owner, subaccount, permission ของ key และค่าคงที่
สำหรับการ sign เทียบกับ environment ที่เลือก reconcile สถานะของ venue หลัง reconnect ก่อนกลับมาส่ง order การเปลี่ยน
เฉพาะ URL กลับไปเป็น v2 ไม่ใช่ขั้นตอนการกู้คืน เพราะ adapter นี้ใช้ request schema ของ v3, nonce แบบ nanosecond
และ signature ของ v3 และไม่รองรับ API v2

Trigger order และ TWAP order ของ v2 ไม่ได้ย้ายมา v3 ตรวจสถานะของ venue ก่อนสร้าง trigger order ที่ต้องการใหม่
adapter นี้ไม่ส่ง TWAP order

## Testnet onboarding {#testnet-onboarding}

ขั้นตอนเพื่อให้ execution client ส่ง order ที่ sign แล้วได้:

1. **Sign in เข้า dashboard ของ testnet** เปิด [testnet.app.derive.xyz](https://testnet.app.derive.xyz/)
   แล้วเชื่อมต่อ EOA หรือ multisig ของเจ้าของ
1. **คัดลอก address ของ wallet เจ้าของ** ใช้ address ของ EOA หรือ multisig เป็น `wallet_address` client จะส่งค่านี้ใน
   header `X-DeriveWallet` V3 ไม่มี address ของ Derive Wallet แยกต่างหาก
1. **เติมเงินเข้า subaccount** ทำตาม[คู่มือการฝากเงิน](https://docs.derive.xyz/getting-started/depositing)ของ Derive
   เพื่อ mint collateral สำหรับทดสอบและฝากบน Sepolia การฝากครั้งแรกจะสร้าง account และ subaccount คัดลอก ID
   แบบจำนวนเต็มของ subaccount ที่มีเงินเป็น `subaccount_id` order ต้องมี collateral เพียงพอสำหรับ margin
1. **สร้าง session key** ทำตาม[คู่มือ session key](https://docs.derive.xyz/authentication/session-keys)ของ Derive
   เพื่อลงทะเบียน key ที่มี protocol scope `trade:orderbook:all` หรือ scope ราย product ที่แคบกว่า โดยจำกัดไว้กับ
   subaccount ที่มีเงิน ความสามารถ `account_info` แบบ off-chain ที่แยกต่างหากไม่ได้ให้สิทธิ์เทรด เลือกวันหมดอายุแล้ว
   ตรวจ record ผ่าน `private/session_keys` คัดลอก private key แบบ secp256k1 ดิบเป็น `session_key` key นี้อยู่ใน
   เครื่องเท่านั้นและถูกซ่อนจาก output ของ `Debug`

   หากต้องการเลิกใช้ key ให้ลงทะเบียนใหม่ด้วยวันหมดอายุในอดีตหรือเป็นศูนย์ การ revoke มี cooldown ขั้นต่ำ [5, 15]
   นาที และไม่มีผลทันที
1. **ตั้ง environment variable** export ค่าสามค่าที่ client อ่านในโหมด testnet (หรือส่งผ่าน
   `DeriveExecutionClientConfig` ซึ่ง field ใน config มีลำดับความสำคัญกว่า):

   ```bash
   export DERIVE_TESTNET_WALLET_ADDRESS="0x..."  # owner EOA or multisig
   export DERIVE_TESTNET_SESSION_PRIVATE_KEY="0x..."  # secp256k1 session-key private key
   export DERIVE_TESTNET_SUBACCOUNT_ID="12345"  # integer subaccount id
   ```

Owner ที่เป็น EOA ลงทะเบียน key ผ่าน `private/set_session_key` ส่วน owner ที่เป็น multisig หรือ contract ต้องเติมเงิน
เข้า account ก่อน แล้วจึงส่ง action Set Session Key จาก owner contract บน Sepolia หรือ Ethereum การลงทะเบียนแบบ
on-chain จะถูกละเว้นเมื่อ account ยังไม่มีอยู่ ทำตาม[คู่มือ multisig](https://docs.derive.xyz/authentication/contract-owned-accounts)ของ venue

### เงินทุนขั้นต่ำ {#minimum-funding}

เงินทุนต้องครอบคลุม initial margin ที่ venue กำหนดและวงเงินที่กันไว้สำหรับ order ที่รออยู่ การรับ order ยังขึ้นกับกฎ
ด้านราคา ปริมาณ และการ execute ของ venue ใช้ตัวเลข margin ของ venue เพื่อกำหนดเงินทุนสำหรับการทดสอบที่เล็กที่สุดที่ทำได้:

- **Smoke test (ส่งและยกเลิก ไม่มี fill)**: เติมเงินให้พอครอบคลุม margin ที่ต้องใช้สำหรับ passive order ที่
  `minimum_amount` ของ instrument
- **Options**: กำหนดยอดฝากให้ครอบคลุม margin ที่ venue ต้องการสำหรับ passive order `public/get_instrument`
  ให้รายละเอียดสัญญาแต่ไม่ได้รายงาน initial margin ที่ต้องใช้

สำหรับการทดสอบ fill ไปกลับ ต้องครอบคลุม margin ที่ venue ต้องการ fee และการเคลื่อนไหวของราคาที่จำกัดขอบเขตไว้ ใช้
quote ที่ execute ได้ในปัจจุบัน ปริมาณที่เล็กที่สุดที่ทำได้ `max_fee_per_contract` และ limit price หรือ
`market_order_slippage_bps` ที่จำกัดไว้ ปิด exposure ของ perpetual หรือ option ที่เกิดขึ้นด้วย market order แบบ
reduce-only หรือ order แบบ `IOC`/`FOK` ปิด exposure ของ spot ด้วย order ฝั่งตรงข้าม แล้วยืนยันว่าไม่เหลือ order
หรือ position ที่เปิดอยู่

สำหรับขีดจำกัดขนาด order และข้อยกเว้นของ taker ดู [การโหลด instrument](#instrument-loading)

### ตรวจสุขภาพของ account {#check-account-health}

ใช้ `private/get_subaccount` หลังเติมเงินเพื่อยืนยันว่า `initial_margin` ยังเป็นบวกหลังคิด initial margin ของ
exposure ที่ตั้งใจไว้ field สุขภาพทั้งสองรวม credit ของ collateral และมูลค่า mark-to-market ของ position หลังหัก
risk charge ของ venue แล้ว:

| Field ของ venue       | ความหมาย                                  | ผลเมื่อเป็นค่าลบ                                                              |
| -------------------- | --------------------------------------- | -------------------------------------------------------------------------- |
| `initial_margin`     | สุขภาพ initial margin สุทธิแบบมีเครื่องหมาย     | Venue จะ reject order ที่เพิ่มความเสี่ยงซึ่งจะทำให้ค่านี้ติดลบ                     |
| `maintenance_margin` | สุขภาพ maintenance margin สุทธิแบบมีเครื่องหมาย | Subaccount เสี่ยงต่อการถูก liquidate                                         |

คำสั่ง `query_account` ของ adapter จะปล่อย snapshot นี้เป็น event `AccountState` เพื่อให้ชั้น strategy ตรวจสุขภาพ
ปัจจุบันได้ ค่าสุขภาพสุทธิเหล่านี้ไม่รวมวงเงินที่กันไว้สำหรับ open order ที่รออยู่ ค่าที่เป็นบวกอย่างเดียวจึงไม่ได้พิสูจน์ว่า
order ถัดไปจะได้รับการยอมรับ `private/order_quote` ของ venue คืนสุขภาพ initial margin ก่อน/หลังสำหรับ order ที่เสนอ
การรับ order จริงยังเป็นตัวตัดสินสุดท้าย ดู [Account state](#account-state) สำหรับการ map

## Mainnet onboarding {#mainnet-onboarding}

Mainnet onboarding ทำเหมือน testnet แต่ทำกับ dashboard ของ production และใช้เงินจริง

1. **Sign in เข้า dashboard ของ mainnet** เปิด [app.derive.xyz](https://app.derive.xyz/) แล้วเชื่อมต่อ wallet
   แบบ EOA หรือ multisig ของเจ้าของ
1. **คัดลอก address ของ wallet เจ้าของ** ใช้ address ของ EOA หรือ multisig เป็น `wallet_address` client ใช้ค่านี้
   สำหรับการยืนยันตัวตนและเป็น owner ของ action ที่ sign
1. **เติมเงินหรือเลือก subaccount** ทำตาม[คู่มือการฝากเงิน](https://docs.derive.xyz/getting-started/depositing)
   ของ Derive เพื่อฝาก USDC หรือ collateral ที่รองรับบน Ethereum การฝากครั้งแรกจะสร้าง account และ subaccount
   คัดลอก ID แบบจำนวนเต็มของ subaccount ที่มีเงินเป็น `subaccount_id` แล้วยืนยันผ่าน `private/get_subaccount`
   ว่ายอดฝากปรากฏใน collateral balance `query_account` ของ adapter แสดงแถวที่ model ไว้ใน
   `AccountState.info.collaterals`

   ตรวจ `net_initial_margin` และ `net_maintenance_margin` ใน metadata ที่ปล่อยออกมาตามที่อธิบายใน
   [ตรวจสุขภาพของ account](#check-account-health) สุขภาพสุทธิที่เป็นบวกอย่างเดียวไม่ได้พิสูจน์ว่า order จะได้รับการยอมรับ
1. **สร้าง session key ของ mainnet** ทำตาม[คู่มือ session key](https://docs.derive.xyz/authentication/session-keys)
   ของ Derive เพื่อลงทะเบียน key ที่มี protocol scope `trade:orderbook:all` หรือ scope ราย product ที่แคบกว่า โดย
   จำกัดไว้กับ subaccount ตรวจ scope และวันหมดอายุผ่าน `private/session_keys` owner ที่เป็น contract ใช้การลงทะเบียน
   แบบ on-chain ตามที่อธิบายใน [Testnet onboarding](#testnet-onboarding) บน Ethereum สำหรับ mainnet คัดลอก private
   key แบบ secp256k1 ดิบ แนะนำให้ใช้ key อายุสั้นสำหรับการรัน tester เพื่อสำรวจ

   หากต้องการเลิกใช้ key ให้ลงทะเบียนใหม่ด้วยวันหมดอายุในอดีตหรือเป็นศูนย์ การ revoke มี cooldown ขั้นต่ำ [5, 15]
   นาที และไม่มีผลทันที
1. **ตั้ง environment variable** export ค่าสามค่าของ mainnet (หรือส่งผ่าน `DeriveExecutionClientConfig` ซึ่ง field
   ใน config มีลำดับความสำคัญกว่า):

   ```bash
   export DERIVE_WALLET_ADDRESS="0x..."  # owner EOA or multisig
   export DERIVE_SESSION_PRIVATE_KEY="0x..."  # secp256k1 session-key private key
   export DERIVE_SUBACCOUNT_ID="12345"  # integer subaccount id
   ```

### เลือกเครือข่ายของตัวอย่าง {#select-the-example-network}

:::warning
ตัวอย่าง Rust แต่ละตัว (`node_data_tester`, `node_exec_tester`, `node_delta_neutral`) ตรึงเครือข่ายไว้ด้วย literal
`const DERIVE_ENVIRONMENT: DeriveEnvironment` ใกล้ส่วนบนของไฟล์ **ตรวจค่าคงที่นี้ทุกครั้งก่อนรัน** และแก้ไขเพื่อสลับ
เครือข่าย ตัวอย่างไม่ได้อ่านเครือข่ายจาก environment
:::

Deployment สำหรับ production เลือกเครือข่ายผ่าน `DeriveDataClientConfig::environment` /
`DeriveExecutionClientConfig::environment`

## การระบุ referral code {#referral-code-attribution}

ทุก order, replace และ trigger order ที่ sign จะมี referral code ของ NautilusTrader ที่ฝังไว้ในโค้ด Derive จ่ายค่า
referral จากรายได้ของตัวเอง ดังนั้นการระบุนี้จึงไม่เพิ่มต้นทุนการเทรด ไม่ต้องขออนุมัติ และไม่สามารถตั้งค่าได้ สิ่งนี้ช่วย
ให้ทีมประเมินการใช้งานจริงของ integration และจัดลำดับความสำคัญของการดูแลรักษาต่อเนื่อง

## ความสามารถ {#capabilities}

### ข้อมูลตลาด {#market-data}

| ความสามารถ                      | รองรับ | หมายเหตุ                                                                           |
| ------------------------------ | ------ | --------------------------------------------------------------------------------- |
| Request instrument (REST)      | ✓      | `public/get_instrument` โหลด instrument หนึ่งตัวเข้า cache ในเครื่อง                   |
| Request all instruments (REST) | ✓      | `public/get_all_instruments` แบบแบ่งหน้าสำหรับแต่ละสกุลเงิน                          |
| Instrument subscription        | -      | *ไม่รองรับ* ตั้ง `currencies` เพื่อให้ refresh ผ่าน REST เป็นระยะ                       |
| Order book deltas (L2_MBP)     | ✓      | Channel: `orderbook.{instrument}.{group}.{depth}`                                 |
| Order book depth (L2_MBP)      | ✓      | Channel ของ order book เดียวกันด้วย `depth=10`                                     |
| Order book at interval         | -      | *ไม่รองรับ* ให้สร้าง book ตามช่วงเวลาจาก delta เองในเครื่อง                             |
| Order book snapshot (REST)     | -      | *ไม่รองรับ* venue ไม่มี endpoint สำหรับ snapshot ของ book                             |
| Historical book deltas (REST)  | -      | *ไม่รองรับ* venue ไม่มี endpoint สำหรับ book ย้อนหลัง                                  |
| Quotes (`ticker_slim`)         | ✓      | Channel: `ticker_slim.{instrument}.{interval}`                                    |
| Quote snapshot (REST)          | ✓      | `public/get_tickers` แบบครั้งเดียว ปล่อย `QuoteTick` หนึ่งตัว                           |
| Historical quotes (REST)       | -      | *ไม่รองรับ* venue มีเฉพาะ snapshot ของ ticker                                        |
| Trades                         | ✓      | Channel: `trades.{instrument_type}.{currency}`                                    |
| Historical trades (REST)       | ✓      | เรียงตามเวลาและตัดรายการซ้ำแล้ว `limit` เก็บ trade ล่าสุดไว้                               |
| Bars / OHLC (REST)             | ✓      | Bar นาที ชั่วโมง วัน และสัปดาห์ที่ปิดแล้ว ประทับเวลาที่การปิด bucket                        |
| Bars / OHLC (WS)               | -      | *ไม่รองรับ* venue ไม่มี channel สำหรับ subscribe candle                               |
| Mark price stream              | ✓      | Derive จาก `ticker_slim` ใช้ subscription ร่วมกับ quote                             |
| Index price stream             | ✓      | Derive จาก `ticker_slim` ใช้ subscription ร่วมกับ quote                             |
| Funding rate stream            | ✓      | Derive จาก field funding rate บน ticker ของ perp                                    |
| Funding rate history (REST)    | ✓      | เรียงตามเวลาสำหรับ perpetual `limit` เก็บแถวที่ valid ล่าสุดไว้                            |
| Instrument status              | -      | *ไม่รองรับ* นิยาม instrument มี `is_active`                                          |
| Instrument close               | -      | *ไม่รองรับ* venue publish การ settle ของ option ผ่าน REST เท่านั้น                      |
| Option greeks                  | ✓      | Derive จาก `option_pricing` บน ticker ของ option                                   |
| Option chain                   | ✓      | รวมจาก quote และ greeks โดย `public/get_tickers` ใช้ bootstrap ATM                  |

#### Quotes {#quotes}

Ticker ทาง WebSocket รายงาน snapshot ที่สมบูรณ์สำหรับ perpetual, option และ spot หากฝั่งใดฝั่งหนึ่งมีราคาหรือขนาด
เป็นศูนย์ adapter จะล้างสถานะ quote ภายในของ instrument นั้นและไม่ปล่อย `QuoteTick` (log ที่ระดับ DEBUG) quote
จะกลับมาเมื่อ snapshot มีทั้งสองฝั่งที่ราคาและขนาดไม่เป็นศูนย์

Quote ล่าสุดที่ publish ไปยังคงอยู่ใน cache ของ Nautilus พร้อม timestamp เดิม การระงับ output ของ quote ไม่ได้ถอน quote
ที่ cache ไว้นั้นออก field อื่นของ ticker ที่ subscribe ไว้ รวมถึง mark price และ index price, funding rate และ option
greeks ยังคงปล่อยออกมาแยกกันตามปกติ

#### การโหลด instrument {#instrument-loading}

`request_instrument` เรียก `public/get_instrument` สำหรับ `InstrumentId` ที่ request และ cache นิยามที่ได้กลับมา
ก่อนปล่อย response instrument ใน cache มี field precision และ increment ที่ใช้ในการ parse quote, trade, book และ
bar ในภายหลัง

Field ปริมาณ map ดังนี้สำหรับ perpetual, option และ spot:

| Field ของ Derive  | Field ของ Nautilus        | การจัดการ                                                    |
| ---------------- | ------------------------ | ---------------------------------------------------------- |
| `minimum_amount` | `info["minimum_amount"]` | เก็บไว้เป็น metadata ของ venue โดยไม่ตั้ง `min_quantity`         |
| `amount_step`    | `size_increment`         | ขั้นของปริมาณ order                                           |
| `maximum_amount` | `max_quantity`           | ปริมาณ order สูงสุด                                          |

:::important
Derive สามารถ fill taker order ที่ต่ำกว่า `minimum_amount` ได้ ขณะที่ order ที่จะ rest บน book อาจถูก reject ด้วย
`11012: Invalid amount` **Derive เป็นผู้ตัดสินว่า order ที่ต่ำกว่าขั้นต่ำ valid หรือไม่**
:::

การโหลด instrument ถือว่า error `12001` ของ venue เป็นผลว่างสำหรับ product type ที่ได้รับผลกระทบ ดังนั้นการไม่มี
listing ของ perp, option หรือ spot สำหรับสกุลเงินหนึ่งจะไม่ขัดขวาง product อื่นของสกุลเงินนั้น แถวที่ parse เป็น domain
instrument ไม่ได้จะถูก log และข้าม ขณะที่แถวที่ valid ยังโหลดต่อ field ใน response ที่ผิดรูปแบบจะทำให้การดึงของสกุลเงิน
นั้นล้มเหลว และ error ในการดึงระหว่างการโหลดครั้งแรกจะทำให้การเชื่อมต่อล้มเหลวด้วย

#### ข้อมูลย้อนหลัง {#historical-data}

| ข้อมูล           | REST endpoint                       |
| ------------- | ----------------------------------- |
| Trades        | `public/get_trade_history`          |
| Bars          | `public/get_tradingview_chart_data` |
| Funding rates | `public/get_funding_rate_history`   |

##### ทิศทางของ trade และการตัดรายการซ้ำ {#trade-direction-and-deduplication}

ประวัติผ่าน public REST และ trade ทาง WebSocket อาจคืนแถวของ maker และ taker ภายใต้ `trade_id` เดียวกัน `direction`
ของแต่ละแถวคือฝั่งของผู้เข้าร่วมนั้น adapter ใช้ฝั่งของ taker เป็นทิศทางของ aggressor และกลับทิศทางของแถว maker
สำหรับคู่ maker/taker ผลลัพธ์จึงไม่ขึ้นกับลำดับของแถว
การ map แถว maker ทาง WebSocket บน v3 แบบ live ยังไม่ได้รับการยืนยัน
สำหรับแถวที่ไม่มี `liquidity_role` adapter ถือว่า `direction` เป็นฝั่งของ taker
และ map role ที่ไม่รู้จักเป็น `AggressorSide::NoAggressor` พร้อม log warning ที่มี identifier ของ trade และ instrument

Request trade ย้อนหลังจะปล่อย `TradeTick` หนึ่งตัวต่อ trade ID ภายใน request feed แบบ live จะระงับแถวที่เป็นคู่และ
การ replay ตอน reconnect ขณะที่ ID ยังอยู่ใน cache ของ trade ล่าสุด 4,096 รายการ การ reconnect อัตโนมัติจะคง cache
นี้ไว้ ส่วนการ disconnect หรือ reset อย่างชัดเจนจะล้าง cache

##### ข้อกำหนดของ bar และขอบเขตเวลา {#bar-requirements-and-time-bounds}

- **Aggregation และราคา**: bar ต้องใช้ aggregation แบบ `EXTERNAL` และ `PriceType::Last` เพราะ candle ของ Derive
  อิงจาก trade
- **ช่วงเวลา**: venue รองรับ step 1, 5, 15 และ 30 นาที, 1, 4 และ 8 ชั่วโมง, 1 วัน และ 1 สัปดาห์ bar specification
  อื่นจะถูกปฏิเสธก่อนส่ง request
- **ขอบเขตเวลา**: ขอบเขต `end` ของ bar เลือก bucket ตามเวลาเริ่มต้นที่ venue response จะละ bucket ที่ปิดหลังเวลา
  ของ request ออก รวมถึง bucket ที่ยังก่อตัวอยู่ซึ่ง venue คืนมาด้วย

#### Order book feed {#order-book-feeds}

Derive เปิดให้ใช้ book delta และ depth snapshot ผ่านกลุ่ม channel `orderbook.{instrument}.{group}.{depth}` เดียวกัน
`subscribe_book_deltas` publish snapshot delta เป็น `OrderBookDeltas` ขณะที่ `subscribe_book_depth` ตรึง `depth=10`
และ publish snapshot แบบ `OrderBookDepth`

[Book update](https://docs.derive.xyz/subscriptions) แบบจัดกลุ่มแต่ละครั้งเป็น snapshot เต็ม ผู้ใช้ delta และ depth
ที่มี argument ของ channel ตรงกันจะใช้ subscription และสถานะการกู้คืนร่วมกัน channel จะยัง active จนกว่าผู้ใช้คนสุดท้าย
จะ unsubscribe

Feed ไม่มี change ID ดังนั้น adapter จึงใช้ timestamp ระดับมิลลิวินาทีเป็น sequence ของ output:

- Adapter รับเฉพาะ timestamp ที่ใหม่กว่า timestamp ล่าสุดที่รับไว้อย่างเคร่งครัด รวมถึงข้ามการ reconnect
- Snapshot ว่างที่เวลาเดินหน้าจะล้าง book และนับเป็นการอัปเดตหนึ่งครั้ง

Derive ไม่ได้ระบุการรับประกันลำดับของ timestamp และ `publish_id` อาจเริ่มนับใหม่หลัง resubscribe

### Execution {#execution}

Derive ใช้ session key ที่ตั้งค่าไว้สำหรับ execution ที่ยืนยันตัวตน:

- Request ส่งและแทนที่ order มี signature แบบ EIP-712 ที่สร้างในเครื่อง
- Execution client แบบ live ส่งการเขียน order ผ่าน WebSocket ที่ยืนยันตัวตน และรับการอัปเดตของ account, order,
  trade และยอดเงินผ่าน private channel
- การสร้าง report, การ refresh account และการค้นหา instrument ใช้ REST

#### Fill และการ settle {#fills-and-settlement}

Adapter รับ fill ทั้งแบบ live และย้อนหลังโดยใช้ `trade_id` และ `order_id` เป็น identity ของ execution ปริมาณ ราคา
และ fee decode เป็น `Decimal` ส่วน commission ปัดตาม precision ของ USDC ใน Nautilus

Order status report และ private fill report ใช้ order label (ถ้ามี) เป็น `client_order_id` เมื่อเป็น identifier ที่ valid
ของ Nautilus adapter จะละ label ที่แทนค่าไม่ได้พร้อม warning และยังคง identity ของ venue และค่าทางการเงินไว้
reconciliation สามารถ resolve fill เหล่านี้ด้วย venue order ID

ค่าทางการเงินที่จำเป็นแต่เป็น null ว่าง หรือหายไป จะทำให้การ decode ล้มเหลว array ของ private trade จะข้ามแถวที่ผิดรูปแบบ
พร้อม warning ค่าทางการเงินบน wire ที่ไม่ถูกต้องจะทำให้ frame ของ public WebSocket หรือหน้าของ public REST ถูกปฏิเสธ
trade ID หรือ order ID ที่จำเป็นแต่แทนค่าไม่ได้จะทำให้การ parse แถวนั้นล้มเหลว adapter จะ warn แล้วทำงานต่อกับแถวที่เหลือ

Metadata ของการ settle ไม่ได้เป็นเงื่อนไขของ fill `batch_status` และ `tx_hash` ของ batch บน Ethereum L1 อาจเป็น null
เมื่อ execution มาถึงครั้งแรก adapter ยังรับกรณีที่ไม่มี metadata `op_uuid` แม้ schema ของ private trade จะกำหนดให้มี
ขั้นตอน (stage) ของ batch ที่ publish และสถานะ `Error` ที่คู่กันอธิบายความคืบหน้าหรือความล้มเหลวในการ settle ของ stage
นั้น adapter จะไม่ย้อนหรือ void fill เมื่อ stage ของ batch รายงาน error
ดู [API changelog](https://docs.derive.xyz/changelog) ของ venue

ความไม่ซ้ำกันของ ID ของ private self-trade และ `trade_fee` รวม `extra_fee` ของ external order ที่ไม่เป็นศูนย์หรือไม่
ยังไม่ได้รับการยืนยัน private fill ตัดรายการซ้ำด้วย `trade_id` ดังนั้นการบัญชี self-trade ที่สมบูรณ์จึงขึ้นกับว่า venue
กำหนด ID ที่ต่างกันให้ทั้งสองขาหรือไม่ adapter ไม่ได้ส่ง `extra_fee`

Trade channel ไม่มี filter ตามสถานะการ settle adapter subscribe `trades.{instrument_type}.{currency}` และ
`{subaccount_id}.trades` query ของ fill report จะข้าม ID ที่อยู่ใน cache ของ trade ที่ปล่อยไปแล้วแบบ live และตัด ID
ที่ซ้ำภายในหน้าเดียวกันและข้ามหน้าของ query เดียวกัน การ query trade ไม่ได้บันทึกว่าถูกปล่อยแล้ว core reconciliation
ใช้ trade ID ของ order ที่ persist ไว้เพื่อปฏิเสธการ apply fill ซ้ำหลัง replay หรือ restart

การ dispatch แบบ live สร้าง fill report และ commission ก่อนเปลี่ยนสถานะ order หากแปลง commission ไม่สำเร็จ trade จะ
ยังไม่ถูกประมวลผลเพื่อให้ replay ลองใหม่ได้ adapter จะบันทึก trade ID ว่าปล่อยแล้วหลังส่งสำเร็จเท่านั้น หากส่งไม่สำเร็จ
จะเก็บ native trade record ไว้

สำหรับ order ที่ติดตามอยู่ native fill สามารถสร้างการผูกกับ venue ครั้งแรกได้ก่อนที่ acknowledgement จะมาถึง adapter
จะปล่อย `OrderAccepted` ก่อน `OrderFilled` fill ที่ฝั่งขัดแย้งกับ order ที่ติดตามจะถูกเลื่อนออกไป แม้ order นั้นจะถึง
สถานะสิ้นสุดแล้วก็ตาม

การอัปเดต private trade อาจมาถึงก่อนที่ประวัติผ่าน REST หรือการอ่าน position จะสะท้อนผล ความคืบหน้าของการ settle
ไม่ได้ทำให้ adapter poll ประวัติเพิ่ม

:::note
`DeriveHttpClient` ยังมี method สำหรับส่ง order ผ่าน HTTP ไว้ใช้กับ tooling และการทดสอบ
:::

Perpetual, option และคู่ spot แบบ ERC-20 ทั้งหมดใช้ Trade module ของ Derive spot ไม่มีเส้นทางการ sign แยกต่างหาก
และ reconciliation ปฏิบัติต่อ spot instrument เหมือน instrument class อื่น ยกเว้น guard ของ reduce-only ที่อธิบายด้านล่าง

Adapter รองรับ request `private/order` ปกติ คือ order แบบ `LIMIT` และ `MARKET` ที่มี time-in-force เป็น `GTC`,
`IOC` หรือ `FOK` และยังรองรับ trigger order ของ Derive สำหรับ stop และ if-touched order type ของ Nautilus ที่ระบุ
ด้านล่าง order type ของ Nautilus ที่ไม่รองรับจะถูกปฏิเสธในเครื่องด้วย `OrderDenied` ก่อนส่ง จึงไม่มีทาง fill ที่ venue

:::important
**Market order ต้องมี quote ใน cache ก่อนส่ง** หากไม่มี adapter จะปล่อย `OrderDenied` และไม่ sign เลย
:::

หลัง submit task แบบ async resolve instrument แล้ว จะ refresh snapshot ของ ticker ปัจจุบันและ derive `limit_price`
ที่ sign ซึ่งเป็นขอบเขต slippage จาก quote ที่ refresh นั้น

#### การแทนที่ order ปกติ {#replacing-regular-orders}

Derive implement การแก้ไข order ปกติเป็น cancel-replace adapter คง client order identity เดียวข้าม native venue order
ID หลายตัว และปล่อย `OrderUpdated` ก่อน route fill ของ child คำสั่งใช้การผูกกับ venue ปัจจุบัน ขณะที่ fill ที่มาช้ายังคง
venue order ID เดิมของตัวเอง

ปริมาณของการแทนที่คือปริมาณรวม (gross) ที่ request หักด้วย fill ก่อนหน้าทั้งหมดที่พิสูจน์ได้ ตัวอย่างเช่น การแทนที่ order
ที่ fill ไปแล้ว `0.3` ด้วยเป้าหมายรวม `1.5` จะ sign ปริมาณของ child เป็น `1.2` request ยังตั้ง `expected_filled_amount`
เป็นเพดาน fill ของขาปัจจุบันที่สังเกตได้ [replace guard](https://docs.derive.xyz/trading/order-types#replace-atomic-cancel-+-replace)
ของ Derive จะปฏิเสธการแทนที่เมื่อปริมาณที่ fill ของเป้าหมายเลยค่านั้นไป report ของ order เชิงตรรกะรวม fill สะสม,
ค่าเฉลี่ยถ่วงน้ำหนักตามปริมาณเมื่อมีค่าเฉลี่ย native ที่จำเป็นครบ และ timestamp การสร้าง native ที่เก่าที่สุดเป็นเวลาที่ถูก accept

#### Reconciliation ตอน startup {#startup-reconciliation}

ก่อนเริ่ม private streaming execution client จะ publish สถานะ account โหลดนิยาม instrument ที่ portfolio แบบ native
และ active order ใน cache ต้องใช้ และรอจน cache ของ engine เห็นนิยามเหล่านั้นก่อนเริ่ม private streaming จากนั้นกู้
context ของ active order ที่เป็นเจ้าของจาก order event ใน cache รวมถึง venue ID ที่ทราบและ trade ID ที่ประมวลผลแล้ว
แล้วตรวจ active order เทียบกับ native open order และประวัติของ instrument นั้น ช่องว่างของ metadata ย้อนหลังไม่ได้ขัดขวาง
การตรวจการผูกนี้

หลังยืนยัน private subscription แล้ว client จะ refresh สถานะ account และนิยาม instrument ที่จำเป็นอีกครั้งก่อนประมวลผล
การอัปเดตจาก stream เพื่อปิดช่องว่างระหว่าง snapshot ของ portfolio ครั้งแรกกับการยืนยัน subscription

เมื่อ `private/get_order` คืน `11006` สำหรับ venue ID ที่ทราบ adapter จะค้นหา ID นั้นแบบตรงตัวใน
`private/get_order_history` แบบแบ่งหน้า โดยไม่อนุมานการผูกกับการแทนที่ที่ยังไม่พิสูจน์จาก order label successor ที่ยัง
resolve ไม่ได้จะขัดขวาง startup หรือการสร้าง report และยังปฏิเสธคำสั่ง cancel และ modify ราย order และคืน error
สำหรับ batch cancel และ `cancel_all_orders` ก่อนส่งออกไป request ที่เปลี่ยนสถานะซึ่งผลกำกวมจะไม่ถูก replay

ขณะที่ process ยังเก็บ signing nonce และเป้าหมายของการแทนที่ไว้ แถว native จาก REST สามารถ resolve ได้ด้วยการตรวจ
parent ID และ nonce แบบเดียวกับแถวจาก WebSocket การอ่านประวัติการแทนที่เริ่มต้นหนึ่งชั่วโมงก่อน timestamp ของ nonce
ที่เก็บไว้ เพื่อเผื่อหน้าต่าง nonce ในอนาคตที่ venue ยอมรับ การไม่พบ successor ไม่ได้พิสูจน์ว่าการแทนที่ล้มเหลว หลัง
restart process event `PendingUpdate` ใน cache ไม่ได้เก็บหลักฐานพอที่จะ resolve การแทนที่ที่กำกวมได้อัตโนมัติ

หากการแทนที่ที่ยัง resolve ไม่ได้ขัดขวาง startup:

1. หยุด trading node และเก็บรักษา cache และประวัติ order event ไว้
1. ตรวจ native order, trade และ position ของ subaccount ผ่านเครื่องมือจัดการ account ของ venue จับคู่ order ที่เป็นเจ้าของ
   ด้วย native ID และสายการแทนที่ (replacement ancestry) label ที่ตรงกันอย่างเดียวไม่ได้ยืนยันความเป็นเจ้าของ successor
1. Resolve venue order ที่ค้างอยู่ และ reconcile สถานะ order ใน cache เทียบกับ native record ที่ยืนยันแล้วก่อนกลับมาเทรด
   adapter ไม่มีการซ่อม cache อัตโนมัติสำหรับกรณีนี้ การล้าง `PendingUpdate` อย่างเดียวไม่ได้ยืนยันผลของการแทนที่

Mass status เก็บขาและ fill ของการแทนที่แบบ native ไว้ สำหรับสายการแทนที่ที่ไม่ได้อยู่ใน cache reconciliation ตอน
startup จะกู้ successor ที่ live และพิสูจน์ได้ พร้อม fill สะสมและ native venue order ID successor ที่กู้คืนรองรับการ query
การแก้ไข และการยกเลิก snapshot ตอน reconnect จะปล่อย report ของการแทนที่ที่ไม่อยู่ใน cache ไว้แบบ native โดยไม่
สร้างสายเหล่านี้ขึ้นใหม่

เมื่อสายการแทนที่ที่ไม่อยู่ใน cache เริ่มก่อนช่วง lookback ที่จำกัดไว้ mass status จะขยายหน้าต่างประวัติทั้งหมดไปถึงขาแรก
Mass status ดึง fill companion ที่ไม่มีในประวัติและ open order ด้วย venue ID แบบตรงตัว รวมถึง order ที่ไม่อยู่ใน cache
และเก็บไว้นอกหน้าต่าง lookback

ขาที่อยู่ใน cache คงการตรวจ identity และการแทนที่ไว้ order และ fill ที่ไม่อยู่ใน cache ซึ่งใช้ label ซ้ำนอกสายการแทนที่
เดียวกันจะละ client order ID การตรวจนี้ใช้ประวัติและสายของ open order ส่วน active open order คง ID ของตัวเองไว้

**ข้อจำกัดของการกู้คืน**:

- Startup ตรวจสายการแทนที่ที่มี label และไม่อยู่ใน cache ทั้งหมดบน subaccount ก่อนที่ filter ของ engine จะทำงาน รวมถึง
  สายของ bot อื่น สายที่ไม่ valid อาจขัดขวาง startup สายที่ยาวจะขยายการอ่านประวัติและอาจเกิน timeout ของ reconciliation
- อย่าให้ process อื่นแทนที่ order ที่ node นี้รับมาดูแล startup จะลงทะเบียน open order ทั้งหมดที่รับมาเพื่อติดตาม
  การแทนที่จากภายนอกที่ไม่มี pending modify ในเครื่องจะทำให้การผูกค้างอยู่ และขัดขวาง report และคำสั่งจนกว่าจะ reconcile
  สถานะ native และ restart node
- Snapshot แบบจำกัดขอบเขตจะทำเครื่องหมายว่าความครอบคลุมไม่สมบูรณ์ หากดึงประวัติไม่สำเร็จหรือ companion ที่ไม่อยู่ใน
  cache ยังหายไปหลัง fallback การดึงประวัติที่ล้มเหลวจะข้ามการค้นหา companion
- การกู้สายการแทนที่ที่ไม่อยู่ใน cache ตอน startup ต้องมีแหล่งข้อมูล order, fill และ position ครบ โดย native fill ต้อง
  ตรงกับแต่ละขาก่อนหน้าและปริมาณ fill สะสม มิฉะนั้น request จะล้มเหลว
- Identity ที่ขัดแย้งกันหรือสายการแทนที่ที่ไม่ valid ทำให้ request ล้มเหลว รวมถึง snapshot แบบจำกัดขอบเขต เมื่อมีประวัติ
  companion ใน cache ที่ใช้ไม่ได้หรือ error ในการค้นหาอื่นนอกจาก `11006` ก็ทำให้ request ล้มเหลวเช่นกัน

Request แบบ open-only คืน active order request สำหรับขาที่ถูกแทนที่ที่ทราบจะคืน order เชิงตรรกะปัจจุบัน query แบบ
order เดียวและแบบ bulk คืน native report สำหรับสายการแทนที่ที่ไม่อยู่ใน cache ส่วน reconciliation ตอน startup จะกู้สายนั้น

Snapshot ตอน reconnect แบบไม่จำกัดขอบเขตจะล้มเหลวหากแหล่งข้อมูลล้มเหลว record หาย หรือ fill ไม่มี order report ที่ตรงกัน
snapshot แบบไม่จำกัดขอบเขตจะหยุดที่ fill companion ตัวแรกที่ใช้ไม่ได้หลัง fallback และเก็บ error ของ venue ไว้เป็นสาเหตุ
โดยไม่ค้นหา companion ที่เหลือ แถวย้อนหลังที่เก็บไว้ซึ่ง precision ถูกอนุมานไม่ทำให้ snapshot เหล่านี้ล้มเหลว snapshot
แบบจำกัดขอบเขตเก็บ report ที่ valid ไว้ และทำเครื่องหมายช่องว่างของ metadata หรือ record ที่หายว่าความครอบคลุมไม่สมบูรณ์
หาก venue เปลี่ยนจำนวน record หรือจำนวนหน้าของประวัติ private ระหว่างการแบ่งหน้า adapter จะทิ้งการ scan และเริ่มใหม่
ไม่เกินสองครั้ง การเปลี่ยนจำนวนครั้งต่อไปจะทำให้ request ล้มเหลว

#### การเลือก margin model {#margin-model-selection}

ตั้ง `DeriveExecutionClientConfig.subaccount_id` เป็น subaccount แบบ Standard Margin (`SM`) หรือ Portfolio Margin
(`PM2`) ที่มีอยู่แล้ว แต่ละ subaccount ผูกกับ manager ที่เลือกตอนสร้างและ risk universe ของมัน adapter อ่าน identity
เหล่านั้นจาก `private/get_subaccount` เส้นทางของ account query, order, position report และ reconnect เดียวกันใช้ได้
กับทั้งสอง model ตั้งค่าสกุลเงินของ data client ให้ตรงกับ product ที่ manager นั้นรองรับ

สร้างหรือเติมเงินเข้า subaccount ผ่านเครื่องมือจัดการ account ของ venue ก่อนเชื่อมต่อ adapter ไม่ได้เปลี่ยน margin model
ของ subaccount และไม่ได้จำลอง portfolio risk engine ของ venue ดู
[managers and risk universes](https://docs.derive.xyz/trading/managers-and-risk-universes) ของ Derive สำหรับ
collateral และ instrument ที่รองรับ

Risk engine แบบ native โดยค่าเริ่มต้นตรวจ initial margin ราย instrument เทียบกับ free balance ในสกุลเงินที่ต้องใช้ ไม่ได้
ใช้ metadata สุขภาพสุทธิของ PM2 ของ Derive สำหรับการรับ order หรือการกำหนดขนาด portfolio ดังนั้นสุขภาพ PM2 ของ venue
ที่เป็นบวกจึงไม่ได้ยืนยันว่า engine แบบ native จะรับ order ที่ใช้ collateral สกุลเงินอื่น พฤติกรรมแบบ live กับ collateral
ข้ามสกุลเงินและ maintenance-margin credit (`mm_credits`) ที่ไม่เป็นศูนย์ยังไม่ได้รับการยืนยัน

#### Account state {#account-state}

Derive ถือ margin ไว้ในระดับ subaccount `AccountState` แยก collateral balance, margin requirement และสุขภาพสุทธิ
ของ account ออกจากกัน

##### Collateral balance {#collateral-balances}

- `total` คือ `amount` ของ collateral (เช่น USDC หรือ ETH) ปัดตาม precision ของสกุลเงินที่ลงทะเบียนไว้
- `locked` เป็นศูนย์ adapter เก็บวงเงินที่กันไว้สำหรับ vault deposit ที่รออยู่ไว้ใน metadata
- `free` เท่ากับ `total` ที่ปัดแล้ว
- `AccountState.info.collaterals` เก็บข้อมูลบางส่วนของแต่ละแถว collateral เป็น string ทศนิยมแบบแม่นยำ
- `collaterals[].initial_margin` คือ credit เป็น USD ที่ collateral นั้นมีส่วนให้ ไม่ใช่เงินที่ถูก lock

##### Margin requirement {#margin-requirements}

Adapter ปล่อย `MarginBalance` ระดับทั้ง account หนึ่งตัว:

| Field ของ Nautilus | Requirement เป็น USD ที่ derive มา                                    |
| ----------------- | ----------------------------------------------------------------- |
| `initial`         | `positions_value - positions_initial_margin - open_orders_margin` |
| `maintenance`     | `positions_value - positions_maintenance_margin`                  |

Field `positions_*_margin` ต้นทางเป็นส่วนที่มีต่อสุขภาพแบบมีเครื่องหมาย ซึ่งรวมมูลค่า mark-to-market ของ position การนำไปหัก
ออกจาก `positions_value` จะได้ risk requirement ที่สอดคล้องกัน `open_orders_margin` เป็นส่วนหักแบบมีเครื่องหมายสำหรับวงเงิน
ที่กันไว้ของ order ที่รออยู่ จึงเพิ่ม initial requirement ที่ derive ได้ adapter ใช้ค่ารวมของ subaccount ซึ่งยังคง offset
ของ portfolio ของ venue ไว้ และไม่ได้ปล่อย margin balance ราย instrument

Requirement ที่ปล่อยออกมาปัดตาม precision ของ USD แบบ native (ทศนิยมสองตำแหน่ง ปัดกรณีกึ่งกลางไปหาเลขคู่) array
`currency` ของ subaccount ไม่ได้กำหนดสกุลเงินของ requirement ค่าต้นทางที่ model ไว้ยังอยู่ใน `AccountState.info`
adapter ไม่ได้คำนวณ haircut ของ collateral หรือปรับค่าอื่นใดสำหรับ `mm_credits`

:::warning
`net_initial_margin` และ `net_maintenance_margin` ใน `AccountState.info` รายงานสุขภาพสุทธิปัจจุบัน รวมมูลค่า
mark-to-market ของ position แต่ไม่รวมวงเงินที่กันไว้สำหรับ open order ที่รออยู่ ใช้ order quote ของ venue สำหรับสุขภาพ
initial margin ก่อน/หลัง การรับ order จริงเป็นตัวตัดสินว่า order เพิ่มเติมจะ execute ได้หรือไม่
:::

##### Metadata ของ account {#account-metadata}

`AccountState.info` มี identity ของ account, สุขภาพสุทธิแบบมีเครื่องหมาย และค่าความเสี่ยงแบบแม่นยำ:

- `net_initial_margin` และ `net_maintenance_margin` เก็บสุขภาพสุทธิ
- `positions_value` เก็บมูลค่า mark-to-market รวมของ position
- `positions_initial_margin` และ `positions_maintenance_margin` เก็บส่วนที่ position มีต่อสุขภาพแบบมีเครื่องหมาย
- `open_orders_margin` เก็บส่วนหักของสุขภาพ initial แบบมีเครื่องหมายสำหรับวงเงินที่กันไว้ของ order ที่รออยู่
- `margin_type` และ `manager_id` ระบุ margin model ส่วน `risk_universe_id` ระบุ risk universe
- `currency` เก็บ array สกุลเงินของ venue
- `collaterals` เก็บข้อมูลบางส่วนของแต่ละแถว collateral เป็น string ทศนิยมแบบแม่นยำ
- `mm_credits` เก็บ maintenance-margin credit จากการกระจายความสูญเสียของ risk universe (loss socialization)
- `projected_margin_change` เก็บการเปลี่ยนแปลงที่คาดการณ์ของ `net_maintenance_margin` หนึ่งนาทีหลังการหมดอายุของ
  option ครั้งถัดไป
- `vault_deposit_holds` เก็บวงเงินที่กันไว้ที่รออยู่ รวมถึงจำนวนแบบ native และ vault ID เป้าหมาย
- `is_under_liquidation` เก็บสถานะการ liquidation

ค่าทศนิยมที่ model ไว้เป็น JSON string ที่แม่นยำภายในช่วงของ `Decimal` และ scale สูงสุด 28 ค่าต้นทางที่เกิน scale
นั้นจะถูกปัดระหว่าง decode balance แบบ native อาจปัดจำนวนที่เล็กกว่า precision ของสกุลเงินเป็นศูนย์ได้ด้วย ใช้ metadata
ของ collateral เมื่อ precision นั้นสำคัญ venue ไม่ได้ระบุว่า vault hold ที่รออยู่รวมอยู่ในจำนวน collateral หรือไม่ adapter
จึงไม่ได้หักออกหรือบวกเข้ายอดเงิน

Snapshot ที่ถูกทำเครื่องหมาย `failed_to_fetch` หรือขาด field ที่จำเป็นจะไม่ปล่อย `AccountState` ทำให้การเชื่อมต่อครั้งแรก
ล้มเหลว และข้าม mass status หลัง reconnect แถวซ้อนที่ไม่ valid รวมถึง position ที่อยู่นอก precision หรือช่วงของ `Quantity`
แบบ native จะทำให้ snapshot ของ account ทั้งหมดล้มเหลว request ของ position report ยังกำหนดให้จำนวนต้องพอดีกับ size
precision ของ instrument ใน cache การแปลง position ที่ล้มเหลวจะทำให้ mass status ทั้งหมดถูกปฏิเสธ รวมถึง order report
และ fill report ซึ่งช่วยรักษา position ใน cache ไว้ แทนที่จะถือว่า position report ที่หายไปเป็น flat

Position report เก็บราคาเข้าเฉลี่ยของ venue ไว้ สำหรับ perpetual ราคานี้รวม fee ตอนเปิดและอาจต่างจากค่าเฉลี่ยที่สร้างขึ้น
จากราคา fill จริง adapter ใช้
[`AvgPxReconciliation::OpeningOnly`](../concepts/execution/reconciliation.md#reported-entry-averages) สำหรับ
perpetual: ปริมาณ position ที่ตรงกันไม่จำเป็นต้องให้ค่าเฉลี่ยที่ derive จาก fill เท่ากับค่าเฉลี่ยที่รายงาน ราคา ปริมาณ และ
commission ของ fill จริงยังครบถ้วน Nautilus ไม่นับ commission ในราคาเข้าเฉลี่ยของ position และหักออกจาก realized PnL
ส่วน instrument type อื่นยังใช้ reconciliation ราคาเฉลี่ยแบบค่าเริ่มต้น

เมื่อไม่มีประวัติ fill การกู้คืนจะเปิด position สังเคราะห์ที่ราคาเฉลี่ยที่รายงาน ปัดตาม price precision ของ instrument
เมื่อมีประวัติบางส่วน การกู้คืนแบบสังเคราะห์อาจได้ cost basis โดยประมาณ ตั้ง lookback ของ reconciliation ให้ครอบคลุม
fill ที่เปิด position ปัจจุบันเมื่อต้องการการบัญชีจาก fill ที่แม่นยำ

#### Conditional order {#conditional-orders}

Trigger order ของ Derive ใช้ `private/order` พร้อม field ของ trigger venue กำหนด order id และเก็บ order ไว้ด้วย
`order_status=untriggered` จนกว่า trigger worker ของ venue จะส่ง child order ที่ sign แล้ว reconciliation จึงอ่านทั้ง
`private/get_open_orders` และ `private/get_trigger_orders`

การ trigger บน v3 แบบ live และการกู้การอัปเดตที่พลาดไปเมื่อประวัติหรือมุมมอง position มาช้า ยังไม่ได้รับการยืนยัน
การ reconnect หรือ restart ที่สำเร็จไม่ได้ยืนยันกรณีเหล่านั้น

##### วันหมดอายุของ signature {#signature-expiry}

API v3 รับ signature ของ order ที่มีอายุในช่วง [10 วินาที, 120 วัน] order แบบ MMP มีอายุสูงสุด 15 นาที
adapter sign trigger order ด้วยวันหมดอายุคงที่ 31 วันและปิด flag MMP ไว้
`signature_expiry_secs` ควบคุม order ปกติและการแทนที่ ต้องมากกว่า 10 วินาทีเพื่อเผื่อ latency ของการ sign โดยสูงสุด
120 วัน หรือ 900 วินาทีเมื่อ `cancel_on_disconnect=True`
ดู [action signing](https://docs.derive.xyz/authentication/action-signing)

##### Type ที่รองรับ {#supported-types}

| Nautilus order type | รองรับ | Derive `order_type` | Derive `trigger_type` | หมายเหตุ                              |
| ------------------- | ------ | ------------------- | --------------------- | ----------------------------------- |
| `StopMarket`        | ✓      | `market`            | `stoploss`            | ใช้ trigger price เป็นขอบเขต           |
| `StopLimit`         | ✓      | `limit`             | `stoploss`            | ส่งทั้ง limit price และ trigger price    |
| `MarketIfTouched`   | ✓      | `market`            | `takeprofit`          | ใช้ trigger price เป็นขอบเขต           |
| `LimitIfTouched`    | ✓      | `limit`             | `takeprofit`          | ส่งทั้ง limit price และ trigger price    |
| `MarketToLimit`     | -      | -                   | -                     | *Derive ไม่รองรับ*                    |
| Trailing stops      | -      | -                   | -                     | *Derive ไม่รองรับ*                    |
| TWAP / algo / RFQ   | -      | -                   | -                     | *Adapter นี้ไม่ได้เปิดให้ใช้*             |

Adapter map `TriggerType::Default` และ `TriggerType::MarkPrice` ของ Nautilus เป็น `trigger_price_type=mark`
ของ Derive เอกสารอ้างอิง error code ของ Derive ระบุว่า trigger price type แบบ index และ last-trade ไม่รองรับ ดังนั้น
adapter จะปฏิเสธ `IndexPrice`, `LastPrice`, `BidAsk` และ trigger price type อื่นในเครื่องด้วย `OrderDenied` ก่อนส่ง

##### การอัปเดต trigger {#updating-a-trigger}

Error `11054` ของ Derive ระบุว่า trigger order แทนที่หรือถูกแทนที่ไม่ได้ adapter จึงปฏิเสธ request modify ของ Nautilus
สำหรับ trigger order ด้วย event `OrderModifyRejected` ให้ยกเลิกแล้วส่งใหม่เพื่ออัปเดต trigger

##### ระยะห่างของ trigger price {#trigger-price-offsets}

Derive ตรวจฝั่งของ trigger price และปฏิเสธ trigger ที่ไม่ได้อยู่เลยราคาปัจจุบันในทิศทางที่คาดไว้ด้วย error `11051`
trigger price ถูกกำหนดตายตัวตอน sign order ดังนั้นระยะห่างที่แคบบน instrument ที่ราคาเคลื่อนไหวเร็วหรือราคาสูงอาจเลื่อน
ไปอยู่ผิดฝั่งก่อนที่ venue จะได้รับ

:::warning
กำหนดระยะห่างของ trigger ให้มากกว่าการเคลื่อนไหวของราคาที่คาดไว้ระหว่างการส่งอย่างเผื่อเหลือ (สำหรับ `ETH-PERP`
คือหลักสิบดอลลาร์ ไม่ใช่ไม่กี่เซนต์) ระยะห่างที่แคบเกินไปจะทำให้เกิดการ reject ด้วย `11051` โดยไม่จำเป็น
:::

#### Cancel on disconnect {#cancel-on-disconnect}

ตั้ง `cancel_on_disconnect=True` ใน `DeriveExecutionClientConfig` เพื่อเปิด
[cancel-on-disconnect (COD)](https://docs.derive.xyz/trading/cancel-on-disconnect) สำหรับ limit order แบบ GTC และ
post-only ที่ resting อยู่ รวมถึงการแทนที่
session key ต้องมี `trade:all` (`Trade(All)`) ครอบคลุมทุก subaccount key สำหรับ onboarding ที่จำกัดไว้ที่
`trade:orderbook:all` ใช้ไม่ได้

เมื่อเปิดตัวเลือกนี้:

- **Startup**: client จะเปิด COD ระดับทั้ง wallet รอการยืนยันจาก account ได้นานสุด 30 วินาที และยืนยันตัวตนการเชื่อมต่อ
  execution ใหม่ก่อนจะพร้อมใช้งาน หากการตั้งค่าหรือการ login ล้มเหลว จะปิดการเชื่อมต่อและทำให้ startup ล้มเหลวโดยไม่ได้
  ปิดค่าตั้งของ wallet เผื่อ connection timeout ของ node ไว้สำหรับการยืนยันและ startup ตามปกติ ซึ่งค่าเริ่มต้นคือ 60 วินาที
- **Order**: การส่งและการแทนที่ที่เข้าเงื่อนไขจะมี `mmp: true` วันหมดอายุของ signature ต้องไม่เกิน 900 วินาที ค่าเริ่มต้น
  คือ 600 order แบบ IOC, FOK, market และ trigger คง flag MMP เดิมไว้ tag MMP ยังทำให้ threshold และกฎการ freeze ของ
  MMP ที่มีอยู่มีผลด้วย adapter ไม่ได้ตั้งค่าหรือ reset MMP
- **วงจรชีวิต**: ค่าตั้งของ wallet คงอยู่ถาวรและมีผลตอน login การ reconnect อัตโนมัติใช้ค่าตั้งนั้นโดยไม่สลับค่า และการ
  shutdown จะไม่ปิดค่านี้ ประสานงานกับ client อื่นที่ใช้ wallet ร่วมกัน การเชื่อมต่อและ order ที่มีอยู่แล้วไม่ได้รับการคุ้มครอง
  ย้อนหลัง

ค่าเริ่มต้น `False` จะไม่เปลี่ยนค่าตั้งของ wallet ที่เก็บไว้และ flag MMP ของ order

:::warning
COD ยกเลิกเฉพาะ order ที่มี `mmp: true` และ RFQ quote ของการเชื่อมต่อที่หลุดเท่านั้น รวมถึงกรณี server shutdown อย่างนุ่มนวล order ที่ไม่ใช่ MMP, trigger order และ algo order
อยู่นอกการคุ้มครองนี้ ให้คงขั้นตอนการยกเลิกและ shutdown อย่างชัดเจนไว้
:::

หากต้องการควบคุมเอง ให้ใช้ `DeriveWebSocketClient::set_cancel_on_disconnect(enabled)` ระดับล่างของ Rust
มีเพียง `"ok"` ที่ยืนยันว่าสำเร็จ error จาก venue และ response ที่ผิดรูปแบบจะถูกส่งต่อขึ้นมา timeout, reply ที่หาย หรือ reply
ที่ผิดรูปแบบจะทำให้ค่าตั้งที่เก็บไว้ไม่แน่นอน ให้ประสานงานกับ client อื่นและยืนยันค่าที่ต้องการอีกครั้ง:

1. รอจน `private/get_account` รายงานค่า `cancel_on_disconnect` ที่ได้รับ acknowledgement ภายในกำหนดเวลาของคุณ
   การอ่าน account อาจช้ากว่า acknowledgement
1. เรียก `disconnect()` แล้ว `connect()` บน client แต่ละตัวที่ได้รับผลกระทบ การเปลี่ยนแปลงไม่มีผลกับการเชื่อมต่อที่มีอยู่

การแปลง order ระดับล่างจะปิด MMP ไว้ ให้ตั้ง `mmp: Some(true)` บน payload ที่เข้าเงื่อนไขซึ่งส่งผ่าน
`execution_handle().submit_order` และ sign ด้วย[อายุไม่เกิน 15 นาที](#signature-expiry)

#### การยกเลิกแบบ bulk {#bulk-cancellation}

##### การเลือกและการ routing {#selection-and-routing}

Derive รองรับ method การยกเลิกหลาย order ทั้งสองแบบที่ `Strategy` มีให้

| Strategy method          | รองรับ | Parameter                                                             | หมายเหตุ                                       |
| ------------------------ | ------ | --------------------------------------------------------------------- | -------------------------------------------- |
| `cancel_orders(...)`     | ✓      | `client_order_ids`, `client_id`, `params`                             | ทุก order ต้องใช้ instrument เดียวกัน              |
| `cancel_all_orders(...)` | ✓      | `instrument_id`, `order_side`, `client_id`, `strategy_only`, `params` | ค่าเริ่มต้นคือ order ของ strategy ที่เรียกใช้           |

Execution client ของ Derive ใช้ method เหล่านี้ดังนี้:

- **Order ที่ระบุชัดเจน**: `cancel_orders` ยกเลิก order ปกติหรือ trigger order ที่ request ทีละตัว
- **Strategy-only**: `cancel_all_orders` ที่ `strategy_only=True` จะขยายรายการที่ตรงกันใน cache เป็นการยกเลิกทีละตัว
- **กรองตามฝั่ง**: `cancel_all_orders` ที่ `strategy_only=False` พร้อม filter ฝั่ง Buy หรือ Sell จะเลือก order ปกติ
  และ trigger order ที่เปิดอยู่จาก cache สำหรับ execution client, account, instrument ที่ตรงตัว และฝั่งที่ตั้งค่าไว้
  แล้วยกเลิกทีละตัว โดยจะไม่ขยายไปทั้งสองฝั่ง
- **ทั้งสองฝั่ง**: `cancel_all_orders` ที่ `strategy_only=False` และไม่มี filter ฝั่ง จะเลือก trigger ที่เปิดอยู่และตรงกัน
  จากขอบเขต execution client, account และ instrument เดียวกัน แล้วยกเลิกทีละตัว จากนั้นจึงส่ง
  `private/cancel_by_instrument` สำหรับ order ปกติ โดยจะไม่ส่ง `private/cancel_all` เลย

`cancel_all_orders` เลือก order ที่เข้าเงื่อนไขจาก cache โดยไม่ refresh สถานะของ venue สำหรับแต่ละ order ที่เลือก การผูก
กับ venue ปัจจุบันของ execution client มีลำดับความสำคัญเหนือ ID ใน cache trigger ที่รออยู่ใช้
`private/cancel_trigger_order` การยกเลิก trigger ที่ activate แล้วแบบทีละตัวใช้ `private/cancel` โหมดทั้งสองฝั่งปล่อยให้
trigger ที่ activate แล้วไปจัดการด้วย `private/cancel_by_instrument` client จะตรวจการ activate ซ้ำหลังรอตาม pacing
และ route trigger ที่ activate ระหว่างรอผ่าน `private/cancel` ก่อนนำ request เข้าคิว

##### การจัดการความล้มเหลว {#failure-handling}

เมื่อ `strategy_only=False` `cancel_all_orders` จะล้มเหลวแบบปิด (fail closed) หาก order ที่เลือกไม่มีทั้งการผูกปัจจุบัน
และ venue order ID ใน cache คำสั่งทั้งหมดจะ log warning ไม่ส่ง request ยกเลิกใด ๆ และไม่ปล่อย order event

`private/cancel_by_instrument` ยกเลิกเฉพาะ order ปกติที่เปิดอยู่ request ที่สำเร็จโดยมี `cancelled_orders == 0` เป็น
no-op ที่คาดไว้และ log ที่ระดับ debug

การยกเลิก trigger ที่ล้มเหลวจะ log warning แต่ไม่ระงับการยกเลิก order ปกติของ instrument request แบบ bulk ที่ล้มเหลว
ไม่มีผลลัพธ์ราย order ให้ปล่อย การอัปเดตจาก private channel และ reconciliation ภายหลังยังคงรับผิดชอบสถานะ order ที่สังเกตได้

#### Execution instruction {#execution-instructions}

| Instruction   | รองรับ | ค่าของ Derive   | หมายเหตุ                                                           |
| ------------- | ------ | ------------- | ----------------------------------------------------------------- |
| `post_only`   | ✓      | `post_only`   | ต้องใช้ `GTC` และจะ reject หาก order จะดึง liquidity                 |
| `reduce_only` | ✓      | `reduce_only` | Perp และ option เท่านั้น เฉพาะ market หรือ `IOC`/`FOK` spot ถูกปฏิเสธ |
| `display_qty` | -      | -             | *Derive ไม่รองรับ*                                                  |

Adapter ไม่รองรับ instruction แบบ iceberg order ที่ตั้ง `display_qty` จะได้รับ `OrderDenied` ก่อนส่ง รวมถึง conditional
order หากขาใดตั้ง `display_qty` adapter จะปฏิเสธทุกขาก่อนส่งขาใด ๆ order ที่มีอยู่แล้วซึ่งมี `display_qty` ยังยกเลิกได้
แต่การแก้ไขจะถูกปฏิเสธ

#### Time in force {#time-in-force}

Derive ระบุ `gtc`, `post_only`, `fok` และ `ioc` เป็นค่า `time_in_force` ของตน ค่าของ Nautilus ที่ไม่มีค่าเทียบเท่าใน
Derive จะถูกปฏิเสธในเครื่องด้วย `OrderDenied` ก่อนส่ง Derive ให้ post-only เป็นค่าหนึ่งของ `time_in_force` ดังนั้น
`post_only` จึงใช้ร่วมกับ `IOC` หรือ `FOK` ไม่ได้

| Time in force  | รองรับ | ค่าของ Derive  | หมายเหตุ                       |
| -------------- | ------ | ------------ | ---------------------------- |
| `GTC`          | ✓      | `gtc`        | Good Till Canceled           |
| `IOC`          | ✓      | `ioc`        | Immediate or Cancel          |
| `FOK`          | ✓      | `fok`        | Fill or Kill                 |
| `GTD`          | -      | -            | *Derive ไม่รองรับ*             |
| `DAY`          | -      | -            | *Derive ไม่รองรับ*             |
| `AT_THE_OPEN`  | -      | -            | *Derive ไม่รองรับ*             |
| `AT_THE_CLOSE` | -      | -            | *Derive ไม่รองรับ*             |

สำหรับ limit แบบ IOC และ FOK ที่เป็น passive และไม่มี liquidity ภายใน limit price Derive จะคืน
[`11009: Zero liquidity for market or IOC/FOK order`](https://docs.derive.xyz/error-codes)
ใน execution test case [TC-E14](../developer_guide/spec_exec_testing.md#tc-e14-limit-ioc-passive---no-fill)
และ [TC-E16](../developer_guide/spec_exec_testing.md#tc-e16-limit-fok-no-fill) ต้องเห็น
`OrderInitialized` -> `OrderSubmitted` -> `OrderRejected` สถานะสิ้นสุด `Rejected` ปริมาณที่ fill เป็นศูนย์ และเหตุผล
การ reject ตรงตามนั้น การ reject ด้วยเหตุผลอื่นถือว่าไม่ผ่าน adapter เก็บการ reject ของ venue ไว้โดยไม่ปล่อย
`OrderAccepted` หรือ `OrderCanceled`

#### Reduce-only order บน spot {#spot-reduce-only-orders}

Spot ของ Derive ไม่มีแนวคิดเรื่อง position ดังนั้น spot order แบบ reduce-only จึงไม่มีทางลดอะไรได้ venue จะ reject ด้วย
error `11025` เสมอ adapter หลีกเลี่ยงการส่งไปกลับนั้นเมื่อรู้ว่า instrument เป็น spot spot instrument ใน cache จะถูกปฏิเสธ
ด้วย `OrderDenied` ส่วน spot instrument ที่ resolve แบบ lazy จะถูก reject ด้วย `OrderRejected` ระหว่างการส่ง

Reduce-only order สำหรับ perpetual และ option ยังคงไปถึง venue ซึ่งผลลัพธ์จะขึ้นกับสถานะ position ของ subaccount
bin `derive-flatten` ปิดเฉพาะ position ของ derivative และไม่ปิด spot เลย เพราะการ flatten ยอด spot จะเทขาย base asset
ไปเป็น quote อื่น

##### ข้อจำกัด time-in-force ของ reduce-only {#reduce-only-time-in-force-limits}

Derive ยอมรับ `reduce_only` เฉพาะบน market order หรือ limit ที่ไม่ rest (`IOC`/`FOK`) limit แบบ `GTC` หรือ post-only
ที่ resting และมี `reduce_only` จะถูกปฏิเสธในเครื่องก่อนส่ง การส่งตรงไปยัง venue จะล้มเหลวด้วย
`11024 Reduce only not supported with this time in force`

:::warning
หากขาใดไม่ผ่านการตรวจ list ล่วงหน้าในเครื่อง adapter จะปฏิเสธทุกขาก่อนส่งขาใด ๆ list ที่มี order ซึ่งมี contingency type
(`OTO`, `OCO` หรือ `OUO`), parent order ID หรือ linked order ID ที่ไม่ว่าง จะถูกปฏิเสธทั้ง list แม้เงื่อนไข order ของแต่ละขา
จะ valid ก็ตาม

การเอา `reduce_only` ออกจาก take-profit แบบ `GTC` ไม่ได้ช่วยเลี่ยงการปฏิเสธนี้ ให้ใช้
[local order emulation](../concepts/orders/emulated.md) หรือส่ง order ทีละตัวแล้วจัดการการ activate, การยกเลิก sibling
และการอัปเดตปริมาณใน strategy เอง list อิสระที่ไม่ผูกกันยังรองรับตามปกติ
:::

#### Semantics ของการ reject order {#order-rejection-semantics}

การเขียนที่เปลี่ยนสถานะ (`submit_order`, `modify_order`, `cancel_order`) จะส่งผ่าน WebSocket ครั้งเดียวและ **ไม่ถูก
replay** ผลลัพธ์ของ WebSocket request เป็นตัวกำหนดว่า adapter จะปล่อยการ reject แบบสิ้นสุดหรือรอ reconciliation

##### ความล้มเหลวที่ชัดเจน {#definitive-failures}

Adapter ปล่อย rejection event แบบสิ้นสุด (`OrderRejected`, `OrderModifyRejected`, `OrderCancelRejected`) สำหรับ
ความล้มเหลวที่ชัดเจนจาก venue:

- การ reject signed action เช่น parameter ไม่ถูกต้อง margin ไม่พอ หรือ order ที่ไม่รู้จัก
- Business code ของ venue เช่น `11009 Zero liquidity`
- การ reject เพราะ post-only ข้ามราคา (`11008 Post only order cannot cross the market`) ซึ่งรายงานเป็น `OrderRejected`
  พร้อม `due_post_only=true`
- Response แบบ rate-limit (`-32000 Rate limit exceeded`) ซึ่ง gateway reject request ก่อนที่ matching engine จะเห็น
- Response ที่สำเร็จของ `private/cancel_by_label` โดยมี `cancelled_orders == 0` ซึ่งหมายความว่าไม่มี open order ที่ตรงกับ
  label ของ client order ID

Response ของ `private/cancel_by_label` แบบไม่จำกัดขอบเขตที่มี `cancelled_orders == -1` เป็นการ acknowledge request
จำนวนที่เป็นบวกก็ต้องรอการอัปเดต private order เช่นกัน ทั้งสองกรณีจะไม่สร้าง event `OrderCanceled` หากไม่มีสถานะ order
ที่ยืนยันแล้ว `query_order` ใช้การค้นหา order status ร่วมกันสำหรับ reconciliation

##### การจำแนก post-only {#post-only-classification}

สำหรับ post-only order ที่ไปถึง venue Derive จะ reject order ที่ข้ามราคาด้วย JSON-RPC `11008` และ message
`Post only order cannot cross the market` adapter ทำเครื่องหมายการ reject แบบสิ้นสุดนั้นด้วย `due_post_only=true` หาก
การ reject จาก WebSocket/order report มีเหตุผลเดียวกัน เส้นทางของ order ที่ติดตามจะจำแนกแบบเดียวกัน การปฏิเสธในเครื่อง
สำหรับชุด post-only IOC/FOK ที่ไม่รองรับเป็น event `OrderDenied` ที่ไม่มี `due_post_only` เพราะไม่ได้แทนการ reject จาก
การข้ามราคาที่ venue

เหตุผลการ reject ที่ส่งให้ strategy จะตัด markup และ control character ออก normalize ช่องว่าง และมีความยาวไม่เกิน 256
ตัวอักษร เหตุผลจาก JSON-RPC คง error code และ message ดั้งเดิมไว้ โดยไม่มี prefix ของ exception หรือ payload วินิจฉัยแบบ
มีโครงสร้าง log จะเก็บ error ต้นฉบับไว้

##### ผลลัพธ์ที่กำกวม {#ambiguous-outcomes}

สำหรับผลลัพธ์ของการเขียนที่กำกวม adapter จะไม่ปล่อย event สิ้นสุด และปล่อยให้ WebSocket reconciliation หรือ status report
ภายหลังเป็นตัวกำหนดสถานะ ชุดของกรณีกำกวมถูกจำกัดให้แคบโดยตั้งใจ:

- `-32603` ซึ่งเป็น internal error ทั่วไปของ JSON-RPC
- `9000` และ `9001` ซึ่งเป็น confirmation timeout ที่ต้อง query สถานะ order ก่อนส่งใหม่
- Response ที่ decode ไม่ได้ (action อาจถูกประมวลผลไปแล้ว)
- Response ที่สำเร็จแต่ venue order ID แทนค่าไม่ได้ adapter จะเก็บ identity ของคำสั่งที่ทราบและสถานะการแทนที่ที่รออยู่ไว้
  สำหรับ reconciliation หาก ID นั้นยังแทนค่าไม่ได้ status report และ fill report จะติดตาม venue order ไม่ได้ ให้ตรวจ
  สถานะของ venue โดยใช้ identity ของคำสั่งที่ log ไว้
- Request timeout, response ที่หายไปตอน reconnect และ error ของ transport

การแยกแยะนี้ปกป้องทั้งสองด้านของวงจรชีวิต order การ reject แบบสิ้นสุดที่ผิดอาจทำให้ engine ถือว่า order ที่ live อยู่
ถูก reject ส่วนผลลัพธ์กำกวมที่ผิดอาจทำให้ order ที่ไม่ได้วางค้างอยู่ใน `Submitted` ตลอดไป เพราะจะไม่มี WebSocket frame มาถึง

## Rate limiting {#rate-limiting}

### ขอบเขตของ budget และการเติม {#budget-scopes-and-refill}

Derive v3 ใช้ [rate limit สองระดับ](https://docs.derive.xyz/rate-limits) budget ของ request ใช้หน้าต่างคงที่ห้าวินาที
budget ของ order ราย instrument เติมต่อเนื่อง โดยมีความจุเท่ากับห้าวินาทีของอัตราการเติมที่ตั้งค่าไว้ bucket ของ instrument
ที่หมดจะได้ token หนึ่งตัวหลังหนึ่งวินาทีที่อัตราเริ่มต้น ขอบหน้าต่างไม่ได้เติมความจุทั้งหมดคืน

HTTP client และ WebSocket client ที่ยืนยันตัวตนบน API host เดียวกันใช้ budget ของ wallet ร่วมกันข้าม session key และ
subaccount request แบบ public รวมถึงการ login ทาง WebSocket ใช้ budget แยกต่างหากภายใน process ต่อ API host ซึ่งสมมติ
แบบระมัดระวังว่ามี source IP เดียว client ที่อยู่หลัง proxy ต่างกันก็ยังใช้โควตานั้นร่วมกัน client ใน process ต่างกันต้อง
ประสานปริมาณ traffic รวมกันเอง URL แบบกำหนดเองที่มี API host ต่างกันใช้สถานะของ limiter แยกกัน

หน้าต่างของ request จะตรงกับเวลาที่สร้าง limiter ที่ใช้ร่วมกัน เพราะสังเกตเฟสของหน้าต่างของ venue ไม่ได้ burst ทั้งสองฝั่ง
ของขอบในเครื่องอาจมาถึงภายในหน้าต่างเดียวของ venue โดยเฉพาะเมื่อคิวของ transport ทำให้การส่งออกล่าช้า venue จะ reject
traffic ที่เกินด้วย `-32000 Rate limit exceeded` และ adapter แสดงผลนี้เป็นการ reject ที่ชัดเจน (ดู
[semantics ของการ reject order](#order-rejection-semantics))

### Request bucket {#request-buckets}

ตารางนี้แสดงค่าเริ่มต้นอ้างอิงและโควตา label-cancel ของ adapter ค่าของ deployment และโควตาที่ตกลงกันอาจต่างออกไป
อ่าน budget ของ request แบบ live ได้ด้วย `public/getRateLimits` endpoint นี้ไม่ได้รายงาน budget ราย instrument บน
testnet endpoint ทาง HTTP จะรายงานโควตาของ public IP แม้จะมี header ยืนยันตัวตน ขณะที่ WebSocket ที่ยืนยันตัวตนจะรายงาน
โควตาของ wallet

| Bucket                             | ขอบเขต                  | โควตาอ้างอิง                         |
| ---------------------------------- | --------------------- | ---------------------------------- |
| Matching                           | Wallet                | 5 request ต่อหน้าต่าง 5 วินาที          |
| Non-matching, authenticated        | Wallet                | 25 request ต่อหน้าต่าง 5 วินาที         |
| Public requests                    | IP                    | 25 request ต่อหน้าต่าง 5 วินาที         |
| `private/cancel_all`               | Wallet และ method     | 5 request ต่อหน้าต่าง 5 วินาที          |
| `private/cancel_by_label` endpoint | Wallet และ method     | Adapter: 50 request ต่อหน้าต่าง       |
| Instrument orders, perpetuals/spot | Wallet และ instrument | 1 token/วินาที ความจุ 5              |
| Instrument orders, options         | Wallet และ instrument | 1 token/วินาที ความจุ 5              |

Venue ยังจำกัดจำนวนการเชื่อมต่อ WebSocket พร้อมกันต่อ IP ค่าอ้างอิงที่เผยแพร่คือสี่การเชื่อมต่อ แต่ค่าของ deployment
อาจต่างกัน

[Integrator reference](https://docs.derive.xyz/integrators/trading/rate-limits) เผยแพร่ 10 TPS สำหรับการยกเลิกด้วย
label แบบไม่จำกัดขอบเขต ส่วน reference หลักไม่ได้ให้ตัวเลขโควตาของ endpoint adapter คง 10 TPS ไว้เป็น budget ของ
endpoint label-cancel

#### การจัดสรรการเขียนแบบ matching {#matching-write-allocation}

Request ของ order, replace, cancel, cancel-by-nonce และ cancel-by-instrument ใช้ทั้ง budget ของ matching request และ
token bucket ของ instrument ที่ระบุ config ราย instrument ใช้กับ perpetual, spot และ option และเป็นอิสระจากโควตา
matching ของ wallet

Adapter กำหนดจังหวะ `cancel_by_label` ตามทั้ง budget ของ matching และของ endpoint ตาม rate-limit reference หลัก
HTTP request ดิบที่ body มี `instrument_name` ก็ใช้ credit ของ instrument ด้วย ขณะนี้ parameter แบบมี type ของ
label-cancel รองรับเฉพาะการเรียกแบบไม่จำกัดขอบเขต ปัจจุบัน testnet คิด budget ของ matching เฉพาะ label cancel แบบจำกัด
ขอบเขต adapter คงกฎตามเอกสารที่เข้มงวดกว่าไว้สำหรับการเรียกแบบไม่จำกัดขอบเขต `cancel_all` ใช้ budget ของ endpoint
ตัวเองเพียงอย่างเดียว

`cancel_trigger_order` และการอ่าน trigger order ใช้ budget ของ non-matching การสร้าง trigger ใช้ `private/order`
และจองโควตา matching ก่อน sign

ค่าตั้งของ wallet และ instrument ที่ระบุชัดเจนครั้งแรกจะใช้กับ limiter ที่ใช้ร่วมกัน constructor ภายหลังที่ไม่ได้ตั้งตัวเลือก
จะคงค่าเหล่านั้นไว้ limiter จะไม่สนใจค่าตั้งที่ระบุชัดเจนแต่ขัดแย้งและ log warning หนี้ของ budget ที่ใช้ร่วมกันคงอยู่ข้ามการ
reconnect และการสร้าง client ใหม่

### การ sign และ response ของ venue {#signing-and-venue-responses}

Header ยืนยันตัวตนของ HTTP และ timestamp ตอน login ทาง WebSocket จะถูก sign หลังการรอตาม pacing execution client
จองโควตาการสร้างและแทนที่ order ทาง WebSocket ก่อนสร้าง nonce และ signature แบบ EIP-712 การจองที่ถืออยู่จะใช้ความจุ
ของ instrument จนกว่าจะ dispatch และการจองที่ถูกทิ้งจะคืน credit ของ request และ instrument ที่ไม่ได้ใช้

การ dispatch ทาง WebSocket ที่จองไว้แล้วจะไม่รออีกหลัง sign หากการจองข้ามขอบหน้าต่างของ request adapter จะคิดหน้าต่าง
request ปัจจุบันทันที หากหน้าต่างนั้นเต็มจะคืน error การ throttle ในเครื่องแบบชัดเจนก่อนนำการเขียนเข้าคิว credit ของ
instrument ถูกคิดครั้งเดียว

Method ส่ง order ทาง HTTP และ method ส่ง signed order ทาง WebSocket โดยตรงรับ body ที่ sign แล้ว ผู้เรียกต้องคำนึงถึง
ความล่าช้าจาก pacing ให้อยู่ภายในช่วงเวลาที่ signature เหล่านั้นยัง valid

การ throttle ของ venue ยังคงเป็นการ reject ที่ชัดเจน ความล้มเหลวของ transport, timeout และผลลัพธ์การเขียนที่กำกวมต้อง
อาศัย reconciliation adapter ไม่ retry การเขียนที่ sign แล้วแบบสุ่มสี่สุ่มห้า

## การกู้คืน WebSocket {#websocket-recovery}

Data client และ execution client จะ reconnect อัตโนมัติหลังอีกฝั่งปิดการเชื่อมต่อ เกิด error ของ transport หรือ heartbeat
timeout adapter ส่ง protocol Ping frame ทุก 30 วินาที และถือว่าการไม่มี frame ขาเข้าใดเลยนาน 60 วินาทีเป็นการเชื่อมต่อที่ตาย
transport ที่เลือกอาจรายงาน Pong ที่หายไปได้เร็วกว่านั้น transport จะ retry การเชื่อมต่อด้วย exponential backoff และ jitter

การกู้คืนเสร็จสิ้นตามลำดับนี้:

1. Transport reconnect หากเกิดการหลุดอีก การกู้คืนจะตามการเชื่อมต่อล่าสุด
1. Session ที่มี credential จะ login ใหม่ จากนั้นทุก session จะ replay subscription ที่ยืนยันแล้วและที่รออยู่ตามลำดับ channel
   ส่วนการ unsubscribe ที่รออยู่และที่ acknowledge แล้วจะไม่ถูก replay
1. Execution client จับ context ของ order ที่เป็นเจ้าของปัจจุบันบน runner thread, refresh สถานะ account แล้วสร้าง mass
   status สำหรับ order, fill และ position หาก snapshot ของ account ถูกปฏิเสธจะข้าม mass status และการแปลง position
   ที่ล้มเหลวจะทำให้ mass status ทั้งหมดถูกปฏิเสธ

Host แบบกำหนดเองต้องประมวลผล message ของ time event จาก runner เพื่อ refresh context ของ order ใน cache ตอน reconnect
การ refresh ใช้กำหนดเวลา `ws_timeout_secs` ที่ตั้งค่าไว้ หากไม่มีตัวส่ง time event client จะคง context เริ่มต้นไว้และปฏิเสธ
reconciliation หลังการผูกแบบสิ้นสุดถูกนำออก

Request ที่เปลี่ยนสถานะทำตาม[semantics ของการ reject order](#order-rejection-semantics) คือ adapter ส่งครั้งเดียวและ
ไม่ replay หลังพยายามกู้การ login หรือ subscription ล้มเหลวสามครั้ง client จะ log error ทำเครื่องหมายว่าตัวเองหลุดการเชื่อมต่อ
และหยุด WebSocket session นั้น

`ws_timeout_secs` ใช้กับการดำเนินการ WebSocket แต่ละครั้งและการ refresh cache ตอน reconnect ไม่ได้ใช้กับการตรวจ heartbeat
หรือ backoff ของการ reconnect

### การกู้คืน order book {#order-book-recovery}

Order book กู้คืนแยกจากการกู้คืน session การกู้คืนจะเริ่มหลัง:

- `book_snapshot_timeout_secs` (ค่าเริ่มต้น `10` วินาที) โดยไม่มี snapshot แรกหรือ snapshot หลัง reconnect
- `book_stale_threshold_secs` (ค่าเริ่มต้น `120` วินาที) โดยไม่ได้รับ snapshot ที่เวลาเดินหน้า

ตั้งตัวเลือกใดเป็น `0` เพื่อปิด timer นั้น เลือก stale threshold ตามความถี่การอัปเดตที่คาดไว้ timestamp ที่ซ้ำกันไม่ได้
refresh ความสดใหม่ แต่ snapshot ว่างที่เวลาเดินหน้า refresh ได้

ราคาหรือจำนวนที่จำเป็นแต่เป็น null หรือว่างจะทำให้การ decode ล้มเหลวและเริ่มการกู้คืน snapshot เหล่านี้ไม่ publish delta
หรือ depth ไม่ refresh ความสดใหม่หรือเลื่อน timestamp ที่ยอมรับ และไม่สามารถทำให้การกู้คืนเสร็จสิ้นได้

การกู้คืนจะ unsubscribe แล้ว resubscribe channel ที่ได้รับผลกระทบ และยอมรับ snapshot หลังได้รับ acknowledgement ของ
subscription เท่านั้น โดยทำตาม
[shared retry budget and ceiling](../developer_guide/adapters.md#recovery-state-and-retry-budgets)
จนกว่าจะยอมรับ snapshot ได้หรือ channel ถูกนำออก

- การ reconnect จะคงรอบการกู้คืนและ budget ของรอบนั้นไว้
- การ unsubscribe ของผู้ใช้คนสุดท้ายหรือการ shutdown client จะยกเลิกการกู้คืนและการเขียนที่รออยู่
- เมื่อปิดกำหนดเวลาของ snapshot budget ของการ retry ครั้งแรกยังจำกัดไว้ที่ 180 วินาที และแต่ละครั้งถัดไปที่ 60 วินาที
  `ws_timeout_secs` ยังเป็นขอบเขตของแต่ละ RPC

:::warning
Notification ของ Derive ไม่มี generation ของ subscription adapter จะปฏิเสธ frame ที่ล้าสมัยซึ่งเข้าคิวไว้ในเครื่องและ
timestamp ที่ไม่เพิ่มขึ้น แต่ไม่สามารถระบุ notification ที่มาช้าของ subscription ก่อนหน้าได้ หาก timestamp ของมันเกิน
timestamp ล่าสุดที่ยอมรับ
:::

### การตรวจสอบการกู้คืนแบบ live {#live-recovery-validation}

`derive-book-stress` ตรวจข้อมูล public ของ Derive testnet โดยไม่ส่ง order โดยตรวจ book แบบ depth-10 ของ `ETH-PERP`
และ `ETH-USDC` เทียบกับข้อตกลงของ book stream และการสร้างใหม่อย่างอิสระของแต่ละ snapshot ที่ venue ส่งต่อมา รวมถึง
snapshot ของ spot ที่ว่าง

ต้องเข้าถึง WebSocket stream ของ testnet และ public instrument REST API ได้ จาก root ของ repository ให้รัน:

```bash
CARGO_BUILD_JOBS=4 bash scripts/strip-adapter-env.bash \
  cargo test -p nautilus-derive --features examples --test derive-book-stress -- \
  --scenario initial --timeout 2 --rounds 60
```

| `--scenario` | ตัวเลือก        | สิ่งที่ตรวจ                                                                                                 |
| ------------ | ------------- | ------------------------------------------------------------------------------------------------------ |
| `churn`      | ค่าเริ่มต้น      | Book ที่ stale, frame ที่มาช้า, การ reconnect ระหว่างกู้คืน, การ unsubscribe/resubscribe และ timestamp ที่ผิด  |
| `initial`    | `--timeout 2` | ทิ้ง snapshot แรกทั้งสองตัว แล้วรัน `churn`                                                                |
| `disabled`   | `--timeout 0` | Book ที่เงียบข้ามการบังคับ reconnect โดยปิด timer ทั้งสอง                                                    |
| `edges`      | `--timeout 1` | Budget แปดครั้งที่หมด, การยอมรับ snapshot ที่มาช้า และการ retry หลังถูกปฏิเสธถาวร                                |

- `--timeout` ตั้ง snapshot timeout เป็นวินาที `0` ปิดกำหนดเวลา stale threshold คือสี่วินาที ยกเว้นใน `disabled` ซึ่งปิด
  timer ทั้งสอง
- `--rounds` ตั้งจำนวนรอบของ churn หรือ disabled (ค่าเริ่มต้น `20`) ส่วน `edges` รันครั้งเดียว

ทุก scenario ตรวจว่า shutdown เสร็จภายในเวลาจำกัด ทุกตัวยกเว้น `disabled` จะเริ่ม shutdown ระหว่างการกู้คืน ดู
[Stress harnesses](../developer_guide/spec_data_testing.md#stress-harnesses) สำหรับ flag และ output ที่ใช้ร่วมกัน

## Deterministic simulation testing {#deterministic-simulation-testing}

Adapter ฝั่ง Rust รองรับ `simulation` ด้วย `cfg(madsim)` ผ่านจุดต่อ (seam) ที่ใช้ร่วมกันของ clock, task, retry และ
transport ให้ตั้ง endpoint แบบ `http://` และ `ws://` ที่ควบคุมได้ และเลือก `TransportBackend::Tungstenite` credential
สังเคราะห์แบบคงที่รองรับการทดสอบแบบ private กับ local peer

ส่วนของ runtime ที่ผ่านการตรวจแล้วครอบคลุมเส้นทางเหล่านี้:

- การค้นหา instrument เดี่ยวแบบ public (`public/get_instrument`) จนถึงการสร้าง domain object
- Public ticker subscription จนถึงการสร้าง quote ที่สมบูรณ์
- Request ของ book subscription และ trade subscription
- Request ของการ login และการยกเลิกแบบ private
- การ replay subscription ตอน reconnect, การจัดลำดับ command/frame และการ shutdown

Integration test ใน `crates/adapters/derive/tests/integration/dst.rs` ตรึง byte บน wire, signature ของการ login และ
field ของ domain รวมถึง ticker interval โดยปล่อย record `DST_TRACE` และ `DST_DOMAIN` เพื่อให้ปลายทางเปรียบเทียบข้าม
process ใหม่ที่มี seed, config และ peer script เท่ากัน `cargo-test-sim` รัน assertion ภายใน repository ส่วนการเปรียบเทียบ
ข้าม process ใหม่อยู่ใน DST harness ปลายทาง ตามที่ระบุไว้ใน[ข้อตกลง DST ของ adapter](../concepts/dst.md#adapter-dst-contract)

Static gate ครอบคลุมเส้นทาง production ต่อไปนี้ภายใต้ `crates/adapters/derive/src/`:

- `book/recovery.rs` และ `book/sync.rs`
- `common/parse.rs`, `common/rate_limit.rs`, `config.rs`, `data.rs`, `execution.rs` และ `providers.rs`
- `http/client.rs`, `http/models.rs`, `http/query.rs` และ `http/parse.rs`
- `signing/encoding.rs`, `signing/nonce.rs`, `signing/auth.rs`, `signing/eip712.rs` และ
  `signing/modules/trade.rs`
- `websocket/client.rs`, `websocket/dispatch.rs`, `websocket/handler.rs`, `websocket/messages.rs`
  และ `websocket/parse.rs`

ค่าคงที่, enum, error, ค่าเริ่มต้นของ endpoint, การประกาศ module, factory ที่ทำแค่การสร้าง, ตัวจำแนก retry, การ resolve
credential และ signing context และ container ของ WebSocket context อยู่นอกรายการ static path ไฟล์เหล่านี้ไม่ได้สร้างขอบเขต
async, clock หรือ randomness ที่เป็นอิสระ การ resolve credential ใช้ config ที่ให้มาหรือ input จาก environment ที่ประกาศไว้
FFI, TLS, สถานะของ venue แบบ live, วงจรชีวิตเต็มของ data/execution client, การส่งและแทนที่ signed order, การ trigger
และส่วนอื่นของ runtime ที่ยังไม่ได้ตรวจ อยู่นอกการอ้างสิทธิ์ของ simulation นี้ ความครอบคลุมแบบ static ไม่ได้พิสูจน์การ
ทำงานขณะ runtime ของส่วนเหล่านั้น

## Subscription parameter {#subscription-parameters}

`subscribe_book_deltas` และ `subscribe_book_depth` รับ key ของ `subscribe_params` ต่อไปนี้:

| Key     | Type   | ค่าเริ่มต้น | ค่าที่อนุญาต                      |
| ------- | ------ | -------- | ------------------------------ |
| `group` | string | `"1"`    | `"1"`, `"10"`, `"100"`         |
| `depth` | string | `"10"`   | `"1"`, `"10"`, `"20"`, `"100"` |

`subscribe_quotes` รับ:

| Key        | Type   | ค่าเริ่มต้น  | ค่าที่อนุญาต         |
| ---------- | ------ | --------- | ----------------- |
| `interval` | string | `"1000"`  | `"100"`, `"1000"` |

ค่าที่ไม่รู้จักจะถูกปฏิเสธตอน subscribe

### Ticker subscription ที่ใช้ร่วมกัน {#shared-ticker-subscription}

Quote, mark price, index price, funding rate และ option greeks ทั้งหมด derive จาก WebSocket subscription
`ticker_slim.{instrument}.{interval}` ตัวเดียวกัน adapter นับการอ้างอิง (reference-count) การเรียก subscribe ของ WS
ที่อยู่เบื้องหลัง:

- Feed แรกที่ subscribe สำหรับ instrument หนึ่งจะเปิด channel
- การ unsubscribe ครั้งสุดท้ายจะปิด channel

:::note
**`interval` ของ subscription แรกเป็นตัวกำหนด** feed ที่ subscribe ภายหลังด้วย interval ต่างกันจะใช้ channel ที่มีอยู่ร่วมกัน
:::

#### Field ของ ticker {#ticker-fields}

Mark price, index price, funding rate และ option greeks อ่าน field จาก payload ของ ticker ทั้งรูปแบบ ticker เต็มและรูปแบบ
`ticker_slim` แบบกะทัดรัดมี field เหล่านี้ feed ที่ derive จึงทำงานได้กับทั้งสองแบบ:

| Field                       | การมีอยู่                         | พฤติกรรมของ feed                                                        |
| --------------------------- | ------------------------------- | --------------------------------------------------------------------- |
| `mark_price`, `index_price` | จำเป็น                           | Field ที่หายไปจะทำให้ deserialize ล้มเหลวและถูก log ไม่ได้ถูกทิ้งแบบเงียบ ๆ   |
| `funding_rate`              | ไม่บังคับ มีเฉพาะ perpetual         | ให้ event ของ funding rate                                             |
| `option_pricing`            | ไม่บังคับ มีเฉพาะ option            | ให้ option greeks                                                     |
| Bid/ask                     | มีในทั้งสองรูปแบบ                    | Quote feed ทำงานได้กับทั้งสองรูปแบบ                                      |

#### ข้อจำกัดตาม instrument class {#instrument-class-restrictions}

Funding rate มีความหมายเฉพาะกับ perpetual และ option greeks เฉพาะกับ option adapter ยอมรับ subscription ของ feed
ที่ไม่ตรงกับ class ของ instrument (เช่น funding rate สำหรับ option) WebSocket subscription จะเปิดได้ แต่ parser จะไม่คืน
event ใดสำหรับ feed นั้น payload ของ venue ไม่มี field funding rate สำหรับสิ่งที่ไม่ใช่ perp และไม่มี `option_pricing` สำหรับ
สิ่งที่ไม่ใช่ option ตรวจ class ของ instrument ก่อน subscribe feed เฉพาะของ derivative

## การตั้งค่า {#configuration}

### ตัวเลือก config ของ data client {#data-client-configuration-options}

Struct: `DeriveDataClientConfig`

| ตัวเลือก                             | ค่าเริ่มต้น  | คำอธิบาย                                                                                         |
| ---------------------------------- | --------- | --------------------------------------------------------------------------------------------- |
| `base_url_rest`                    | `None`    | Override REST base URL                                                                        |
| `base_url_ws`                      | `None`    | Override WebSocket base URL                                                                   |
| `proxy_url`                        | `None`    | Proxy URL (ไม่บังคับ) สำหรับ transport แบบ HTTP และ WebSocket                                      |
| `environment`                      | `Mainnet` | ตัวเลือกเครือข่าย (`Mainnet` หรือ `Testnet`)                                                       |
| `http_timeout_secs`                | `10`      | Timeout ของ REST request เป็นวินาที                                                             |
| `ws_timeout_secs`                  | `None`    | Timeout ของ WebSocket ต่อการดำเนินการ (login, subscribe, read, write) เป็นวินาที หากไม่ตั้งจะใช้ 10s   |
| `book_snapshot_timeout_secs`       | `10`      | เวลารอ snapshot เป็นวินาที `0` ปิดกำหนดเวลา                                                       |
| `book_stale_threshold_secs`        | `120`     | จำนวนวินาทีที่ไม่มี snapshot ที่เวลาเดินหน้าก่อนเริ่มกู้คืน `0` ปิด timer นี้                                  |
| `update_instruments_interval_mins` | `60`      | ช่วงเวลา refresh instrument เป็นนาที `0` ปิดการ refresh เป็นระยะ                                    |
| `currencies`                       | `[]`      | สกุลเงินที่จะ bulk-load ตอนเชื่อมต่อ ว่างหมายถึงโหลดแบบ lazy ตามต้องการ                                  |
| `include_expired`                  | `false`   | รวมรายการ option ที่ยัง live และหมดอายุแล้วจาก `public/get_all_instruments`                          |
| `auto_load_missing_instruments`    | `true`    | โหลด instrument ที่ไม่รู้จักแบบ lazy ก่อนส่ง subscribe request                                       |
| `transport_backend`                | `Sockudo` | Backend ของ transport แบบ WebSocket (`Sockudo` หรือ `Tungstenite`)                              |

Data client จะ refresh นิยาม instrument สำหรับสกุลเงินที่ตั้งค่าไว้ทุกรอบ refresh และ publish ออกไป รวมถึง listing ใหม่
การ refresh ที่ล้มเหลวยังคงนิยามใน cache ไว้ใช้ และจะลองใหม่ในรอบถัดไป `currencies` ที่ว่างจะปิดการ refresh เป็นระยะ

:::important
**`auto_load_missing_instruments` ครอบคลุมเฉพาะคำสั่ง subscribe** ก่อน request quote, trade, bar หรือ funding rate
ให้ bulk-load สกุลเงินของ instrument หรือ subscribe ก่อน
:::

- `request_quotes`, `request_trades`, `request_bars` และ `request_funding_rates` จะล้มเหลวเมื่อ instrument ยังไม่อยู่ใน cache
- `request_instrument` เป็นข้อยกเว้น เพราะดึง `public/get_instrument` เสมอ
- Option-chain subscription ต้องมี option จาก series นั้นใน cache เพื่อดึงราคาอ้างอิงเริ่มต้น

### ตัวเลือก config ของ execution client {#execution-client-configuration-options}

Struct: `DeriveExecutionClientConfig`

#### Account และ credential {#account-and-credentials}

| ตัวเลือก           | ค่าเริ่มต้น     | คำอธิบาย                                                           |
| ---------------- | ------------ | ---------------------------------------------------------------- |
| `account_id`     | `DERIVE-001` | Identifier ของ account ใน Nautilus                                 |
| `wallet_address` | `None`       | Address ของ EOA หรือ multisig ของเจ้าของ หากไม่ตั้งจะใช้ env var ด้านล่าง    |
| `session_key`    | `None`       | Private key ของ session key แบบ secp256k1 หากไม่ตั้งจะใช้ env var ด้านล่าง |
| `subaccount_id`  | `None`       | Subaccount id ของ Derive หากไม่ตั้งจะใช้ env var ด้านล่าง                 |

#### การเชื่อมต่อและการ retry {#connectivity-and-retries}

| ตัวเลือก                   | ค่าเริ่มต้น  | คำอธิบาย                                                                                       |
| ------------------------ | --------- | ------------------------------------------------------------------------------------------- |
| `base_url_rest`          | `None`    | Override REST base URL                                                                      |
| `base_url_ws`            | `None`    | Override WebSocket base URL                                                                 |
| `proxy_url`              | `None`    | Proxy URL (ไม่บังคับ) สำหรับ transport แบบ HTTP และ WebSocket                                    |
| `environment`            | `Mainnet` | ตัวเลือกเครือข่าย (`Mainnet` หรือ `Testnet`)                                                     |
| `http_timeout_secs`      | `10`      | Timeout ของ REST request เป็นวินาที                                                           |
| `ws_timeout_secs`        | `None`    | Timeout ของ WebSocket ต่อการดำเนินการ (login, subscribe, read, write) เป็นวินาที หากไม่ตั้งจะใช้ 10s |
| `transport_backend`      | `Sockudo` | Backend ของ transport แบบ WebSocket (`Sockudo` หรือ `Tungstenite`)                            |
| `max_retries`            | `3`       | จำนวนครั้งที่ retry สำหรับการอ่าน REST ที่ idempotent การเขียน order ส่งครั้งเดียวและไม่ถูก replay        |
| `retry_delay_initial_ms` | `100`     | Delay เริ่มต้นของการ retry เป็นมิลลิวินาที                                                        |
| `retry_delay_max_ms`     | `5,000`   | Delay สูงสุดของ backoff เป็นมิลลิวินาที                                                          |

Adapter เปิด `nautilus-network/transport-sockudo` เสมอ ตั้ง `transport_backend` เป็น `Tungstenite` เพื่อเลือก
backend ทางเลือก

สำหรับการอ่านผ่าน HTTP `Retry-After` จะกำหนด delay ขั้นต่ำซึ่งอาจเกิน `retry_delay_max_ms` ได้ หาก delay นั้นไม่พอดีกับ
budget การ retry 180 วินาทีที่เหลือ client จะคืน error เดิมโดยไม่ retry

#### การ sign order และราคา {#order-signing-and-pricing}

:::important
`max_fee_per_contract` เป็นค่าที่จำเป็นและต้องมากกว่าศูนย์ การสร้าง execution client จะล้มเหลวก่อนสร้าง venue client
เมื่อ field นี้ไม่มีหรือไม่เป็นบวก
:::

| ตัวเลือก                      | ค่าเริ่มต้น | คำอธิบาย                                                                              |
| --------------------------- | -------- | ----------------------------------------------------------------------------------- |
| `max_fee_per_contract`      | จำเป็น    | เพดาน fee เป็น USDC ต่อสัญญาที่เป็นค่าบวก ซึ่ง sign เข้าไปในทุก order                          |
| `domain_separator`          | `None`   | Override EIP-712 domain separator (ไม่บังคับ)                                         |
| `action_typehash`           | `None`   | Override EIP-712 action typehash (ไม่บังคับ)                                          |
| `trade_module_address`      | `None`   | Override address ของ Trade module contract (ไม่บังคับ)                                |
| `signature_expiry_secs`     | `600`    | TTL ของ order/replace ดู[วันหมดอายุของ signature](#signature-expiry) trigger ใช้ 31 วัน  |
| `market_order_slippage_bps` | `50`     | ขอบเขต slippage สำหรับ limit price ของ market order                                   |
| `cancel_on_disconnect`      | `False`  | คุ้มครอง limit order ที่ resting อยู่ ดู [cancel on disconnect](#cancel-on-disconnect)    |

#### ขีดจำกัด matching request {#matching-request-limits}

ขีดจำกัดทั้งสองมีค่าเริ่มต้นเป็น **1 request ต่อวินาที** เมื่อไม่ได้ตั้งค่า ตั้งค่าที่ตกลงกันได้แยกกัน: โควตาของ wallet ใช้
หน้าต่างคงที่ และ credit ของ instrument เติมต่อเนื่อง client ของ wallet เดียวกันบน API host เดียวกันใช้ค่าตั้งที่ระบุชัดเจน
ครั้งแรกร่วมกัน ตรวจ budget ของ request แบบ live ก่อนเพิ่มโควตาของ wallet budget ราย instrument ไม่สามารถค้นหาได้ด้วย
`public/getRateLimits`

| ตัวเลือก                                            | ค่าเริ่มต้น | คำอธิบาย                                              |
| ------------------------------------------------- | -------- | --------------------------------------------------- |
| `max_matching_requests_per_second`                | `None`   | Matching request ของ wallet ต่อวินาที                  |
| `max_per_instrument_matching_requests_per_second` | `None`   | อัตราการเติม token ต่อ wallet/instrument ต่อวินาที       |

#### Environment variable ของ credential {#credential-environment-variables}

`wallet_address`, `session_key` และ `subaccount_id` จะใช้ environment variable เมื่อไม่ได้ตั้งค่า:

| Field            | Variable ของ mainnet          | Variable ของ testnet                  |
| ---------------- | ---------------------------- | ------------------------------------ |
| `wallet_address` | `DERIVE_WALLET_ADDRESS`      | `DERIVE_TESTNET_WALLET_ADDRESS`      |
| `session_key`    | `DERIVE_SESSION_PRIVATE_KEY` | `DERIVE_TESTNET_SESSION_PRIVATE_KEY` |
| `subaccount_id`  | `DERIVE_SUBACCOUNT_ID`       | `DERIVE_TESTNET_SUBACCOUNT_ID`       |

Session key คือ private key แบบ secp256k1 ที่ลงทะเบียนบน wallet สำหรับการ sign API field `session_key` จะถูกซ่อนใน
output ของ `Debug`

### Data client ฝั่ง Rust {#rust-data-client}

```rust
use nautilus_derive::{
    common::enums::DeriveEnvironment,
    config::DeriveDataClientConfig,
};

let config = DeriveDataClientConfig {
    environment: DeriveEnvironment::Testnet,
    currencies: vec!["ETH".to_string(), "BTC".to_string()],
    ..Default::default()
};
```

### Execution client ฝั่ง Rust {#rust-execution-client}

```rust
use nautilus_derive::{
    common::enums::DeriveEnvironment,
    config::DeriveExecutionClientConfig,
};
use rust_decimal::Decimal;

let config = DeriveExecutionClientConfig {
    wallet_address: Some("0x...".to_string()),
    session_key: Some("0x...".into()),
    subaccount_id: Some(1),
    environment: DeriveEnvironment::Testnet,
    max_fee_per_contract: Some(Decimal::from(1000)),
    ..Default::default()
};
```

## ข้อจำกัดที่ทราบ {#known-limitations}

- `request_instruments` ต้องมีสกุลเงินที่ตั้งค่าไว้อย่างน้อยหนึ่งสกุลใน `DeriveDataClientConfig::currencies` adapter
  กรอง `public/get_all_instruments` ตามแต่ละสกุลเงินที่ตั้งค่าไว้ และไม่ได้ไล่รายการสกุลเงินทั้งหมด
- Venue ไม่ได้ push instrument status, instrument close หรือ candle subscription นิยาม instrument มี `is_active` และ
  timestamp ของการ activate/deactivate ตามกำหนดการ และ bar มีให้ทาง REST เท่านั้น
- Venue ไม่มี endpoint REST สำหรับ snapshot ของ book และ endpoint สำหรับ book delta ย้อนหลัง / quote ย้อนหลัง ดูตาราง
  ความสามารถข้างต้น
- เอกสาร REST อย่างเป็นทางการของ Derive ระบุว่า `public/get_ticker` เลิกใช้แล้ว (deprecated) โดยให้ใช้
  `public/get_tickers` แทน ตั้งแต่วันที่ 1 ธันวาคม 2025 adapter ใช้ `public/get_tickers` สำหรับ snapshot ของ quote และ
  การ bootstrap ราคาอ้างอิงของ option chain
