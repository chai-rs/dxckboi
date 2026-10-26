# OrderPendingCancel

`OrderPendingCancel` บันทึกว่าระบบได้ส่งคำขอ `CancelOrder` ไปยัง venue แล้วและกำลังรอการตอบรับ
`ExecutionEngine` จะนำ event นี้ไปปรับใช้กับ order, อัปเดต `Cache` และ publish บน `MessageBus`

การเปลี่ยนสถานะโดยทั่วไป: `ACCEPTED` -> `PENDING_CANCEL` Handler: `on_order_pending_cancel`

## ฟิลด์ {#fields}

นอกเหนือจาก[ฟิลด์ร่วมของ order event](index.md#common-python-order-event-fields)แล้ว
`OrderPendingCancel` ยังมีฟิลด์ต่อไปนี้:

| Field            | Required/default | คำอธิบาย                                       |
| ---------------- | ---------------- | --------------------------------------------- |
| `venue_order_id` | `None`           | identifier ของ order ที่ venue กำหนด (ถ้าทราบ)    |
| `account_id`     | Required         | account ที่เกี่ยวข้องกับ order (ถ้าทราบ)           |
| `reconciliation` | Required         | ถูกสร้างระหว่าง reconciliation หรือไม่             |

## ตัวอย่าง {#example}

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และฟิลด์ร่วมของ order event
- [Orders](../orders/) - ประเภทของ order และ state machine
