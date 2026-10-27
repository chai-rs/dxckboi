---
title: "OrderSubmitted"
---

`OrderSubmitted` บันทึกว่าระบบได้ส่ง order ไปยัง venue แล้วและกำลังรอการตอบรับ (acknowledgement)
`ExecutionEngine` จะนำ event นี้ไปใช้กับ order อัปเดต `Cache` และ publish ออกไปบน `MessageBus`

การเปลี่ยนสถานะโดยทั่วไป: `INITIALIZED` / `RELEASED` -> `SUBMITTED` Handler: `on_order_submitted`

## ฟิลด์ [#fields]

นอกเหนือจาก[ฟิลด์ร่วมของ order event](index.md#common-python-order-event-fields)แล้ว
`OrderSubmitted` ยังมีฟิลด์ต่อไปนี้:

| ฟิลด์        | จำเป็น/ค่าเริ่มต้น | คำอธิบาย                              |
| ------------ | ---------------- | ------------------------------------- |
| `account_id` | จำเป็น            | account ที่เกี่ยวข้องกับ order นี้       |

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และฟิลด์ร่วมของ order event
- [Orders](../orders/) - ประเภทของ order และ state machine
