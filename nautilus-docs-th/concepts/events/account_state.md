# AccountState

`AccountState` เก็บ snapshot ของยอด balance และ margin ของ account ระบบจะ publish event นี้เมื่อ:

- venue รายงานการอัปเดต account ผ่าน execution client
- `Portfolio` คำนวณ account state ใหม่หลังจาก position มีการอัปเดต สำหรับ margin account ที่เปิด
  `calculate_account_state` ไว้

`Portfolio` subscribe event เหล่านี้ภายในเพื่อติดตาม exposure และ balance

flag `is_reported` ใช้แยก snapshot ที่ venue รายงานมา ออกจาก snapshot ที่ระบบคำนวณเอง

## ฟิลด์ {#fields}

| Field           | Required/default | คำอธิบาย                                                                 |
| --------------- | ---------------- | ------------------------------------------------------------------------- |
| `account_id`    | Required         | ID ของ account (รวม venue)                                                |
| `account_type`  | Required         | ประเภท account (`CASH`, `MARGIN`, `BETTING` หรือ `WALLET`)                |
| `base_currency` | `None`           | สกุลเงินหลักของ account (`None` สำหรับ account แบบหลายสกุลเงิน)          |
| `is_reported`   | Required         | state นี้รายงานมาจาก exchange หรือไม่ (ถ้าไม่ใช่ แปลว่าระบบคำนวณเอง)     |
| `balances`      | Required         | balance ของ account (อาจว่างได้)                                          |
| `margins`       | Required         | margin balance (อาจว่างได้)                                               |
| `event_id`      | Required         | ID ของ event                                                              |
| `ts_event`      | Required         | UNIX timestamp (nanoseconds) ตอนที่ event เกิดขึ้น                        |
| `ts_init`       | Required         | UNIX timestamp (nanoseconds) ตอนที่ object ถูกสร้าง                       |
| `info`          | `None`           | ข้อมูล account เฉพาะของ venue ที่ไม่มีฟิลด์แบบ typed รองรับ (เป็น dict ว่างเมื่อไม่ได้ตั้งค่า) |

## ตัวอย่าง {#example}

โดยปกติจะอ่าน account state ผ่าน `Portfolio` แทนการเขียน handler แยกเฉพาะ

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

ผลลัพธ์ที่ได้แยกขาดจาก Portfolio การแก้ไข account ที่ได้คืนมาจะไม่เปลี่ยน account ตัวหลักที่ engine
ถืออยู่

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Events](index.md) - ประเภทของ event และการ dispatch
- [Accounting](../accounting.md) - ประเภท account, balance และ margin model
- [Portfolio](../portfolio.md) - account state ป้อนข้อมูลให้การติดตาม exposure และ balance อย่างไร
