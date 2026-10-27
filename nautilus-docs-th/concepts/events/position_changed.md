---
title: "PositionChanged"
---

`PositionChanged` บันทึกการอัปเดตที่ทำให้ position ยังคงเปิดอยู่ `ExecutionEngine` จะส่ง event นี้ออกมาเมื่อ
fill เปลี่ยนแปลง position ที่เปิดอยู่โดยไม่ได้ปิดมัน หรือเมื่อการแก้ไข fill (fill correction) ทำให้ position
ที่ถูกแก้ไขยังคงเปิดอยู่ ดู[จาก fill สู่ position](index.md#from-fill-to-position-the-causal-chain)
Handler: `on_position_changed`

## ฟิลด์ [#fields]

ดูตารางฟิลด์ทั้งหมดได้ที่[ฟิลด์ของ position event](index.md#position-event-fields) นอกเหนือจาก
ฟิลด์ snapshot ตอนเปิด position แล้ว `PositionChanged` ยังมีฟิลด์ต่อไปนี้:

| ฟิลด์             | คำอธิบาย                                                       |
| ----------------- | -------------------------------------------------------------- |
| `peak_quantity`   | quantity ตามทิศทางที่มากที่สุดที่ position เคยไปถึง                  |
| `peak_qty`        | alias สำหรับความเข้ากันได้ของ `peak_quantity`                     |
| `avg_px_close`    | ราคาปิดเฉลี่ยจนถึงปัจจุบัน หากมี quantity ใดถูกปิดไปแล้ว              |
| `realized_return` | ผลตอบแทนที่เกิดขึ้นจริง (realized return) ของ position             |
| `realized_pnl`    | realized PnL หากมี                                              |
| `unrealized_pnl`  | engine ตั้งค่าเป็นศูนย์ ไม่ใช่การคำนวณแบบ mark-to-market            |
| `ts_opened`       | UNIX timestamp (นาโนวินาที) ที่ position เปิด                     |

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และลำดับเหตุการณ์จาก fill สู่ position
- [Positions](../positions.md) - วงจรชีวิตของ position, การรวม (aggregation) และ PnL
- [Orders](../orders/) - order ที่ fill แล้วเปิดและปิด position
