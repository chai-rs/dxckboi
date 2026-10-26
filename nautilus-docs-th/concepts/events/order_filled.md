# OrderFilled

`OrderFilled` บันทึกการ execute บางส่วนหรือทั้งหมดของ order `ExecutionEngine` จะนำ event นี้ไปใช้กับ order
อัปเดต `Cache` และ publish ลงบน `MessageBus` fill ที่มาจาก live execution, reconciliation และ simulated
matching เป็นตัวขับเคลื่อน event ในวงจรชีวิตของ position

การเปลี่ยนสถานะทั่วไป: `ACCEPTED` -> `FILLED` / `PARTIALLY_FILLED` handler: `on_order_filled`

fill ที่มี `trade_id` ซึ่งมีอยู่แล้วใน order แต่มาพร้อม `last_px` ใหม่ ถือเป็น **price restatement** (การแก้ไขราคา)
คือเป็นการแก้ราคาทางบัญชีของ trade นั้น โดยไม่มีผลต่อ quantity ไม่เปลี่ยนสถานะ และไม่กระทบตัวเลขทางเศรษฐศาสตร์ของ
portfolio

handler ที่สะสม `last_qty` ข้ามหลาย fill event ต้องข้าม restatement เหล่านี้ไป ตรวจจับได้โดยดูใน `order.events`
ว่ามี `OrderFilled` ก่อนหน้าที่มี `trade_id` เดียวกันแต่ `event_id` ต่างกันหรือไม่

## ฟิลด์ {#fields}

นอกเหนือจาก[ฟิลด์ทั่วไปของ order event](index.md#common-python-order-event-fields) แล้ว
`OrderFilled` ยังมีฟิลด์ต่อไปนี้:

| ฟิลด์             | บังคับ/ค่าเริ่มต้น | คำอธิบาย                                                                 |
| ---------------- | ---------------- | ------------------------------------------------------------------------ |
| `venue_order_id` | Required         | ตัวระบุ order ที่ venue กำหนด                                              |
| `account_id`     | Required         | account ที่เกี่ยวข้องกับ fill                                               |
| `trade_id`       | Required         | trade match ID ที่ venue กำหนด                                             |
| `position_id`    | `None`           | position ID ที่เกี่ยวข้องกับ fill                                            |
| `order_side`     | Required         | ฝั่งของ order ที่ execute                                                   |
| `order_type`     | Required         | ประเภทของ order ที่ execute                                                 |
| `last_qty`       | Required         | quantity ของ fill ในการ execute ครั้งนี้                                       |
| `last_px`        | Required         | ราคา fill ของการ execute ครั้งนี้ ไม่ใช่ราคาเฉลี่ย                                 |
| `currency`       | Required         | สกุลเงินของราคา fill                                                       |
| `commission`     | `None`           | ค่า commission ของ fill หากมีการรายงาน                                       |
| `liquidity_side` | Required         | liquidity side ของการ execute (`MAKER`, `TAKER` หรือ `NO_LIQUIDITY_SIDE`) |
| `info`           | `None`           | metadata เพิ่มเติมของ fill ที่เฉพาะกับ venue หรือ adapter                       |
| `reconciliation` | Required         | event นี้ถูกสร้างขึ้นระหว่าง reconciliation หรือไม่                              |

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Events](index.md) - ประเภทของ event, การ dispatch และฟิลด์ทั่วไปของ order event
- [Positions](../positions.md) - position ที่ถูกสร้างและแก้ไขจาก fill
- [Orders](../orders/) - ประเภทของ order และ state machine
