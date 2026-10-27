---
title: "OrderEmulated"
---

`OrderEmulated` บันทึกว่า `OrderEmulator` ได้รับ order เข้ามาจำลองแบบ local แล้ว โดย emulator จะ apply
event นี้กับ order, อัปเดต `Cache` และ publish event ลงบน `MessageBus`

การเปลี่ยนสถานะที่พบบ่อย: `INITIALIZED` -> `EMULATED` Handler: `on_order_emulated`

## Field [#fields]

`OrderEmulated` มีเฉพาะ
[field ร่วมของ order event](index.md#common-python-order-event-fields) เท่านั้น

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และ field ร่วมของ order event
- [Emulated orders](../orders/emulated.md) - วงจรชีวิตของการจำลอง order แบบ local
