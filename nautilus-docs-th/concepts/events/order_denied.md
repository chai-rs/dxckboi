# OrderDenied

`OrderDenied` บันทึกว่าระบบ Nautilus ไม่สามารถ submit order ที่ถูกต้องตามรูปแบบได้ เช่น เพราะติด
risk limit หรือใช้ฟีเจอร์ที่ไม่รองรับ execution pipeline จะนำ event นี้ไป apply กับ order อัปเดต `Cache`
และ publish ลงบน `MessageBus`

risk engine, execution engine, execution algorithm และ execution client ล้วนสามารถ deny order ได้

transition ที่พบบ่อย: `INITIALIZED` -> `DENIED` handler: `on_order_denied`

## ฟิลด์ {#fields}

นอกจาก[ฟิลด์ทั่วไปของ order event](index.md#common-python-order-event-fields) แล้ว
`OrderDenied` ยังมี:

| ฟิลด์     | Required/default | คำอธิบาย                                                              |
| -------- | ---------------- | -------------------------------------------------------------------- |
| `reason` | Required         | รหัสเหตุผลการ deny แบบมาตรฐาน อาจตามด้วยข้อความวินิจฉัยเพิ่มเติม (suffix) |

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และฟิลด์ทั่วไปของ order event
- [Execution](../execution/) - risk check และ pipeline ก่อนส่ง order (pre-trade)
- [เหตุผลของ order denied](../execution/index.md#order-denied-reasons) - ชุดรหัสมาตรฐานและ
  รูปแบบข้อความ
- [Orders](../orders/) - ประเภทของ order และ state machine
