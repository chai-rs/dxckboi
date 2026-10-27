---
title: "Emulated orders"
---

Emulation ช่วยให้ใช้ order type บางประเภทได้ แม้ว่า trading venue จะไม่รองรับ order type นั้นโดยตรง

`OrderEmulator` จะเฝ้าดู market data ที่เลือกไว้ด้วย `emulation_trigger` เมื่อ order ภายในเครื่องตรงกับ
เงื่อนไขการ release แล้ว emulator จะแปลงมันเป็น order แบบ `MARKET` หรือ `LIMIT` แล้ว route order นั้นไปยัง
execution algorithm ที่ตั้งค่าไว้หรือ risk engine ตัวอย่างเช่น `STOP_LIMIT` ที่ถูก emulate จะกลายเป็น
order `LIMIT` หลังจากที่ราคา stop ถูก trigger

## การ submit order เพื่อ emulate [#submitting-an-order-for-emulation]

**Trigger ที่รองรับ**: กำหนด `emulation_trigger` ใน constructor ของ order หรือ method ของ `OrderFactory`
emulator ภายในเครื่องรับค่าต่อไปนี้

| Trigger type | Market data ที่ใช้                                    |
| ------------ | ----------------------------------------------------- |
| `DEFAULT`    | quote โดยมีพฤติกรรมภายในเครื่องเหมือนกับ `BID_ASK`   |
| `BID_ASK`    | quote bid และ ask ที่ดีที่สุด                          |
| `LAST_PRICE` | trade                                                 |

ปล่อย `emulation_trigger` เป็น `None` เพื่อปิดการ emulate ภายในเครื่องและ submit ผ่านเส้นทางปกติ

ค่า `TriggerType` อื่น ๆ อธิบายวิธีการ trigger ที่บาง venue รองรับ แต่ `OrderEmulator` ภายในเครื่อง
ไม่รับค่าเหล่านั้นเป็น `emulation_trigger`

:::warning
emulator จะ cancel order ที่ submit มาด้วยค่า `emulation_trigger` อื่นนอกเหนือจากนี้ และ log
trigger type ที่ไม่รองรับเป็น error
:::

**พฤติกรรมของ trigger**: การ emulate ขึ้นอยู่กับ trigger type ที่เลือก

- สำหรับ stop order emulator จะเปรียบเทียบราคา trigger กับ market data ที่เลือก
- สำหรับ trailing-stop order จะอัปเดต trailing trigger จาก market data นั้น
- สำหรับ order `LIMIT` ที่ถูก emulate จะเปรียบเทียบราคา limit กับ market data นั้น และ release
  เป็น order `MARKET` เมื่อตรงเงื่อนไข

**ข้อมูลใน cache**: emulator สร้าง matching core สำหรับ trigger instrument แต่ละตัว
`emulation_trigger` ของ order แรกจะเป็นตัวเลือกว่าจะใช้ market data ใดใน cache มาตั้งค่าเริ่มต้นให้ core นั้น

- `DEFAULT` และ `BID_ASK` ใช้ bid และ ask จาก quote ล่าสุดใน cache
- `LAST_PRICE` ใช้ราคา trade ล่าสุดใน cache

order ที่มีสิทธิ์และตรงกับข้อมูลนี้จะถูก release ทันทีตอน submit **โดยไม่มี event `OrderEmulated`**
หากแหล่งข้อมูลที่เลือกยังไม่มีข้อมูลใน cache order จะยังคงถูก emulate จนกว่าจะมี market data ที่ตรงกันเข้ามา

## รายละเอียดทางเทคนิค [#technical-details]

