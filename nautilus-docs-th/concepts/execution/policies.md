---
title: "Execution Policies"
---

NautilusTrader ประสาน state ภายในเครื่องกับ trading venue ข้ามขอบเขตแบบกระจาย (distributed boundary)
หน้านี้กำหนด policy ที่ควบคุม order command, order event, persistence และ reconciliation
รวมถึงพฤติกรรมที่ระบุไว้และข้อจำกัดที่ทราบ ใช้หน้านี้เมื่อต้องตีความ state ของ order,
implement execution adapter หรือออกแบบขั้นตอนการกู้คืนระบบในการเทรดจริง

สำหรับโมเดลของ component และการ routing ดู [Execution](index.md) สำหรับสถานะของ order แต่ละแบบและ
การเปลี่ยนสถานะหลัก ดู [Orders](../orders/index.md#order-state-flow)

## สรุป policy [#policy-summary]

| ขอบเขต                    | พฤติกรรม                                                                                         | ข้อจำกัด                                                                                         |
| ------------------------- | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| Order state               | event ทุกตัวที่ถูกนำไปใช้ต้องผ่าน state machine ของ order และการตรวจสอบ identity                 | สถานะเพียงอย่างเดียวไม่ได้บอกว่าเกิดจากหลักฐานจาก venue หรือจาก reconciliation                    |
| ผลลัพธ์ของ command         | adapter แยกแยะความล้มเหลวภายในที่ชัดเจน, ผลลัพธ์จาก venue ที่ชัดเจน และผลลัพธ์ที่ไม่ทราบ           | ผลลัพธ์ในระดับ transport ไม่ได้พิสูจน์เสมอไปว่า venue ทำอะไร                                    |
| การส่ง command             | ตาม contract จะ retry command ที่เปลี่ยน state เฉพาะเมื่อการส่ง request เดิมซ้ำนั้นปลอดภัย         | ไม่ใช่ทุก adapter ที่ทำตาม contract นี้ และการส่งไม่ได้เป็นแบบ exactly once                       |
| การนำ event ไปใช้          | การตรวจ identity ของ order และการเปลี่ยนสถานะจะปฏิเสธ event ที่ไม่ถูกต้อง fill จะปฏิเสธ `trade_id` ที่ซ้ำ | ไม่มีการรับประกัน exactly-once แบบครอบคลุมทุกประเภท event หรือเมื่อ state ที่เก็บไว้สูญหาย          |
| Persistence ก่อนส่ง         | cache จะ enqueue order และ origin ของ execution client ที่ resolve แล้ว ก่อนเรียก client             | cache backend ในตัวไม่ได้รอให้เขียนลง storage ถาวรเสร็จก่อนที่ client จะส่งได้                     |
| การกู้คืนแบบมีขอบเขต        | reconciliation กู้คืน state ของ order ที่ venue รายงาน โดยไม่นำตัวเลข fill ที่ไม่มีหลักฐานรองรับไปใช้ | ประวัติที่ไม่ครบไม่สามารถพิสูจน์ตัวเลขของ position ในอดีต, realized PnL หรือราคาเฉลี่ยได้          |
| Terminal policy           | reconciliation อาจปิดสถานะ order ที่หายไปหรือ timeout หลังจาก retry ครบตามที่ตั้งค่าไว้             | การปิดสถานะโดย policy ไม่ใช่การ reject หรือ cancel ที่ venue ยืนยัน                               |

## Order state [#order-state]

Order ใช้แนวทาง event sourcing order ทุกตัวเริ่มต้นด้วย `OrderInitialized` และ `OrderCore::apply` จะตรวจสอบ
event กับ identity ของ order และการเปลี่ยนสถานะที่อนุญาตจากสถานะปัจจุบัน มันจะปฏิเสธการเปลี่ยนสถานะ
ที่ไม่ถูกต้องหรือ fill ที่มี `trade_id` ซ้ำก่อนจะแก้ไข order จากนั้นจึงต่อ event ที่ยอมรับแต่ละตัวเข้าไปใน
ประวัติ event ของ order

[Order state flow](../orders/index.md#order-state-flow) แสดง lifecycle หลัก โมเดลนี้ยังรองรับ
กรณีการกู้คืนและกรณีขอบ (edge case) ในโลกจริงบางกรณีด้วย เช่น fill ที่ได้รับระหว่างที่ command
ยัง pending อยู่ และ fill ที่มาช้าสำหรับ order ที่ถูก cancel ไปแล้ว หน้า event แต่ละหน้าภายใต้
[Events](../events/) อธิบาย field และการเปลี่ยนสถานะทั่วไปของ event นั้น

### ลำดับ [#ordering]

ภายใน live node หนึ่งตัว runner จะจัดการ message branch ที่เลือกแต่ละอันจนเสร็จก่อนจะเลือกอันถัดไป
ทำให้การแก้ไข order ฝั่ง kernel เป็นแบบลำดับเดียว (serialized) order จะต่อ event ที่ยอมรับแต่ละตัว
ตามลำดับที่นำไปใช้ และไม่เรียงประวัติใหม่ตาม timestamp ของ event

เมื่อมีหลาย channel พร้อมพร้อมกัน priority ของ runner จะเป็นตัวกำหนดว่าจะจัดการอันไหนก่อน event
จาก task ของ adapter หรือ venue ที่เป็นอิสระต่อกันจึงอาจสลับกันได้ และ timestamp ของ event ไม่ได้กำหนด
ลำดับ FIFO แบบ global ดู [Dispatch priority and overload behavior](../live.md#dispatch-priority-and-overload-behavior)

### การนำ event ไปใช้ซ้ำ [#duplicate-application]

NautilusTrader ไม่ได้ใช้ `event_id` เป็น key สำหรับตัดข้อมูลซ้ำในระดับ order แบบครอบจักรวาล และไม่รับประกัน
การนำ event ไปใช้แบบ exactly-once สำหรับ order event ทุกตัว การป้องกันที่มีแคบกว่านั้น ได้แก่:

- order จะปฏิเสธ fill ที่ซ้ำที่ราคาล่าสุดที่บันทึกไว้ของ `trade_id` นั้น fill ซ้ำที่ราคาใหม่จะปรับ
  ราคาเฉลี่ยของ trade นั้นใหม่โดยไม่เพิ่ม quantity
- execution engine ป้องกันไม่ให้ `trade_id` เดิมเพิ่ม quantity ให้ position เป้าหมายซ้ำอีก
  แต่จะส่งการปรับราคาไปยังราคาเฉลี่ยของ position ที่เปิดอยู่แทน
- การยกเลิก fill (fill void) ใช้ `trade_id` เดิม และจะปฏิเสธการแก้ไขที่ซ้ำ, เก่า, ขัดแย้งกัน
  หรือรวมกันแล้วเกินจริง
- lifecycle event อื่นที่ซ้ำยังต้องผ่านการเปลี่ยนสถานะตามปกติ update บางตัวที่ไม่เปลี่ยน state
  และ request ที่ pending ซ้ำถือเป็น event ที่ถูกต้องและอาจถูกต่อเข้าไปอีกได้

การตรวจสอบเหล่านี้ขึ้นอยู่กับหลักฐานของ order และ position ที่เก็บไว้ใน cache state ที่กู้คืนมาจะยัง
มี trade ID และประวัติ event เดิมให้ใช้หลัง restart หากไม่มีทั้ง state ดังกล่าวและประวัติจาก venue
ที่จำเป็น NautilusTrader ก็ไม่สามารถอนุมานการนำไปใช้แบบ exactly-once จากหลักฐานที่ขาดหายได้

report ที่อธิบาย fill เชิงตรรกะเดียวกันแต่มี trade ID ต่างกันจะถูกถือว่าแยกกัน และต้องผ่านการตรวจ
overfill และ integrity ตามปกติ

[Event store](../event_sourcing.md) ที่เป็นตัวเลือกเสริมมีขอบเขตแยกต่างหาก capture adapter ของมัน
จะตัดการ dispatch ซ้ำของ message identity เดียวกันภายในช่วงเวลาล่าสุดที่จำกัด และการ replay
จะนำแต่ละ sequence entry ที่เก็บไว้ไปใช้หนึ่งครั้ง แต่สิ่งนี้ไม่ได้ทำให้ capture ที่ยังไม่ commit
กลายเป็นข้อมูลถาวร หรือทำให้การส่งจาก venue เป็นแบบ exactly once

## ผลลัพธ์ของ command [#command-outcomes]

execution command จะได้ข้อสรุปตามหลักฐานที่มี:

| หลักฐาน                    | ความหมาย                                                          | ผลลัพธ์                                                                                             |
| ------------------------- | ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| ความล้มเหลวภายในที่ชัดเจน   | การ validate พิสูจน์ได้ว่า command ไม่ได้ถูกส่งออกไป               | deny การ submit หรือ reject การ modify/cancel เมื่อความล้มเหลวนั้นเกิดจาก command นั้นจริง            |
| ผลลัพธ์จาก venue ที่ชัดเจน   | matching engine หรือ venue ยืนยันผลลัพธ์อย่างชัดแจ้ง               | นำ event accepted, updated, canceled หรือ rejected ที่ตรงกันไปใช้                                   |
| ผลลัพธ์ live ที่ไม่ทราบ      | command อาจไปถึง venue แล้ว แต่ยังไม่ทราบผล                        | คง command ไว้ในสถานะ in flight เพื่อรอ stream update, polling, query หรือ reconciliation           |

event ที่บอกความล้มเหลวขึ้นอยู่กับ command และช่วงเวลาที่ความล้มเหลวนั้นชัดเจน:

| Command                             | Event                 | ความหมาย                                                          |
| ----------------------------------- | --------------------- | ----------------------------------------------------------------- |
| Submit หรือ submit order list        | `OrderDenied`         | การตรวจภายในขัดขวางการ submit ไม่มี event `OrderSubmitted`         |
| Submit หรือ submit order list        | `OrderRejected`       | การ submit เข้าสู่ execution แล้ว และภายหลังพิสูจน์ได้ว่าไม่สำเร็จ    |
| Modify                              | `OrderModifyRejected` | พิสูจน์ได้ว่าการแก้ไขที่ร้องขอไม่สำเร็จ                             |
| Cancel, cancel-all หรือ batch cancel | `OrderCancelRejected` | พิสูจน์ได้ว่าการ cancel ที่ร้องขอไม่สำเร็จ                          |

submit command จะถูกประเมินเฉพาะกับ order ที่มีสิทธิ์เท่านั้น order ใน cache จะหมดสิทธิ์เมื่อออกจาก
`INITIALIZED` หรือ `RELEASED` ไปแล้ว และยังหมดสิทธิ์ในช่วงระหว่างที่ dispatch ไปยัง execution client
จนถึงตอนที่ `ExecutionEngine` นำ status event แรกของ client นั้นไปใช้

สำหรับ order ที่หมดสิทธิ์:

- `SubmitOrder` ที่ซ้ำจะถูกข้าม
- `SubmitOrderList` ที่ระบุ order นั้นจะถูก deny โดยไม่ไปถึง client โดย `OrderDenied`
  (`ORDER_LIST_DENIED`) จะใช้กับเฉพาะสมาชิกของ list ที่มีสิทธิ์เท่านั้น การ submit ครั้งก่อน
  ยังคง lifecycle ของมันไว้ และ event จาก client ที่ตามมาจะถูกนำไปใช้ตามปกติ

**บันทึกการ dispatch เป็นข้อมูลเฉพาะใน process**: การ reset จะล้างมัน และจะไม่มีอยู่หลัง restart
หลัง restart order ที่ dispatch แล้วแต่ยังเป็น `INITIALIZED` จะถูกปฏิบัติเหมือน order อื่นที่ยังไม่ถูก route
ไปจนกว่า event จาก client หรือ report การ cancel หรือหมดอายุจาก venue จะปิดสถานะมัน

สำหรับการเตรียม modify หรือ cancel NautilusTrader จะ emit rejection ที่ตรงกันเฉพาะเมื่อความล้มเหลวนั้น
เกิดจาก command นั้นจริงและพิสูจน์ได้ว่าไม่ได้ถูกส่งออกไป มิฉะนั้นจะ log ความล้มเหลวไว้โดยไม่สร้าง
ผลลัพธ์ขึ้นมาเอง

response ของ batch ที่สำเร็จยังอาจมีความล้มเหลวรายออร์เดอร์ที่ชัดเจนอยู่ได้ ความล้มเหลวทั้ง request
ที่ไม่มีหลักฐานรายออร์เดอร์ ไม่ได้พิสูจน์ว่า child command ทั้งหมดล้มเหลว

:::warning[ผลลัพธ์ live ที่ไม่ทราบ]
transport error, timeout, การหลุดการเชื่อมต่อ, การ cancel task, การ retry request ของ adapter จนหมด,
acknowledgement ที่หายไป และการ parse ล้มเหลวหลังจากส่งไปแล้ว โดยทั่วไปจะทำให้ไม่ทราบผลลัพธ์ที่ venue
HTTP status code และ rate limit จะถือว่าชัดเจนก็ต่อเมื่อความหมายเฉพาะของ venue พิสูจน์ได้ว่า
command ไม่ได้ถูกยอมรับ
:::

**In-flight order** คือ order ที่กำลังรอข้อสรุป:

- `SUBMITTED`: การ submit ครั้งแรกที่รอการ accept หรือ reject
- `PENDING_UPDATE`: การแก้ไขที่รอการยืนยัน
- `PENDING_CANCEL`: การ cancel ที่รอการยืนยัน

### ข้อจำกัดของการส่งและการ retry [#delivery-and-retry-limits]

request อาจไปถึง venue แล้วแม้ว่า response จะหายไป NautilusTrader ไม่รับประกันการส่งแบบ
exactly-once สำหรับ command submit, modify หรือ cancel

#### การทดสอบความปลอดภัย [#safety-test]

adapter จะ retry ความล้มเหลวชั่วคราว (transient failure) เฉพาะเมื่อการส่งซ้ำนั้นปลอดภัย การส่งซ้ำจะ
ปลอดภัยเมื่อ:

- ไม่สามารถทำให้การเปลี่ยน state เกิดขึ้นสองครั้ง
- ไม่สามารถเปลี่ยน order ที่ความพยายามครั้งแรกไม่ได้ระบุ
- ไม่สามารถรายงานความสำเร็จก่อนหน้าว่าเป็น rejection

การส่งซ้ำที่เพียงแค่สร้าง session token อายุสั้นเพิ่มอีกหนึ่งตัว โดยไม่ revoke session ที่มีอยู่
หรือสร้าง credential ถาวร ถือว่าปลอดภัย

การส่งซ้ำแบบอื่นจะปลอดภัยได้ก็ต่อเมื่อ protocol ของ venue รองรับ ผ่าน request identity ที่คงที่และ
การตรวจจับข้อมูลซ้ำ หรือผ่านความหมายแบบ idempotent สำหรับเป้าหมายเดียวกัน การรับประกันนี้ต้องคงอยู่
ตลอดช่วง retry ทั้งหมด รวมถึงหลังจากที่ order เป้าหมาย fill หรือถูก cancel ไปแล้ว และจนกว่า
นาฬิกาตัดข้อมูลซ้ำของ venue จะหมดอายุ

| Request                                                                                                              | ผลของการส่งซ้ำ                                   | จำนวนครั้งที่พยายาม                     |
| -------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ | -------------------------------------- |
| Submit หรือ submit order list                                                                                         | อาจวาง live order ตัวที่สอง                       | ครั้งเดียว เว้นแต่ผ่านการทดสอบความปลอดภัย |
| Modify                                                                                                               | อาจแก้ไขซ้ำเป็นครั้งที่สอง                         | ครั้งเดียว เว้นแต่ผ่านการทดสอบความปลอดภัย |
| Cancel หรือ batch cancel โดยระบุ ID ชัดเจน                                                                            | response ครั้งที่สองอาจรายงานว่าเป็น rejection     | ครั้งเดียว เว้นแต่ผ่านการทดสอบความปลอดภัย |
| Cancel-all                                                                                                           | อาจ cancel order ที่ถูกวางระหว่างความพยายามแต่ละครั้ง | ครั้งเดียว เว้นแต่ผ่านการทดสอบความปลอดภัย |
| request อื่นที่เปลี่ยน state                                                                                          | อาจทำให้การเปลี่ยนแปลงนั้นเกิดสองครั้ง              | ครั้งเดียว เว้นแต่ผ่านการทดสอบความปลอดภัย |
| request ขอ session token ที่เพียงสร้าง token อายุสั้นที่ยังไม่ถูกใช้ ไม่ revoke session และไม่สร้าง credential ถาวร       | สร้าง token ที่ยังไม่ถูกใช้เพิ่มอีกตัว               | เมื่อเกิด transient failure             |
| request อ่านข้อมูลหรือ request อื่นที่เป็น idempotent                                                                  | ไม่มีการเปลี่ยน state ที่ venue                    | เมื่อเกิด transient failure             |

เมื่อไม่ผ่านการทดสอบความปลอดภัย adapter จะส่ง request เพียงครั้งเดียว และผลลัพธ์จะยังไม่ทราบ
จนกว่า stream update, query, polling หรือ reconciliation จะให้ข้อสรุปสำหรับ command เชิงความหมายเดียวกัน

:::warning[การปฏิบัติตามยังไม่ครบถ้วน]
transient failure อาจทำให้ order command บางตัวถูกส่งสองครั้ง รวมถึงบน Bybit, OKX และ Kraken spot
รายการนี้ไม่ได้ครบถ้วนทั้งหมด ตาราง
[retry gate conformance](../../developer_guide/adapters.md#retry-gate-conformance) บันทึก
HTTP gate ที่ตรวจสอบแล้ว
:::

#### ผลลัพธ์ [#outcome]

การที่ความล้มเหลวหนึ่งปลอดภัยที่จะ retry หรือไม่นั้นเป็นคนละเรื่องกับผลลัพธ์ที่ venue ความล้มเหลวอาจ
ปลอดภัยที่จะ retry ในขณะที่ความพยายามครั้งก่อนยังกำกวมอยู่ เมื่อความพยายามครั้งหนึ่งอาจไปถึง venue แล้ว
ความล้มเหลวที่ตามมาก็จะยังกำกวม เว้นแต่จะมีหลักฐานที่เชื่อถือได้ให้ข้อสรุปสำหรับ command เชิงความหมายเดียวกัน

## Persistence ก่อน transport [#persistence-before-transport]

การสร้าง client order ID, การ publish `OrderInitialized` และการส่ง `SubmitOrder` เป็นการทำงานภายใน
process สิ่งเหล่านี้ไม่ได้ยืนยันการเขียนลง storage ถาวรด้วยตัวเอง

สำหรับการ submit ที่จัดการโดย execution engine ในตัว:

1. order อยู่ใน cache พร้อม `ClientOrderId` และ event `OrderInitialized` ก่อนการเรียก
   execution client ครั้งสุดท้าย
1. engine เลือกและตรวจสอบ execution client
1. cache enqueue origin ของ order ไปยัง client ที่ resolve แล้วเพื่อ persist จากนั้นจึงอัปเดต
   origin index ในหน่วยความจำ
1. engine เรียก `ExecutionClient` ที่เลือกไว้
1. cache backend ประมวลผลการเขียนที่อยู่ในคิวโดยไม่ขึ้นกับ transport และ acknowledgement ของ venue

ขั้นตอน enqueue จะล้มเหลวก่อนการเรียก client เมื่อ cache backend ไม่สามารถรับได้ การ enqueue
สำเร็จไม่ได้หมายความว่า backing store ได้ commit order หรือ origin แล้ว cache backend Redis และ
PostgreSQL ในตัวจะประมวลผลการเขียนเหล่านี้แบบ asynchronous

ขั้นตอนสุดท้ายเป็นเพียงการเรียกเข้าไปใน `ExecutionClient` ของ adapter เท่านั้น adapter เป็นผู้รับผิดชอบ
การส่งข้อมูลจริงทางเครือข่ายในภายหลัง และแปลง response หรือ stream update จาก venue เป็น order event
ทั้งการเรียก client ที่สำเร็จหรือ event `OrderSubmitted` ต่างก็ไม่ได้พิสูจน์ว่า venue ยอมรับแล้ว

:::warning[ช่องว่างของ persistence]
process อาจล้มเหลวหลังจากที่ venue ได้รับ order แล้ว แต่ก่อนที่ order และ origin ภายในเครื่องจะ
ถูกเขียนลง storage ถาวร startup reconciliation สามารถกู้คืน order นั้นได้เมื่อ venue รายงานมา
แต่ประวัติจาก venue ที่ไม่ครบอาจทำให้ node ไม่มีหลักฐานพอที่จะสร้างประวัติ execution ทั้งหมดขึ้นใหม่
:::

[Event store](../event_sourcing.md) ที่เป็นตัวเลือกเสริมก็ capture แบบ asynchronous เช่นกัน และไม่ได้
รอ commit ถาวรก่อน dispatch message การ restart ในการเทรดจริงยังคงใช้ state ที่กู้คืนจาก cache
ร่วมกับ reconciliation จาก venue

## อำนาจของ reconciliation [#reconciliation-authority]

order event stream ที่อยู่ใน cache เป็นแหล่งที่มาของ state ของ order ที่คำนวณภายในเครื่อง ระหว่างการกู้คืน
ในการเทรดจริง report จาก adapter เป็นหลักฐานจาก venue ที่ใช้ปรับ state นั้นให้ตรงกัน reconciliation
นำ report ไปใช้เป็นเฟส order-status, fill และ position ตามลำดับ เพื่อให้การตรวจ position อยู่บน
state ของ order และ fill ที่ reconcile แล้ว

การกู้คืนหมายถึงการคืน state จาก cache เท่าที่มี, การ reconcile report จาก venue เท่าที่มี และการ
กั้นไม่ให้ strategy เริ่มทำงานจนกว่า startup reconciliation จะเสร็จ มันไม่ได้พิสูจน์ว่า venue ส่ง
ประวัติครบถ้วน หรือว่าผลลัพธ์ของ command ที่ไม่ทราบได้ข้อสรุปครบทั้งหมดแล้ว

:::info[Position report คือ market exposure]
**Position report ที่ระบุชัดเจนถือเป็นข้อมูลที่เชื่อถือได้ ระหว่าง startup reconciliation
engine จะปรับให้ตรงกับ report นั้นภายใน tolerance ของ reconciliation หรือไม่ก็ล้มเหลวแบบปิด (fail closed)**

Position report ที่เชื่อถือได้:

- report ที่ระบุว่าเปิดอยู่อย่างชัดเจน พร้อม quantity และทิศทาง
- report ที่ระบุว่า flat อย่างชัดเจน

สิ่งที่ไม่ใช่หลักฐานว่า position เป็น flat:

- report ที่หายไป
- quantity ที่เป็น null
- venue ที่ไม่เผยแพร่ position

ช่วงเวลาของ fill (fill window) ไม่ได้เป็นตัวตัดสินว่า report เชื่อถือได้หรือไม่ report ที่หายไป
ไม่ได้แปลว่า flat

โดยค่าเริ่มต้น `generate_missing_orders` เปิดใช้งานอยู่ engine จะสร้าง order และ fill ที่จำเป็น
เพื่อปรับ state ภายในให้ตรงกับ report การปิดการสร้างนี้ไม่ได้ทำให้ report ที่ยังไม่ได้ข้อสรุป
ผ่าน startup ไปได้

การรับประกันนี้ครอบคลุม report ที่ผ่าน filter ของ position report และ instrument โดยเปิดใช้
reconciliation อยู่ report ที่ยังไม่ได้ข้อสรุปจะทำให้ actor และ strategy เริ่มทำงานไม่ได้
:::

report ของ order ที่ไม่มีอยู่ใน cache สามารถสร้างเป็น external order ได้ claim ที่ active อยู่จะกำหนด
external order ให้กับ strategy ส่วน order ที่ไม่มีใคร claim จะใช้ strategy `EXTERNAL` ดู
[External order creation](reconciliation.md#external-order-creation)

external order และ fill ยังคงมีส่วนในการติดตาม position และการคำนวณ portfolio เมื่อหลักฐานของมัน
ผ่านกฎ reconciliation เดียวกัน มาตรการป้องกันเรื่องประวัติที่มีขอบเขตจะมีผลไม่ว่า strategy จะ claim
กิจกรรมนั้นหรือไม่

Startup reconciliation รันก่อนที่ component ของ trader จะเริ่มทำงาน หาก startup ล้มเหลว node
จะไม่เริ่มทำงาน เว้นแต่จะมี compatibility path ที่ระบุไว้รองรับเงื่อนไขนั้นโดยเฉพาะ

### ที่มาของ terminal reconciliation [#terminal-reconciliation-provenance]

field `reconciliation` ระบุว่า event นั้นถูกสร้างผ่าน reconciliation แต่ตัวมันเองไม่ได้แยกแยะว่า
มาจาก status report ของ venue หรือจากการตัดสินโดย policy ภายใน การตัดสินโดย policy ภายในด้านล่างนี้
จะใช้เมื่อการเก็บรักษาการ submit (submission retention) ไม่ได้บังคับให้ order ค้างสถานะไว้ status report
จาก venue ที่ระบุชัดเจนยังคง reconcile order ที่ถูกเก็บรักษาไว้ได้:

| เส้นทางของหลักฐาน                                                    | สถานะก่อนหน้า                           | Terminal event                  | ที่มาของ event ที่มีให้                                                                                                               |
| ------------------------------------------------------------------- | -------------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| status report จาก venue ที่ระบุชัดเจน                                 | การเปลี่ยนสถานะใดก็ได้ที่โมเดลอนุญาต       | `OrderRejected`/`OrderCanceled` | `reconciliation=true` rejection เก็บเหตุผลที่รายงานมา หรือ `UNKNOWN` หากไม่มี ส่วน cancellation เก็บเหตุผลที่รายงานมาถ้ามี            |
| retry ของ in-flight หมด                                              | `SUBMITTED`                            | `OrderRejected`                 | `reconciliation=true`, reason `INFLIGHT_TIMEOUT`                                                                                      |
| retry ของ in-flight หมด                                              | `PENDING_UPDATE`/`PENDING_CANCEL`      | `OrderCanceled`                 | `reconciliation=true`, `reason=None`                                                                                                  |
| order ที่มีประวัติเต็มยังหายไปหลัง retry และ query แบบเจาะจง             | `SUBMITTED`/`ACCEPTED`                 | `OrderRejected`                 | `reconciliation=true`, reason `NOT_FOUND_AT_VENUE`                                                                                    |
| order ที่มีประวัติเต็มยังหายไปหลัง retry และ query แบบเจาะจง             | `PARTIALLY_FILLED`                     | `OrderCanceled`                 | `reconciliation=true`, `reason=None`                                                                                                  |

ค่า `submission_recovery_policy` มีค่าเริ่มต้นเป็น `SubmissionRecoveryPolicy::ResolveLocally` เมื่อใช้
`SubmissionRecoveryPolicy::RetainUnresolved` order ที่ไม่เคยถูก accept จะคง state ที่ยังไม่ได้รับ
acknowledgement ไว้หลังจากการกู้คืนหมดความพยายาม รวมถึง cancel หรือ update ที่ pending อยู่
Polymarket client ต้องใช้พฤติกรรมนี้เสมอไม่ว่าจะตั้ง policy ไว้อย่างไร

สำหรับ order ที่ถูกเก็บรักษาไว้เหล่านี้ query การกู้คืนอัตโนมัติราย order จะหยุดลง แต่ event และ
report จาก venue ที่ตามมายังสามารถให้ข้อสรุปกับ order ได้ order ที่ถูก accept แล้วยังคงใช้กฎการตัดสิน
ภายในด้านบน

:::warning[Terminal state ภายในไม่ใช่การยืนยันจาก venue]
แถวแรกมีสถานะจาก venue ที่ระบุชัดเจนรองรับ ส่วนแถวอื่นเป็นการคืน terminal state ภายในหลังจาก
retry policy ที่ผู้ดูแลระบบตั้งไว้หมดอายุ มันไม่ได้พิสูจน์ว่า venue reject การ submit หรือ cancel
working order นั้นจริง
:::

`OrderCanceled.reason` สามารถบรรจุเหตุผลการ cancel ที่ venue รายงานได้ แต่เส้นทาง reconciliation
สังเคราะห์ทั้งสองจะไม่ใส่ค่านี้ เมื่อไม่มีเหตุผล ตัว event เพียงอย่างเดียวไม่สามารถแยกแยะระหว่าง
การ cancel ที่ venue รายงานกับเส้นทางสังเคราะห์เหล่านั้นได้ ผู้ใช้ข้อมูลที่ต้องการแยกแยะเรื่องนี้
ต้องเก็บรักษา input ของ reconciliation และ operational log ที่เกี่ยวข้องไว้ event store ที่เป็น
ตัวเลือกเสริมจะ capture report ดิบจาก venue เมื่อเปิดใช้ แต่ไม่มี field ใดของ `OrderCanceled`
ที่บรรจุเหตุผลจาก policy

ดู [Runtime checks](reconciliation.md#runtime-checks) สำหรับการประสาน query, การป้องกัน order
ล่าสุด และพฤติกรรมเมื่อ order หายไป

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Execution](index.md): บทบาทของ component, การ routing, พฤติกรรม OMS และการตรวจสอบความเสี่ยง
- [Execution algorithms](algorithms.md): TWAP, algorithm ที่เขียนเอง และ order ที่ถูก spawn
- [Orders](../orders/): ประเภท order, สถานะ และ state flow หลัก
- [Events](../events/): field ของ event, การ dispatch และผลจาก order ไปสู่ position
- [Execution reconciliation](reconciliation.md): การกู้คืนตอน startup, การตรวจสอบต่อเนื่อง และ
  invariant ของ reconciliation
- [Live trading](../live.md): lifecycle ของ node, dispatch policy, metric และการ shutdown
- [ตั้งค่า live trading node](../../how_to/configure_live_trading.md): การตั้งค่า live execution
