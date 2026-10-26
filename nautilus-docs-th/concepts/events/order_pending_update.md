# OrderPendingUpdate

`OrderPendingUpdate` บันทึกว่าระบบได้ส่งคำขอ `ModifyOrder` ไปยัง venue แล้ว และกำลังรอการตอบรับ
`ExecutionEngine` จะนำ event นี้ไปใช้กับ order อัปเดต `Cache` และ publish ลงบน `MessageBus`

การเปลี่ยนสถานะโดยทั่วไป: `ACCEPTED` -> `PENDING_UPDATE` handler: `on_order_pending_update`

## ฟิลด์ {#fields}

นอกเหนือจาก[ฟิลด์ทั่วไปของ order event](index.md#common-python-order-event-fields)
แล้ว `OrderPendingUpdate` ยังมีฟิลด์ต่อไปนี้:

| Field            | Required/default | คำอธิบาย                                         |
| ---------------- | ---------------- | ------------------------------------------------ |
| `venue_order_id` | `None`           | ตัวระบุ order ที่ venue กำหนดให้ (ถ้าทราบ)          |
| `account_id`     | Required         | account ที่เกี่ยวข้องกับ order (ถ้าทราบ)            |
| `reconciliation` | Required         | เป็น event ที่สร้างขึ้นระหว่าง reconciliation หรือไม่ |

## ตัวอย่าง {#example}

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Events](index.md) - ประเภทของ event, การ dispatch และฟิลด์ทั่วไปของ order event
- [Orders](../orders/) - ประเภทของ order และ state machine
