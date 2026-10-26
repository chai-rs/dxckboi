# OrderCanceled

`OrderCanceled` บันทึกว่า order เข้าสู่สถานะ terminal `CANCELED` แล้ว execution pipeline จะนำ event
นี้ไปใช้กับ order อัปเดต `Cache` และ publish บน `MessageBus` โดย event อาจมาจาก trading venue,
simulated matching engine, local order emulator หรือ reconciliation ซึ่ง reconciliation สามารถสร้าง event
นี้จาก report ของ venue หรือจาก policy ภายในเรื่อง timeout หรือ order ที่หายไป

Transition ทั่วไป: `PENDING_CANCEL`/`ACCEPTED` -> `CANCELED` ส่วนเส้นทางจากภายนอกและเส้นทางกู้คืน
ยังอนุญาต `INITIALIZED`, `EMULATED`, `RELEASED`, `SUBMITTED`, `PENDING_UPDATE`, `TRIGGERED` หรือ
`PARTIALLY_FILLED` -> `CANCELED` ด้วย Handler: `on_order_canceled`

การปิดซ้ำ (re-close) สามารถเพิ่ม `CANCELED` ต่อท้ายได้ขณะที่ order ถูกยกเลิกไปแล้ว ในกรณีที่มี fill
มาถึงช้าหลังการยกเลิกครั้งก่อน

## ฟิลด์ {#fields}

นอกจาก[ฟิลด์ร่วมของ order event](index.md#common-python-order-event-fields) แล้ว `OrderCanceled`
ยังมีฟิลด์:

| ฟิลด์             | จำเป็น/ค่าเริ่มต้น  | คำอธิบาย                                                                  |
| ---------------- | ---------------- | ------------------------------------------------------------------------ |
| `venue_order_id` | `None`           | ตัวระบุ order ที่ venue กำหนด หากทราบ                                        |
| `account_id`     | `None`           | account ที่เกี่ยวข้องกับ order หากทราบ                                       |
| `reason`         | `None`           | เหตุผลการยกเลิกที่ venue ให้มา                                               |
| `reconciliation` | Required         | event ถูกสร้างโดย reconciliation หรือไม่ ซึ่งไม่ได้หมายความว่า venue ยืนยันแล้ว      |

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และฟิลด์ร่วมของ order event
- [Orders](../orders/) - ชนิดของ order และ state machine
- [Execution policies](../execution/policies.md#terminal-reconciliation-provenance) - หลักฐานจาก venue
  และ policy สถานะ terminal แบบสังเคราะห์
