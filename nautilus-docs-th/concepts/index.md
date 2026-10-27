---
title: "แนวคิด (Concepts)"
---

คู่มือในหมวดนี้อธิบาย component หลัก สถาปัตยกรรม และการออกแบบของ NautilusTrader

## ภาพรวม [#overview]

ฟีเจอร์หลักและกรณีการใช้งานที่แพลตฟอร์มออกแบบมารองรับ

## สถาปัตยกรรม [#architecture]

[Architecture](architecture.md) อธิบาย component, ขอบเขตของ runtime และการไหลของข้อมูล

## Actors [#actors]

`DataActor` คือ component พื้นฐานสำหรับโต้ตอบกับระบบเทรด
ครอบคลุมความสามารถและรายละเอียดการ implement

## Strategies [#strategies]

วิธี implement trading strategy ด้วย component `Strategy`

## Instruments [#instruments]

นิยาม instrument สำหรับสินทรัพย์และสัญญาที่ซื้อขายได้

## Synthetics [#synthetics]

Instrument ที่ผู้ใช้กำหนดเอง ซึ่งราคาคำนวณจากการประเมิน expression เชิงตัวเลขบนราคาของ instrument ที่เป็นองค์ประกอบ

## Continuous Futures [#continuous-futures]

สัญญา futures ที่ต่อเนื่องกันรวมเป็น bar series ที่ปรับแล้วชุดเดียวผ่าน roll table ที่กำหนดชัดเจน คู่มือนี้ครอบคลุม
โหมดการปรับ 4 แบบ ขั้นตอน request และ subscription และนโยบายขอบเขตการ roll กลาง bar

## Value Types [#value-types]

Numeric type แบบ immutable (`Price`, `Quantity`, `Money`) ที่ใช้ทั่วทั้งแพลตฟอร์ม
รวมถึงพฤติกรรมทางคณิตศาสตร์ การจัดการ precision และข้อจำกัดเฉพาะของแต่ละ type

## Data [#data]

Data type ที่มีมาในตัวสำหรับโดเมนการเทรด และวิธีทำงานกับ custom data

## Events [#events]

Event type ที่ขับเคลื่อนระบบ ได้แก่ order event, position event, account event และ time event ครอบคลุมการ
dispatch ไปยัง handler, ห่วงโซ่เหตุและผลตั้งแต่ order fill ไปจนถึง position event และการติดตามจาก order ไปยัง position

## Event Sourcing [#event-sourcing]

Log ของ event store แบบถาวรสำหรับ message ที่มีผลต่อสถานะ รวมถึงขอบเขตการบันทึก, correlation header,
โหมด replay, recovery anchor และพฤติกรรมของ verifier

## Options [#options]

Type ของ option instrument, การ stream greeks ที่ venue ให้มา, การ subscribe option chain พร้อมการกรองช่วง strike
และการรวม snapshot

## Greeks [#greeks]

Option greeks (delta, gamma, vega, theta) จากสองทาง คือ greeks แบบ real-time ที่ venue ให้มาผ่าน type
`OptionGreeks` ของ Rust และการคำนวณ Black-Scholes ในเครื่องผ่าน `GreeksCalculator` การคำนวณในเครื่อง
รองรับ shock scenario, beta weighting และการรวมระดับ portfolio

## Custom Data [#custom-data]

ระบบ custom data ทำงานอย่างไร: การลงทะเบียน การ persist การ encode แบบ Arrow และการ routing ขณะ runtime
ผ่าน actor และ strategy

## Order Book [#order-book]

Order book ประสิทธิภาพสูง การติดตาม order ของตัวเอง มุมมองแบบกรองสำหรับ net liquidity และการรองรับ binary market

## Execution [#execution]

การ execute การเทรดและการจัดการ order ข้ามหลาย strategy และหลาย venue พร้อมกัน (ต่อ instance) รวมถึง
การไหลของ component และ message, นโยบายที่ขอบเขตของ venue, execution algorithm และ reconciliation

## Orders [#orders]

ประเภท order ที่มี, execution instruction ที่รองรับ, ประเภท order ขั้นสูง และ emulated order

## Positions [#positions]

