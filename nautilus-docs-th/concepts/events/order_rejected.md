---
title: "OrderRejected"
---

`OrderRejected` บันทึกว่า order เข้าสู่สถานะสุดท้าย (terminal) `REJECTED` โดย `ExecutionEngine`
จะ apply event นี้กับ order, อัปเดต `Cache` และ publish ลงบน `MessageBus` โดยปกติ event นี้
มาจากการที่ venue ปฏิเสธ order อย่างชัดเจน แต่ reconciliation ก็สามารถสร้าง event นี้จาก report ของ venue หรือ
หลังจาก timeout ฝั่ง local หรือ policy สำหรับ order ที่หายไปหมดอายุได้เช่นกัน

transition ทั่วไป: `SUBMITTED` -> `REJECTED` ส่วนเส้นทางจากภายนอกและจาก reconciliation ยังอนุญาต
`INITIALIZED`, `ACCEPTED`, `PENDING_UPDATE`, `PENDING_CANCEL` หรือ `TRIGGERED` -> `REJECTED` ได้ด้วย
Handler: `on_order_rejected`

## ฟิลด์ [#fields]

นอกเหนือจาก[ฟิลด์ทั่วไปของ order event](index.md#common-python-order-event-fields)
`OrderRejected` ยังมีฟิลด์ต่อไปนี้:

| Field            | Required/default | Description                                                                          |
| ---------------- | ---------------- | ------------------------------------------------------------------------------------ |
| `account_id`     | Required         | account ที่เกี่ยวข้องกับ order                                                        |
| `reason`         | Required         | เหตุผลจาก venue หรือเหตุผลจาก reconciliation policy ฝั่ง local                         |
| `due_post_only`  | `False`          | ถูกปฏิเสธเพราะเป็น post-only แต่จะถูก execute ทันทีในฐานะ taker หรือไม่                 |
| `reconciliation` | Required         | reconciliation เป็นผู้สร้าง event นี้หรือไม่ ทั้งนี้ไม่ได้หมายความว่า venue ยืนยันแล้ว   |

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และฟิลด์ทั่วไปของ order event
- [Orders](../orders/) - ประเภท order และ state machine
- [Execution policies](../execution/policies.md#terminal-reconciliation-provenance) - หลักฐานจาก venue
  และ policy สำหรับสถานะสุดท้ายแบบสังเคราะห์
