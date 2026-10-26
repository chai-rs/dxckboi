# Accounting

ระบบย่อย accounting ทำหน้าที่ติดตาม balance, margin และ PnL ของแต่ละ account คู่มือนี้อธิบาย data model,
query API ที่ strategy ใช้ และข้อตกลงที่ผู้เขียน adapter ต้องปฏิบัติตามเพื่อให้ทำงานสอดคล้องกันในทุก venue

เนื้อหานี้ใช้ได้ทั้งกับ backtest และ live trading สำหรับ config เฉพาะของ backtest (starting balance, การเลือก
margin model ต่อ venue) ดู [Backtesting](backtesting/)

## ประเภทของ account {#account-types}

เมื่อกำหนด config ของ venue สำหรับ live trading หรือ backtest ให้ใช้ `account_type` เพื่อเลือก Cash, Margin หรือ Betting

Wallet ใช้จำลองสถานะของ wallet บน on-chain โดย Blockchain adapter เป็นตัวเลือกใช้ execution client ของ adapter นี้
รองรับ market swap บน Uniswap V3 ที่ sign แบบ local แต่ยังไม่พร้อมใช้งานจริงใน production

| ประเภท account | กรณีใช้งานทั่วไป                                   | การจอง (reservation)                                                        |
| ------------ | ----------------------------------------------- | ------------------------------------------------------------------------- |
| Cash         | Spot trading (เช่น BTC/USDT, หุ้น)               | Notional ของ buy order ที่ค้างอยู่ และปริมาณ base asset ของ sell order ที่ค้างอยู่ |
| Margin       | Derivatives หรือผลิตภัณฑ์ใดก็ตามที่ใช้ leverage ได้     | Initial margin ของแต่ละ order บวก maintenance margin ของ position ที่เปิดอยู่  |
| Betting      | Sports betting, bookmaking                      | Stake ที่ venue กำหนด ไม่มี leverage                                         |
| Wallet       | Blockchain wallet (DeFi)                        | จำนวนที่จองไว้แบบ local สำหรับ order ที่ค้างอยู่ ไม่มี leverage และไม่มีการกู้ยืม     |

### Cash account {#cash-accounts}

Cash account **settle trade เต็มจำนวน** ไม่มี leverage จึงไม่มีแนวคิดเรื่อง margin ยอด locked balance สะท้อนมูลค่าที่จองไว้
สำหรับ order ที่ค้างอยู่ ได้แก่ มูลค่า notional ของ buy order แต่ละตัว และปริมาณที่ sell order แต่ละตัวจะต้องส่งมอบ

#### Precision และการปัดเศษ {#precision-and-rounding}

กระแสเงินสดและการจองใช้การคำนวณแบบ checked domain หรือ `Decimal` แล้วแปลงเป็น precision ดั้งเดิมของ `Money`
ด้วยการปัดแบบ midpoint-to-even การอัปเดต balance จะบวกและลบจำนวน `Money` โดยไม่ปัดเป็น display precision ของสกุลเงิน
การแปลงสกุลเงินก็คง precision ดั้งเดิมของ `Money` ไว้เช่นกัน

