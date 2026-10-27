---
title: "Execution Algorithms"
---

`ExecutionAlgorithm` รับ primary order ที่ถูกเลือกด้วย `exec_algorithm_id` และสามารถแบ่ง order นั้นออกเป็น
spawned order ย่อย ๆ ได้ NautilusTrader รองรับ algorithm แบบกำหนดเอง และมี TWAP ที่ implement แบบ native
ใน Rust มาให้ หน้านี้อธิบายการตั้งค่า TWAP, การเขียน algorithm และการจัดการ spawned order

สำหรับโมเดลของ component และการ routing ดูที่ [Execution](index.md#execution-flow)

## TWAP (time-weighted average price) [#twap-time-weighted-average-price]

TWAP กระจาย primary order ออกเป็นช่วงเวลาสม่ำเสมอ เพื่อลด market impact จากการส่งปริมาณทั้งหมดในครั้งเดียว
algorithm แบบ native นี้ลงทะเบียนกับ `BacktestEngine` ที่ initialize แล้วได้ในชื่อ `"TwapAlgorithm"`
โดยใช้ `ExecAlgorithmId` เป็น `"TWAP"`

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่างการลงทะเบียนแบบ Python จึงตัดออก

order ที่ route ไปยัง TWAP ต้องมี `exec_algorithm_params` ซึ่งเป็นค่า string ดังนี้:

| Key             | ความหมาย                                                |
| --------------- | ------------------------------------------------------- |
| `horizon_secs`  | ช่วงเวลารวม ใช้ร่วมกับ interval เพื่อกำหนดจำนวน slice      |
| `interval_secs` | ระยะเวลาระหว่างแต่ละ slice                               |

ทั้งสองค่าต้อง parse เป็นตัวเลขบวกได้ และ `horizon_secs` ต้องไม่น้อยกว่า `interval_secs`
algorithm จะส่ง slice แรกทันที และส่ง slice ที่เหลือตาม interval ที่ตั้งไว้

TWAP จะ deny primary order ก่อนการส่ง หากประเภท order, instrument หรือตารางเวลาไม่รองรับหรือไม่ถูกต้อง

## Execution algorithm แบบกำหนดเอง [#custom-execution-algorithms]

execution algorithm แบบกำหนดเองรับ order ผ่าน `on_order(...)` และมีสิทธิ์เข้าถึง cache และ portfolio,
clock สำหรับ timer, signal และเมธอดสำหรับ spawn order

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่างการเขียน algorithm แบบ Python จึงตัดออก

หลังลงทะเบียนแล้ว message bus จะ route order ไปยัง algorithm ที่มี `ExecAlgorithmId` ตรงกับ
`exec_algorithm_id` ของ order ฟิลด์ `exec_algorithm_params` (ไม่บังคับ) เป็น map จาก string ไปยัง string
override `on_order_list(...)` หากต้องการจัดการ list ของ order เป็นหน่วยเดียว ส่วน implementation เริ่มต้นจะส่ง
order แต่ละตัวไปยัง `on_order(...)`

ตรวจสอบว่ามี key ที่จำเป็นใน `exec_algorithm_params` ครบ และ parse ค่า string ให้เรียบร้อยก่อน execute order
หาก execute order ไม่ได้ ให้เรียก `deny_order(...)` พร้อม [reason code](index.md#order-denied-reasons)
มาตรฐาน เช่น `VALIDATION_FAILED: horizon_secs not found in exec_algorithm_params`

order ที่ execution algorithm ได้รับคือ **primary order** ใช้เมธอดต่อไปนี้เพื่อสร้าง **spawned order**:

- `spawn_market(...)`: สร้าง order แบบ `MARKET`
- `spawn_market_to_limit(...)`: สร้าง order แบบ `MARKET_TO_LIMIT`
- `spawn_limit(...)`: สร้าง order แบบ `LIMIT`

ทุกเมธอดรับ primary order เป็น argument แรก โดยค่าเริ่มต้นเมธอดจะลดปริมาณของ primary order ลงเท่ากับ
`quantity` ที่ spawn ออกไป ตั้ง `reduce_primary` เป็น false หากต้องการให้ปริมาณของ primary คงเดิม

:::warning
เมื่อ `reduce_primary` เป็น true ปริมาณที่ spawn ต้องไม่เกิน `leaves_qty` ของ primary order
(ปริมาณที่ยังไม่ถูก fill)
:::

ขณะที่ primary ยังอยู่ในระบบ local ผลลัพธ์ของ spawned order จะส่งผลต่อปริมาณของ primary ดังนี้:

- **ปริมาณที่ไม่ถูก fill**: spawned order อาจถูก deny, reject, cancel, expire หรือถูกปฏิเสธก่อนการส่ง ในกรณีเหล่านี้
  สัดส่วนที่ไม่ถูก fill จะถูกคืนกลับไปยังปริมาณของ primary order ในหน่วยปริมาณของ primary กรณีนี้ใช้กับ
  spawn แบบ quote-quantity ที่ venue แปลงเป็น base quantity ด้วย
- **Late fill**: late fill บน spawn ที่ถูก cancel แล้วจะหักปริมาณที่เคยคืนไปกลับออกอีกครั้ง ตราบใดที่ primary
  ยังแก้ไขได้ในระบบ local หากปริมาณนั้นถูกนำไปใช้โดย spawn ถัดไปแล้ว ส่วนเกินจะถูกหักจากปริมาณที่คืนให้
  spawn นั้นแทน

เมื่อส่งต่อ primary ไปยังขั้นตอนการ submit แล้ว ปริมาณของ primary จะถูก commit และจะไม่เปลี่ยนแปลงจากผลลัพธ์
ของ spawn ที่ตามมาภายหลัง

spawn แบบ quote-quantity ที่ถูกแปลงแล้ว จะคำนวณปริมาณที่ยังไม่ถูก fill จาก fill สะสม และปัดลงตาม
quantity precision ของ primary ยอดหักรวมไม่ขึ้นกับว่ามี fill event กี่ครั้งที่รายงานปริมาณที่ fill

หาก [fill ถูก void](../events/order_fill_voided.md) หลังจากที่ปริมาณที่ไม่ถูก fill ของ spawn ใดถูกคืนไปแล้ว
การแก้ไขนี้จะคืนปริมาณที่ไม่ถูก fill เพิ่มเติมให้ ตราบใดที่ primary ยังอยู่ใน local โดยจะหักกลบกับปริมาณจาก
late fill ที่ยังหักจาก primary ไม่ได้ก่อน

execution algorithm อาจ spawn order ต่อไปเรื่อย ๆ, submit ส่วนที่เหลือของ primary order หรือทำทั้งสองอย่าง
TWAP ในตัวจะ submit ส่วนที่เหลือของ primary order ใน interval สุดท้าย

## Spawned order [#spawned-orders]

spawned order แต่ละตัวจะตั้ง `exec_spawn_id` เป็น `client_order_id` ของ primary order ส่วน `client_order_id`
ของตัวมันเองเป็นไปตามรูปแบบนี้:

```text
{exec_spawn_id}-E{spawn_sequence}
```

เช่น order แรกที่ spawn จาก `O-20230404-001-000` จะมี ID เป็น `O-20230404-001-000-E1`

:::note
คำว่า primary และ spawned ใช้แยกการแบ่ง execution ออกจากความสัมพันธ์แบบ parent และ child
ของ contingent order
:::

## การ query order ของ execution algorithm [#execution-algorithm-order-queries]

`Cache` มี query หลักสองตัว:

- `orders_for_exec_algorithm(...)`: คืน order ของ algorithm หนึ่ง โดยกรองตาม venue,
  instrument, strategy, account และ side ได้ (ไม่บังคับ)
- `orders_for_exec_spawn(...)`: คืน primary order และ spawned order ทั้งหมดของ primary
  `ClientOrderId` หนึ่ง

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Execution](index.md): การ routing ของ component, พฤติกรรม OMS, risk check และผลลัพธ์ของ command
- [Execution policies](policies.md): ขอบเขตของ order state และการส่ง command
- [Orders](../orders/): ประเภทของ order, instruction และการเปลี่ยนสถานะ
