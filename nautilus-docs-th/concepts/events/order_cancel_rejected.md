---
title: "OrderCancelRejected"
---

`OrderCancelRejected` บันทึกว่า trading venue ปฏิเสธคำสั่ง `CancelOrder` จากนั้น `ExecutionEngine`
จะนำ event นี้ไปใช้กับ order อัปเดต `Cache` และ publish บน `MessageBus`

การเปลี่ยนสถานะโดยทั่วไป: `PENDING_CANCEL` -> สถานะก่อนหน้า (เช่น `ACCEPTED`) Handler:
`on_order_cancel_rejected`

## ฟิลด์ [#fields]

นอกเหนือจาก[ฟิลด์ทั่วไปของ order event](index.md#common-python-order-event-fields) แล้ว
`OrderCancelRejected` ยังมีฟิลด์ต่อไปนี้:

| Field            | Required/default | คำอธิบาย                                         |
| ---------------- | ---------------- | ------------------------------------------------ |
| `reason`         | Required         | เหตุผลที่ปฏิเสธการ cancel order                  |
| `venue_order_id` | `None`           | order identifier ที่ venue กำหนด (ถ้าทราบ)       |
| `account_id`     | `None`           | account ที่ผูกกับ order (ถ้าทราบ)                |
| `reconciliation` | Required         | event นี้ถูกสร้างระหว่าง reconciliation หรือไม่  |

## ตัวอย่าง [#example]

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python (การอ่าน event ใน handler ของ strategy) จึงตัดออก

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - ประเภทของ event, การ dispatch และฟิลด์ทั่วไปของ order event
- [Orders](../orders/) - ประเภทของ order และ state machine
