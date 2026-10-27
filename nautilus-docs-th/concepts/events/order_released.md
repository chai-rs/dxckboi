---
title: "OrderReleased"
---

`OrderReleased` บันทึกว่า `OrderEmulator` ได้ release order หลังจากเงื่อนไข trigger เป็นจริงแล้ว
emulator จะ apply event นี้กับ order อัปเดต `Cache` และ publish ลงบน `MessageBus` ก่อนจะส่ง order
ต่อไปยังขั้นถัดไป

transition ที่พบบ่อย: `EMULATED` -> `RELEASED` handler: `on_order_released`

## ฟิลด์ [#fields]

นอกจาก[ฟิลด์ทั่วไปของ order event](index.md#common-python-order-event-fields) แล้ว
`OrderReleased` ยังมี:

| ฟิลด์             | Required/default | คำอธิบาย                                    |
| ---------------- | ---------------- | ------------------------------------------ |
| `released_price` | Required         | ราคาที่ทำให้ order ถูก release ออกจาก emulator |

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และฟิลด์ทั่วไปของ order event
- [Emulated orders](../orders/emulated.md) - lifecycle ของการ emulate order ในเครื่อง
