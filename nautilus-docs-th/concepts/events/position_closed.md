---
title: "PositionClosed"
---

`PositionClosed` บันทึก snapshot สุดท้ายของ position โดย `ExecutionEngine` จะส่ง event นี้ออกมาเมื่อ
fill ทำให้ position กลายเป็น flat หรือเมื่อการแก้ไข fill ทำให้ position ที่ถูกแก้ไขอยู่ในสถานะปิด ดู
[จาก fill ถึง position](index.md#from-fill-to-position-the-causal-chain) Handler:
`on_position_closed`

## ฟิลด์ [#fields]

ดูตารางฟิลด์ทั้งหมดได้ที่ [ฟิลด์ของ position event](index.md#position-event-fields) ฟิลด์ที่อธิบาย
การปิดและผลลัพธ์สุดท้ายมีดังนี้:

| Field              | คำอธิบาย                                                     |
| ------------------ | ------------------------------------------------------------ |
| `closing_order_id` | client order ID ที่ปิด position (ถ้ามี)                       |
| `peak_quantity`    | ปริมาณตามทิศทางสูงสุดที่ position เคยไปถึง                    |
| `peak_qty`         | alias เพื่อความเข้ากันได้ของ `peak_quantity`                  |
| `avg_px_close`     | ราคาปิดเฉลี่ย (ถ้ามี)                                         |
| `realized_return`  | realized return สุดท้ายของ position                           |
| `realized_pnl`     | realized PnL สุดท้าย (ถ้ามี)                                  |
| `unrealized_pnl`   | engine ตั้งค่าเป็นศูนย์                                       |
| `duration`         | ระยะเวลาทั้งหมดที่ position เปิดอยู่ หน่วย nanoseconds        |
| `ts_opened`        | UNIX timestamp (nanoseconds) ตอนที่ position เปิด             |
| `ts_closed`        | UNIX timestamp (nanoseconds) ตอนที่ position ปิด              |

เมื่อปิด position แล้ว `side` จะเป็น `FLAT` และ `unrealized_pnl` จะเป็นศูนย์

## ตัวอย่าง [#example]

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python (การอ่าน event ใน handler ของ strategy) จึงตัดออก

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - ประเภทของ event, การ dispatch และลำดับเหตุการณ์จาก fill ถึง position
- [Positions](../positions.md) - วงจรชีวิตของ position, การรวม และ PnL
- [Orders](../orders/) - order ที่ fill แล้วเปิดและปิด position
