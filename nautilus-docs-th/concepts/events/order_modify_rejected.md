# OrderModifyRejected

`OrderModifyRejected` บันทึกว่าคำสั่ง `ModifyOrder` ถูกปฏิเสธ `ExecutionEngine` จะนำ event นี้ไปใช้กับ order
อัปเดต `Cache` และ publish ลงบน `MessageBus` ผู้ปฏิเสธคำขออาจเป็น trading venue, simulated matching engine
หรือ risk control ภายในระบบก็ได้

การเปลี่ยนสถานะทั่วไป: `PENDING_UPDATE` -> สถานะก่อนหน้า (เช่น `ACCEPTED`) handler:
`on_order_modify_rejected`

ในสถานะ `SUBMITTED`, `ACCEPTED`, `TRIGGERED` และ `PARTIALLY_FILLED` การปฏิเสธจะคงสถานะและเงื่อนไขของ order ไว้
ตามเดิม รวมถึงการแก้ไขที่ `OrderUpdated` นำไปใช้แล้ว ส่วนในสถานะ `PENDING_CANCEL` จะคงการยกเลิกที่ค้างอยู่ไว้

## ฟิลด์ {#fields}

นอกเหนือจาก[ฟิลด์ทั่วไปของ order event](index.md#common-python-order-event-fields) แล้ว
`OrderModifyRejected` ยังมีฟิลด์ต่อไปนี้:

| ฟิลด์             | บังคับ/ค่าเริ่มต้น | คำอธิบาย                                        |
| ---------------- | ---------------- | ---------------------------------------------- |
| `reason`         | Required         | เหตุผลที่การอัปเดต order ถูกปฏิเสธ                   |
| `venue_order_id` | `None`           | ตัวระบุ order ที่ venue กำหนด หากทราบ               |
| `account_id`     | `None`           | account ที่เกี่ยวข้องกับ order หากทราบ               |
| `reconciliation` | Required         | ถูกสร้างขึ้นระหว่าง reconciliation หรือไม่             |

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Events](index.md) - ประเภทของ event, การ dispatch และฟิลด์ทั่วไปของ order event
- [Orders](../orders/) - ประเภทของ order และ state machine
