# Callback Dispatch Contract

หน้านี้กำหนดข้อกำหนดด้าน ownership, ลำดับ และความคืบหน้า (progress) ของ callback ของ actor และ strategy
แบบเข้าคิว (queued) ส่วน [design principles](design_principles.md#queued-callback-dispatch-requirements)
อธิบายนโยบายเบื้องหลัง

:::info การส่งแบบเข้าคิวยังไม่เปิดใช้งาน
ข้อกำหนดเหล่านี้เป็นข้อจำกัดด้านการออกแบบสำหรับการส่ง callback ของ actor และ strategy แบบเข้าคิว ไม่ใช่สิ่งที่
รับประกันในเส้นทาง dispatch แบบ synchronous ที่มีอยู่ตอนนี้
การรองรับ reentry แบบ synchronous ของ message bus ไม่ได้เปิดใช้งาน callback แบบเข้าคิวของ actor หรือ strategy
:::

อ่านหน้านี้ตามจุดประสงค์:

- **สัญญาเชิงบรรทัดฐาน (normative contract)**: [ลำดับ](#ordering-and-reentrancy), observable state,
  bounded progress และ [การออกแบบ drain boundary](#drain-boundary-design) กำหนดข้อกำหนดสำหรับการเปิดใช้งาน
- **implementation ปัจจุบัน**: [ข้อจำกัดของ implementation](#implementation-limits) และ
  [การ integrate กับ runtime](#runtime-integration) แยกกลไกที่มีอยู่แล้วออกจากช่องว่างที่ยังต้องปิดก่อนเปิดใช้งาน
  [Runtime Conformance Contract](runtime_conformance.md) จับคู่ซอร์สโค้ดกับการตรวจสอบที่เป็นตัวแทน
- **รายละเอียด implementation ภายใน**: [dispatch primitive](#private-dispatch-primitives), command transport
  และการนับ storage อธิบายว่ากลไกบังคับใช้ข้อกำหนดเหล่านั้นอย่างไร

ข้อความ error และวิธีแก้ไข ดูได้ที่ [reentrancy และ dispatch diagnostics](#reentrancy-and-dispatch-diagnostics)

## ตัวอย่างลำดับ callback {#worked-callback-sequence}

ตัวอย่างนี้แสดงสัญญาแบบเข้าคิว callback แบบ canonical ใน production ยังคงเป็น synchronous สมมติว่า event A
มีผู้รับที่มีสิทธิ์สองตัว คือ strategy S1 และ S2 และ event B มีผู้รับคือ S3 ไม่มีงานอื่นที่ไม่เกี่ยวข้องค้างอยู่ และทุก
drain boundary สามารถเข้าถึงทุกอย่างที่ต้องใช้ได้

```text
Publication A
  |  Reserve publication ordinal before synchronous subscribers run
  |  Admit S1(A), creating root R; admit S2(A) under the same root
  v
Callback queue: [ S1(A) | S2(A) ]                         root R
  |
  |  Enclosing runtime borrows end
  v
Safe boundary: drain callbacks
  |
  +-- S1(A) runs                                          root R
  |     |
  |     +-- publish B --> queue S3(B) after S2(A)          root R
  |     |                (no recursive S3 invocation)
  |     |
  |     +-- send C -----> command transport retains C     root R
  |                      (owner thread; context preserved)
  |
  +-- S2(A) runs                                          root R
  |
  +-- S3(B) runs                                          root R

Callback order: S1(A) --> S2(A) --> S3(B), even across bounded drain passes
Budget:         three completed deliveries charged to R,
                even across bounded drain passes

When the runtime processes C (timing depends on the runtime):
  restore R --> handle C --> any resulting callbacks retain R
                            and queue by publication/admission order
  C itself consumes no callback delivery budget
```

publication B ที่ซ้อนอยู่จะรอต่อจากผู้รับของ A ที่ยังค้างอยู่ callback และ command ต่างถือ root R ไว้

ลำดับ callback ข้างต้นไม่ได้ระบุว่า C จะทำงานเมื่อใดเมื่อเทียบกับ S2(A) และ S3(B)
[settlement ของ backtest และการจัดตารางของ live](#runtime-integration) เป็นตัวกำหนดการประมวลผล command
ระหว่างรอบของ callback R ยังมีชีวิตอยู่ตราบที่ C หรืองานลูกหลานอื่นที่ถูก retain ไว้ยังเป็นเจ้าของ การ yield ไม่ได้
reset budget ของมัน การส่งจาก thread อื่นหรือจาก task ที่ไม่เกี่ยวข้องจะไม่สืบทอด R โดยอัตโนมัติ

## คำศัพท์ {#terminology}

| คำ            | ความหมาย                                                                                      |
| ------------- | --------------------------------------------------------------------------------------------- |
| Publication   | การ publish event หนึ่งครั้ง ซึ่งจอง ordinal ไว้ก่อนงานของ subscriber แบบ synchronous         |
| Admission     | การจองจำนวน callback, storage ที่ทราบขนาด และตำแหน่งในคิว ก่อนสร้าง capture                   |
| Slot          | entry หนึ่งรายการในคิวที่เรียงลำดับ ไม่ว่าจะอยู่ในสถานะจองไว้ พร้อม หรือถูกยกเลิก             |
| Capture       | ค่าที่ถูก own ไว้สำหรับ callback หรือ invocation                                              |
| Root          | ownership เชิงเหตุผล (causal) และการนับ delivery สะสมที่งานลูกหลานใช้ร่วมกัน ไม่มี rollback    |
| Retained work | งานที่เข้าคิวหรือถูกพักไว้ ซึ่งทำให้ root ยังมีชีวิตอยู่เลยการเรียกหรือ drain ปัจจุบัน         |
| Drain         | การไล่ callback slot ตามลำดับหนึ่งรอบแบบมีขอบเขต                                              |
| Safe boundary | จุดที่ runtime เป็นเจ้าของ ซึ่ง borrow ที่ครอบอยู่สิ้นสุดแล้ว และอนุญาตให้เรียก callback ได้   |

## ข้อจำกัดของ implementation {#implementation-limits}

| ด้าน                                                  | สิ่งที่ implement แล้ว                                        | ข้อจำกัด                                                                |
| ----------------------------------------------------- | ------------------------------------------------------------- | ----------------------------------------------------------------------- |
| การส่งให้ actor                                       | primitive ภายในสำหรับการส่งแบบเรียงลำดับ                     | การส่งแบบเข้าคิวให้ actor และ strategy ยังไม่เปิดใช้งาน                 |
| [การส่งต่อ root](#callback-roots-and-budgets)         | causal root ผ่าน retained work และ command/event channel      | สายบรรพบุรุษอยู่บน owner thread เท่านั้น การ fire ตามตารางเป็นอิสระต่อกัน |
| [ความปลอดภัยของ drain](#draining-and-progress)        | boundary ของ running loop ใน backtest และ live                | การ integrate กับ lifecycle ของ live และการเปิดใช้งานยังไม่สมบูรณ์       |
| [Progress budget](#callback-roots-and-budgets)        | นับ callback delivery ที่เสร็จแล้วต่อ root                    | ไม่จำกัดระยะเวลาของ callback หรือ loop ที่มีแต่ command                  |
| [การนับ memory](#storage-limits)                      | ขีดจำกัดของ retained unit และ storage ที่ทราบขนาด             | ไม่ใช่เพดาน memory ของทั้ง process ผู้ใช้ตั้งค่าไม่ได้                   |
| [การจัดการ failure](#failure-cleanup)                 | การคืนสภาพเมื่อ unwind และ teardown แบบมี guard               | fatal error หยุดทุก root ไม่มี rollback ของผลที่เกิดขึ้นแล้ว              |
| [การเข้าถึงและ backend](#backend-compatibility)       | checked-access guard ภายใน                                    | unchecked access และความเท่าเทียมระหว่าง native/dynamic อยู่นอกการรับประกัน |
| [Observable state](#maintenance-and-observable-state) | ผลแบบ synchronous ต่อ cache และ facade                        | เป็น state ปัจจุบันของ cache ไม่ใช่ snapshot ณ เวลาที่เกิด event         |

## ลำดับและ reentrancy {#ordering-and-reentrancy}

:::tip reentrancy คืออะไร
reentrancy เกิดขึ้นเมื่อโค้ดถูกเรียกเข้าซ้ำก่อนที่การเรียกก่อนหน้าจะ return ตัวอย่างเช่น subscriber ของ message bus
publish ข้อความอื่น ทำให้เกิดการส่งซ้อนกันก่อนที่ publication เดิมจะเสร็จ สิ่งนี้เกิดได้บน thread เดียว ไม่จำเป็นต้อง
มีการทำงานแบบขนาน
:::

ภายใน runtime thread หนึ่ง callback แบบ canonical ของ actor และ strategy ต้องรักษา **ลำดับของ publication**
ข้าม component และ topic กฎนี้ใช้เหมือนกันทั้งกับ component ที่ว่างและที่กำลังทำงาน publication ที่ซ้อนอยู่ต้อง
ไม่แซงการส่งที่ยังค้างอยู่ของ publication ก่อนหน้า รวมถึงผู้รับทุกตัวของ publication ก่อนหน้านั้น node ที่เป็นอิสระต่อกัน
ไม่มีการรับประกันลำดับร่วมแบบ global

callback ต้องการ **การเข้าถึงแบบ exclusive** ต่อ component ของมัน และต้องการ delivery boundary ที่ mutable
runtime borrow ที่ครอบอยู่สิ้นสุดแล้ว component ทุกตัวต้องปฏิบัติตามสัญญาด้านลำดับเดียวกัน

## Maintenance และ observable state {#maintenance-and-observable-state}

maintenance ของ component ทำงานพร้อมกับ event ที่เข้าคิวแต่ละตัว **ก่อน author callback ของ event นั้น**
ซึ่งรวมถึง:

- การอัปเดต indicator
- การเก็บกวาด timer
- การจัดการ contingent order

maintenance ยังคงใช้กฎ lifecycle ที่เกี่ยวข้องเมื่อ author callback ถูกระงับ event ที่ maintenance ส่งออกมาจะ
เข้าสู่กลไก dispatch แบบเรียงลำดับเดียวกัน

### Cache state และสิทธิ์ในการรับ {#cache-state-and-eligibility}

การเปลี่ยนแปลง [cache](../concepts/architecture.md#cache) ของ engine และผลโดยตรงของ facade ยังคงเป็น
synchronous callback จะเห็น **state ปัจจุบันของ cache** การส่งแบบเรียงลำดับไม่ได้ให้ snapshot ของ cache ณ เวลา
ที่เกิด event payload ของ event ที่ immutable บันทึก event ไว้ ขณะที่ cache อาจสะท้อนการเปลี่ยนแปลงที่เกิดหลังจากนั้น
แล้ว การให้การอัปเดต indicator ไปพร้อมกับการส่ง event จะรักษาลำดับของมันเมื่อเทียบกับ callback ที่สอดคล้องกัน

ตัวอย่างเช่น engine อาจใช้ order update E1 เข้าคิว callback ของมัน แล้วใช้ E2 ก่อนที่ callback นั้นจะถูก drain
callback ของ E1 จะได้รับ payload ที่ immutable ของ E1 แต่การค้นใน cache อาจคืนสถานะของ order หลัง E2 ให้ใช้
payload สำหรับข้อเท็จจริงเกี่ยวกับ E1 และใช้ cache สำหรับ runtime state ปัจจุบัน

author callback ต้องมีสิทธิ์รับทั้ง **ตอนที่ event มาถึงและตอนส่ง** การ stop, reset หรือการปลดออก (retirement)
ต้องไม่พา callback เก่าเข้าไปยังการลงทะเบียนใหม่หรือ [lifecycle](../concepts/actors.md#lifecycle) generation ใหม่

## Bounded progress {#bounded-progress}

**drain** ประมวลผล callback ที่เข้าคิวตามลำดับ โดยพยายามส่งเมื่อได้รับอนุญาต callback ที่ค้างอยู่เป็นส่วนหนึ่งของ
เงื่อนไขการ drain ของ runtime การทำงานแบบ live ใช้ drain batch แบบมีขอบเขตและ yield ระหว่าง batch ส่วน backtest
ทำงานที่ค้างอยู่ให้เสร็จก่อนเลื่อนเวลาจำลอง
chain ของ callback ที่ควบคุมไม่ได้จะทำให้เกิด fault อย่างชัดเจน

queue overflow จะบันทึก fatal error และหยุดการทำงานที่ safe boundary ต้องไม่ทิ้ง callback แบบเงียบ ๆ และต้องไม่
ขัดจังหวะการดำเนินการระหว่างที่ผลแบบ synchronous ยังทำไม่เสร็จ callback dispatch ไม่ rollback ผลที่เกิดขึ้นแล้ว

## การออกแบบ drain boundary {#drain-boundary-design}

การ integrate กับ runtime ต้องรวมการ drain callback ไว้ที่ ownership boundary ที่ชัดเจนจำนวนน้อย **runtime เป็น
เจ้าของการจัดตาราง drain** method ของ dispatch และ flush แต่ละตัวต้องไม่มีนโยบาย drain ของตัวเอง วิธีนี้ทำให้การปล่อย
borrow, ลำดับของ callback และการจัดการ failure อยู่ในไม่กี่จุด และจำกัดการเรียก drain ซ้ำ, การส่งต่อแบบ async และ
การเดินสาย error

แต่ละ boundary ต้องระบุ:

- borrow ของ component, engine และ cache ตัวใดที่สิ้นสุดแล้ว
- callback ใดที่อาจทำงานต่อไป
- เส้นทาง lifecycle ที่มีอยู่เส้นทางใดที่จัดการ failure

ให้การปิดระบบและการเก็บกวาด ownership อยู่ภายใต้เจ้าของ lifecycle นั้น callback dispatch ต้องไม่สร้าง lifecycle
หรือ recovery state machine คู่ขนานขึ้นมา เมื่อเกิด failure ให้หยุด dispatch ต่อ เก็บ error ไว้ และปล่อย ownership ที่
retain ไว้ตามลำดับที่สัญญาการเก็บกวาดกำหนด

ใช้ชุด boundary ที่เล็กที่สุดที่ตอบข้อกำหนดด้านลำดับ, progress และ ownership การย้าย drain ออกไปด้านนอกทำได้ก็ต่อเมื่อ
ยังรักษาข้อกำหนดเหล่านั้นไว้ ให้ตรวจสอบตำแหน่งของ boundary ด้วย test ที่ระบุลำดับ callback อย่างแม่นยำ รวมถึงการทำงาน
ต่อเนื่องข้าม drain pass แบบมีขอบเขต จำนวนจุด drain ที่น้อยลงอย่างเดียวไม่ได้พิสูจน์ความถูกต้อง ใน synchronous core
และ backtest ลำดับ callback ที่ deterministic ยังคงเป็นเกณฑ์ที่ต้องผ่านก่อนเปิดใช้งาน

### boundary ของการจัดตารางและ ownership {#scheduling-and-ownership-boundaries}

ให้ `actor::drain_callbacks(budget)` เป็นการดำเนินการร่วม: ประมวลผล batch แบบมีขอบเขต และ return ว่ายังมี
callback เหลืออยู่หรือ dispatch ล้มเหลว settlement ของ backtest และการจัดตารางของ live เป็นเจ้าของการวนซ้ำรอบ
การดำเนินการนั้น ให้เลือกใช้ loop เล็ก ๆ ที่เฉพาะของแต่ละ runtime แทน scheduler ร่วมที่มี runtime mode,
lifecycle state หรือ policy object แชร์โค้ดเพิ่มก็ต่อเมื่อช่วยลดการซ้ำซ้อนโดยไม่ต้องเพิ่มกลไกเหล่านั้น

แผนผัง boundary แยกการ integrate และกฎ ownership ที่มีอยู่ ออกจากหลักฐานที่ต้องมีเพื่อเปิดใช้งาน queued callback
column สุดท้ายระบุข้อกำหนดของ test ไม่ใช่การตรวจสอบที่ทำเสร็จแล้ว

| Runtime และ boundary            | การ integrate ที่มีอยู่                                                                  | ownership ที่ retain ไว้และการปล่อย                                                       | หลักฐานที่ต้องมีเพื่อเปิดใช้งาน                                                                              |
| ------------------------------- | --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Backtest startup                | settle data command และ callback ปล่อย trading command ไว้ในคิว                        | engine borrow ตอน startup สิ้นสุดก่อน settlement                                          | ลำดับ callback ตอน startup ก่อน input แรก trading command ยังอยู่ในคิว                                     |
| Backtest event settlement       | `drain_command_queues` หลังงานของ engine และหลัง timer handler แต่ละตัว                | engine และ venue borrow ที่ครอบอยู่สิ้นสุดก่อน settlement                                 | command ที่ callback สร้างต้อง settle ก่อน timer ถัดไปหรือการเลื่อนเวลา                                    |
| Backtest normal stop            | settle งานที่ถึงกำหนด, stop trader, settle command ที่เกิดจากการ stop แล้วจึง stop engine | trader และ engine borrow สิ้นสุดก่อนส่ง callback                                         | ลำดับที่แม่นยำของ command และ callback ที่เกิดจากการ stop                                                  |
| Backtest failure และ teardown   | การเก็บกวาดเมื่อ abort และการล้าง callback แบบมี guard                                 | งาน synchronous ที่ถูก own ปล่อยก่อนล้าง capture                                          | การทิ้งเมื่อ fatal abort และการปฏิเสธขณะที่ยังมี external root                                             |
| Live startup                    | drain ครั้งแรกของ running loop หลัง startup สำเร็จ                                      | future ของการเชื่อมต่อและ mass-status ถือ engine borrow จนกว่าจะเสร็จหรือถูกยกเลิก          | instrument มาก่อนการเชื่อมต่อ execution callback ทำงานหลัง borrow ที่ครอบอยู่สิ้นสุด                      |
| Live event settlement           | pass ของ running loop แบบมีขอบเขต พร้อม yield และตรวจ stop                              | borrow ของ event handler แบบ synchronous สิ้นสุดก่อน drain ถัดไป                          | ลำดับ callback ที่แม่นยำ, การต่อ batch, สิทธิ์ในการ stop และการจัดตารางเฉพาะ channel ที่ไม่เปลี่ยน         |
| Live report request             | ทำ report family แบบทีละตัว command ใช้ client access แบบ shared การอัปเดต instrument ถูกเลื่อน | shared client borrow คร่อมหลายรอบของ loop และสิ้นสุดก่อน flush instrument ที่เลื่อนไว้ | พัก report ไว้ระหว่างทดสอบเส้นทาง callback จริง พิสูจน์ความปลอดภัยของการเข้าถึงและ progress ที่ตามมา    |
| Live normal stop                | deadline ที่เหลือและ dispatch ที่ buffer ไว้รอบสุดท้าย standalone runner ยังถือ channel | report future ถูก drop ก่อน flush instrument ที่เลื่อนไว้และก่อนตัดการเชื่อมต่อ client     | การเสร็จของ report ที่ค้าง, timeout และการยกเลิกตอนปิดระบบต้องรักษาลำดับ                                 |
| Live fatal failure และ disposal | การปิดระบบที่มีอยู่จัดการ failure disposal ล้างโดยไม่ส่ง callback                        | disposal ของ kernel มาก่อนการปล่อย runner ที่ retain ไว้และการล้างแบบมี guard              | งานที่เกิดจากการ stop ถูกทิ้ง external root ยังคงปฏิเสธการล้าง                                            |
| Live hosted-run abandonment     | การ drop future ของ run อาจทำให้การปิดระบบไม่สมบูรณ์                                    | report future ถูก drop ก่อน node return disposal ภายหลังจะพยายามเก็บกวาด                  | แยกการยกเลิกแบบ await ออกจากการทิ้ง ตรวจสอบการปล่อย capture และการเก็บกวาดแบบมี guard                    |

ownership ของ live ถูก implement โดย [execution-client facade](../../crates/live/src/execution/client.rs),
[lifecycle ของ report task](../../crates/live/src/node/reconciliation.rs) และ
[lifecycle ของ node](../../crates/live/src/node/mod.rs)

### boundary ของ backtest {#backtest-boundaries}

settlement ของ backtest เป็นแบบอย่างของการจัดตารางแบบรวมศูนย์ ไม่ใช่การ drain ครั้งเดียวตอนจบ timestamp timer
handler จะ settle งานของตัวเองก่อน handler ถัดไป แม้จะอยู่ที่ timestamp เดียวกัน การประมวลผลของ venue จะปล่อย
borrow ก่อน settlement การย้าย drain ทั้งหมดไปที่ loop ชั้นนอกสุดจะเปลี่ยน boundary เหล่านี้ pass ของ data command
และ callback ตอน startup ก็ใช้ settlement แบบไม่จำกัดไม่ได้ โดยไม่เปลี่ยนเวลาที่ trading command ทำงาน

### Live startup และ lifecycle แบบ standalone {#live-startup-and-standalone-lifecycle}

#### ลำดับตอน startup {#startup-ordering}

รักษาลำดับตอน startup: instrument event ต้องถึง cache ก่อนที่ execution client จะเชื่อมต่อ และก่อนจัดการ
subscription command

หลัง live startup สำเร็จ `run_with_mode` จะถึง drain ครั้งแรกของ running loop หลังจาก trader start, plug-in
start และ system dispatch ตอน startup ก่อนเลือกข้อความถัดไปจาก channel ให้ใช้ boundary ที่มีอยู่นี้ เว้นแต่จะมี
producer และข้อกำหนดด้านลำดับที่ระบุได้ว่าจำเป็นต้อง drain เร็วกว่านั้น

future ของการเชื่อมต่อถือ mutable engine borrow คร่อม await ของมัน เช่นเดียวกับ mass-status request ระหว่าง
startup reconciliation การที่สิ่งเหล่านี้เสร็จอย่างเดียวไม่ใช่เหตุผลพอที่จะเพิ่มจุด drain ความล้มเหลวของ replay และ
startup ใช้ทางออกแยกกัน และต้องมีการพิสูจน์ ownership ของตัวเอง

#### ข้อจำกัดของ lifecycle แบบ standalone {#standalone-lifecycle-restrictions}

การเรียก `LiveNode::start` และ `stop` แบบ standalone ไม่ได้รัน event loop ต่อเนื่อง `start` จะ return โดยยังถือ
runner ไว้ ส่วน stop หรือ abort จะประมวลผลข้อความที่ค้างอยู่ของ runner เส้นทางนี้ยังรองรับในฐานะส่วนประกอบสำหรับ
test และการฝัง (embedding) พร้อม callback แบบ synchronous ที่มีอยู่ การ drain เพียงครั้งเดียวใน
`finish_startup_trader` ไม่สามารถให้การทำงานต่อเนื่องของ queued callback ได้

การเปิดใช้งาน queued callback ครั้งแรกสำหรับ `LiveNode` จำกัดไว้เฉพาะ `run`/`run_with_mode` ของ Rust ซึ่งเป็น
เจ้าของ event loop ของ node การส่งแบบเข้าคิวผ่าน `start`/`stop` แบบ standalone ถูกเลื่อนออกไป ก่อน admission ของ
callback การเปิดใช้งานต้องปฏิเสธการส่งแบบเข้าคิวเมื่อ startup แบบ standalone และแจ้ง error ที่ชัดเจน ต้องไม่ถอยกลับไป
ใช้การส่งแบบ synchronous อย่างเงียบ ๆ และต้องไม่ปล่อยให้ callback ไม่ถูกส่ง ข้อจำกัดนี้เกี่ยวกับ startup แบบ standalone
ไม่ใช่คำขอ stop ผ่าน handle ของ node ที่กำลังรัน event loop อยู่

การส่งแบบเข้าคิวให้ actor และการปฏิเสธนี้ยังไม่ได้ implement การบังคับใช้ข้อจำกัดของ lifecycle เป็นข้อกำหนดก่อน
เปิดใช้งาน และไม่เปลี่ยน API แบบ standalone ที่มีอยู่

#### การยกเลิกและการทิ้ง hosted run {#hosted-run-cancellation-and-abandonment}

การยกเลิก hosted run แบบ await จะขับให้เกิดการปิดระบบอย่างเรียบร้อย การทิ้ง run นั้นขณะที่ host loop ยังทำงานอยู่
อาจ drop future ของมันก่อนการปิดระบบจะเสร็จ ในเส้นทางนั้นไม่รับประกันการส่ง callback การทดสอบก่อนเปิดใช้งานต้อง
ตรวจสอบการปล่อย ownership และการเก็บกวาดตอน disposal แบบมี guard หลังการทิ้ง แยกจากการปิดระบบอย่างเรียบร้อย

เมื่อ run future ถูก drop จะปล่อย report borrow แต่ไม่ได้รันการเก็บกวาดการยกเลิก report task เอง ไม่ flush การอัปเดต
instrument ที่เลื่อนไว้ และไม่ลบ marker ของ targeted query

### Live stop และการเก็บกวาดขั้นสุดท้าย {#live-stop-and-terminal-cleanup}

`LiveNode::finalize_stop` ไม่สามารถเป็นเจ้าของการ teardown ของ callback เพียงลำพังได้: ผู้เรียกยังถือ channel
receiver และการปิดระบบปกติจะประมวลผล event ที่ buffer ไว้หลังจากนั้น เส้นทาง `start`/`stop` ยังถือ runner ไว้
ส่วน `run_with_mode` เป็นเจ้าของ receiver ภายในตัวเอง เจ้าของการเก็บกวาดขั้นสุดท้ายคือ `LiveNode::dispose`
ไม่ใช่การ stop ปกติหรือการจบของ running loop

:::info การเก็บกวาด ownership ขั้นสุดท้าย
`LiveNode::dispose` เก็บกวาด callback ตามลำดับนี้:

1. dispose kernel ตรวจให้แน่ใจว่าไม่มี engine หรือ component borrow เหลืออยู่ก่อนปล่อย runner
1. ปล่อย runner ที่ retain ไว้ และทิ้งข้อความที่ค้างอยู่ รวมถึงงานที่ stop callback เข้าคิวไว้ระหว่าง disposal
1. log callback failure ที่ latch ไว้ (ถ้ามี) แล้วพยายามล้าง callback state log การปฏิเสธการเก็บกวาดโดยไม่บังคับ reset

disposal ต้องไม่กลายเป็น loop ส่ง callback อีกตัว ให้รักษาการ stop ปกติและการนำ runner กลับมาใช้ใหม่ไว้ อย่าปิด
channel ใน stop ปกติหรือตอนจบ running loop เพื่อให้การล้างสำเร็จ disposal เป็นเจ้าของการปล่อย runner ที่ retain ไว้
:::

การ drop runner ไม่ได้พิสูจน์ว่า root ทั้งหมดถูกปล่อยแล้ว การล้างยังต้องปฏิเสธ scope ที่ active หรือ root ที่ถูก
retain จากภายนอก และรายงาน failure โดยไม่บังคับ reset การล้างที่ถูกปฏิเสธจะคง callback state ไว้ตามเดิม สามารถลอง
disposal ใหม่ได้หลังปล่อย ownership ที่ขวางอยู่แล้ว ให้ตรวจสอบว่า:

- ข้อความที่มี root ใน runner ที่ retain ไว้ขวางการล้างก่อน disposal และถูกปล่อยโดย disposal
- ข้อความที่มี root ซึ่งเกิดจากการ stop ถูกทิ้งโดย disposal โดยไม่ส่ง callback
- fatal latch ของ running loop ถูกล้างตอน disposal เมื่อ ownership อนุญาต
- ownership จากภายนอกยังคงขวางการล้าง

### การตรวจสอบ boundary ที่ต้องทำ {#required-boundary-verification}

#### ข้อกำหนดด้านลำดับ {#sequence-requirements}

การเปลี่ยน boundary ต้องมี assertion ของลำดับที่แม่นยำสำหรับ:

- data command และ callback ตอน startup ของ backtest ก่อน input แรก โดย trading command ยังอยู่ในคิว
- การประมวลผล instrument ของ live ก่อนการเชื่อมต่อ execution รวมถึงลำดับใด ๆ ที่เส้นทางที่เปิดใช้งานต้องการ
- timer ที่ timestamp เดียวกันและ command ที่ callback สร้าง ก่อน timer ถัดไปหรือการเลื่อนเวลา
- การทำงานต่อเนื่องข้าม pass แบบมีขอบเขต, สิทธิ์ในการ stop ของ live และการจัดตารางเฉพาะ channel ที่ไม่เปลี่ยน
- งานที่เกิดจาก stop ปกติเทียบกับการทิ้งเมื่อ fatal abort รวมถึงการปฏิเสธการเก็บกวาดขณะที่ยังมี root
- การปฏิเสธการส่งแบบเข้าคิวเมื่อ live startup แบบ standalone ก่อน admission ของ callback ขณะที่ยังรักษาพฤติกรรม
  synchronous แบบ standalone และการปิดระบบผ่าน handle ของ node ที่กำลังรันอยู่

การตรวจลำดับเหล่านี้เป็นข้อกำหนดก่อนเปิดใช้งาน ไม่ใช่หลักฐานว่า implement การส่งแบบเข้าคิวของ live แล้ว ให้ทดสอบ
เส้นทางที่เปิดใช้งานผ่าน component จริง test ของ dispatcher ภายในเพียงอย่างเดียวไม่ได้พิสูจน์ว่า runtime borrow
สิ้นสุดที่ boundary ที่เลือก

#### การบังคับใช้สิทธิ์การเข้าถึง {#access-enforcement}

สำหรับแต่ละเส้นทางที่เปิดใช้งาน ให้จองผ่านกลไก admission ที่มีอยู่ และขอ checked component access สำหรับการเรียก
ระบุ engine, client และ cache access ทุกตัวที่ callback อาจเข้าถึงได้: allocation guard ของ actor ไม่ได้ป้องกัน
borrow ที่แยกต่างหากเหล่านั้นหรือ unchecked handle implementation ต้องสร้างการเข้าถึงที่เข้ากันได้ก่อนเรียก callback
การได้ actor guard สำเร็จเพียงอย่างเดียวยังไม่พอ

#### Test สำหรับ report ที่ค้างอยู่ {#pending-report-tests}

ใช้ synchronization point สำหรับ test อย่างชัดเจนเพื่อพัก report request ไว้ ทดสอบผ่านเส้นทาง callback จริง
assert ลำดับของ event และ command อย่างแม่นยำทั้งก่อนและหลังปล่อย report ครอบคลุมทั้งการเสร็จสำเร็จ, timeout,
การยกเลิกตอนปิดระบบ และการทิ้ง hosted run ในกรณีที่เส้นทางนั้นไปถึงได้

shared command access ทำงานต่อได้ขณะที่ report ค้างอยู่ mutable client access ต้องรอจนกว่า borrow ที่ขัดแย้งกัน
จะสิ้นสุด ถ้าเส้นทางต้องเลื่อนออกไป ให้รักษาลำดับของ publication และ progress ของ report future ไว้ อย่าเปลี่ยน
busy head ที่ safe drain boundary ที่ประกาศไว้ให้กลายเป็นการ retry แบบไม่จำกัด
[กฎ fatal stalled-delivery](#draining-and-progress) ยังคงใช้อยู่

[test ของ live node](../../crates/live/tests/integration/node.rs) ที่มีอยู่ครอบคลุมการทำ report แบบทีละตัว,
การอัปเดต instrument ที่เลื่อนไว้ และการเก็บกวาดเมื่อ timeout และปิดระบบ แต่ไม่ได้ทดสอบการส่งแบบเข้าคิวให้ actor
ใน production และใช้แทน test ก่อนเปิดใช้งานเฉพาะเส้นทางเหล่านี้ไม่ได้

## การ integrate กับ runtime {#runtime-integration}

[test ของการส่งต่อ root](../../crates/common/src/actor/dispatch.rs) ครอบคลุม continuation ที่ retain ไว้ รวมถึง
data command และ trading command ใน runtime แบบ synchronous และ live รวมถึง trading child ที่เลื่อนไว้ event
channel ของ data และ execution ใน live ก็รักษา root ผ่านการข้าม channel หลายแบบผสมกันและ
[startup buffering](../../crates/live/src/node/mod.rs)

### Backtest {#backtests}

backtest เรียก boundary drain หลัง data command batch ตอน startup, หลัง settlement ของ command และ execution
event และหลัง timer handler pass ของ callback แต่ละครั้งประมวลผล slot จำนวนจำกัด settlement จะทำต่อไปตราบที่ยังมี
callback หรือ synchronous command ค้างอยู่ ก่อนเลื่อนเวลาจำลอง root ที่ retain ไว้อย่างเดียวไม่ทำให้ settlement
ทำงานต่อ ตอน startup ยังปล่อย trading command ไว้ในคิวจนถึง settlement ปกติ

dispatch error จะ propagate ออกจากการทำงานของ backtest และทำให้เกิดการเก็บกวาดแบบ abort การ start trader ที่
ล้มเหลวก็จะ stop engine และทิ้ง synchronous command ที่ค้างอยู่ด้วย abort (`abort_run`), reset และ disposal
ปล่อย synchronous command batch ก่อนล้าง callback capture งานที่ถูก retain จากภายนอกยังอาจขวางการเก็บกวาด callback
ได้ การจบแบบปกติ (`end_impl`) จะ drain งานที่เหลือก่อนล้าง dispatcher

### Running loop ของ live {#live-running-loops}

running loop ของ live จะ drain callback batch แบบมีขอบเขตหนึ่ง batch ก่อนเลือก event ถัดไป และ yield เมื่อยังมี
callback ค้างอยู่ callback ที่ค้างจะทำงานต่อโดยไม่ต้องรอข้อความจาก channel อีก สัญญาณ stop ยังมีสิทธิ์ระหว่าง batch
ลำดับความสำคัญของ channel ที่มีอยู่และการ yield เฉพาะ channel ยังคงไม่เปลี่ยน

เมื่อ callback ล้มเหลว:

- **Standalone runner**: return error โดยไม่ปิด channel หรือทิ้งข้อความใน channel
- **Live node**: เรียกการปิดระบบที่มีอยู่ และข้ามทั้งช่วงเวลา residual event และ dispatch ที่ buffer ไว้รอบสุดท้าย
  ทิ้งข้อความที่ค้างใน channel รวมถึง command ที่ stop callback เข้าคิวไว้ เมื่อ receiver ถูก drop

ทั้งสองเส้นทางยังคง fatal callback latch ไว้ และไม่ล้าง callback ownership

flush ตอน startup และ residual flush ไม่ได้ integrate การ drain callback disposal ปล่อยงานที่ถูก own ก่อนพยายาม
เก็บกวาด callback โดยไม่ส่ง callback การเปิดใช้งานยังต้องมีการจัดตารางและ ownership ที่ครอบคลุมเส้นทาง lifecycle
ของ live อย่างชัดเจน โดยไม่สมมติว่า lifecycle method ทุกตัวต้องมี drain

report future ของ live อาจถือ shared client borrow คร่อมหลายรอบของ loop การ drain ที่ต้น loop อย่างเดียวไม่ได้
พิสูจน์ความปลอดภัยสำหรับ callback ที่ต้องการ mutable client access การเปิดใช้งาน queued callback ต้องคำนึงถึง
borrow ที่ถูกถือไว้เหล่านั้น

### ข้อกำหนดก่อนเปิดใช้งาน {#activation-requirements}

boundary เหล่านี้ไม่ได้เปิดใช้งานการส่งแบบเข้าคิวให้ actor ก่อนเปิดใช้งาน การ integrate กับ runtime ต้อง:

- รักษา [ingress boundary ที่เป็นอิสระ](#sender-types-and-ingress) เมื่อเปิดใช้งานเส้นทาง callback เพิ่มเติมหรือ
  นำ invocation storage แบบใช้ซ้ำได้เข้ามา
- ทำ drain boundary ของ lifecycle ของ live และการครอบคลุม ownership นอกเหนือจาก terminal disposal ให้สมบูรณ์
- บังคับใช้ [ข้อจำกัดของ lifecycle ของ live](#live-startup-and-standalone-lifecycle) ก่อน admission ของ queued callback
- สร้างความปลอดภัยของ ownership สำหรับทุกเส้นทาง callback ที่เปิดใช้งาน
- ตรวจสอบ queued callback ผ่าน lifecycle ของ runtime ทั้ง backtest และ live อย่างครบถ้วน
- พิสูจน์ลำดับ callback ที่ deterministic ผ่าน component ใน synchronous core และ backtest รวมถึง publication
  ที่ซ้อนกัน, fan-out, command ที่ callback สร้าง, งานที่ timestamp เดียวกัน และการทำงานต่อเนื่องข้าม drain batch
  จำนวนนับและ balance ตอนจบอย่างเดียวไม่ได้พิสูจน์ลำดับ

trading message สาธารณะและเส้นทาง `dispatch()` โดยตรงไม่ได้พา callback context ข้าม thread synchronous queue
เป็นเจ้าของ context ของมันเป็นการภายใน command channel ของ live ใช้ envelope ที่อธิบายด้านล่าง การ drain ของ
runtime อย่างเดียวไม่ได้พิสูจน์การ integrate กับ lifecycle ของ live ที่สมบูรณ์หรือความเท่าเทียมของ backend

## Dispatch primitive ภายใน {#private-dispatch-primitives}

actor module มี primitive ภายในสำหรับ access, admission, publication, invocation และ storage data command และ
trading command รักษา [callback root](#callback-roots-and-budgets) ไว้ใน runtime แบบ synchronous และ live
ข้อความ data, execution และ system ของ live ที่ส่งออกภายในเครื่อง รวมถึง time event รักษา root ผ่าน channel ของ
ตัวเอง

การค้น actor, component access และเส้นทางของ message bus ใน production ไม่ได้ใช้การส่ง callback แบบเข้าคิว
การจะเปิดใช้งานการส่งแบบนั้น runtime boundary ต้องรองรับ ownership ก่อน primitive และการ drain ของ runtime
อย่างเดียวไม่ได้พิสูจน์ความปลอดภัยของ ownership ใน runtime และไม่ได้พิสูจน์ความเท่าเทียมของ callback แบบ native,
direct และ dynamic

### Publication และ admission {#publication-and-admission}

publication scope จะจอง **ordinal** (หมายเลขลำดับ) ก่อนที่ subscriber แบบ synchronous จะทำงาน การจอง callback
เรียงตาม publication ordinal แล้วตามด้วย admission ordinal ดังนั้น publication ที่ซ้อนอยู่จะตามหลังผู้รับที่ค้างอยู่
ทุกตัวของ publication ที่ครอบอยู่

scope ซ้อนกันและถูก drop ตามลำดับ stack การ drop scope จะคืน publication state โดยไม่ส่ง callback การ unwind
ระหว่าง publication จะ latch fatal error

`publish_any` และเส้นทาง typed publication ที่ใช้ร่วมกันจะเข้า scope ก่อนที่ bus tap และ subscriber แบบ synchronous
จะทำงาน `try_publish_any` ทำเช่นเดียวกันหลังตรวจสิทธิ์เข้าถึง bus ครั้งแรก scope รักษาลำดับการจองข้าม publication
แบบ typed และแบบ `Any` ที่ซ้อนกัน การ unwind ผ่านเส้นทางเหล่านี้จะ latch fatal dispatch error แม้ไม่มี callback
เข้าคิว หรือผู้เรียกจะ catch panic ภายหลัง

publication scope เรียงลำดับการจอง แต่ไม่ได้ admit callback แบบ canonical หรือ drain มัน

admission จองจำนวน, storage ที่ทราบขนาด และ queue slot **ก่อนที่ผู้เรียกจะสร้าง capture ที่ถูก own**:

- การจองที่สำเร็จจะรับ capture ของมัน แม้การดำเนินการอื่นจะ latch failure ภายหลัง
- การจองที่ยังไม่เสร็จจะขวางการส่งและ teardown ที่ตามมา
- การยกเลิกจะเหลือ slot ไว้ให้ drain หรือ teardown ที่ชัดเจนเป็นผู้ปล่อย
- การจองของ actor เก็บ identity ของการลงทะเบียนไว้ ดังนั้นการแทนที่หรือการลงทะเบียนซ้ำของ allocation เดียวกันจะข้าม
  handler ที่ล้าสมัย การส่งที่ถูกข้ามยังนับเป็นการส่งที่เสร็จแล้วใน budget ของ root

### การ drain และ progress {#draining-and-progress}

drain ประมวลผลได้มากที่สุดเท่ากับ **slot budget** ที่ให้มา รวมถึง slot ที่ถูกยกเลิก คิวจะมี **busy head** เมื่อ
callback ตัวแรกไม่สามารถได้การเข้าถึงแบบ exclusive ต่อ component ของมัน ซึ่งจะขวางการส่งที่ตามมา

boundary drain ภายในเพิ่มสัญญาที่เข้มงวดกว่า drain แบบชัดเจน ผู้เรียกต้องสร้าง delivery point ที่ปลอดภัย:

| ผลของ drain แบบชัดเจน       | ความหมาย                                                                 | พฤติกรรมของ boundary drain                 |
| --------------------------- | ------------------------------------------------------------------------ | ------------------------------------------ |
| Empty queue                 | ไม่มี slot ในคิวเหลืออยู่                                                | return ตามปกติ                             |
| Exhausted slot budget       | ยังมี slot เหลือหลัง pass                                                 | return ตามปกติ ไม่ถือว่าเป็น stall          |
| Deferred entry              | publication, recursive drain, teardown หรือ checked access ขวางการเข้า   | รายงานว่ามีงาน active ไม่ latch failure     |
| Unfinished head reservation | การจองตัวแรกยังไม่มี capture                                             | รายงานว่ามีงาน active ไม่ latch failure     |
| Busy head                   | ไม่สามารถเข้าถึง component แบบ exclusive ได้                              | latch fatal stalled-delivery error         |

deferred entry ไม่ทำงานใด ๆ unfinished head reservation หรือ busy head อาจเกิดหลังการส่งก่อนหน้าใน pass เดียวกัน
**ผลลัพธ์ที่เป็น error ไม่คืนจำนวนการส่ง** busy head ที่ boundary ถือเป็น fatal แม้ callback ก่อนหน้าจะมี progress
แล้วก็ตาม การใช้ slot budget จนหมด รวมถึงกับ slot ที่ถูกยกเลิก ไม่ถือว่าเป็น stalled delivery

คิวว่างไม่ได้หมายความว่า root ที่ retain ไว้ทั้งหมดถูกปล่อยแล้ว การนับ ownership อย่างเดียวไม่ใช่เหตุผลที่จะ drain ต่อ
boundary drain จะเก็บ fatal error ตัวแรกไว้ และปล่อยให้ capture ในคิวยังถูก own อยู่จนกว่าจะ teardown อย่างชัดเจน

:::warning ปล่อย borrow ที่ครอบอยู่ก่อน drain
การทำลาย guard เพียงปล่อยสิทธิ์การเข้าถึง ไม่ได้ drain callback ผู้เรียกต้องสิ้นสุด borrow ของ engine, cache และ
borrow อื่นที่ไม่ได้ติดตามที่ครอบอยู่ก่อน drain ด้วย
:::

#### การ drain ระหว่าง unwind {#draining-during-unwind}

drain ที่สำเร็จตามปกติแต่ถูกเรียกเข้าระหว่าง panic unwinding จะ latch fatal delivery failure ตอนออก แม้คิวจะว่าง
ผลลัพธ์ที่ return ยังเป็นสำเร็จ failure นั้นตรวจพบได้ผ่านการตรวจ callback failure และจะหยุด boundary drain ครั้งถัดไป

### Callback root และ budget {#callback-roots-and-budgets}

**root** เป็นเจ้าของ delivery budget ของ publication หรือการจองที่เข้ามา และงาน callback ทั้งหมดที่มันก่อให้เกิด
ลำดับของ publication ไม่ขึ้นกับว่า root ใดเป็นเจ้าของงาน

- publication และการจองที่ซ้อนอยู่สืบทอด root ที่ active
- publication ที่ไม่มี root ที่ active จะเริ่ม root ใหม่เมื่อ admit callback ครั้งแรกหรือส่ง command ครั้งแรก
  scope ที่ว่างจะไม่จอง root
- นอก publication scope การ admit แต่ละครั้งจะเริ่ม root แยก เว้นแต่มี root ที่ active อยู่แล้ว

#### Delivery budget เทียบกับ slot budget {#delivery-budget-versus-slot-budget}

การส่งที่เสร็จแต่ละครั้งนับหนึ่งครั้งใน budget ของ root ความพยายามที่ busy และ slot ที่ถูกยกเลิกไม่นับ แม้ slot ที่ถูก
ยกเลิกยังใช้ slot budget ของ drain

เมื่อถึงขีดจำกัดของ chain dispatch จะ latch fatal runaway error ก่อนรัน callback ที่พร้อมตัวถัดไป callback ที่ถูกขวาง
จะอยู่ในคิวรอ teardown อย่างชัดเจน **fatal error หยุด dispatcher ทั้งหมด** แม้ root แต่ละตัวจะมี budget แยกกัน

### Retained work และการเก็บกวาด {#retained-work-and-cleanup}

queued slot, active scope และ invocation work ที่ retain ไว้ทำให้ root ยังมีชีวิตอยู่ **คิวว่างไม่ได้ reset budget
ของ root ที่ยังอยู่** storage ที่ retain ไว้จะ capture root ที่ active หรือเริ่ม root ใหม่ถ้าไม่มี การขยาย storage นั้น
ยังคงรักษา root เดิม

#### การทำงานต่อและการใช้ storage ซ้ำ {#resumption-and-storage-reuse}

การทำ retained work ต่อผ่าน `with_chain` ทำให้ root ของมัน active ระหว่างการเรียก การ return หรือ unwind จะคืน
root ที่ครอบอยู่ invocation batch ใช้กลไกนี้กับค่าที่ retain ไว้แต่ละค่า การทำงานต่อไม่นับเป็นการส่งในตัวเอง
invocation batch แต่ละตัวเป็นของการ preparation และ invocation หนึ่งครั้ง allocation ที่อยู่เบื้องหลังไม่ได้เลือก root
ของค่าที่มาทีหลัง: ค่าแต่ละค่าจะ capture context ที่ active ตอน admission ของตัวเอง

#### การ fire ของ timer ที่เป็นอิสระ {#independent-timer-firings}

การลงทะเบียน callback และ causal ownership มีอายุแยกกัน การใช้ timer callback ที่ลงทะเบียนแล้วซ้ำไม่ได้ใช้ budget
ของการ fire ครั้งก่อนซ้ำ time-event envelope ของ live ที่เป็นอิสระแต่ละตัวจะสร้าง scope ของตัวเองก่อนค้นหาและเรียก
callback แม้การส่งจะเกิดภายใน scope อื่นที่ active อยู่ continuation ที่ retain ไว้ยังคงใช้ root เดิม การใช้ storage ซ้ำ
อย่างเดียวไม่ได้ทำให้ event ที่ไม่เกี่ยวข้องกลายเป็น continuation

#### การทำลายและการปล่อย {#destruction-and-release}

destructor ของ capture ที่ส่งแล้วทำงานโดยมี root ของการส่งนั้น active รวมถึงระหว่าง unwind ค่าใน batch ที่ยังไม่ถูกเรียก
ก็คืน root ของตัวเองเมื่อถูกทำลาย

root และ storage charge ของมันจะถูกปล่อยเมื่อเจ้าของคนสุดท้าย drop การยกเลิก, batch ที่ล้มเหลว และ teardown ที่ชัดเจน
ใช้การเก็บกวาด ownership แบบเดียวกับ storage อื่นที่ retain ไว้ retained work ขวาง teardown จนกว่าจะถูกปล่อย

### การเก็บกวาดเมื่อ failure {#failure-cleanup}

overflow จะ latch fatal failure ตัวแรกและปฏิเสธการจองต่อ ๆ ไปโดยไม่สร้าง capture ผลแบบ synchronous ของ facade
อาจทำงานเสร็จก่อนที่ runtime จะรายงาน failure ที่ safe boundary queued slot ยังถูก own ไว้รอ teardown อย่างชัดเจน
invocation batch ที่ล้มเหลวจะปล่อย capture หลังจาก preparation ออกไปและ guard ของมันถูกปล่อยแล้ว

ลำดับของ teardown เป็นข้อบังคับ:

1. รายงาน fatal error ก่อนที่การล้างจะ reset latch ซึ่งเป็นความรับผิดชอบของผู้เรียก
1. สิ้นสุด publication, reservation, invocation, chain, drain และ access scope ที่ active ทั้งหมดก่อนเริ่ม teardown
   การล้างจะปฏิเสธ scope ที่ active และ root ที่ถูก retain จากภายนอก
1. แยก storage ออกก่อนปล่อย capture
1. ปล่อย capture

teardown ปฏิเสธ admission ที่เกิดจาก destructor ตลอดการล้าง

#### ความปลอดภัยของ preparation และ destructor {#preparation-and-destructor-safety}

invocation preparation เก็บเจ้าของ batch ไว้นอกการเรียกที่ขอ guard เมื่อถูกปฏิเสธหรือ unwind capture ที่โอนไปก่อนหน้า
จะยังอยู่จนกว่า guard เหล่านั้นจะถูกปล่อย ผู้เรียกจะจองก่อนสร้าง capture แต่ละตัว และขอ preparation guard ภายใน
การเรียกนั้น primitive ไม่สามารถควบคุมตัวแปร local ใด ๆ ที่โค้ดของผู้เขียนสร้างหรือทำลายเองขณะถือ borrow ได้
destructor ต้องไม่ panic ระหว่าง unwind ที่กำลังเกิดขึ้น

## Reentrancy และ dispatch diagnostics {#reentrancy-and-dispatch-diagnostics}

### ความขัดแย้งในการเข้าถึง component {#component-access-conflicts}

[ComponentAccessError](../../crates/common/src/component.rs) อธิบาย resource และการดำเนินการที่พยายามทำ เมื่อ
checked component-state access ล้มเหลว diagnostic เหล่านี้ใช้กับเส้นทาง synchronous อยู่แล้ว และไม่ได้หมายความว่า
การส่งแบบเข้าคิวเปิดใช้งานแล้ว ความขัดแย้งอาจเกิดจาก callback reentry หรือจาก borrow ปกติที่ถือไว้นานเกินไป
ข้อความไม่ได้ระบุผู้ถือ borrow หรือ call stack ของมัน

| Variant         | Prefix ของข้อความ                                                           | ความหมายและสิ่งที่ต้องทำ                                                          |
| --------------- | --------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| `NotRegistered` | `Cannot access {resource} during {operation}: the actor is not registered`  | การลงทะเบียนยังไม่ได้ให้ resource นั้น ให้ลงทะเบียน component ก่อนเข้าถึง          |
| `ReadConflict`  | `Cannot read {resource} during {operation}: it is already mutably borrowed` | ปล่อย exclusive borrow ที่มีอยู่ก่อนอ่าน                                           |
| `WriteConflict` | `Cannot modify {resource} during {operation}: it is already borrowed`       | ปล่อย shared หรือ exclusive borrow ที่มีอยู่ก่อนแก้ไข                              |

`try_cache_ref()` และ `try_clock_mut()` แบบ native คืน error เหล่านี้ ส่วน `cache_ref()` และ `clock_mut()`
ที่คู่กันจะ panic ด้วย diagnostic เดียวกัน ดู [native access method](../concepts/rust.md#dataactornative-methods)

facade ของ Rust แบบ portable ก็รายงานความขัดแย้งในการเข้าถึงเช่นกัน:

- method อ่านแบบ infallible ของ [CacheApi](../../crates/common/src/cache/api.rs) จะ panic ด้วย `ReadConflict` และ
  operation `cache read` ส่วน `get()` คืนความขัดแย้งผ่าน `anyhow::Result` ด้วย operation `get` การค้นหาแบบ
  fallible `try_*` คืนผ่าน variant `Access` ของ lookup error โดยใช้ชื่อ method เป็น operation เช่น `try_position`
- [ClockApi](../../crates/common/src/clock/api.rs) ที่มี native backing จะ panic ด้วย `ReadConflict` เมื่ออ่าน
  ขัดแย้ง setter ของ timer และ alert คืน `WriteConflict` ผ่าน `anyhow::Result` ส่วน `cancel_timer()` และ
  `cancel_timers()` จะ panic เมื่อเกิดความขัดแย้งนั้น diagnostic ของ clock ระบุ method ที่พยายามเรียก clock ที่มี
  handler เป็น backing จะส่งต่อให้ handler แทนการตรวจ borrow แบบ native เหล่านี้

วิธีวิเคราะห์ความขัดแย้ง:

1. หาการดำเนินการที่พยายามทำใน traceback
1. ไล่ borrow ที่ครอบอยู่ผ่าน synchronous publication, command dispatch หรือ lifecycle call
1. สิ้นสุด borrow นั้นก่อนที่โค้ดจะเข้า resource เดิมซ้ำได้ ในโค้ด native ให้อ่านหรือคัดลอก state ที่ต้องใช้ใน scope สั้น ๆ
   ปล่อย guard แล้วจึง publish หรือ dispatch

อย่าข้ามการตรวจ borrow หรือ retry ซ้ำ ๆ ขณะที่ scope ที่ขัดแย้งยัง active อยู่ การย้าย callback drain ต้องมีการพิสูจน์
ownership และลำดับตาม[การออกแบบ drain boundary](#drain-boundary-design)

operation ของ component lifecycle registry ปฏิเสธการเข้าถึงที่ทับซ้อนกันแยกต่างหากด้วย
`Component '{id}' is already mutably borrowed` การตรวจตาม ID นี้แยกจากทั้ง resource borrow และ callback
allocation guard ภายใน และไม่มีตัวใดพิสูจน์ความปลอดภัยสำหรับ unchecked access

### ความล้มเหลวของ callback dispatcher {#callback-dispatcher-failures}

[CallbackDispatchError](../../crates/common/src/actor/dispatch.rs) มี diagnostic ต่อไปนี้ `InvalidDestination`,
`Stalled` และ `Runaway` อธิบายความล้มเหลวของการส่งแบบเข้าคิว เส้นทางใน production ปัจจุบันไม่ได้ admit callback
เหล่านั้น variant เหล่านี้จึงยังเป็นพฤติกรรมของการ integrate ภายในและ test เท่านั้น diagnostic อื่นอาจเกิดได้แล้วผ่าน
publication scope, การนับ command root, runtime drain หรือการเก็บกวาด ขณะที่การส่งแบบเข้าคิวให้ actor ยังไม่เปิดใช้งาน

dispatcher เก็บ fatal failure ตัวแรกไว้ boundary drain ครั้งต่อ ๆ ไปจะรายงาน failure นั้นโดยไม่ส่ง callback เพิ่ม
ผลแบบ synchronous และผลของ callback ที่เกิดขึ้นแล้วจะไม่ถูก rollback

| Variant              | ข้อความ                                        | ความหมายและสิ่งที่ต้องทำ                                                                                                              |
| -------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `Overflow`           | `Callback storage limit exceeded`              | callback admission, retained storage หรือการจอง command root เกินความจุ ตรวจสอบ capture และ retained work                           |
| `InvalidDestination` | `Invalid callback destination`                 | checked actor delivery พบ type ไม่ตรงกัน ตรวจสอบ type ของ actor ที่ลงทะเบียนและปลายทางของ handler                                     |
| `SequenceExhausted`  | `Callback publication sequence exhausted`      | ตัวนับลำดับเดินต่อไม่ได้ หยุด dispatch และทำตามขั้นตอน teardown                                                                       |
| `PublicationUnwound` | `Callback publication unwound`                 | publication scope เกิด unwind ตรวจสอบ panic ต้นทาง แม้ผู้เรียกจะ catch ไว้                                                           |
| `DeliveryUnwound`    | `Callback delivery unwound`                    | drain scope ออกระหว่าง unwind ตรวจสอบ panic ต้นทาง                                                                                   |
| `Runaway`            | `Callback chain delivery limit exceeded`       | root หนึ่งใช้ delivery budget สะสมจนหมด ตรวจสอบ feedback loop ระหว่าง callback และ command                                           |
| `Stalled`            | `Callback delivery stalled at a safe boundary` | หัวคิวทำงานไม่ได้ที่ safe boundary ที่ประกาศไว้ แก้ไข access ownership หรือตำแหน่งของ boundary                                       |
| `Active`             | `Callback work or access is still active`      | การเข้า drain, การจองที่ยังไม่เสร็จ หรือ ownership ของ teardown ขวางการดำเนินการ ปล่อย scope หรือ retained root ที่ขวางอยู่           |

`Active` เป็นการปฏิเสธการดำเนินการ และไม่ได้ latch fatal failure ในตัวเอง running loop ของ live ยังคงออกเมื่อเจอ
drain error นี้ และ node จะเริ่มปิดระบบถ้ากำลังทำงานอยู่ busy head ที่ safe boundary จะ latch `Stalled` การ drain
ซ้ำ ๆ ไม่ใช่กลยุทธ์การกู้คืน การจองที่ยังไม่เสร็จอาจปฏิเสธ drain หลังจาก callback ก่อนหน้าทำงานไปแล้ว และผลลัพธ์ที่เป็น
error ไม่มีจำนวนการส่ง

เก็บ failure ต้นทางไว้ก่อนทำตาม[การเก็บกวาดเมื่อ failure](#failure-cleanup) ระหว่าง live disposal ข้อความ
`Callback dispatch failed before disposal cleanup: ...` รายงาน latch ที่มีอยู่ ส่วน
`Failed to clear callback dispatch during disposal: ...` รายงานการปฏิเสธการเก็บกวาด ให้ปล่อยงานที่ถูก retain
จากภายนอกก่อนลอง disposal ใหม่ อย่าบังคับล้าง ownership หรือถือว่าการเก็บกวาดเป็นการ rollback

## Synchronous command {#synchronous-commands}

สายบรรพบุรุษของ command ขึ้นกับ context ตอนส่ง:

- เมื่อมี root ที่ active command จะ capture root นั้น รวมถึงระหว่างการส่ง callback และการทำ retained work ต่อ
- ภายใน publication scope หรือ command-processing scope การส่งจะเริ่ม root ของ scope นั้นถ้าจำเป็น
- นอก scope เหล่านั้นและไม่มี root ที่ active การส่งจะไม่มี root การประมวลผลจะสร้าง root อิสระเมื่อ admit งาน callback
  ครั้งแรกหรือส่ง command ที่ซ้อนอยู่

command ใน drain batch เดียวกันไม่ได้ใช้ root ร่วมกันเพียงเพราะทำงานด้วยกัน การประมวลผลและการทำลายจะคืน root ที่
ครอบอยู่เมื่อ return หรือ unwind

### ลำดับของ batch และ child {#batch-and-child-ordering}

command drain ประมวลผล batch ที่รวบรวมไว้ตอนเข้าตามลำดับ command ที่ handler เข้าคิวผ่าน synchronous sender
จะอยู่ในคิวรอ drain ครั้งถัดไป trading handler ยัง capture child ที่เลื่อนไว้ได้: child เหล่านี้ทำงานแบบ depth-first
ตามลำดับที่ capture ก่อน command ถัดไปใน batch ที่รวบรวมไว้ child แต่ละตัว capture root ที่ active ตอน capture
รวมถึง nested context ชั่วคราว แทนที่จะใช้ context ของ parent หลังจาก handler return การ route ไปยัง endpoint
โดยตรงยังคงไม่เปลี่ยน

borrow ของคิวและ dispatch frame สิ้นสุดก่อนที่ handler จะทำงาน หรือก่อนทำลาย command ที่ถูกทิ้ง การประมวลผล command
ไม่นับเป็นการส่ง callback และไม่ drain callback โดยอัตโนมัติ

### Panic และ overflow {#panic-and-overflow}

ถ้า handler panic panic จะ propagate ออกไป command ที่ยังไม่ประมวลผลใน batch ที่รวบรวมไว้และ child ที่เลื่อนไว้ที่ยัง
ค้างอยู่จะถูกทำลายภายใต้ context ของตัวเอง command ที่เพิ่งเข้าคิวจะยังอยู่ในคิว root ที่ถูก capture ทำให้การนับ
callback ยังมีชีวิตอยู่ และขวาง teardown ของ dispatcher อย่างชัดเจนจนกว่า command เหล่านั้นจะปล่อย command ที่
ถูกทิ้งก็ปล่อย ownership ที่ capture ไว้ตอน thread teardown

ขีดจำกัดของ callback storage ไม่ได้ปฏิเสธการส่ง command ถ้าการส่งต้องใช้ root และการจองนั้นเกินขีดจำกัด จะ latch
callback overflow ขณะที่ command ยังอยู่ในคิว command entry ไม่ได้ใช้ขีดจำกัด retained unit ของ callback และ
payload ของ command กับความจุของคิวไม่ได้นับรวมใน callback storage ที่ทราบขนาด การจอง root แต่ละครั้งคิดค่าเพียงครั้งเดียว

:::warning command transport ไม่ใช่การจัดสรร callback budget
ขีดจำกัดเหล่านี้ไม่ได้จำกัด memory ของ command queue หรือ loop ที่สร้างแต่ command
:::

## Command และ event channel ของ live {#live-command-and-event-channels}

data command sender และ trading command sender ของ live จะ capture root เฉพาะบน thread ที่สร้าง sender เท่านั้น
ให้สร้าง sender บน runtime thread การส่งจาก thread อื่นถือเป็น ingress ที่เป็นอิสระ แม้ runtime อื่นจะมี root ที่ active
อยู่บน thread นั้น dispatch envelope พา token ที่ปลอดภัยต่อการส่ง callback root และการนับยังคงเป็น thread-local
การประมวลผล envelope ที่มี root บน thread อื่นจะ panic ก่อน dispatch

### Dispatch และ buffering {#dispatch-and-buffering}

envelope รักษาสายบรรพบุรุษผ่าน startup buffer, การ poll ของ runner และการ drain ตอนปิดระบบ trading child
capture root ที่ active ของตัวเองและรักษาลำดับแบบ depth-first บนเส้นทาง command ที่ node poll นั้น execution
observer จะทำงานก่อน dispatch ไปยัง endpoint ที่ตรงกันแต่ละครั้ง รวมถึง child การประมวลผล envelope ที่ไม่มี root
จะเริ่ม root scope แบบ lazy ที่เป็นอิสระ
สิ่งนี้ไม่ได้เปิดใช้งาน queued callback หรือ drain มันโดยอัตโนมัติ

startup buffering รักษา root ของแต่ละ event ไว้เมื่อแยก execution batch เป็น order แต่ละตัว batch ที่เป็นอิสระยัง
เป็นอิสระจนถึง dispatch report ยังคง drain ก่อน order event ที่ buffer ไว้ และ account event ยังประมวลผลทันที
execution observation ของ node และการบันทึก fill ทำงานภายใต้ context ของ event ที่ได้รับ event transport เองไม่ได้
ใช้ callback delivery budget

### ประเภทของ sender และ ingress {#sender-types-and-ingress}

event sender ของ data, execution และ system ใน live ใช้ `EventSender<T>` ซึ่งเป็น alias ของ `DispatchSender<T>`
ด้วย envelope `DispatchMessage<T>` เดียวกับ command system command และ time event ก็ใช้ sender นี้ การ bind
กับ runtime เป็นตัวกำหนด owner thread event ที่ส่งออกแบบ synchronous บน thread นั้นจะสืบทอด root ที่ active
การส่งจาก thread อื่นและการส่งที่ไม่มี scope ที่ active ยังคงเป็นอิสระ context ไม่ได้ส่งต่อข้าม task ที่ spawn ใด ๆ
หรือข้ามจุด await

### System event และ time event {#system-and-time-events}

system event และ command รักษา envelope ไว้ผ่าน startup buffering, การ poll และการปิดระบบ การส่ง time event
จะคืน context ของ envelope ก่อน resolve และเรียก callback การลงทะเบียน timer ไม่ได้ capture root: การ fire ตาม
ตารางเข้ามาอย่างอิสระ และการใช้ callback ซ้ำไม่ได้แชร์ budget ระหว่างการ fire แต่ละครั้ง time event ที่ส่งแบบ
synchronous จาก callback scope ที่ active จะสืบทอด scope นั้นแทน callback lease, การยกเลิก และการเก็บกวาดยังคงมี
พฤติกรรมเดิม callback transport เองไม่ได้ใช้ delivery budget

### Standalone sender {#standalone-senders}

client แบบ standalone สามารถแปลง Tokio sender ธรรมดาเป็น `EventSender<T>` เพื่อคง receiver ของ domain event
โดยตรงไว้ได้ โหมดนั้นไม่พา callback context และไม่ได้ใช้ใน channel binding ของ live runner
payload ของ event และความจุของ channel อยู่นอกการนับ callback storage เช่นเดียวกับ payload ของ command และคิว

### การทำลายข้าม thread {#destruction-across-threads}

| Thread ที่ทำลาย    | Context ของ payload                                   | การปล่อย root                                                               |
| ------------------ | ----------------------------------------------------- | --------------------------------------------------------------------------- |
| Owner thread       | คืน root ของข้อความระหว่าง drop payload               | ปล่อย ownership บน owner thread                                             |
| Foreign thread     | ไม่สามารถคืน context ของ owner thread ได้             | ปล่อย token ผ่าน cleanup queue โดยไม่เข้าถึง root ของเจ้าของ               |

เจ้าของจะเรียกคืนการปล่อยจาก foreign thread เมื่อ capture หรือ dispatch ของ channel ครั้งถัดไป, เมื่อตรวจ callback
quiescence หรือเมื่อล้าง dispatcher thread teardown ก็ปล่อยสิ่งเหล่านี้เช่นกัน dispatch envelope, ความจุของ
thread-local context registry และ storage ของ cleanup queue อยู่นอกขีดจำกัดของ callback storage

## ขีดจำกัดของ storage {#storage-limits}

ขีดจำกัดภายในมีดังนี้:

- **Retained unit**: ไม่เกิน 65,536 invocation capture และการจองความจุของ batch ใช้ตัวนับร่วมกับ queued callback
  ดังนั้นตัวนับอาจเกินจำนวน callback จริงแบบเผื่อไว้
- **Storage ที่ทราบขนาด**: ไม่เกิน 64 MiB
- **Callback chain**: ไม่เกิน 1,048,576 การส่งที่เสร็จแล้วต่อ root

ขีดจำกัดเหล่านี้เป็นฟิวส์ความปลอดภัยเชิงป้องกัน ไม่ใช่คำแนะนำในการกำหนดขนาด workload มีไว้หยุดการ retain ที่มากเกินไป
หรือ callback chain ที่ยาวเกินไป ไม่ได้บอกอัตรา event ที่รองรับได้อย่างยั่งยืนหรืองบ memory ของ process ที่ปลอดภัย
ขีดจำกัดเหล่านี้เป็นค่าภายในและผู้ใช้ตั้งค่าไม่ได้

### Storage ที่นับรวม {#included-storage}

storage ที่ทราบขนาดประกอบด้วย:

- ค่าของ callback, queue slot และความจุของคิวที่มองเห็นได้
- allocation ของ chain และความจุของ invocation vector
- heap charge ที่ผู้เรียกระบุ

การวัด payload ครอบคลุมความจุของ string และ vector ที่เข้าถึงได้, เนื้อหา JSON แบบ recursive, metadata entry,
collection ของ order event, book entry ที่มองเห็นได้ และ option-chain entry เมื่อเปิด `defi` จะครอบคลุม string
ของ blockchain ที่ถูก own โดยตรงด้วย

storage ของ payload ที่ใช้ร่วมกันอาจถูกนับมากกว่าหนึ่งครั้ง ชื่อ data type, topic และ identifier เปิดเผยเป็น string
slice ดังนั้นการนับจะครอบคลุมความยาว แต่ไม่รวมความจุส่วนเกินที่เข้าถึงไม่ได้ การนับจะถูกถือไว้ระหว่างที่การส่งยังดำเนินอยู่
และระหว่างที่ batch ยัง retain capture

:::warning ไม่ใช่เพดาน memory ของทั้ง process
การนับนี้ไม่ได้จำกัด memory ของทั้ง process และไม่รวม:

- bookkeeping ของ allocator
- ส่วนภายในของ collection ที่การวัดเหล่านี้มองไม่เห็นความจุ
- storage ของ intern pool และ allocation ของ actor ที่ลงทะเบียนไว้
- payload ของ custom data แบบ opaque
- storage ของ dynamic module
:::

ผู้เรียกต้องระบุ heap charge ที่ทราบขนาดสำหรับ type ของ capture ของตัวเอง การจองแบบ generic ไม่สามารถตรวจฟิลด์
ที่ถูก own ใด ๆ ได้

## ความเข้ากันได้ของ backend {#backend-compatibility}

allocation guard ภายในปฏิเสธ checked access ที่ทับซ้อนกัน **unchecked access และ engine/cache borrow ที่ครอบอยู่
ยังอยู่นอก guard เหล่านั้น** ความเท่าเทียมระหว่าง native และ dynamic backend ยังไม่ได้รับการพิสูจน์

backend แบบ direct และ dynamic ต้องรักษาข้อกำหนดด้านลำดับของ callback, การเข้าถึง component แบบ exclusive และ
สิทธิ์ตาม lifecycle แบบเดียวกัน ผลของ facade ยังคงเป็น synchronous
[กฎของ plug-in boundary](plugins.md#boundary-rules) ใช้กับทุกค่าที่ข้าม boundary ของ dynamic library
callback dispatch ไม่ได้โอน allocation ownership ข้าม boundary นั้นโดยนัย

admission ticket และ cleanup scope เป็นกลไกของ framework field ของผู้เขียน, signature ของ callback และ facade API
แบบ canonical ไม่ได้เปิดเผยสิ่งเหล่านี้
