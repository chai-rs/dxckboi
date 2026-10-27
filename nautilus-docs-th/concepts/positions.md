---
title: "Positions"
---

คู่มือนี้อธิบายว่า NautilusTrader สร้างและอัปเดต position จาก order fill อย่างไร และคำนวณกำไรขาดทุน (PnL)
อย่างไร รวมถึงอธิบายว่า NautilusTrader เก็บรักษารอบ (cycle) ที่ปิดไปแล้วอย่างไรภายใต้การตั้งค่า order management
system (OMS) แบบ `NETTING`

## ภาพรวม [#overview]

Position บันทึก exposure ต่อ instrument หนึ่งในช่วงรอบเปิด-ปิด โดยรวบรวม fill ที่ถูกกำหนดให้กับ position ID เดียว
และติดตามปริมาณ ราคาเฉลี่ย realized PnL, commission และ identifier ที่เกี่ยวข้อง ใช้ราคาตลาดร่วมกับ method ของ
position เพื่อคำนวณ unrealized PnL และมูลค่า notional

Execution engine สร้าง position เมื่อ order ถูก fill และติดตามตั้งแต่เปิดจนปิด การตั้งค่า OMS เป็นตัวกำหนดว่า fill
จะใช้ net position ร่วมกัน หรือแยกอยู่ใน position แบบ hedged ต่างหาก

## วงจรชีวิตของ position [#position-lifecycle]

### การสร้าง [#creation]

ระบบจะเปิด position เมื่อเกิด fill แรก:

- **NETTING OMS**: เปิดเมื่อเกิด fill แรกของ instrument และ strategy นั้น position จะใช้ ID แบบ deterministic
  `{instrument_id}-{strategy_id}`
- **HEDGING OMS**: เปิดเมื่อเกิด fill แรกของ `position_id` ใหม่ (มีได้หลาย position ต่อ instrument)

Position ติดตามข้อมูลต่อไปนี้:

- รายละเอียดของ order และ fill ที่เปิด position
- ฝั่งของการเข้า (`BUY` หรือ `SELL`)
- ปริมาณและราคาเข้าเฉลี่ยหลังนำ fill ที่เปิดมาคำนวณ
- Timestamp ของการ initialize และการเปิด

:::tip
คุณเข้าถึง position ได้ผ่าน Cache ด้วย method `position(position_id)` หรือ `positions(...)` (กรองด้วย
`instrument_id` ได้) จากภายใน actor หรือ strategy
:::

### การอัปเดต [#updates]

เมื่อเกิด fill เพิ่มเติม position จะ:

- รวมปริมาณจาก fill ฝั่ง buy และ sell
- คำนวณราคาเข้าและราคาออกเฉลี่ยใหม่
- อัปเดต peak quantity ของรอบปัจจุบัน
- ติดตาม order ID และ trade ID ของรอบปัจจุบัน
- สะสม commission แยกตามสกุลเงิน

### การปิด [#closure]

Position จะปิดเมื่อ **net quantity เป็นศูนย์** (`FLAT`) เมื่อปิด:

