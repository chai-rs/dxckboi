---
title: "OrderExpired"
---

`OrderExpired` บันทึกว่า order หมดอายุแล้ว execution pipeline จะนำ event นี้ไปใช้กับ order อัปเดต `Cache`
และ publish ออกไปบน `MessageBus` event นี้อาจมาจาก trading venue, simulated matching engine
หรือ reconciliation ก็ได้ เช่น เมื่อ order แบบ GTD ถึงเวลาหมดอายุ

การเปลี่ยนสถานะที่พบบ่อย: `ACCEPTED` -> `EXPIRED` Handler: `on_order_expired`

fill ที่ได้รับหลังหมดอายุแล้วยังคงอัปเดต order และ position:

- partial fill จะทำให้ order ยังคงเป็น `EXPIRED`
- fill ที่ทำให้ quantity ครบจะเปลี่ยน order เป็น `FILLED`

ถ้ามี fill เข้ามาหลังการหมดอายุครั้งล่าสุด และ order ยังเป็น `EXPIRED` หรือ `FILLED` อยู่ Nautilus จะบันทึก
และ publish `OrderExpired` ถัดไป event นี้ไม่เปลี่ยนสถานะหรือ timestamp ของ order และ event หมดอายุ
ที่ตามมาจะถูกปฏิเสธจนกว่าจะมี fill ใหม่เข้ามา

## ฟิลด์ [#fields]

นอกเหนือจาก[ฟิลด์ร่วมของ order event](index.md#common-python-order-event-fields)
`OrderExpired` ยังมีฟิลด์ต่อไปนี้:

| Field            | Required/default | คำอธิบาย                                     |
| ---------------- | ---------------- | -------------------------------------------- |
| `venue_order_id` | `None`           | ตัวระบุ order ที่ venue กำหนดให้ (ถ้าทราบ)       |
| `account_id`     | `None`           | account ที่เกี่ยวข้องกับ order (ถ้าทราบ)         |
| `reconciliation` | Required         | ถูกสร้างขึ้นระหว่าง reconciliation หรือไม่      |

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - ประเภทของ event, การ dispatch และฟิลด์ร่วมของ order event
- [Orders](../orders/) - ประเภทของ order และ state machine
