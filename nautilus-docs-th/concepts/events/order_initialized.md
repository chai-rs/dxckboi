---
title: "OrderInitialized"
---

`OrderInitialized` คือ event ตั้งต้น (seed event) ของ order ใหม่ ซึ่งมีข้อมูลเพียงพอสำหรับส่ง order
ออกไปทางเครือข่าย และสร้าง order ขึ้นมาใหม่ได้ด้วยคุณสมบัติเดิมทุกประการ execution pipeline จะเก็บ
order ไว้ใน `Cache` และ publish event นี้บน `MessageBus`

event นี้ใช้เป็นจุดตั้งต้นทั้งของ order ที่สร้างขึ้นภายในระบบ และ order ภายนอกที่ถูกสร้างขึ้นระหว่าง
reconciliation
Handler: `on_order_initialized`

## ฟิลด์ [#fields]

นอกเหนือจาก[ฟิลด์ร่วมของ order event](index.md#common-python-order-event-fields)แล้ว
`OrderInitialized` ยังมีฟิลด์ต่อไปนี้:

| Field                   | Required/default | คำอธิบาย                                             |
| ----------------------- | ---------------- | --------------------------------------------------- |
| `order_side`            | Required         | ฝั่งของ order (side)                                 |
| `order_type`            | Required         | ประเภทของ order                                     |
| `quantity`              | Required         | จำนวนของ order                                      |
| `time_in_force`         | Required         | time in force ของ order                             |
| `post_only`             | Required         | order นี้ให้ liquidity เท่านั้นหรือไม่                    |
| `reduce_only`           | Required         | order นี้มีคำสั่ง reduce-only หรือไม่                   |
| `quote_quantity`        | Required         | จำนวนระบุเป็นหน่วยของ quote currency หรือไม่            |
| `reconciliation`        | Required         | event นี้ถูกสร้างระหว่าง reconciliation หรือไม่           |
| `price`                 | `None`           | ราคา limit                                          |
| `activation_price`      | `None`           | ราคา activation ของ trailing-stop order               |
| `trigger_price`         | `None`           | ราคา trigger ของ stop                                |
| `trigger_type`          | `None`           | ประเภทของ trigger                                    |
| `limit_offset`          | `None`           | ระยะ trailing offset ของราคา limit                    |
| `trailing_offset`       | `None`           | ระยะ trailing offset ของราคา trigger                  |
| `trailing_offset_type`  | `None`           | ประเภทของ trailing offset                            |
| `expire_time`           | `None`           | UNIX timestamp ที่หมดอายุ หน่วย nanosecond             |
| `display_qty`           | `None`           | จำนวนที่แสดงบน order book สาธารณะ                      |
| `emulation_trigger`     | `None`           | trigger จากราคาตลาดสำหรับการ emulate ภายในระบบ           |
| `trigger_instrument_id` | `None`           | instrument ที่ใช้เป็นแหล่ง trigger ของการ emulate        |
| `contingency_type`      | `None`           | ประเภท contingency ของ order                         |
| `order_list_id`         | `None`           | ID ของ order list ที่เกี่ยวข้อง                         |
| `linked_order_ids`      | `None`           | client order ID ที่เชื่อมโยงกัน                         |
| `parent_order_id`       | `None`           | client order ID ของ order แม่                         |
| `exec_algorithm_id`     | `None`           | ID ของ execution algorithm                          |
| `exec_algorithm_params` | `None`           | พารามิเตอร์ของ execution algorithm                    |
| `exec_spawn_id`         | `None`           | client order ID ของ primary order ที่ให้กำเนิด order นี้ |
| `tags`                  | `None`           | tag ที่ผู้ใช้กำหนดเอง                                   |

## ตัวอย่าง [#example]

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และฟิลด์ร่วมของ order event
- [Orders](../orders/) - ประเภทของ order และ state machine
