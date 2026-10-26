# OrderTriggered

`OrderTriggered` บันทึกว่า conditional order แบบ limit ถูก trigger แล้ว execution pipeline จะนำ event
นี้ไปใช้กับ order อัปเดต `Cache` และ publish บน `MessageBus` โดย trading venue, simulated matching
engine หรือ reconciliation สามารถรายงานการ trigger ของ order แบบ `StopLimit`, `LimitIfTouched` หรือ
`TrailingStopLimit` ได้

Transition ทั่วไป: `ACCEPTED` -> `TRIGGERED` Handler: `on_order_triggered`

## ฟิลด์ {#fields}

นอกจาก[ฟิลด์ร่วมของ order event](index.md#common-python-order-event-fields) แล้ว `OrderTriggered`
ยังมีฟิลด์:

| ฟิลด์             | จำเป็น/ค่าเริ่มต้น  | คำอธิบาย                                          |
| ---------------- | ---------------- | ------------------------------------------------ |
| `venue_order_id` | `None`           | ตัวระบุ order ที่ venue กำหนด หากทราบ                |
| `account_id`     | `None`           | account ที่เกี่ยวข้องกับ order หากทราบ               |
| `reconciliation` | Required         | event ถูกสร้างระหว่าง reconciliation หรือไม่          |

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และฟิลด์ร่วมของ order event
- [Orders](../orders/) - ชนิดของ order และ state machine