วงจรชีวิตของ position การรวมจาก order fill การคำนวณ PnL และการทำ snapshot ของ position
สำหรับการตั้งค่า OMS แบบ netting

## Cache [#cache]

`Cache` คือที่เก็บข้อมูลส่วนกลางใน memory สำหรับข้อมูลทั้งหมดที่เกี่ยวกับการเทรด
ครอบคลุมความสามารถและแนวปฏิบัติที่ดี

## Message Bus [#message-bus]

`MessageBus` ทำให้ component ส่ง message ถึงกันได้แบบไม่ผูกติดกัน รองรับรูปแบบ point-to-point,
publish/subscribe และ request/response

## Accounting [#accounting]

ประเภท account (cash, margin, betting และ wallet), data model ของ `AccountBalance` และ `MarginBalance`
และขอบเขต margin แบบต่อ instrument เทียบกับทั้ง account คู่มือนี้ยังครอบคลุม API สำหรับ query จาก strategy,
margin model ที่มีมาในตัว และแนวปฏิบัติของ adapter สำหรับ venue แบบ live

## Portfolio [#portfolio]

`Portfolio` ติดตาม position ทั้งหมดข้าม strategy และ instrument ทำให้เห็นภาพรวมของการถือครอง
ความเสี่ยง และผลการดำเนินงานในที่เดียว

## Reports [#reports]

Execution report การวิเคราะห์ portfolio การบัญชี PnL และการวิเคราะห์หลังรัน backtest

## Logging [#logging]

ระบบ logging ประสิทธิภาพสูงสำหรับทั้ง backtest และ live trading ซึ่ง implement ด้วย Rust

## Backtesting [#backtesting]

Backtest API การตั้งค่าข้อมูลและ venue ลำดับการ execute การจำลอง fill
account, funding และการตั้งค่า margin

## Behavioral Models [#behavioral-models]

[Behavioral Models](behavioral_models.md) อธิบายว่า model ที่ implement ขึ้นเข้าสู่ runtime ได้อย่างไร

## Visualization [#visualization]

Tearsheet แบบ interactive สำหรับวิเคราะห์ผล backtest รวมถึงกราฟ theme ตัวเลือกการปรับแต่ง และ
visualization แบบกำหนดเองผ่าน chart registry ที่ขยายได้

## Configuration [#configuration]

Config struct ทำงานอย่างไร: การกำหนดค่าเริ่มต้น แนวปฏิบัติ `T` เทียบกับ `Option<T>`
รูปแบบ builder และ field ทั่วไปที่ใช้ร่วมกันระหว่าง adapter และ engine

## Live Trading [#live-trading]

การรันโค้ด strategy และ execution algorithm ชุดเดียวกันในตลาดจริง โดยคำนึงถึงพฤติกรรมของ venue, transport,
เวลา, การ persist, กิจกรรมจากภายนอก และ reconciliation

## Adapters [#adapters]

ข้อกำหนดและแนวปฏิบัติที่ดีสำหรับพัฒนา adapter เพื่อ integrate กับผู้ให้บริการข้อมูลและ trading venue

## Networking [#networking]

สถาปัตยกรรม นโยบายด้านความน่าเชื่อถือ และคุณลักษณะด้านประสิทธิภาพของ client สำหรับ HTTP, WebSocket
และ raw TCP socket ที่ใช้ร่วมกัน

## Rust [#rust]

การเขียน actor และ strategy รวมถึงการรัน backtest และ live trading ด้วย Rust ล้วน
โดยใช้ implementation ใน `crates/` โดยตรง

## DST [#dst]

Deterministic simulation testing (DST): ข้อตกลงสำหรับการ execute ที่ replay ได้ด้วย seed และจุดต่อในระดับ source
ที่ implement ข้อตกลงนั้น คู่มือนี้ยังครอบคลุม pre-commit hook ที่บังคับใช้ข้อตกลงและขอบเขตที่ทราบอยู่แล้ว

:::note
หากคู่มือเหล่านี้ขัดแย้งกับ API reference ให้ถือ API reference ([Rust API docs](https://nautilustrader.io/docs/core-latest/)) เป็นหลัก
:::