- บันทึก closing order ID ไว้ หากเป็นการ settle ตอนสัญญาหมดอายุ ค่านี้จะว่าง
- คำนวณระยะเวลาตั้งแต่เปิดจนปิด
- คำนวณ realized PnL สุดท้าย
- ไม่ว่าจะใช้ OMS แบบใด เมื่อ position เปิดใหม่ภายหลังด้วย ID เดิม engine จะทำ snapshot สถานะที่ปิดไว้เพื่อเก็บ
  PnL ย้อนหลัง (ดู [Position snapshotting](#position-snapshotting))

### การ settle ตอนสัญญาหมดอายุ [#settlement-at-contract-expiration]

ใน live node `InstrumentClose` ประเภท `CONTRACT_EXPIRED` จะ settle position ของ binary option เฉพาะบน venue
ที่ execution client ของ node ให้บริการอยู่เท่านั้น ภายใต้เงื่อนไขนั้น ระบบจะ settle position ที่เปิดอยู่ทุกตัวของ
instrument นั้นที่ราคาปิด (เช่น `1` สำหรับผลลัพธ์ที่ชนะ และ `0` สำหรับผลลัพธ์ที่แพ้) สำหรับแต่ละ position
execution engine จะ:

- ปิดรอบปัจจุบันที่ราคาปิดโดยไม่สร้าง order หรือ fill
- บันทึก realized PnL สำหรับปริมาณที่ settle ส่วนปริมาณ fill และ commission คงเดิมตามที่ fill ดังนั้นค่าธรรมเนียม
  ตอนเปิดยังคงอยู่ใน realized PnL
- ปล่อย `PositionClosed` หนึ่งครั้งโดยไม่มี closing order ID

การปิดครั้งแรกที่ apply กับ instrument ถือเป็นค่าที่เชื่อถือได้ การปิดซ้ำหรือขัดแย้งกัน หรือการ restart ที่กู้คืน position
ที่ settle แล้ว จะไม่ settle ซ้ำอีก หลัง settle แล้ว fill หรือ fill void ของ instrument นั้นยังคงอัปเดต order แต่จะไม่
เปลี่ยน position อีก engine จะ log warning และไม่ปล่อย position event

การ settle ไม่ได้ redeem สินทรัพย์ที่ venue และไม่ได้เปลี่ยนยอดเงินใน account รายงาน account จาก venue ยังคงเป็น
แหล่งข้อมูลที่เชื่อถือได้สำหรับเงินสด ดังนั้นเงินที่ยังไม่ได้ redeem จะนับรวมในยอดเงินก็ต่อเมื่อ venue รายงานเงินที่
redeem แล้วเท่านั้น

Backtest และ sandbox paper trading ยังคงใช้การจัดการการหมดอายุของ venue จำลองแทน matching engine จะปิด
position ด้วย expiration fill และ fill เหล่านั้นจะเข้า account จำลอง live node จะไม่ใช้การ settle ของ engine
บน venue ที่ execution client settle สัญญาที่หมดอายุเอง เช่นเดียวกับที่ sandbox client ทำ

## การรวม order fill [#order-fill-aggregation]

Position รวม order fill เพื่อรักษามุมมองของ exposure ในตลาดให้ถูกต้อง กระบวนการรวมจัดการกิจกรรมการเทรดทั้งสองฝั่ง:

### Fill ฝั่ง buy [#buy-fills]

เมื่อ order ฝั่ง BUY ถูก fill:

- เพิ่ม exposure ฝั่ง long หรือลด exposure ฝั่ง short
- อัปเดตราคาเข้าเฉลี่ยสำหรับ trade ที่เปิด position
- อัปเดตราคาออกเฉลี่ยสำหรับ trade ที่ปิด position
- คำนวณ realized PnL ของส่วนที่ถูกปิด

### Fill ฝั่ง sell [#sell-fills]

เมื่อ order ฝั่ง SELL ถูก fill:

- เพิ่ม exposure ฝั่ง short หรือลด exposure ฝั่ง long
- อัปเดตราคาเข้าเฉลี่ยสำหรับ trade ที่เปิด position
- อัปเดตราคาออกเฉลี่ยสำหรับ trade ที่ปิด position
- คำนวณ realized PnL ของส่วนที่ถูกปิด

### การคำนวณ net position [#net-position-calculation]

Position มี field `signed_qty` ที่แทน net exposure:

- ค่าบวกหมายถึง position แบบ `LONG`
- ค่าลบหมายถึง position แบบ `SHORT`
- ศูนย์หมายถึง position แบบ `FLAT` (ปิดแล้ว)

```text
# Example: Position aggregation
# Initial BUY 100 units at $50
signed_qty = +100  # LONG position

# Subsequent SELL 150 units at $55
signed_qty = -50  # Closes the LONG cycle and opens a SHORT cycle

# Final BUY 50 units at $52
signed_qty = 0  # Position FLAT (closed)
```

### การบัญชีเมื่อกลับทิศ (reversal) [#reversal-accounting]

Fill ฝั่งตรงข้ามที่มีขนาดใหญ่กว่าปริมาณที่เปิดอยู่ จะปิด exposure เดิมและเปิดส่วนที่เหลือในทิศทางตรงข้าม execution
engine จะแยก fill นี้เป็นการปิดและการเปิดใหม่ โดยการเก็บสถานะที่ปิดแล้วเป็นไปตาม
[Position snapshotting](#position-snapshotting)

Fill แบบกลับทิศที่ไม่ได้ถูกแยกและ apply โดยตรงกับ `Position` ตัวเดียว จะเริ่มช่วงการบัญชี (accounting episode)
ใหม่สำหรับ exposure ส่วนที่เหลือ ช่วงใหม่นี้มีการ reset ดังนี้ รวมถึงระหว่าง replay ของ fill void ด้วย:

- `avg_px_open` เปลี่ยนเป็นราคาของ fill ที่กลับทิศ
- `avg_px_close` เป็น `None` และ `realized_return` เป็นศูนย์จนกว่าจะมี fill ที่ปิดครั้งถัดไป
- `buy_qty` และ `sell_qty` เริ่มใหม่ โดยมีเฉพาะส่วนที่เหลือที่เปิดใหม่ในฝั่งการเข้าใหม่ และเป็นศูนย์ในอีกฝั่ง ดังนั้น
  ค่าเฉลี่ยการปิดในภายหลังจะไม่รวม volume จากทิศทางก่อนหน้า

**เฉพาะส่วนที่ปิด** เท่านั้นที่ realize PnL เทียบกับราคาเข้าเดิม `realized_pnl` และยอดรวม commission ของ object
ยังคงสะสมต่อเนื่องข้ามการกลับทิศนี้ โดยนับ commission ของ fill เพียงครั้งเดียว การ reset ไม่ได้ล้างประวัติ fill,
timestamp ตอนเปิด หรือ peak quantity หาก position ไปถึง `FLAT` แล้วมี fill ภายหลังเปิดใหม่ จะ reset ทั้งรอบ
รวมถึง realized PnL และ commission

การ reset ช่วงการบัญชีเหล่านี้ใช้กับการกลับทิศที่เกิดจาก fill เท่านั้น การปรับปริมาณที่เปลี่ยนฝั่งของ position
จะไม่ทำการ reset แบบเดียวกัน

## การปรับ position [#position-adjustments]

Position adjustment บันทึกการเปลี่ยนแปลงปริมาณหรือ PnL ที่เกิดขึ้นนอกเหนือจาก order fill ปกติ ระบบแทน
การเปลี่ยนแปลงเหล่านี้ด้วย event `PositionAdjusted`

### Commission ในสกุล base currency [#base-currency-commissions]

เมื่อเทรดคู่เงินแบบ spot (เช่น BTC/USDT) หรือ FX spot commission ที่จ่ายเป็น base currency จะส่งผลโดยตรงต่อ
ปริมาณสุทธิที่ได้รับหรือส่งมอบ:

- **Fill ที่เปิด**: หัก commission ออกจากปริมาณที่เทรด การซื้อ 1.0 BTC โดยเสีย commission 0.001 BTC
  จะได้ net long position 0.999 BTC
- **Fill ที่ปิด**: commission ถูก apply กับ `signed_qty` เพราะกระทบ inventory จริง การขาย position LONG
  0.999 BTC โดยเสีย commission 0.000999 BTC จะทำให้คุณ SHORT อยู่ 0.000999 BTC ไม่ใช่ FLAT เพราะคุณ
  สละไปทั้งหมด 0.999999 BTC
- **Flip**: commission ส่งผลต่อขนาด position สุดท้ายทั้งสองฝั่งของการ flip

:::note
Commission ในสกุล base currency ใช้กับคู่เงินแบบ spot และ instrument แบบ FX spot ที่สกุลเงินของ commission
ตรงกับ `instrument.base_currency` เท่านั้น สำหรับ instrument อื่น commission จะถูกติดตามแยกและไม่กระทบปริมาณ position
:::

### Funding payment [#funding-payments]

Funding adjustment ติดตามการจ่ายเงินตามรอบของ perpetual futures โดยไม่กระทบปริมาณ position ใช้
`quantity_change = None` และอาจมีการเปลี่ยนแปลง PnL รวมอยู่ด้วย

### การติดตาม adjustment [#adjustment-tracking]

Position เปิดให้เข้าถึง adjustment ที่เก็บไว้:

- `position.adjustments()` คืน list ของ event `PositionAdjusted` ทั้งหมด
- Adjustment แต่ละรายการมีประเภท (`COMMISSION` หรือ `FUNDING`), การเปลี่ยนแปลงปริมาณหรือ PnL, เหตุผล,
  event ID และ timestamp
- ประวัติ adjustment ปัจจุบันจะถูกล้างเมื่อ position ที่ปิดแล้วเปิดใหม่
- หลัง `purge_events_for_order()` position ที่มี fill จะสร้าง commission adjustment ใหม่จาก fill ที่เหลืออยู่และ
  apply adjustment ที่ไม่ใช่ commission ซ้ำ หากไม่มี fill เลย position จะกลายเป็นเปลือก `FLAT` ว่าง ๆ และล้าง
  ประวัติ adjustment

## ประเภท OMS และการจัดการ position [#oms-types-and-position-management]

NautilusTrader รองรับโหมดการจัดการ position สองแบบ strategy ที่ตั้งค่าเป็น `OmsType.UNSPECIFIED` จะใช้ประเภท
OMS ของ execution client ที่เป็นเจ้าของ สำหรับการ resolve ความเป็นเจ้าของและกฎของ position ID ดู
[คู่มือ Execution](execution/index.md#order-management-system-oms)

### `NETTING` [#netting]

ในโหมด `NETTING` fill ของแต่ละ instrument และ strategy จะถูกรวมเป็น position เดียว:

- หนึ่ง position ต่อ instrument และ strategy
- ทุก fill ส่งผลต่อ position เดียวกัน
- Fill ที่ข้ามศูนย์จะปิดรอบปัจจุบันและเปิดรอบใหม่ในฝั่งตรงข้าม
- Snapshot ย้อนหลังเก็บรักษาสถานะของ position ที่ปิดแล้ว

### `HEDGING` [#hedging]

ในโหมด `HEDGING` สามารถมีหลาย position สำหรับ instrument เดียวกัน:

- มี position แบบ `LONG` และ `SHORT` หลายตัวพร้อมกันได้
- แต่ละ position มี position ID ไม่ซ้ำกัน
- แต่ละ position ถูกติดตามแยกกัน
- ไม่มีการ netting ข้าม position โดยอัตโนมัติ
- Fill ที่มี position ID ใหม่จะสร้าง position แยกต่างหาก หาก fill ภายหลังใช้ position ID ที่ปิดแล้วซ้ำ engine จะ
  archive รอบที่ปิดแล้วก่อนแทนที่สถานะใน cache
- Virtual position flip จะสร้าง ID ใหม่และคง position เดิมที่ปิดแล้วไว้ใน cache ดังนั้นเส้นทางนี้ไม่จำเป็นต้องมี
  snapshot ของรอบที่ปิด

หลัง virtual flip fill ภายหลังของ order เดียวกันจะไปที่ position ปลายทาง แม้จะมี position ID เดิมติดมาก็ตาม engine
หา position ปลายทางจากประวัติ order และ position ที่เก็บไว้ รวมถึงหลังการกู้ cache และการแก้ไขด้วย fill void
หากประวัตินั้นไม่ครบหรือกำกวม engine จะ reject fill

Virtual position แบบ `HEDGING` จะเก็บประวัติสำหรับ replay ข้ามการเปิดใหม่เสมอ แม้จะตั้ง
`carry_replay_events_on_reopen=False` ก็ตาม เพื่อรักษาการ routing นี้เมื่อมีการใช้ position ID ซ้ำ

:::warning
`HEDGING` อาจเพิ่มความต้องการ margin เมื่อ venue ดูแล position ฝั่ง long และ short แยกกัน venue ที่ใช้ OMS แบบ
`NETTING` จะแสดงเฉพาะ net position แม้ NautilusTrader จะติดตาม virtual position หลายตัวก็ตาม ตรวจสอบ position
mode และกฎ margin ของ venue
:::

### OMS ของ strategy เทียบกับของ venue [#strategy-vs-venue-oms]

ประเภท OMS ของ strategy และของ venue อาจต่างกันได้:

| Strategy OMS | Venue OMS | ผลลัพธ์                                                                |
| ------------ | --------- | --------------------------------------------------------------------- |
| `NETTING`    | `NETTING` | หนึ่ง position ต่อ instrument และ strategy                               |
| `HEDGING`    | `HEDGING` | หลาย position ต่อ instrument และ strategy                                |
| `NETTING`    | `HEDGING` | Virtual position หนึ่งตัวครอบคลุม position ทั้งหมดของ venue                 |
| `HEDGING`    | `NETTING` | Virtual position หลายตัวเทียบกับ net position เดียวของ venue               |

:::tip
ตั้งประเภท OMS ของ strategy และ venue ให้ตรงกัน เว้นแต่ strategy ต้องการ virtual position ดูคู่มือ integration
สำหรับการตั้งค่า position mode ของ venue
:::

## Position snapshotting [#position-snapshotting]

Position snapshotting เก็บรักษารอบที่ปิดแล้วไว้สำหรับการติดตาม PnL และการรายงาน เมื่อ fill ภายหลังเปิด position
ที่ปิดแล้วขึ้นมาใหม่

### ทำไม snapshotting จึงสำคัญ [#why-snapshotting-matters]

เมื่อ position ปิด (เป็น `FLAT`) แล้วเปิดใหม่ด้วย ID เดิมจาก trade ใหม่ object ของ position จะ reset เพื่อติดตาม
exposure ใหม่ หากไม่มี snapshotting realized PnL ย้อนหลังของรอบก่อนหน้าจะหายไป

### ทำงานอย่างไร [#how-it-works]

เมื่อ fill เปิด position ที่ปิดแล้วขึ้นใหม่ด้วย ID เดิม execution engine จะ archive สถานะที่ปิดไว้ก่อนเปิดรอบถัดไป
ใช้ได้กับทั้ง OMS แบบ `NETTING` และ `HEDGING` ส่วน flip ของ `HEDGING` ที่ใช้ ID แบบไม่ใช่ virtual
จะไปอีกเส้นทางหนึ่ง คือใช้ ID ซ้ำโดยไม่ archive รอบที่ปิด

Snapshot เก็บรักษา:

- ปริมาณและราคาสุดท้าย
- Realized PnL
- Fill event ทั้งหมด
- ยอดรวม commission

Cache เก็บ snapshot ตาม position ID โดย entry ที่ active ใน cache จะแทนรอบใหม่ ส่วน snapshot ก่อนหน้ายังเข้าถึง
ได้ Portfolio จะรวม realized PnL ของ snapshot เหล่านี้เข้าในยอดรวมของ instrument

Fill void ที่แก้ไข fill จากรอบก่อนหน้าเป็นข้อยกเว้นเดียว การแก้ไขนี้เลื่อนขอบเขตของรอบที่ snapshot ที่เก็บไว้อธิบาย
ดังนั้น engine จะแทนที่ snapshot เหล่านั้นด้วยรอบที่ประวัติที่แก้ไขแล้วปิดจริง โดยนับแต่ละรอบเพียงครั้งเดียว ดู
[Position replay across NETTING cycles](execution/index.md#position-replay-across-netting-cycles)

:::note
Archive ของรอบที่ปิดนี้ต่างจาก position state snapshot ซึ่งเป็นตัวเลือกเสริม:

- `snapshot_positions=True` จะ publish สถานะเมื่อ position เปิด เปลี่ยนแปลง หรือปิด
- `snapshot_positions_interval_secs` จะ publish position ที่เปิดอยู่ทั้งหมดเป็นระยะ

Cache ที่มี Redis หรือ Postgres เป็น backing จะ persist snapshot เหล่านี้ด้วย หากไม่มี cache backing ทั้งสองเส้นทาง
จะ publish snapshot บน message bus ภายใน process โดยไม่ persist ดูค่าตั้งเหล่านี้ได้ที่ `LiveExecutionEngineConfig`
:::

### ตัวอย่างสถานการณ์ [#example-scenario]

```text
# NETTING OMS Example
# Cycle 1: Open LONG position
BUY 100 units at $50   # Position opens
SELL 100 units at $55  # Position closes, PnL = $500
# Snapshot taken preserving $500 realized PnL

# Cycle 2: Open SHORT position
SELL 50 units at $54   # Position reopens (SHORT)
BUY 50 units at $52    # Position closes, PnL = $100
# Snapshot taken preserving $100 realized PnL

# Total realized PnL = $500 + $100 = $600 (from snapshots)
```

## การคำนวณ PnL [#pnl-calculations]

การคำนวณ PnL ของ position คำนึงถึง specification ของ instrument และธรรมเนียมของตลาด

### Realized PnL [#realized-pnl]

ส่วนของราคาใน realized PnL จะคำนวณเมื่อ fill ปิด position บางส่วนหรือทั้งหมด หรือเมื่อ
[การ settle ตอนสัญญาหมดอายุ](#settlement-at-contract-expiration)ปิด position commission ในสกุล cost currency
ของ position จะส่งผลต่อ realized PnL ทันทีที่แต่ละ fill เข้ามา

```text
# For standard instruments
# LONG: realized_pnl = (exit_price - entry_price) * closed_quantity * multiplier
# SHORT: realized_pnl = (entry_price - exit_price) * closed_quantity * multiplier

# For inverse instruments (side-aware)
# LONG: realized_pnl = closed_quantity * multiplier * (1/entry_price - 1/exit_price)
# SHORT: realized_pnl = closed_quantity * multiplier * (1/exit_price - 1/entry_price)
```

ฝั่งของ position เป็นตัวเลือกสูตร inverse instrument ที่อิง premium (เช่น option ที่ settle เป็นเหรียญ) quote premium
เป็น base currency ดังนั้นจึงใช้สูตรมาตรฐาน และรายงาน PnL และมูลค่า notional เป็น base currency

### Unrealized PnL [#unrealized-pnl]

`unrealized_pnl()` คำนวณ PnL ของ position ที่เปิดอยู่จาก `price` ที่ส่งเข้าไป คุณส่งราคา bid, ask, mid, last
หรือ mark ก็ได้ เช่น ใช้ราคา last trade, ใช้ bid เพื่อประเมินแบบระมัดระวังสำหรับ position LONG หรือใช้ ask เพื่อ
ประเมินแบบระมัดระวังสำหรับ position SHORT

สำหรับ position แบบ `FLAT` จะคืน `Money(0, cost_currency)` เสมอไม่ว่าจะส่งราคาใดเข้าไป

### Total PnL [#total-pnl]

`total_pnl()` รวมส่วน realized และ unrealized เข้าด้วยกัน (คืนค่า `realized_pnl + unrealized_pnl` ที่ราคาปัจจุบันที่ส่งเข้าไป)

### ข้อพิจารณาเรื่องสกุลเงิน [#currency-considerations]

- PnL คำนวณในสกุล cost currency ของ instrument คือ quote สำหรับสัญญาแบบ linear, base สำหรับสัญญาแบบ inverse
  และ settlement สำหรับสัญญาแบบ quanto
- สำหรับ Forex โดยทั่วไป cost currency คือ quote currency
- Portfolio รวม realized PnL ต่อ instrument ในสกุล cost currency
- ยอดรวมหลายสกุลเงินต้องแปลงค่านอก Position

## Commission และต้นทุน [#commissions-and-costs]

Position ติดตาม commission ของ fill:

- Commission สะสมแยกตามสกุลเงิน
- Commission ของแต่ละ fill ถูกบวกเข้ายอดรวมสะสม
- รองรับ commission หลายสกุลเงิน
- Realized PnL รวม commission เฉพาะที่อยู่ในสกุล cost currency ของ position
- Commission อื่นถูกติดตามแยกและอาจต้องแปลงค่า

`position.commissions()` คืนยอดรวม commission แยกตามสกุลเงิน ส่วน `position.notional_value(current_price)` คืน
`Money` ในสกุล quote (linear), base (inverse) หรือ settlement currency (quanto)

:::warning
การคำนวณ notional value จะล้มเหลวภายใต้เงื่อนไขต่อไปนี้:

- Inverse position ไม่มี base currency
- ราคาที่ส่งเข้าไปไม่เป็นบวกสำหรับ inverse position ที่ไม่อิง premium
- ผลลัพธ์ไม่สามารถแทนด้วย `Money` ได้

ฝั่ง Rust ใช้ `try_notional_value()` เพื่อจัดการ error ของการคำนวณเหล่านี้ได้ ส่วน `notional_value()` จะ panic
หากการคำนวณล้มเหลว
:::

## Property และสถานะของ position [#position-properties-and-state]

### Identifier [#identifiers]

- `id`: identifier ของ position ที่ไม่ซ้ำกัน
- `instrument_id`: instrument ที่เทรด
- `account_id`: account ที่ถือ position
- `trader_id`: trader ที่เป็นเจ้าของ position
- `strategy_id`: strategy ที่จัดการ position
- `opening_order_id`: client order ID ที่เปิด position
- `closing_order_id`: client order ID ที่ปิด position หากปิดด้วย fill

### สถานะของ position [#position-state]

- `side`: ฝั่งของ position ปัจจุบัน (`LONG`, `SHORT` หรือ `FLAT`)
- `entry`: ฝั่งที่เปิดของรอบปัจจุบัน (`BUY` สำหรับ `LONG`, `SELL` สำหรับ `SHORT`) อัปเดตเมื่อ position กลับทิศ
- `quantity`: ขนาด position ปัจจุบันแบบค่าสัมบูรณ์
- `signed_qty`: ขนาด position แบบมีเครื่องหมายเป็น Decimal (บวกสำหรับ `LONG`, ลบสำหรับ `SHORT`)
- `peak_qty`: ปริมาณสูงสุดที่เคยถึงในรอบเปิด-ปิดปัจจุบัน
- `is_open`: position เปิดอยู่หรือไม่
- `is_closed`: position ปิดแล้วหรือไม่ (`FLAT`)
- `is_long`: ฝั่งของ position เป็น `LONG` หรือไม่
- `is_short`: ฝั่งของ position เป็น `SHORT` หรือไม่

### ราคาและการประเมินมูลค่า [#pricing-and-valuation]

- `avg_px_open`: ราคาเข้าเฉลี่ยแบบ Decimal
- `avg_px_close`: ราคาออกเฉลี่ยแบบ Decimal เมื่อปิด
- `realized_pnl`: กำไร/ขาดทุนที่ realize แล้ว
- `realized_return`: อัตราผลตอบแทนแบบ floating-point (เช่น 0.05 สำหรับ 5%)
- `quote_currency`: quote currency ของ instrument
- `base_currency`: base currency หากมี
- `settlement_currency`: สกุลเงินสำหรับ settle PnL

ดู [การบัญชีเมื่อกลับทิศ](#reversal-accounting) สำหรับการ reset ราคาเฉลี่ยและผลตอบแทน ในขณะที่ realized PnL
ยังสะสมต่อเนื่อง เมื่อ fill ที่ไม่ถูกแยกทำให้ position หนึ่งกลับทิศ

### Specification ของ instrument [#instrument-specifications]

- `multiplier`: contract multiplier
- `price_precision`: precision ทศนิยมของราคา
- `size_precision`: precision ทศนิยมของปริมาณ
- `is_inverse`: instrument เป็น inverse หรือไม่

### Timestamp [#timestamps]

- `ts_init`: เวลาที่ position ถูก initialize
- `ts_opened`: เวลาที่ position ถูกเปิด
- `ts_last`: timestamp ของการอัปเดตล่าสุด
- `ts_closed`: เวลาที่ position ถูกปิด หากปิดแล้ว
- `duration_ns`: ระยะเวลาตั้งแต่เปิดจนปิดหน่วย nanosecond หรือเป็นศูนย์ขณะยังเปิดอยู่

### ข้อมูลที่เกี่ยวข้อง [#associated-data]

- `symbol`: ticker symbol ของ instrument
- `venue`: trading venue
- `client_order_ids`: client order ID ที่ไม่ซ้ำของ fill ที่เก็บไว้ในรอบปัจจุบัน
- `venue_order_ids`: venue order ID ที่ไม่ซ้ำของ fill ที่เก็บไว้ในรอบปัจจุบัน
- `trade_ids`: trade ID ที่ไม่ซ้ำของ fill ที่เก็บไว้ในรอบปัจจุบัน
- `events`: order fill event ที่เก็บไว้ในรอบปัจจุบัน
- `adjustments`: position adjustment ที่เก็บไว้ในรอบปัจจุบัน
- `event_count`: จำนวน fill event ที่เก็บไว้ในรอบปัจจุบัน
- `last_event`: fill event ล่าสุดที่เก็บไว้
- `last_trade_id`: trade ID ของ fill ล่าสุดที่เก็บไว้

:::info
ดูข้อมูล type ทั้งหมดและเอกสารของ property โดยละเอียดได้ที่ [Rust API docs](https://nautilustrader.io/docs/core-latest/)
:::

## Event และการติดตาม [#events-and-tracking]

Object `Position` แต่ละตัวบันทึก fill และ adjustment ของรอบเปิด-ปิดปัจจุบัน:

- Fill event เรียงตามลำดับที่ apply
- Accessor ของ client order ID, venue order ID และ trade ID คืนค่าที่เรียงแล้วและไม่ซ้ำกัน
- `event_count` รายงานจำนวน fill event ที่เก็บไว้
- รอบ `NETTING` ที่ปิดแล้วเก็บประวัติ event ไว้ใน cache snapshot ตามที่อธิบายข้างต้น

ข้อมูลนี้รองรับ:

- การวิเคราะห์ position โดยละเอียด
- การ reconcile trade
- Performance attribution
- Audit trail

:::tip
ใช้ `position.events()` เพื่อเข้าถึง fill ที่เก็บไว้ในรอบปัจจุบันสำหรับ reconciliation ผลลัพธ์ของ
`position.trade_ids()` ช่วยในการจับคู่กับ statement ของ broker ดู[คู่มือ Execution](execution/)
สำหรับแนวปฏิบัติที่ดีในการทำ reconciliation
:::

## ความแม่นยำเชิงตัวเลข [#numerical-precision]

`Position` ใช้การคำนวณแบบ checked ทั้งในระดับ domain และ raw สำหรับการดำเนินการกับปริมาณและค่าธรรมเนียมแบบ
แม่นยำ และใช้ `Decimal` สำหรับปริมาณแบบมีเครื่องหมาย ราคาเฉลี่ย cost basis และเศษ PnL ส่วน `Price`, `Quantity`
และ `Money` ยังคงใช้การแทนค่าแบบ fixed-point `realized_return` ยังคงเป็นค่าประมาณแบบ floating-point สำหรับ
การวิเคราะห์ และไม่ได้ป้อนเข้าการบัญชี

การคำนวณมีขอบเขตจำกัด คือ coefficient ขนาด 96 บิตและ decimal scale สูงสุด 28 การแปลง input ต้องพอดีแบบไม่มี
การสูญเสีย ผลคูณที่ derive มา ผลรวมต้นทุน การหาร และค่ากลางตามสัดส่วน จะปัดภายในขอบเขตนั้น การดำเนินการจะล้มเหลว
หาก input สูญเสีย precision ค่ากลาง overflow หรือค่ากลางที่ไม่เป็นศูนย์หายไป ที่ขีดจำกัด precision ของ Decimal
ผลรวม PnL อาจดูดซับเศษที่หายไปซึ่งเล็กกว่าหนึ่งหน่วย `Money` ของ settlement แบบ native ได้ แต่ operand ที่หายไป
ซึ่งมีขนาดอย่างน้อยหนึ่งหน่วย native ยังคงทำให้ล้มเหลว ส่วนเศษที่แทนค่าได้จะยังอยู่ใน position
การดำเนินการ fill, adjustment, settlement, purge และ fill-void ที่อาจล้มเหลวได้ จะไม่เปลี่ยน position เมื่อเกิด error

### Cost basis และการปัดเศษ [#cost-basis-and-rounding]

ตัวสะสมต้นทุนการเปิดและปิดเก็บยอดเงินที่อยู่เบื้องหลังราคาเฉลี่ยไว้ ค่าเฉลี่ยที่ปัดแล้วจะไม่ถูกป้อนเข้า cost basis
การลดบางส่วนจะเก็บต้นทุนการเปิดที่ยังไม่ได้จัดสรรไว้ และการลดครั้งสุดท้ายจะใช้ส่วนที่เหลือนั้นทั้งหมด เศษ PnL แบบ
Decimal ที่แยกต่างหากจะเก็บเศษส่วนที่ต่ำกว่า scale ของ Money แบบ native ไว้ระหว่างการอัปเดต

ยอด Money ที่ derive มาจะปัดแบบ midpoint to even ที่ scale แบบ native คือ 9 ตำแหน่งทศนิยมในโหมดมาตรฐาน และ
16 ตำแหน่งในโหมด high-precision สกุลเงินที่รองรับ precision ละเอียดกว่าจะใช้ precision นั้นแทน precision สำหรับ
แสดงผลของสกุลเงินไม่ได้ตัดยอดที่เก็บไว้ การจัดสรรค่าธรรมเนียมเมื่อกลับทิศทำงานในหน่วย Money แบบ native โดยส่วน
ที่ปิดจะตัดเศษเข้าหาศูนย์ และส่วนที่เปิดจะได้รับเศษที่เหลือ ทำให้ยอดรวมค่าธรรมเนียมคงเดิม

| การดำเนินการ                                                     | ผลทางบัญชี                                                  |
| -------------------------------------------------------------- | --------------------------------------------------------- |
| เปิดหนึ่งหน่วยโดยมีค่าธรรมเนียม `0.000001 USD`                          | Realized PnL เป็น `-0.000001 USD`                         |
| ซื้อ `0.01` ที่ `0.1234` แล้วขายที่ `0.1235` โดยไม่มีค่าธรรมเนียม          | Realized PnL เป็น `0.000001 USD`                          |
| ปิด long `0.9` ด้วยการขาย `1` และค่าธรรมเนียม `0.006 USD`             | ค่าธรรมเนียมปิดเป็น `0.0054 USD` ค่าธรรมเนียมเปิดเป็น `0.0006 USD` |

Quantity precision ที่รองรับอาจเกิน display precision ของ instrument ได้ เช่น หลังหักค่าธรรมเนียมในสกุล base
currency position จะเก็บปริมาณนั้นไว้แทนการปัดให้เป็น flat close order ของ strategy จะปัดปริมาณดังกล่าวลงเป็น
ผลคูณของ size increment ของ instrument เศษเล็ก ๆ (dust) จะยังเปิดอยู่ และจะไม่ส่ง close order เมื่อปริมาณทั้งหมด
ปัดแล้วเป็นศูนย์ PnL ของ inverse ที่ไม่อิง premium ต้องใช้ราคาที่เป็นบวก เมื่อเปิด feature `defi` จะรองรับ input
ที่มีทศนิยม 17 และ 18 ตำแหน่ง หาก input และผลทางบัญชีที่ได้อยู่ภายในขอบเขตของ Decimal และ domain

### การ persist และ replay [#persistence-and-replay]

Position ที่ serialize แล้วรวมต้นทุนการเปิด ต้นทุนการปิด ปริมาณที่ปิดแล้ว และเศษ PnL ต้องเก็บรักษา field เหล่านี้
ไว้เมื่อกู้คืน position ที่จะประมวลผล fill ต่อ ค่าทางการเงินของวงจรชีวิตและ snapshot ของ position ใช้ข้อความ
ทศนิยมใน Arrow ส่วน SQL เก็บปริมาณแบบมีเครื่องหมายและราคาเฉลี่ยเป็น `NUMERIC`

Payload ของ position และการบัญชีวงจรชีวิตใน EventStore จาก build รุ่นเก่าไม่เป็นไปตามข้อตกลงนี้ ให้สร้าง store
เหล่านั้นใหม่จาก input ต้นฉบับก่อน replay ด้วยเวอร์ชันนี้ marker ของรูปแบบ envelope ใน EventStore ไม่ได้ทำ version
ให้กับ payload การบัญชีแต่ละรายการ

การ replay ของ EventStore derive การบัญชีจาก fill, adjustment และ settlement notification ของวงจรชีวิตใช้ตรวจสอบ
การบัญชีนั้น แทนที่จะแทนด้วยค่าเฉลี่ยที่ละเลย cost basis และเศษ layout นี้ไม่มีตัวอ่านสำหรับรูปแบบการบัญชีแบบ
floating-point เดิม

## การทำงานร่วมกับ component อื่น [#integration-with-other-components]

Position ทำงานร่วมกับ component หลักหลายตัว:

- **Portfolio**: รวม exposure และ PnL ของ position ข้าม instrument และ strategy
- **ExecutionEngine**: สร้างและอัปเดต position จาก fill
- **Cache**: เก็บสถานะ position ปัจจุบันและ snapshot ของรอบที่ปิดแล้ว
- **RiskEngine**: อ่าน position ที่เปิดอยู่เมื่อตรวจสอบว่า order ลด exposure หรือไม่

:::info
ไม่มีการสร้าง position สำหรับ spread instrument contingent order ยังคง trigger สำหรับ spread ได้ แต่ทำงานโดยไม่ผูกกับ
position engine จัดการ spread instrument แยกจาก position ปกติ
:::

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](events/): fill สร้าง position event อย่างไร
- [Orders](orders/): order ที่สร้างและแก้ไข position
- [Execution](execution/): การจัดการ fill ที่อัปเดต position
- [Portfolio](portfolio.md): การรวม position ในระดับ portfolio