component `OrderEmulator` ตัวเดียวกันจัดการ order type ที่รองรับการ emulate ในทุก
[environment context](../architecture.md#environment-contexts)

:::note
NautilusTrader ไม่ได้กำหนดจำนวนสูงสุดของ emulated order ไว้ตายตัว หน่วยความจำที่มีและต้นทุนของ
การประมวลผล market data เป็นตัวจำกัดในทางปฏิบัติ
:::

## Lifecycle [#lifecycle]

**Lifecycle**: emulated order จะผ่านขั้นตอนต่อไปนี้

1. `Strategy` submit order ผ่าน `submit_order` ซึ่ง route order ไปยัง `OrderEmulator`
1. `OrderEmulator` ตรวจว่าจะ release ทันทีได้หรือไม่ หาก order ไม่ตรงเงื่อนไข จะถือ order ไว้และ
   เฝ้าดูภายในเครื่อง
1. สำหรับ order ที่ถูกถือไว้ market update ที่ตรงเงื่อนไขจะแปลง order เป็น `MARKET` หรือ `LIMIT` และ
   release เมื่อมีราคาที่จำเป็นครบ
1. emulator จะ route order ที่ release แล้วไปยัง `ExecutionAlgorithm` เมื่อมีการตั้ง `exec_algorithm_id`
   ไว้ หรือไปยัง `RiskEngine` ในกรณีอื่น

:::warning[การตรวจสอบความเสี่ยงมีผลตอน release]
การ submit ครั้งแรกและการแก้ไข emulated order ที่ถูกถือไว้จะข้าม `RiskEngine` order ที่ถูกถือไว้
ไม่ได้กันเงินใน account ไว้ การ release ที่ไม่มี execution algorithm จะผ่าน `RiskEngine` ก่อนถึง
`ExecutionEngine` method `submit_order` ของ execution algorithm ก็ route การ submit ผ่าน risk engine
เช่นกัน

หากไม่ได้ข้ามการตรวจสอบความเสี่ยง การตรวจที่ล้มเหลวสำหรับการ release ที่ไม่มี execution algorithm
จะทำให้ order นั้นเป็น `DENIED` โดยไม่มีการ retry อัตโนมัติ รวมถึง protective stop ด้วย: `HALTED`
จะ deny order เหล่านี้ ส่วน `REDUCING` จะอนุญาตเฉพาะการ submit แบบ reduce-only ที่มีสิทธิ์และระบุ
position ที่เปิดอยู่ได้ ดู [Trading state](../execution/index.md#trading-state)

การ deny ยังเรียกการจัดการ contingency ด้วย ซึ่งอาจ cancel order ลูกแบบ OTO หรือ order พี่น้องแบบ
OCO/OUO ที่จัดการภายในเครื่อง stop ที่ถูก deny จึงอาจ cancel take-profit order ที่ผูกกันอยู่ได้
:::

:::note
strategy สามารถแก้ไขหรือ cancel emulated order ได้ และ request แบบ cancel-all ก็รวม order เหล่านี้ด้วย
:::

:::info
emulated order จะคง client order ID เดิมไว้เมื่อถูกแปลง ดังนั้น query ใน cache จึงใช้ ID เดิมต่อไปได้
:::

### Emulated order ที่ถูกถือไว้ [#held-emulated-orders]

**ระหว่างถือไว้**: `OrderEmulator` จะทำสิ่งต่อไปนี้ขณะที่ถือ order ไว้

- cache command `SubmitOrder` ต้นฉบับไว้
- ประมวลผล order ใน matching core ภายในเครื่อง
- subscribe quote หรือ trade ที่จำเป็น หากยังไม่มี subscription ที่ตรงกัน
- รับการแก้ไขจาก strategy และการอัปเดตที่ขับเคลื่อนด้วยตลาดจนกว่าจะ release หรือ cancel

### Emulated order ที่ release แล้ว [#released-emulated-orders]

**ลำดับการ release**: เมื่อ market data ตรงกับ emulated order การ release จะทำสิ่งต่อไปนี้ตามลำดับ

1. แปลง order เป็น `MARKET` หรือ `LIMIT` ผ่าน event `OrderInitialized` อีกตัว ซึ่งตั้ง
   `emulation_trigger` เป็น `None` ทำให้ component ต่าง ๆ ไม่ถือว่า order นี้ถูก emulate อีกต่อไป
1. อัปเดต command `SubmitOrder` ต้นฉบับให้อธิบาย order ที่ถูกแปลงแล้ว
1. นำ `OrderReleased` ไปใช้และ publish ก่อนส่ง command
1. ส่ง command นั้นไปยัง `ExecutionAlgorithm` ที่ตั้งค่าไว้ หรือไปยัง `RiskEngine` ในกรณีอื่น
   การ submit ที่ผ่านการตรวจความเสี่ยงจะไปถึง `ExecutionEngine` ซึ่งจะ route order ที่มีสิทธิ์ไปยัง
   `ExecutionClient`

## Order type ที่ emulate ได้ [#order-types-that-can-be-emulated]

**Type หลัง release**: type ของ order หลัง release ขึ้นอยู่กับ type ของ emulated order ต้นฉบับ

| Order type ที่จะ emulate  | Emulate ได้ | Type หลัง release |
| ------------------------ | ----------- | ----------------- |
| `MARKET`                 | -           | N/A               |
| `MARKET_TO_LIMIT`        | -           | N/A               |
| `LIMIT`                  | ✓           | `MARKET`          |
| `STOP_MARKET`            | ✓           | `MARKET`          |
| `STOP_LIMIT`             | ✓           | `LIMIT`           |
| `MARKET_IF_TOUCHED`      | ✓           | `MARKET`          |
| `LIMIT_IF_TOUCHED`       | ✓           | `LIMIT`           |
| `TRAILING_STOP_MARKET`   | ✓           | `MARKET`          |
| `TRAILING_STOP_LIMIT`    | ✓           | `LIMIT`           |

การ release เป็น `MARKET` จะคง time in force เดิมไว้ ยกเว้น `GTD` จะกลายเป็น `GTC` เพราะ market order
ไม่รองรับ `GTD`

:::warning
`TRAILING_STOP_LIMIT` ที่ถูก emulate และ submit โดยไม่มีราคา limit จะได้ราคา limit จากการคำนวณ trailing
หาก order ถูก trigger ก่อนที่การคำนวณนั้นจะสำเร็จ emulator จะ log warning และคง order ไว้ในสถานะ
emulate เมื่อ market update ทำให้การคำนวณได้ราคา limit แล้ว update ที่ตรงเงื่อนไขครั้งถัดไปจะ release order

ตัวอย่างเช่น trigger type `LAST_OR_BID_ASK` ต้องใช้ราคา trade ล่าสุด แต่ emulation trigger แบบ `BID_ASK`
ทำให้ emulator subscribe เฉพาะ quote ดังนั้น order จะยังถูกถือไว้จนกว่าข้อมูล trade จะเข้ามาใน cache
(เช่น จาก trade subscription ของ strategy)
:::

## การ query [#querying]

ใช้ cache หรือตัว order object เพื่อ query สถานะการ emulate

### ผ่าน cache [#through-the-cache]

**Method ของ cache**: `Cache` มี method ต่อไปนี้

- `orders_emulated(...)` คืน emulated order ทั้งหมดที่ตรงกับ filter
- `is_order_emulated(...)` ตรวจสอบ client order ID หนึ่งตัว
- `orders_emulated_count(...)` คืนจำนวน emulated order ที่ตรงกับ filter

ดูรายละเอียดเพิ่มเติมได้ที่ [Rust API reference](https://nautilustrader.io/docs/core-latest/)

### Query จาก order โดยตรง [#direct-order-queries]

ใช้ `is_emulated` ของ order object เพื่อ query โดยตรง ค่า false หมายความว่า order ถูก release แล้ว
หรือไม่เคยถูก emulate เลย

:::warning
อย่าถือ reference ภายในไปยัง emulated order เพราะ order object จะถูกแปลงเมื่อ emulated order
ถูก *release* ให้ใช้ `Cache` แทน
:::

## Persistence และการกู้คืน [#persistence-and-recovery]

ตอน startup `OrderEmulator` จะเปิดใช้งาน emulated order ที่ cache database ที่ตั้งค่าไว้กู้คืนเข้ามาใน
cache อีกครั้ง ทำให้ state ของ order เหล่านี้คงอยู่ข้ามการ restart

## แนวปฏิบัติที่ดี [#best-practices]

**แนวปฏิบัติที่ดี**: เมื่อทำงานกับ emulated order ให้ทำตามกฎต่อไปนี้

- query จาก `Cache` แทนการเก็บ reference ของ order ไว้เอง
- เผื่อกรณีที่ order type เปลี่ยนไปตอน release
- จัดการ denial และ rejection จากเส้นทาง execution ของ order ที่ release แล้ว

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Orders](index.md) - แนวคิดของ order, execution instruction และ order factory
- [Advanced orders](advanced.md) - order list, ประเภท contingency และ bracket order
- [Strategies](../strategies.md) - การจัดการ order จาก strategy
