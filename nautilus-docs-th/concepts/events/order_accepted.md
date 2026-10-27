---
title: "OrderAccepted"
---

`OrderAccepted` บันทึกว่า trading venue ยืนยันแล้วว่าได้รับ order และ order นั้นถูกต้อง
(มักตรงกับ OrdStatus `NEW` ของ FIX) `ExecutionEngine` จะนำ event นี้ไปใช้กับ order อัปเดต `Cache`
และ publish ออกไปบน `MessageBus`

การเปลี่ยนสถานะที่พบบ่อย: `SUBMITTED` -> `ACCEPTED` Handler: `on_order_accepted`

## ฟิลด์ [#fields]

นอกเหนือจาก[ฟิลด์ร่วมของ order event](index.md#common-python-order-event-fields)
`OrderAccepted` ยังมีฟิลด์ต่อไปนี้:

| Field            | Required/default | คำอธิบาย                                |
| ---------------- | ---------------- | --------------------------------------- |
| `venue_order_id` | Required         | ตัวระบุ order ที่ venue กำหนดให้           |
| `account_id`     | Required         | account ที่เกี่ยวข้องกับ order             |
| `reconciliation` | Required         | ถูกสร้างขึ้นระหว่าง reconciliation หรือไม่ |

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - ประเภทของ event, การ dispatch และฟิลด์ร่วมของ order event
- [Orders](../orders/) - ประเภทของ order และ state machine
