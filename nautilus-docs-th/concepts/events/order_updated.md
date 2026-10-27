---
title: "OrderUpdated"
---

`OrderUpdated` บันทึกการเปลี่ยนแปลง quantity, price, trigger price หรือ protection price ที่คำนวณได้
ของ order execution pipeline จะ apply event นี้กับ order อัปเดต `Cache` และ publish ลงบน
`MessageBus` การเปลี่ยนแปลงอาจมาจาก trading venue, simulated matching engine, order emulator
ในเครื่อง หรือ reconciliation

transition ที่พบบ่อย: `PENDING_UPDATE` -> สถานะก่อนหน้า (เช่น `ACCEPTED`) handler:
`on_order_updated`

## สัญญา (contract) [#contract]

`quantity` คือ quantity แบบ **gross** ของ order: รวม filled quantity และ voided quantity ที่ไม่ได้
reopen (ดู [OrderFillVoided](order_fill_voided.md)) Nautilus คำนวณ leaves ที่ยังทำงานอยู่เป็น

```text
leaves_qty = max(quantity - filled_qty - non_reopened_voided_qty, 0)
```

ดังนั้น void ที่ไม่ได้ reopen จะถูกตัดออกจาก leaves เสมอแม้มีการ update ครั้งต่อ ๆ ไป โดย adapter ไม่ต้อง
หักออกจาก `quantity` เอง adapter ที่รายงาน `quantity` แบบหัก void ที่ไม่ได้ reopen ออกแล้ว จะทำให้
leaves ถูกหัก quantity นั้นซ้ำสองครั้ง

**Terminal reconciliation** เป็นข้อยกเว้น reconciliation update (`reconciliation=True`) ที่ `quantity`
เท่ากับ `filled_qty` ที่ไม่เป็นศูนย์ของ order จะปิด order เป็น `FILLED` ในกรณีนี้ `quantity` คือ filled
quantity ที่มีผลจริง หลังหัก voided quantity แล้ว ดังนั้น adapter ที่ปิด order ตามจำนวนที่ fill ได้จะรายงาน
`filled_qty` แทน gross quantity

## ฟิลด์ [#fields]

นอกจาก[ฟิลด์ทั่วไปของ order event](index.md#common-python-order-event-fields) แล้ว
`OrderUpdated` ยังมี:

| ฟิลด์                | Required/default | คำอธิบาย                                                  |
| ------------------- | ---------------- | -------------------------------------------------------- |
| `venue_order_id`    | `None`           | order identifier ที่ venue กำหนด (ถ้าทราบ)                  |
| `account_id`        | `None`           | account ที่ผูกกับ order (ถ้าทราบ)                            |
| `quantity`          | Required         | quantity ปัจจุบันของ order                                  |
| `price`             | `None`           | ราคาปัจจุบันของ order                                       |
| `trigger_price`     | `None`           | trigger price ปัจจุบันของ order                             |
| `protection_price`  | `None`           | protection price ที่คำนวณได้ของ order                       |
| `is_quote_quantity` | `False`          | order quantity เป็นหน่วยของ quote currency หรือไม่            |
| `reconciliation`    | Required         | สร้างขึ้นระหว่าง reconciliation หรือไม่                        |

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และฟิลด์ทั่วไปของ order event
- [Orders](../orders/) - ประเภทของ order และ state machine