ดูเหตุผลเชิงบัญชีและตัวอย่างการคำนวณได้ที่ [Precision และ settlement](#precision-and-settlement)

#### การจองสำหรับ sell {#sell-reservations}

การ sell คือการส่งมอบ base asset ของ instrument ดังนั้นการจองจึงควรเป็นสกุลของ asset นั้น instrument ที่
`base_currency()` เป็น `None` เช่น binary option และหุ้น ไม่ได้จำลอง asset ดังกล่าว sell order ที่ค้างอยู่บน
instrument เหล่านี้จึง **ไม่ lock อะไรเลย** และรายได้จากการขายจะไม่ถูกนำไปเทียบกับ free quote balance

การจองปริมาณ order เป็น quote currency จะเท่ากับการตีความจำนวนหุ้นหรือสัญญาว่าเป็นเงิน และ lock หลักประกันที่การขาย
ไม่ได้ใช้จริง ส่วนการนำรายได้จากการขายไปเทียบกับหลักประกันที่ว่างอยู่ จะทำให้การขายถูกปฏิเสธเพราะขาด balance ที่การขาย
นั้นเองจะเป็นตัวเติมให้

ใน live trading ความพร้อมของ inventory สำหรับ instrument เหล่านี้จะไม่ถูกติดตามหรือตรวจสอบแบบ local รวมถึงในกรณีที่มี
sell หลายตัวพร้อมกัน venue เป็นผู้บังคับใช้ และการปฏิเสธใด ๆ จะมาตามวงจรชีวิตปกติของ order ขอบเขตนี้ตั้งใจออกแบบไว้
เพราะการทำซ้ำการตรวจสอบ inventory ของ venue แบบ local ต้องใช้ balance ราย asset ซึ่ง adapter ไม่ได้รับมา ส่วนการจอง
สำหรับ buy และการตรวจสอบ sell ของ instrument ที่มีการจำลอง base asset ยังคงเหมือนเดิม

#### การตรวจสอบ short-selling ในการจำลอง {#simulated-short-selling-checks}

Backtest และ sandbox จะตรวจสอบ sell ของหุ้นหรือ binary option แต่ละตัวบน cash account ตอน submit ปริมาณคงเหลือ
ของ order ต้องไม่เกิน inventory ฝั่ง long ใน position ที่เลือก ซึ่งรวมถึง fill ที่ execute แล้วแต่ยังรอการอัปเดต position
sell ที่ลดหรือปิด position นั้นยังคงใช้ได้

การตรวจสอบนี้ใช้ราคาของ order หรือ bid ปัจจุบันในการคำนวณจำนวนหน่วยสัญญาสำหรับ sell ที่ระบุขนาดเป็น quote
order แบบ trigger-style market และ trailing อาจแปลงที่ราคาต่างออกไปเมื่อ fill ทำให้ปริมาณสัญญาเปลี่ยนไป

Order ฝั่งเข้า (entry) จะเพิ่ม inventory ให้กับ order ฝั่งออก (exit) ที่ผูกอยู่ก็ต่อเมื่อมี fill เกิดขึ้นเท่านั้น โดยค่าเริ่มต้น
`oto_full_trigger` เป็น `false` ดังนั้น exit แบบเต็มขนาดอาจถูกปฏิเสธหลังจาก entry ถูก fill เพียงบางส่วน ให้ตั้ง
`oto_full_trigger` เป็น `true` เพื่อปล่อย exit หลังจาก entry ถูก fill เต็มจำนวนแล้วเท่านั้น ซึ่งใช้กับกรณีที่การอัปเดต
position ถูกเลื่อนออกไปด้วย

การตรวจสอบตอน submit นี้ไม่ได้จอง inventory ไว้ให้ sell ตัวอื่นที่กำลังทำงานอยู่ และไม่ตรวจซ้ำเมื่อปริมาณเปลี่ยนในภายหลัง

### Margin account {#margin-accounts}

Margin account รองรับ instrument ที่ต้องใช้หลักประกัน เช่น futures หรือ crypto perp ที่มี leverage โดยติดตาม balance
ของ account, จอง margin สำหรับ order และ position ที่เปิดอยู่ และใช้ leverage ที่กำหนดได้ต่อ instrument margin balance
มีสองขอบเขต (scope) ดู [Margin scope](#margin-scopes) ด้านล่าง

**คำศัพท์**:

- **Leverage**: ขยาย exposure เทียบกับ equity ของ account leverage ที่สูงขึ้นจะเพิ่มทั้งผลตอบแทนที่เป็นไปได้และความเสี่ยง
- **Initial margin**: หลักประกันที่จองไว้เมื่อ submit order
- **Maintenance margin**: หลักประกันขั้นต่ำที่ต้องมีเพื่อคง position ที่เปิดอยู่
- **Locked balance**: เงินที่จองไว้เป็นหลักประกัน ไม่สามารถใช้กับ order ใหม่ได้

:::note
Reduce-only order ลด exposure ได้อย่างเดียว จึงไม่นับรวมใน `balance_locked` ของ cash account และไม่เพิ่ม initial margin
ของ margin account แต่ order ของ wallet ยังคงจอง asset ฝั่ง input ไว้ เพราะ transaction บน on-chain ใช้ asset นั้นจริง
แม้ order จะเป็นการลด position ก็ตาม
:::

### Betting account {#betting-accounts}

Betting account ออกแบบมาเฉพาะสำหรับ venue ที่วางเงินเดิมพัน (stake) เพื่อได้หรือเสียตามจำนวน payout ที่กำหนดไว้
(prediction market, sports book) โดยจะ lock เฉพาะ stake ที่ venue กำหนดเท่านั้น leverage และ margin ไม่มีผล

### Wallet account {#wallet-accounts}

Wallet account แทน blockchain wallet: การถือครองหลายสกุลเงินแบบไม่มี leverage ทั้ง native และ ERC-20 token
โดยไม่มี margin และไม่มีการกู้ยืม

สำหรับสถานะที่รายงานเข้ามา:

- `total` คือ balance ที่สังเกตได้บน on-chain
- `locked` ติดตามการจองแบบ local สำหรับ order ที่ค้างอยู่
- `free = total - locked`

Account state event **ให้เฉพาะค่า total**: account จะไม่สนใจค่า `locked` และ `free` ที่เข้ามา, คงการจองแบบ local ไว้
และคำนวณ `free` ใหม่ ระหว่าง startup ของ live trading จะสร้างการจองชั่วคราวขึ้นใหม่จาก order ที่ submit แล้วและที่เปิดอยู่

ขณะที่การแก้ไข order (amendment) ยังค้างอยู่ account จะจอง balance ที่สังเกตได้ทั้งหมดของสกุลเงินฝั่ง debit เพราะ pending
event ไม่ได้มีเงื่อนไขที่ร้องขอมาด้วย หากจำนวนที่จองเกินค่า total ล่าสุดที่สังเกตได้ `locked` จะถูกจำกัดไว้ที่ `total` และ
`free` จะเป็นศูนย์จนกว่า balance หรือการจองจะเปลี่ยน

Balance ที่มี total ติดลบจะถูกปฏิเสธแทนที่จะถูกนำไปใช้ ERC-20 allowance เป็นการอนุญาตให้ spender ใช้เงิน
และจะไม่ถูกแสดงเป็น balance หรือเงินที่ lock ไว้

## Balance model {#balance-model}

`AccountBalance` เก็บค่าสามค่าในสกุลเงินเดียวกัน:

- `total`: ตัวเลข balance รวมที่ venue รายงาน (wallet, net liquidation หรือ margin balance แล้วแต่ venue)
- `locked`: จำนวนที่จองไว้สำหรับ order และ position ที่เปิดอยู่
- `free`: จำนวนที่ใช้ได้สำหรับ order ใหม่ (`total - locked`)

**Balance invariant** `total == locked + free` ต้องเป็นจริงเสมออย่างแม่นยำในรูปแบบ Money ที่จัดเก็บ

### การสร้าง balance {#constructing-balances}

โค้ด adapter ที่เขียนด้วย Rust มี derived constructor ที่บังคับ invariant ไว้จากศูนย์กลาง ควรใช้ constructor เหล่านี้
แทน `AccountBalance::new` เมื่อ venue รายงานค่ามาเพียงสองในสามค่า:

| Rust constructor                              | ใช้เมื่อ                                                                           |
| --------------------------------------------- | -------------------------------------------------------------------------------- |
| `AccountBalance::from_total_and_locked`       | Venue รายงาน total และ locked โดย `free` คำนวณจากสองค่านี้                          |
| `AccountBalance::from_total_and_free`         | Venue รายงาน total และ free โดย `locked` คำนวณจากสองค่านี้                          |
| `AccountBalance::from_money_total_and_locked` | total และ locked ที่คำนวณไว้เป็นจำนวน Money อยู่แล้ว คง precision ดั้งเดิมไว้           |
| `AccountBalance::new`                         | ทราบค่าทั้งสามและสอดคล้องกันอยู่แล้ว (test, pass-through)                              |

Derived constructor แต่ละตัวจะ clamp ค่า locked หรือ free ที่ส่งเข้ามาให้อยู่ในช่วง `[0, total]` เมื่อ `total >= 0`
เพื่อไม่ให้ค่าที่เกินมาชั่วคราวจากการปัดเศษของ venue ทำให้ account อยู่ในสถานะที่เสียหาย

## ข้อกำหนดด้านสกุลเงินและการประเมินมูลค่า {#currency-and-valuation-contracts}

ค่าทางบัญชีจะคงสกุลเงินต้นทางไว้จนกว่าการแปลงสกุลเงินแบบ explicit จะสำเร็จ เพื่อป้องกันไม่ให้ตัวเลขที่ถูกต้องถูกติด
ป้ายสกุลเงินผิด หรือค่าที่ไม่มีอยู่ถูกตีความว่าเป็นศูนย์

| ค่า                          | ข้อกำหนดด้านสกุลเงิน                                                   |
| ---------------------------- | -------------------------------------------------------------- |
| Cost currency ของ instrument | Base สำหรับ inverse, settlement สำหรับ quanto และ quote สำหรับกรณีอื่น |
| PnL ของ position              | Cost currency ของ instrument ที่บันทึกไว้ตอนเปิด position            |
| Lock และ margin ที่คำนวณได้     | สกุลเงินของแต่ละจำนวนที่คำนวณได้ โดยแปลงแยกกันอย่างอิสระ                |
| ค่ารวมระดับ portfolio          | แยก bucket ตามสกุลเงินดั้งเดิม หรือเป็น base ของ account เมื่อแปลงสำเร็จ |

การรวมค่าจะรวมเฉพาะค่า `Money` ที่เข้ากันได้ account ที่ไม่มี base currency จะแยก bucket ตามสกุลเงินดั้งเดิม
query realized PnL ของ instrument เดียวจะคืนค่า unavailable แทนที่จะรวมสกุลเงินที่ปนกัน

เส้นทางด้าน accounting และการประเมินมูลค่ายังเป็นไปตามกฎเหล่านี้:

- ผลลัพธ์ notional, PnL, fee, locked balance และ margin ที่ไม่ถูกต้องหรือแสดงค่าไม่ได้ จะให้ผลเป็น error, ค่า unavailable
  หรือสถานะ unpriced โดยไม่แทนด้วยศูนย์ การคำนวณ realized PnL ใหม่ที่ล้มเหลวจะล้างผลลัพธ์ที่ cache ไว้ก่อนหน้าด้วย
- `equity()` นับ base asset แบบ non-inverse ที่ได้รับเข้ามาเพียงครั้งเดียวสำหรับ cash account หลายสกุลเงินที่ไม่มี base currency
  ส่วน `mark_values()` ยังคงเป็น query มูลค่า position แบบ gross และรวม asset นั้นไว้
- MTM snapshot แยกแยะระหว่าง input เก่าที่ถูกยกมา กับ position ที่ไม่เคยมีข้อมูลการประเมินมูลค่าครบถ้วนเลย
  metadata ของราคาเก่าครอบคลุมเฉพาะคู่ instrument และฝั่งของ position ที่เปิดอยู่

ดู [Portfolio](portfolio.md#equity-and-mark-to-market) สำหรับสูตร equity, การเลือกราคาและ xrate, metadata ของ snapshot
และขอบเขตของ query กรณีไม่มีราคา

## Precision และ settlement {#precision-and-settlement}

PnL ของ position และ balance ของ account ที่คำนวณได้จะเก็บจำนวนที่ละเอียดกว่า display precision ของสกุลเงินไว้
เพื่อให้กำไรและค่าธรรมเนียมเล็ก ๆ ยังคงใช้ในการคำนวณครั้งต่อไปได้

### Cost basis และเศษคงเหลือ {#cost-basis-and-residuals}

การคำนวณทางการเงินใช้ checked domain type และ `Decimal`:

- Position คำนวณราคาเฉลี่ยจากต้นทุนเปิดและปิดที่เก็บไว้ ราคาเฉลี่ยที่ปัดแล้วจะไม่ถูกนำไปใช้เป็น cost basis
- การปิดบางส่วนจะจัดสรรต้นทุนเปิดตามสัดส่วนและเก็บต้นทุนที่ยังไม่ได้จัดสรรไว้ การปิดครั้งสุดท้ายจะใช้ต้นทุนที่เหลือทั้งหมด
- Position จะยกเศษ PnL แบบ Decimal ที่ต่ำกว่าสเกลดั้งเดิมของ `Money` ไปใช้ในการอัปเดตครั้งต่อไป การจัดสรรค่าธรรมเนียม
  ตอน reversal จะคงยอดค่าธรรมเนียมรวมเดิมไว้

`Money` ที่คำนวณได้จะปัดแบบ midpoint to even ที่ทศนิยมเก้าตำแหน่งในโหมดมาตรฐาน และ 16 ตำแหน่งในโหมด high-precision
หรือที่ precision ของสกุลเงินที่ละเอียดกว่าที่รองรับ การปัดเพื่อแสดงผลไม่เปลี่ยนจำนวนที่จัดเก็บ

Decimal มี coefficient ขนาด 96 bit และสเกลสูงสุด 28 input ต้องพอดีอย่างแม่นยำ การคำนวณที่ได้จะปัดภายในขอบเขตนั้น และ
ปฏิเสธกรณี overflow หรือค่าที่ไม่ใช่ศูนย์หายไป ผลรวม PnL อาจดูดซับเศษที่เล็กกว่าหนึ่งหน่วย `Money` ของ settlement ดั้งเดิม
หาก Decimal เก็บไว้ไม่ได้ แต่ operand ที่หายไปซึ่งมีขนาดอย่างน้อยหนึ่งหน่วยดั้งเดิมยังคงทำให้ล้มเหลว ดู
[ข้อกำหนดการคำนวณฉบับเต็ม](positions.md#numerical-precision)

### ตัวอย่างการคำนวณ {#worked-examples}

ทั้งสองตัวอย่างใช้ instrument แบบ linear ที่มี multiplier เท่ากับหนึ่ง, ราคาเป็น USD และไม่มีค่าธรรมเนียม

#### กำไรที่น้อยกว่าหนึ่งเซนต์ {#profit-below-a-cent}

ซื้อ `0.01` หน่วยที่ `0.1234 USD` แล้วขายที่ `0.1235 USD`:

- ต้นทุนการซื้อ: `0.01 * 0.1234 = 0.001234 USD`
- รายได้จากการขาย: `0.01 * 0.1235 = 0.001235 USD`
- Realized PnL และ cash balance ที่คำนวณได้เพิ่มขึ้น: `0.000001 USD`

การแสดงผลทศนิยมสองตำแหน่งจะเห็นเป็น `0.00 USD` แต่กำไรที่จัดเก็บไว้ยังคงใช้ในการคำนวณครั้งต่อไปได้
หากปัดกระแสเงินสดแต่ละรายการเป็นเซนต์ กำไรนี้จะหายไป

#### การปิดบางส่วนเมื่อราคาเฉลี่ยเป็นทศนิยมซ้ำ {#partial-closes-with-a-repeating-average}

ซื้อสามหน่วยที่ `0.10 USD` และสี่หน่วยที่ `0.20 USD` ต้นทุนเปิดคือ `1.10 USD` ราคาเฉลี่ย `1.10 / 7` เป็นทศนิยมซ้ำ
ปิดที่ `0.30 USD` ด้วยสอง fill ที่ precision ดั้งเดิมแบบมาตรฐาน:

| การกระทำ               | ปริมาณที่ถือ | Realized PnL สะสม (USD) |
| -------------------- | ---------- | ----------------------------- |
| หลังซื้อทั้งสองครั้ง        | 7          | 0.000000000                   |
| ขายหนึ่งหน่วยที่ 0.30 USD | 6          | 0.142857143                   |
| ขายหกหน่วยที่ 0.30 USD   | 0          | 1.000000000                   |

การขายครั้งแรกเก็บต้นทุนที่ยังไม่ได้จัดสรรและส่วนต่างระหว่าง PnL แบบ Decimal กับผลลัพธ์ `Money` ที่ปัดแล้วไว้
การปิดครั้งสุดท้ายใช้เศษทั้งสองส่วนนี้: PnL รวมคือ `2.10 - 1.10 = 1.00 USD` การทิ้ง PnL ส่วนที่เป็นเศษ หรือการนำ
ราคาเฉลี่ยที่ปัดแล้วมาใช้ซ้ำ อาจทำให้เกิดการคลาดเคลื่อนสะสม (drift)

### การ settle เงินสดของ venue {#venue-cash-settlement}

กฎการ settle ของ venue กำหนดว่าจะปัดเงินสดที่บันทึกที่ใดและอย่างไร ซึ่งอาจต่างจาก PnL ภายใน เมื่อ
`calculate_account_state=False` cash และ margin account จะข้ามการคำนวณ balance จาก fill และใช้ balance ที่ venue
รายงานแทน FX rollover คำนวณด้วย Decimal และปัดแต่ละรายการที่บันทึกเป็น precision ของสกุลเงิน

[วิธีมาตรฐานสำหรับ futures ของ CME (2015)](https://www.cmegroup.com/clearing/files/CME-Money-Calculations-Futures-and-Options.pdf)
ปัด `price * contract value factor` เป็น precision ของสกุลเงินที่ทั้งสองราคา นำค่าเหล่านั้นมาลบกัน แล้วคูณด้วยปริมาณ
การแบ่ง trade ที่ราคาเดียวกัน หรือการเพิ่ม mark ระหว่างทางสำหรับการถือครองที่ไม่เปลี่ยนแปลง จะคงยอดรวมไว้เท่าเดิม
ค่ากึ่งกลางจะปัดออกจากศูนย์ ส่วนผลิตภัณฑ์แบบ notional จะปัดหลังจากคูณการเปลี่ยนแปลงราคาด้วยปริมาณและ contract value factor

[Eurex เปลี่ยน variation margin และกระแสเงินสดอื่น ๆ ในปี 2023](https://www.eurex.com/ec-en/find/circulars/clearing-circular-3572446)
จากการปัดรายสัญญาเป็นการปัดราย transaction หรือราย position ตามความต้องการของลูกค้า การทำให้ตรงกับ clearing statement
ต้องใช้กฎ settlement ของผลิตภัณฑ์นั้น PnL ที่ precision ดั้งเดิมเพียงอย่างเดียวไม่สามารถจำลองทุกข้อตกลงได้

## Margin scope {#margin-scopes}

`MarginBalance` มีสี่ field ได้แก่ `initial`, `maintenance`, `currency` และ `InstrumentId` แบบ optional ซึ่งใช้เลือก
หนึ่งในสอง scope

### Scope ราย instrument {#per-instrument-scope}

`MarginBalance.instrument_id` ถูกตั้งเป็น instrument ที่ระบุ ใช้สำหรับ:

- Isolated margin (หลักประกันราย position)
- Margin ใน backtest หรือที่คำนวณขึ้น ซึ่ง `AccountsManager` คำนวณ margin แบบ local จาก order และ position
  ที่เปิดอยู่ราย instrument

### Scope ระดับทั้ง account {#account-wide-scope}

`MarginBalance.instrument_id` เป็น `None` รายการนี้ใช้ `currency` (สกุลเงินของหลักประกัน) เป็น key ใช้สำหรับ venue
แบบ cross-margin ที่รายงานค่ารวมค่าเดียวต่อสกุลเงินหลักประกัน venue อาจส่งรายการระดับทั้ง account มาเพียงรายการเดียว
(cross margin แบบหลักประกันเดียว) หรือหลายรายการ (หนึ่งรายการต่อ collateral coin)

ทั้งสอง scope อยู่ร่วมกันบน `MarginAccount` เดียวกันโดยแยก store ภายใน event `AccountState` สามารถมีรายการใน scope ใด
scope หนึ่งหรือทั้งสองก็ได้ `MarginAccount.apply()` จะส่งแต่ละรายการไปยัง store ที่ถูกต้องตามว่ามีการตั้ง `instrument_id` หรือไม่

:::warning
`MarginAccount.apply()` **แทนที่** margin store ทั้งสองด้วยข้อมูลจาก event ที่เข้ามา โดยไม่ merge กับสถานะก่อนหน้า
event ที่ไม่มีทั้ง balance และ margin จะคง store เดิมไว้

Adapter ที่ส่ง snapshot แบบบางส่วนต้องใส่รายการ margin ที่ยังมีผลอยู่ทุกรายการในทุกการอัปเดต มิฉะนั้นรายการเหล่านั้น
จะหายไปจนกว่าจะมี snapshot แบบเต็มครั้งถัดไป

Balance ที่ event มีมาด้วยจะแทนที่รายการที่จัดเก็บไว้ของสกุลเงินนั้น ส่วนสกุลเงินที่ event ไม่ได้ระบุจะถูกคงไว้
:::

## Query API สำหรับ strategy {#strategy-query-api}

ใช้ query ที่ตรงกับรูปแบบการรายงานของ venue หาก venue รายงาน margin ราย instrument ให้ query ด้วย `InstrumentId`
หากรายงาน margin ระดับทั้ง account ให้ query ด้วย `Currency`

| Scope          | Query                                                                        |
| -------------- | ---------------------------------------------------------------------------- |
| ราย instrument  | `margin`, `initial_margin` และ `maintenance_margin`                          |
| ทั้ง account     | `account_margin`, `account_initial_margin` และ `account_maintenance_margin`  |
| ทั้งสอง scope    | `total_initial_margin` และ `total_maintenance_margin`                        |

Query แบบจุด (point query) จะไม่คืนค่า (none) เมื่อไม่มีรายการนั้น ส่วน query แบบผลรวมจะคืน `Money` เสมอ
(เป็นศูนย์ในสกุลเงินนั้นหากไม่มีรายการที่ตรง)

### Query ราย instrument (`MarginAccount`) {#per-instrument-queries-marginaccount}

- `margin(instrument_id)`: คืน `MarginBalance` หรือไม่มีค่า
- `initial_margin(instrument_id)`: คืน `Money` หรือไม่มีค่า
- `maintenance_margin(instrument_id)`: คืน `Money` หรือไม่มีค่า
- `margins()`: map จาก `InstrumentId` ไป `MarginBalance` (รายการราย instrument ทั้งหมด)
- `initial_margins()`: map จาก `InstrumentId` ไป `Money`
- `maintenance_margins()`: map จาก `InstrumentId` ไป `Money`

Method เหล่านี้เห็นเฉพาะ store ราย instrument บน venue แบบ cross-margin จะคืน map ว่างหรือไม่มีค่า
ให้ใช้ query ระดับทั้ง account ด้านล่างแทน

### Query ระดับทั้ง account (`MarginAccount`) {#account-wide-queries-marginaccount}

- `account_margin(currency)`: คืน `MarginBalance` หรือไม่มีค่า
- `account_initial_margin(currency)`: คืน `Money` หรือไม่มีค่า
- `account_maintenance_margin(currency)`: คืน `Money` หรือไม่มีค่า
- `account_margins()`: map จาก `Currency` ไป `MarginBalance` (รายการระดับทั้ง account ทั้งหมด)
- `account_initial_margins()`: map จาก `Currency` ไป `Money`
- `account_maintenance_margins()`: map จาก `Currency` ไป `Money`

### ผลรวม (`MarginAccount`) {#totals-marginaccount}

Method เหล่านี้รวมค่าจากทั้งรายการราย instrument และรายการระดับทั้ง account สำหรับสกุลเงินที่ระบุ:

- `total_initial_margin(currency)`: คืน `Money`
- `total_maintenance_margin(currency)`: คืน `Money`

มีประโยชน์เมื่อ strategy เทรดบน venue ที่อาจมีทั้งสอง scope (เช่น isolated position ควบคู่กับหลักประกันแบบ cross-margin)

### Query ระดับ portfolio {#portfolio-level-queries}

#### Query ด้าน margin {#margin-queries}

- `portfolio.instrument_initial_margins` (รับ `venue` และ `account_id`): คืน map จาก `InstrumentId` ไป `Money` หรือไม่มีค่า
- `portfolio.instrument_maintenance_margins` (รับ `venue` และ `account_id`): คืน map จาก `InstrumentId` ไป `Money` หรือไม่มีค่า

`account_id` ใช้เลือก account นั้น ส่วน query ที่ระบุเฉพาะ venue จะ resolve ได้ก็ต่อเมื่อมี account ภายใต้ venue นั้นเพียง
account เดียว ดังนั้นให้ส่ง `account_id` เมื่อมีหลาย account ใช้ venue ร่วมกัน

เมื่อ resolve margin account ได้ query เหล่านี้จะคืนมุมมอง money ราย instrument แบบเดียวกับ
`MarginAccount.initial_margins` และ `MarginAccount.maintenance_margins` หาก resolve margin account ไม่ได้จะไม่คืนค่า

สำหรับข้อมูลระดับทั้ง account บน venue แบบ cross-margin ให้ query จาก account โดยตรงผ่าน
`portfolio.account(venue=venue).account_initial_margin(ccy)` account ที่ได้คืนมาเป็น **snapshot ที่แยกออกมา**
และไม่สามารถแก้ไขสถานะของ Portfolio ได้

#### Query ด้าน PnL และการประเมินมูลค่า {#pnl-and-valuation-queries}

Query ด้าน PnL, exposure, mark-to-market และ equity ทั้งหมดรับ `venue` และ `account_id` แบบ optional เพื่อจำกัดขอบเขต
สำหรับ venue ที่มีหลาย account:

- `portfolio.unrealized_pnls` (รับ `venue`, `account_id`, `target_currency`): คืน map จาก `Currency` ไป `Money`
- `portfolio.realized_pnls` (รับ `venue`, `account_id`, `target_currency`): คืน map จาก `Currency` ไป `Money`
- `portfolio.total_pnls` (รับ `venue`, `account_id`, `target_currency`): คืน map จาก `Currency` ไป `Money`
- `portfolio.net_exposures` (รับ `venue`, `account_id`, `target_currency`): คืน map จาก `Currency` ไป `Money` หรือไม่มีค่า
- `portfolio.mark_values` (รับ `venue`, `account_id`): คืน map จาก `Currency` ไป `Money`
- `portfolio.equity` (รับ `venue`, `account_id`): คืน map จาก `Currency` ไป `Money`
- `portfolio.missing_price_instruments` (รับ `venue`, `account_id`): คืนรายการ `InstrumentId`

หากระบุ argument ด้านขอบเขตทั้งสองตัว ทั้งสองต้องชี้ไปที่ account เดียวกัน การไม่มีราคา, การแปลงเป็น target currency
ล้มเหลว หรือการคำนวณ overflow จะทำให้ทั้ง collection ที่ได้รับผลกระทบใช้ไม่ได้: query จะไม่คืนผลรวมแบบบางส่วนหรือ
แบบสกุลเงินปนกันเลย

ดู [คู่มือ Portfolio](portfolio.md#equity-and-mark-to-market) สำหรับสูตร equity, ลำดับการ fallback ของราคา, พฤติกรรมการ
แปลงเป็น base currency และตัวติดตามกรณีไม่มีราคาที่แจ้งเตือนเพียงครั้งเดียว

## Margin model {#margin-models}

NautilusTrader มี margin model ที่ยืดหยุ่นสำหรับเส้นทางแบบคำนวณ (backtest และ live strategy ที่รันด้วย
`calculate_account_state=True` เพื่อ reconciliation) margin ที่ venue รายงานมาจะเข้าสู่ store `margins` หรือ
`account_margins` ของ account โดยตรงโดยไม่ผ่าน model

### ภาพรวม {#overview}

แต่ละ venue จัดการ leverage ต่างกัน:

- **Broker แบบดั้งเดิม** (เช่น Interactive Brokers): ใช้เปอร์เซ็นต์ margin คงที่ไม่ว่า leverage จะเป็นเท่าใด
- **Crypto exchange** (Binance และอื่น ๆ): leverage อาจลด margin ที่ต้องใช้ลง

Model ที่มีมาให้ทั้งสองตัวคำนวณ margin เป็นเปอร์เซ็นต์ของ notional โดยใช้ field `margin_init` และ `margin_maint`
ของ instrument ต่างกันเพียงว่า leverage ช่วยลดยอดจองหรือไม่ สำหรับ venue ที่ใช้ margin คงที่รายสัญญาจริง
(CME / ICE) ให้ตั้ง `instrument.margin_init` และ `margin_maint` ให้เปอร์เซ็นต์ได้จำนวนเงินดอลลาร์ที่ต้องการ

### การ netting ในโหมด HEDGING {#hedging-mode-netting}

ภายใต้ `OmsType.HEDGING` fill แรกของ position ID ใหม่จะเปิด `Position` และ fill ต่อ ๆ มาจะอัปเดต position นั้นได้
account จึงสามารถถือ sub-position ที่เปิดอยู่จำนวนมากของ instrument เดียวกันได้ accounts manager จะ net sub-position
เหล่านั้นเข้าเป็น NETTING position สมมุติตามลำดับ `ts_opened` แล้วรัน margin model ครั้งเดียวบนปริมาณสุทธิแบบมีเครื่องหมาย
และราคาเปิดเฉลี่ยที่ได้

การ replay เป็นไปตามกฎเดียวกับ `Position.apply`:

- Fill ฝั่งเดียวกันให้ราคาเปิดเฉลี่ยถ่วงน้ำหนักตามปริมาณ
- Fill ฝั่งตรงข้ามจะปิดบางส่วนที่ราคาเฉลี่ยเดิม
- Fill ที่ข้ามศูนย์จะทำให้ส่วนที่เหลือใช้ราคาของ fill ที่พลิกฝั่งนั้น

Sub-position ที่มี `ts_opened` เดียวกันจะถูกรวมตามลำดับ `(ts_opened, position_id)` เพื่อให้ผลลัพธ์ไม่ขึ้นกับลำดับการ
iterate ของ cache

เมื่อใช้ margin model และ leverage เดียวกัน maintenance margin ของ HEDGING และ NETTING จะตรงกันหากปริมาณสุทธิ
และราคาเปิดเฉลี่ยที่รวมได้ตรงกัน margin ที่ต้องใช้จึงแปรผันตาม **exposure ทางเศรษฐกิจสุทธิ**

### Model ที่มีให้ใช้ {#available-models}

#### `StandardMarginModel` {#standardmarginmodel}

ใช้ **เปอร์เซ็นต์คงที่โดยไม่หารด้วย leverage** ตรงกับพฤติกรรมของ broker แบบดั้งเดิม

- Initial margin: `notional_value * instrument.margin_init`
- Maintenance margin: `notional_value * instrument.margin_maint`

**กรณีใช้งาน**: broker แบบดั้งเดิม (Interactive Brokers), forex broker ที่มี margin requirement คงที่

#### `LeveragedMarginModel` {#leveragedmarginmodel}

หาร margin requirement **ด้วย leverage**

- Initial margin: `(notional_value / leverage) * instrument.margin_init`
- Maintenance margin: `(notional_value / leverage) * instrument.margin_maint`

**กรณีใช้งาน**: crypto exchange ที่ลด margin ตาม leverage และ venue ที่ leverage มีผลต่อ margin requirement

### พฤติกรรมเริ่มต้น {#default-behavior}

`MarginAccount` ใช้ `LeveragedMarginModel` เป็นค่าเริ่มต้น backtest เลือกใช้ `StandardMarginModel` ได้โดยส่ง model นี้
ให้ `BacktestVenueConfig.margin_model` โดยตรง ดู [Backtesting](backtesting/accounts-and-margin.md#margin-models)

### ตัวอย่างการคำนวณ: EUR/USD {#worked-example-eurusd}

- **Instrument**: EUR/USD
- **ปริมาณ**: 100,000 EUR
- **ราคา**: 1.10000
- **Notional**: $110,000
- **Leverage**: 50x
- **`instrument.margin_init`**: 3%

| Model     | การคำนวณ               | ผลลัพธ์ | เปอร์เซ็นต์ |
| --------- | ---------------------- | ------ | -------- |
| Standard  | $110,000 × 0.03        | $3,300 | 3.00%    |
| Leveraged | ($110,000 ÷ 50) × 0.03 | $66    | 0.06%    |

บน account ขนาด $1,000: standard model จะบล็อก trade นี้ ส่วน leveraged model จะอนุญาต

## ข้อตกลงสำหรับ adapter {#adapter-convention}

Live adapter แปลง response ของ venue เป็น instance ของ `AccountBalance` และ `MarginBalance` ข้อตกลงที่ผู้เขียน adapter
ต้องปฏิบัติตาม:

### การสร้าง `AccountBalance` {#building-accountbalance}

ควรใช้ derived constructor เพื่อให้การ clamp และ invariant `total == locked + free` ถูกบังคับใช้จากศูนย์กลาง การคำนวณ
สาม field ด้วยตนเองแล้วส่งให้ `AccountBalance::new` เหมาะเฉพาะกับเส้นทางแบบ pass-through ที่ค่าทั้งสามเชื่อถือได้อยู่แล้ว
(เช่น test)

### การสร้าง `MarginBalance` {#building-marginbalance}

เลือก scope ให้ตรงกับสิ่งที่ venue รายงาน:

| Venue รายงาน                                    | Scope          | สร้างด้วย                                                     |
| ---------------------------------------------- | -------------- | ---------------------------------------------------------- |
| ราย instrument (isolated position)              | ราย instrument  | `MarginBalance::new(initial, maint, Some(id))`             |
| ค่ารวมเดียวต่อหลักประกัน (cross margin)             | ทั้ง account     | `MarginBalance::new(initial, maint, None)`                 |
| หลายค่ารวม หนึ่งค่าต่อหลักประกัน                      | ทั้ง account     | `MarginBalance` หนึ่งตัวต่อสกุลเงิน โดย `instrument_id=None` |

:::info
ไม่มีการใช้ `InstrumentId` สังเคราะห์แบบ `ACCOUNT.{VENUE}` หรือ `ACCOUNT-{COIN}.{VENUE}` เป็น placeholder
รายการระดับทั้ง account จะมี `instrument_id=None` และใช้ `currency` เป็น key
:::

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Backtesting](backtesting/): starting balance, margin model และการตั้งค่า account เฉพาะของ backtest
- [Portfolio](portfolio.md): PnL ระดับ portfolio, exposure และการแปลงสกุลเงิน
- [Positions](positions.md): วงจรชีวิตของ position, การ aggregate และ PnL
- [Adapters](adapters.md): ข้อกำหนดและแนวปฏิบัติที่ดีสำหรับผู้เขียน adapter
- [Blockchain](../integrations/blockchain.md): adapter ที่เลือกใช้ wallet account และสถานะด้าน execution ของมัน
