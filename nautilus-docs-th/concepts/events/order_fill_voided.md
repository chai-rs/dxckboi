---
title: "OrderFillVoided"
---

`OrderFillVoided` บันทึกว่า fill ที่เคยรายงานไปแล้วทั้งหมดหรือบางส่วนไม่มีผลทางเศรษฐกิจอีกต่อไป
`ExecutionEngine` จะนำ correction แต่ละรายการที่สำเร็จไปใช้กับ order

เมื่อ fill ที่อ้างถึงมีอยู่ในฝั่ง local แล้ว engine จะสร้าง position ที่ได้รับผลกระทบแต่ละตัวขึ้นใหม่ และ
refresh cache ของ position และ PnL ใน portfolio ก่อน publish correction ส่วน venue adapter จะ refresh
ยอดคงเหลือของ account จาก endpoint ของ account ที่เป็นแหล่งข้อมูลจริง (authoritative)

strategy จะได้รับ `OrderFillVoided` หลังจากสถานะ cache ที่แก้ไขแล้วพร้อมใช้งาน เมื่อ position ใน cache
เปลี่ยนแปลง engine จะ publish `PositionChanged` ถ้า position ยังเปิดอยู่ หรือ `PositionClosed` ถ้าปิดแล้ว
correction ที่สำเร็จแต่มีผลกับ order เท่านั้นจะไม่สร้าง position event

**correction** ไม่ใช่ fill ฝั่งตรงข้าม แต่จะคงตัวตนของ trade เดิมไว้ เพื่อให้ replay, reconciliation
และประวัติการ audit ของ strategy อธิบายสิ่งที่ venue ทำได้ตรงตัว

Handler: `on_order_fill_voided`

## ข้อกำหนด [#contract]

`voided_qty` และ `commission_voided` เป็นค่าสะสมของ `trade_id` ที่อ้างถึง correction ด้าน quantity
จะลดลงไม่ได้ สำหรับ fill ที่นำไปใช้ใน local แล้ว correction ด้านค่าธรรมเนียมก็ลดลงไม่ได้เช่นกัน
revision ที่ตามมาอาจเพิ่มค่าใดค่าหนึ่ง หรือเปลี่ยน `is_reopened` ที่ quantity เดิมก็ได้ correction ที่ซ้ำ,
เก่า (stale) หรือ void เกินจำนวนจะถูกปฏิเสธ

การที่ `OrderFilled` ที่อ้างถึงมีอยู่ในประวัติ order ฝั่ง local แล้วหรือไม่ เป็นตัวกำหนดว่า Nautilus
จะตีความ correction อย่างไร:

| Fill อยู่ใน local | `is_reopened` | ผลลัพธ์                                                               |
| ---------------- | ------------- | --------------------------------------------------------------------- |
| Yes              | `false`       | นำไปใช้ quantity ที่แก้ไขจะไม่กลับมาเป็น working                         |
| Yes              | `true`        | นำไปใช้ quantity ที่แก้ไขกลับมาเป็น working ภายใต้กฎของสถานะ terminal      |
| No               | `false`       | นำไปใช้ order ทั้งหมดกลายเป็น terminal โดยมี leaves เป็นศูนย์              |
| No               | `true`        | ปฏิเสธ                                                                |

correction แบบไม่ reopen ที่ไม่ได้นำไปใช้กับ fill ใน local ถือเป็น **การยืนยันว่า order อยู่ในสถานะ terminal**
(order-level terminal assertion) ซึ่งยังเป็นจริงแม้ `voided_qty` จะน้อยกว่า quantity ของ order
เพราะค่านี้บันทึก quantity ของ fill ที่ไม่มีผล ไม่ใช่ leaves ที่ยัง working อยู่

สำหรับ correction แบบไม่ reopen ที่ไม่ได้นำไปใช้กับ fill ใน local:

- event ต้องตรงกับตัวตนของ order
- `voided_qty` ต้องไม่เกิน quantity ของ order
- event ต้องไม่ void commission ที่ไม่เป็นศูนย์
- Nautilus จะไม่ย้อน exposure ของ position หรือ account เมื่อไม่มี fill ใน local

### ข้อกำหนดสำหรับ adapter [#adapter-requirements]

- publish และ persist `OrderFilled` ที่อ้างถึงก่อน correction แบบ reopen หรือ correction บางส่วนใดๆ
  ที่ควรทำให้ order ยัง execute ได้ การ replay บังคับลำดับเดียวกับการประมวลผลแบบ live
- ส่ง correction โดยไม่มี fill ที่อ้างถึงได้ก็ต่อเมื่อ order ทั้งหมดอยู่ในสถานะ terminal อย่างเป็นทางการแล้วเท่านั้น
- อย่าพึ่ง `OrderStatusReport` แบบ working ที่ตามมาทีหลังเพื่อแก้ลำดับของ event continuous
  reconciliation จะไม่สนใจการลดลงของ fill ใน report แบบ working ที่ไม่มีหลักฐานการ void อย่างชัดเจน,
  `VOIDED` จะไม่ reopen และ snapshot reconciliation จะสร้าง correction จาก fill ที่เก็บไว้เท่านั้น
