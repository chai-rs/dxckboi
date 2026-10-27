---
title: "Behavioral Models"
---

หน้านี้อธิบาย model แบบ pluggable ที่ใช้เปลี่ยนหรือขยายพฤติกรรมของ NautilusTrader

**behavioral model** คือส่วนที่กำหนดกฎสำหรับการคำนวณหรือการตัดสินใจเฉพาะอย่างหนึ่งของระบบ
เช่น fill model กำหนดว่า order ใดมีสิทธิ์ถูก fill แบบจำลองและมี liquidity เท่าใด ส่วน fee model คำนวณค่า
commission ของแต่ละ fill engine จะเรียก model ที่ตั้งค่าไว้ผ่าน interface ที่นิยามไว้ ผู้ใช้จึงเปลี่ยนกฎเหล่านี้ได้
โดยไม่ต้องแก้ engine หรือ strategy ของตัวเอง

model ถูกส่งเข้ามาเป็น object ใน config หรือผ่าน runtime API ผู้ใช้เลือก implementation ที่มีมาให้ในตัว
ปรับพารามิเตอร์ หรือส่ง implementation ของตัวเองได้ในกรณีที่ model family และ API นั้นรองรับ คำว่า pluggable
ในที่นี้หมายถึงการแทนที่พฤติกรรมผ่าน interface เหล่านี้
[การโหลด native library ตอน runtime](#native-extension-boundary) เป็นความสามารถอีกเรื่องหนึ่งแยกต่างหาก

## Model family [#model-families]

| Family  | ควบคุมอะไร                                                                                | ตัวอย่างที่มีในตัว                                                |
| ------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| Fill    | สิทธิ์การ fill ของ limit order แบบจำลอง, slippage และ liquidity สังเคราะห์ (ถ้าเปิดใช้)        | `DefaultFillModel`, `BestPriceFillModel`, `TwoTierFillModel`. |
| Fee     | commission ที่คำนวณจาก order, fill, instrument และ pricing context (ถ้ามี)                 | `MakerTakerFeeModel`, `FixedFeeModel`, `PerContractFeeModel`. |
| Margin  | initial margin ของ order และ maintenance margin ของ position                             | `StandardMarginModel`, `LeveragedMarginModel`.                |
| Latency | delay แบบจำลองของการ submit, modify และ cancel order                                     | `StaticLatencyModel`.                                         |

fill model มีตั้งแต่แบบที่ใช้ book ที่บันทึกไว้ ไปจนถึงแบบที่สร้าง liquidity สังเคราะห์ที่มีพฤติกรรมแบบหลายชั้น (tiered),
partial fill, คำนึงถึงขนาด, คำนึงถึงการแข่งขัน, อ่อนไหวต่อ volume หรือขึ้นกับเวลาทำการของตลาด ดู
[fill models](backtesting/fill-models.md) สำหรับรายการที่มีในตัวทั้งหมดและผลของ book type ต่อการจำลอง fill

fee model ยังมี `ProbabilityPriceFeeModel`, `CappedOptionFeeModel` และ `TieredNotionalOptionFeeModel`
สำหรับ instrument ที่ตั้งราคาแบบความน่าจะเป็น และตารางค่าธรรมเนียมของ option

[margin models](backtesting/accounts-and-margin.md#margin-models) เลือกว่าจะลด margin requirement ของ
instrument ตาม leverage ของ account หรือไม่ `StaticLatencyModel` เพิ่ม base delay เข้ากับ delay ของ insert,
update และ cancel ที่ตั้งค่าแยกกัน

fill model และ latency model ควบคุมเฉพาะ execution แบบจำลอง ไม่ได้กำหนดว่า venue จริงใน live จะ fill order
หรือไม่และเมื่อใด พฤติกรรมของ model มีผลที่ขอบเขตของ engine หรือ account ที่ใช้ model นั้น การเปลี่ยน model
ไม่ได้แทนที่การตรวจสอบ state ของ order, การ route execution หรือส่วนอื่นของ runtime

## Simulation modules [#simulation-modules]

[Simulation modules](backtesting/simulation-modules.md) เป็นจุดขยายที่เกี่ยวข้องกันสำหรับพฤติกรรมตลอด
lifecycle ของ exchange แบบจำลอง แทนที่จะตอบคำถามการคำนวณ fill, fee, margin หรือ latency module จะประมวลผล
state ของ exchange แล้วคืนค่า account adjustment module FX rollover และ CFD swap ที่มีในตัวใช้ interface นี้
module ที่เขียนเองก็ขยายพฤติกรรมการจำลองผ่าน lifecycle และ acknowledgement contract เดียวกันได้

## การส่ง implementation [#supplying-implementations]

ฝั่ง Rust สามารถ implement trait ของ model family นั้น แล้วส่ง implementation ผ่าน runtime handle ของมัน
พารามิเตอร์ของ model เรียงลำดับเป็น `fill_model`, `fee_model`, `margin_model` และ `latency_model`
เมื่อมี `modules` จะตามหลังกลุ่ม model

### Latency model [#custom-python-latency-models]

ค่า delay ที่ latency model คืนให้ต้องรวม base latency ไว้แล้ว เพราะ engine ไม่บวกเพิ่มให้

ใน backtest engine จะเรียก model เฉพาะกับ command ที่เข้าคิว venue ที่สร้างจาก `BacktestVenueConfig`
จะเข้าคิว command เสมอ

callback แต่ละตัวได้รับ command จริงและ `ts_now` ซึ่งคือเวลาปัจจุบันของ clock เป็น Unix nanoseconds:
เวลาจำลองใน backtest และเวลาจาก clock ของ sandbox client ใน sandbox

| Method               | Command types                                          |
| -------------------- | ------------------------------------------------------ |
| `get_insert_latency` | `SubmitOrder`, `SubmitOrderList`.                      |
| `get_update_latency` | `ModifyOrder`, `BatchModifyOrders`.                    |
| `get_delete_latency` | `CancelOrder`, `BatchCancelOrders`, `CancelAllOrders`. |

method เดียวกันนี้รับ batch command ทั้งชุด และคืน delay เดียวสำหรับสมาชิกทั้งหมดในชุด engine เรียก model
หนึ่งครั้งต่อหนึ่ง command

เวลาที่ command มาถึงใน backtest คือ `command.ts_init + delay` โดย engine จะประมวลผล command ที่
[settlement point](backtesting/execution-flow.md#latency-delayed-commands) แรกที่มีสิทธิ์ ณ เวลานั้นหรือหลังจากนั้น
ส่วนใน sandbox เวลาที่มาถึงคือ `ts_now + delay` โดย `ts_now` คือเวลาที่ได้รับ command

## โครงสร้างของ model family [#model-family-structure]

behavioral model family ใช้รูปแบบการแทนค่าร่วมกันทั้งใน simulation และ execution:

- Rust trait `<Family>Model` นิยาม behavioral contract
- Rust type ที่เป็น concrete type ใช้ implement model ที่มีในตัว
- enum `<Family>ModelAny` รวม built-in หลักและ language bridge ที่ต้องเก็บเป็น enum แล้ว implement trait ผ่าน
  enum dispatch แบบ explicit
- `<Family>ModelHandle` เก็บ trait object แบบ shared ในจุดที่ runtime component รับ Rust implementation ที่
  link เข้ามา นอกเหนือจาก variant ของ enum

model เฉพาะของ adapter อยู่ใน crate ของ adapter นั้น ในกรณีที่การเพิ่ม variant ใน core enum จะทำให้เกิด reverse
dependency โค้ด Rust ระดับล่างส่ง model เหล่านี้ผ่าน handle ที่ตรงกัน

## ขอบเขตของ dispatch [#dispatch-boundary]

| รูปแบบ                    | implementation ที่รับได้                   | Dispatch     | บทบาท                                          |
| ------------------------ | -------------------------------------- | ------------ | --------------------------------------------- |
| Concrete type หรือ generic | concrete implementation ตัวเดียว          | Static       | ส่วนภายในของ model และผู้เรียกที่เฉพาะทาง          |
| `<Family>ModelAny`       | built-in ที่ประกาศไว้และ bridge variant      | Enum match   | config หรือการจัดเก็บ built-in และ bridge         |
| `<Family>ModelHandle`    | Rust trait implementation ใดก็ได้ที่รับได้     | Trait object | การจัดเก็บ runtime แบบ shared และ type ที่เขียนเอง |

`<Family>ModelAny` ใช้ enum dispatch ส่วน `<Family>ModelHandle` ใช้ dynamic dispatch ผ่าน trait object
ดังนั้น built-in ที่แปลงจาก enum เป็น handle จะผ่าน vtable ก่อนถึง enum match การจัดเก็บที่มีเฉพาะ built-in จึง
คง type เป็น `<Family>ModelAny` ไว้ในจุดที่การเลี่ยง trait-object dispatch มีความสำคัญ

handle ไม่มี fast path แยกสำหรับ built-in โดยใช้รูปแบบเดียวที่เรียบง่ายกว่า เพราะยังไม่มีกรณีด้าน performance ที่วัดผล
แล้วคุ้มค่ากับความซับซ้อนของ variant และ dispatch ที่เพิ่มขึ้น

## ขอบเขตของ native extension [#native-extension-boundary]

[plug-in crate](../developer_guide/plugins.md) แบบ open-source นิยาม artifact ABI ไว้ แต่การลงทะเบียน model
และ host ที่ทำหน้าที่โหลดไม่ได้อยู่ใน repository นี้ distribution แบบ open-source ไม่มี native model plugin แบบ
runtime ให้ใช้ native model จะถูกประกอบเข้าด้วยกันตอน compile และส่งผ่าน enum หรือ handle ที่ตรงกัน

[Simulation modules](backtesting/simulation-modules.md) ใช้รูปแบบการแทนค่าเหล่านี้ที่ขอบเขตของ backtest
config และ runtime
