---
title: "PositionOpened"
---

`PositionOpened` บันทึก snapshot ตอนเปิดของ position ใหม่ `ExecutionEngine` จะส่ง event นี้ออกมาเมื่อ
fill สร้าง position ขึ้น (ดู
[จาก fill สู่ position](index.md#from-fill-to-position-the-causal-chain)) Handler:
`on_position_opened`

## ฟิลด์ [#fields]

ดูตารางฟิลด์ทั้งหมดได้ที่[ฟิลด์ของ position event](index.md#position-event-fields) `PositionOpened`
มี snapshot ตอนเปิด position ส่วน event ที่ตามมาภายหลังจะมีฟิลด์เกี่ยวกับยอดรวมและวงจรชีวิตเพิ่มเติม
ฟิลด์สถานะหลักมีดังนี้:

| ฟิลด์          | คำอธิบาย                                                  |
| -------------- | --------------------------------------------------------- |
| `entry`        | ฝั่งของ order ขาเข้าที่เปิด position นี้                        |
| `side`         | ฝั่งปัจจุบันของ position (`LONG` หรือ `SHORT`)                |
| `signed_qty`   | quantity ของ position แบบมีเครื่องหมาย                       |
| `quantity`     | quantity ที่เปิดอยู่ในปัจจุบัน                                 |
| `last_qty`     | quantity ของ fill ที่เปิด position                           |
| `last_px`      | ราคาของ fill ที่เปิด position                                |
| `currency`     | quote currency ของ position                                |
| `avg_px_open`  | ราคาเปิดเฉลี่ย                                              |
| `realized_pnl` | realized PnL ของรอบปัจจุบัน ในสกุลเงินต้นทุน (cost currency) |

ตอนเปิด position นั้น realized PnL จะหักค่าคอมมิชชันของ fill ที่เปิด position ออก หากค่าคอมมิชชันนั้น
อยู่ใน[สกุลเงินต้นทุน](../positions.md#currency-considerations)ของ instrument มิฉะนั้นจะเป็นศูนย์

position แบบ `NETTING` ที่ถูกเปิดใหม่อีกครั้งจะเริ่มรอบใหม่ ส่วน realized PnL ของรอบก่อนหน้าจะยังคงอยู่ใน
[snapshot ของ position ที่ปิดแล้ว](../positions.md#position-snapshotting)

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Events](index.md) - หมวดหมู่ของ event, การ dispatch และลำดับเหตุการณ์จาก fill สู่ position
- [Positions](../positions.md) - วงจรชีวิตของ position, การรวม (aggregation) และ PnL
- [Orders](../orders/) - order ที่ fill แล้วเปิดและปิด position