- รายงาน `OrderUpdated.quantity` แบบ gross (ดู [OrderUpdated](order_updated.md#contract)) คือรวม quantity
  ที่ fill แล้วและ quantity ที่ void แบบไม่ reopen Nautilus หัก quantity ที่ void แบบไม่ reopen ออกจาก leaves
  ให้อยู่แล้ว adapter ที่หักออกจาก `quantity` อีกจะเป็นการหักซ้ำสองครั้ง ข้อยกเว้นคือ update จาก
  reconciliation ที่ปิด order เป็น `FILLED` ซึ่ง `quantity` คือ `filled_qty` ที่มีผลจริง

### พฤติกรรมของสถานะเมื่อมี fill ใน local [#status-behavior-with-a-local-fill]

โดยค่าเริ่มต้น quantity ที่แก้ไขจะไม่กลับมา execute ได้:

- order ที่ fill แล้วจะกลายเป็น `VOIDED` ซึ่งเป็นสถานะ terminal แม้จะยังมี filled quantity ที่มีผลเหลืออยู่บางส่วน
- order ที่ partially filled จะคงส่วนที่เหลือที่ working อยู่แล้วไว้ สถานะของ order คำนวณจาก fill ที่มีผลซึ่งยังเหลืออยู่
  และ leaves จะไม่รวม quantity ที่ void แบบไม่ reopen การไม่รวมนี้คงอยู่ต่อเนื่องผ่าน event
  [`OrderUpdated`](order_updated.md) ที่ตามมา Nautilus คำนวณ leaves เป็น
  `max(quantity - filled_qty - non_reopened_voided_qty, 0)` เสมอ ดังนั้นการเปลี่ยน quantity ภายหลัง
  จะไม่ทำให้ส่วนที่ void กลับเข้าไปใน leaves
- order ที่ถูก cancel หรือหมดอายุแล้วจะคงสถานะ terminal ไว้
- correction ที่มี `is_reopened=true` จะคืน quantity ที่แก้ไขกลับเข้า working leaves ด้วย โดย order
  จะเป็น `ACCEPTED` เมื่อ quantity ของ fill ที่มีผลเป็นศูนย์ หรือ `PARTIALLY_FILLED` เมื่อยังมี quantity เหลืออยู่

`VOIDED` เป็นสถานะ terminal ไม่ว่าจะมาจาก correction เส้นทางใด fill, cancel, update, correction
และ status report แบบ working ที่ตามมาภายหลังจะไม่ทำให้มัน reopen

:::warning[อัปเกรดฝั่งผู้อ่านข้อมูลก่อนอ่านข้อมูลที่ผ่าน correction]
schema จะเพิ่ม event และสถานะนี้ต่อท้ายโดยไม่เปลี่ยน record เดิม reader รุ่น v2 ที่เก่ากว่าจะไม่รู้จักค่าใหม่เหล่านี้
จึงต้องอัปเกรดฝั่งผู้อ่านข้อมูล (consumer) ก่อนที่จะอ่าน stream หรือข้อมูลใน catalog ที่ผ่าน correction
:::

## ฟิลด์ [#fields]

นอกเหนือจาก[ฟิลด์ร่วมของ order event](index.md#common-python-order-event-fields)
`OrderFillVoided` ยังมีฟิลด์ต่อไปนี้:

| Field               | Required/default | คำอธิบาย                                                  |
| ------------------- | ---------------- | --------------------------------------------------------- |
| `venue_order_id`    | Required         | ตัวระบุ order ที่ venue กำหนดให้                             |
| `account_id`        | Required         | account ที่เกี่ยวข้องกับ fill เดิม                            |
| `correction_id`     | Required         | ตัวตนของ correction revision นี้                            |
| `trade_id`          | Required         | trade ID เดิมจาก venue                                     |
| `voided_qty`        | Required         | quantity ที่ไม่มีผลแบบสะสมของ trade นั้น                       |
| `commission_voided` | `None`           | การแก้ไขค่าธรรมเนียมแบบสะสมของ trade นั้น                     |
| `order_side`        | Required         | side ของ fill เดิม                                         |
| `order_type`        | Required         | ประเภทของ order เดิม                                       |
| `last_px`           | Required         | ราคาของ fill เดิม                                          |
| `currency`          | Required         | สกุลเงินของราคา fill เดิม                                   |
| `liquidity_side`    | Required         | liquidity side ของ fill เดิม                               |
| `position_id`       | `None`           | position ID ที่เกี่ยวข้องกับ fill เดิม                        |
| `reason`            | `None`           | เหตุผลของ correction จาก venue หรือ reconciliation           |
| `info`              | `None`           | metadata เพิ่มเติมของ correction จาก venue                   |
| `is_reopened`       | `False`          | venue ยืนยันได้หรือไม่ว่า order กลับมา execute ได้อีก             |
| `reconciliation`    | Required         | ถูกสร้างขึ้นระหว่าง reconciliation หรือไม่                     |

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Execution](../execution/) - การนำ correction ไปใช้และลำดับการ publish
- [OrderFilled](order_filled.md) - event ของ fill เดิม
- [Orders](../orders/) - สถานะของ order และลำดับการเปลี่ยนสถานะ
