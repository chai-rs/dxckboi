---
title: "Runtime Conformance Contract"
---

ใช้เอกสารอ้างอิงนี้เพื่อหาขอบเขตของ implementation และการตรวจสอบตัวอย่างสำหรับ
[design principle](design_principles.md) ที่เลือกไว้

แต่ละ section บันทึก source baseline แยกกัน ลิงก์ไปยัง source จะ resolve ตาม revision ของเอกสารนี้
baseline ระบุ revision ที่ใช้เป็นหลักฐานของ section นั้น เว้นแต่จะระบุว่าเป็นข้อมูลในอดีต (historical) พฤติกรรม
ที่เกิดภายหลังให้ดูจาก source และ test ที่ลิงก์ไว้ ณ revision ของเอกสารนี้ ทั้งสองอย่างไม่ได้เป็นการรับประกัน
สำหรับการเปลี่ยนแปลงในอนาคต

รายการต่าง ๆ อธิบาย source และ test coverage ของ Rust ไม่ได้รับรองทุก adapter, entry point,
config หรือรูปแบบความล้มเหลว test ที่ระบุชื่อเป็นเพียงการอ้างอิง source ไม่ใช่บันทึก
ผลการรัน test

## ที่มาของหลักฐานและผลลัพธ์ [#evidence-and-outcome-provenance]

Historical source baseline: [46f87cd1b7af576495418761bbf11db23e89124c](https://github.com/nautechsystems/nautilus_trader/commit/46f87cd1b7af576495418761bbf11db23e89124c).
พฤติกรรม submission-retention ด้านล่างครอบคลุมโดย source และ test ที่ลิงก์ไว้ ณ revision ของเอกสารนี้

[Execution policy](../concepts/execution/policies.md#terminal-reconciliation-provenance)
แยกหลักฐานจาก venue ออกจากการ resolve ด้วย policy ภายในเครื่อง ใน Rust live execution manager
`check_inflight_orders` จัดการกรณีที่ retry ครบตามที่ตั้งค่าไว้ดังนี้:

- **Order ที่ submit แล้ว**: manager สร้าง rejection ด้วยเหตุผล `INFLIGHT_TIMEOUT` เมื่อ
  submission retention ไม่มีผล
- **Update และ cancellation ที่ pending**: manager สร้าง `OrderCanceled` เว้นแต่ order นั้น
  ไม่เคยถูก accept และต้อง retain ไว้

event เหล่านี้มี `reconciliation=true` manager จะ retain order ที่ยังไม่ได้รับการยืนยันภายใต้
`SubmissionRecoveryPolicy::RetainUnresolved` หรือเมื่อ client ของ order return `true` จาก
`retain_unresolved_submissions`

- **Implementation**: [Execution manager](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/src/execution/manager.rs),
  `check_inflight_orders`, `register_submission_retention`, `submission_is_retained` และ
  `resolve_missing_order`
- **การตรวจสอบตัวอย่าง**: [Manager integration tests](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/tests/integration/manager.rs),
  `test_inflight_order_generates_rejection_after_max_retries`,
  `test_retained_submission_survives_recovery_and_accepts_late_evidence`,
  `test_check_open_orders_submitted_missing_at_venue_obeys_recovery_policy`,
  `test_inflight_pending_update_generates_canceled` และ
  `test_inflight_pending_cancel_generates_canceled`
- **ข้อจำกัด**: การ retry ครบไม่ได้ยืนยันผลลัพธ์ที่ venue flag reconciliation เพียงอย่างเดียว
  ไม่ได้แยก report จาก venue ออกจากการ resolve ด้วย policy ภายในเครื่อง และ `OrderCanceled` ไม่มี
  ฟิลด์ reason ผู้ใช้งานต้องอาศัย input และ log ที่เกี่ยวข้องเพื่อคงความแตกต่างนั้นไว้

## ลำดับและความเป็นเจ้าของของ callback [#callback-ordering-and-ownership]

Source reviewed at: [16f2163a3cefd5ae212cbddda7dc5253f8fe0bbc](https://github.com/nautechsystems/nautilus_trader/commit/16f2163a3cefd5ae212cbddda7dc5253f8fe0bbc).

[Callback dispatch contract](callback_dispatch.md) กำหนดให้มีลำดับการ publish ข้ามผู้รับ และ
การเข้าถึง component แบบ exclusive primitive ภายในของ Rust จะสงวนลำดับการ publish และปฏิเสธการเข้าถึง
allocation แบบ checked ที่ซ้อนทับกัน การส่ง callback ใน production ไม่ได้ใช้ primitive เหล่านี้ backtest ใช้
boundary drain และ callback teardown ส่วน live running loop ใช้ drain แบบจำกัดขอบเขตก่อนเลือก event ถัดไป
โดยมีการ yield และตรวจสอบการหยุดระหว่าง batch ที่ pending

runtime เป็นเจ้าของการ scheduling ที่ขอบเขตความเป็นเจ้าของที่ระบุชัดเจนจำนวนไม่มาก เส้นทาง startup และ shutdown ยัง
ต้องมี coverage ด้านความเป็นเจ้าของและการ scheduling ก่อน activation ซึ่งไม่ได้หมายความว่าต้องมี drain ในทุก lifecycle หรือ
flush method

- **Live startup**: หลัง startup สำเร็จ drain ครั้งแรกของ running loop คือขอบเขตการส่งที่มีอยู่
- **Live disposal**: ปล่อย runner ที่ retain ไว้หลังจาก dispose kernel แล้วจึงพยายาม cleanup callback
  root ภายนอกยังสามารถขัดขวางการ clear ได้
- **Standalone lifecycle**: `start`/`stop` ไม่มีกำหนดการส่ง callback ในคิวอย่างต่อเนื่อง และ
  คงพฤติกรรมแบบ synchronous เดิมไว้

[Live lifecycle contract](callback_dispatch.md#live-startup-and-standalone-lifecycle) จำกัดให้
activation ของคิวครั้งแรกเกิดได้เฉพาะผ่าน `run`/`run_with_mode` ของ Rust เท่านั้น activation ต้อง
ปฏิเสธการส่งแบบคิวเมื่อเป็น standalone startup ก่อนจะรับ callback ซึ่งการปฏิเสธนี้ยังไม่ได้ implement

- **Implementation**:

  | Source                                                       | Symbols                                              |
  | ------------------------------------------------------------ | ---------------------------------------------------- |
  | [Dispatch](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/common/src/actor/dispatch.rs)        | `PublicationScope` และ `drain`                       |
  | [Allocation access](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/common/src/actor/access.rs) | `AllocationGuard`                                    |
  | [Live bounded drain](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/src/dispatch.rs)      | `drain_callbacks` ที่ห่อ `actor::drain_callbacks`     |
  | [Runner](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/src/runner.rs)                    | `AsyncRunner::run`                                   |
  | [Live node](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/src/node/mod.rs)               | `run_with_mode` และ `dispose`                        |

- **การตรวจสอบตัวอย่าง**:

  - `nested_publication_reserves_all_outer_recipients` ใน dispatch module ตรวจสอบลำดับของผู้รับชั้นนอก
    ข้าม publication ที่ซ้อนกัน
  - `test_actor_and_component_views_share_access` ใน access module ตรวจสอบการ exclusion ข้าม view
  - `test_runner_callback_failure_stops_before_next_command` ใน runner module ตรวจสอบการส่งต่อความล้มเหลว
    และการ retain message ที่ pending รวมถึง fatal latch
  - `test_runner_stop_preserves_messages_and_channel_only_scheduling` ใน runner module ตรวจสอบการนำกลับมาใช้
    ตามลำดับหลังหยุด โดยไม่เพิ่มการ yield สำหรับงานที่มีเฉพาะ channel
  - `test_callback_failure_stops_later_live_events` ใน live node module ตรวจสอบพฤติกรรมเมื่อล้มเหลว ความล้มเหลว
    ต้องหยุดการ dispatch event ที่ตามมาและหยุด trader และคง latch ไว้จนกว่าจะ dispose
  - `test_dispose_releases_retained_callback_roots` ใน live node module ตรวจสอบการปล่อยความเป็นเจ้าของ runner
    และการ cleanup fatal latch รวมถึงการปฏิเสธขณะที่ยังมี root ภายนอกอยู่ และ diagnostic
    ของ error ตอน dispose
  - `test_dispose_releases_stop_generated_callback_roots` ใน live node module ตรวจสอบว่าการ dispose
    ปล่อย rooted message ที่เกิดจากการ stop โดยไม่ส่งมันออกไป
- **ข้อจำกัด**: การตรวจสอบเหล่านี้ไม่ได้ยืนยันลำดับ callback หรือความปลอดภัยของความเป็นเจ้าของใน production การ
  integrate กับ runtime ต้องจบ mutable borrow ที่ครอบอยู่ก่อน drain และคงคุณสมบัติด้าน lifecycle ของ
  native และ dynamic backend ไว้ การเข้าถึงแบบ unchecked ยังอยู่นอก allocation guard ภายใน

  ก่อน activation ต้องมี test ลำดับ callback ที่แน่นอนผ่าน component จริง เพื่อยืนยันลำดับที่
  deterministic ข้อกำหนดนี้ใช้กับ synchronous core และ backtesting coverage ต้องครอบคลุม
  publication ที่ซ้อนกัน, fan-out, command ที่สร้างจาก callback, งานใน timestamp เดียวกัน และการทำงานต่อเนื่องข้าม
  pass ที่จำกัดขอบเขต จำนวนและยอดคงเหลือสุดท้ายเพียงอย่างเดียวไม่ได้พิสูจน์ลำดับ ดู [ข้อกำหนดของ
  activation](callback_dispatch.md#activation-requirements)

## การกู้คืน [#recovery]

Source baseline: [46f87cd1b7af576495418761bbf11db23e89124c](https://github.com/nautechsystems/nautilus_trader/commit/46f87cd1b7af576495418761bbf11db23e89124c).

สำหรับ Rust live node ที่เปิด execution reconciliation ไว้ startup จะทำ reconciliation ก่อนเริ่ม
component ของ trader หาก reconciliation error จะยกเลิก startup integration test ของ startup ด้านล่างส่ง report ของ
order และ fill ที่อยู่ในสถานะสิ้นสุดผ่าน test execution client และตรวจสอบจำนวนที่กู้คืน ราคา identity ของ trade
ค่า commission จำนวนของ position และสถานะสิ้นสุด

- **Implementation**: [Live node](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/src/node/mod.rs),
  `perform_startup_reconciliation` และผู้เรียกของมัน
- **การตรวจสอบตัวอย่าง**: [Node integration tests](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/tests/integration/node.rs),
  `test_live_node_startup_recovers_terminal_fill_exactly`
- **ข้อจำกัด**: การตรวจสอบนี้ครอบคลุมการกู้คืนตอน startup จาก report ไม่ใช่ความทนทานของ backing store, การ restart
  โดย supervisor หรือการกู้คืนจาก panic ใด ๆ reconciliation สามารถปิดได้ การ replay จาก event store ก็ข้าม
  การเชื่อมต่อ live client และ reconciliation ด้วย ความครบถ้วนของข้อมูลจาก venue ยังขึ้นกับ [reconciliation
  policy](../concepts/execution/policies.md#reconciliation-authority)

## การจัดการภาวะ overload [#overload-handling]

Source reviewed at: [16f2163a3cefd5ae212cbddda7dc5253f8fe0bbc](https://github.com/nautechsystems/nautilus_trader/commit/16f2163a3cefd5ae212cbddda7dc5253f8fe0bbc).

[Live runner](../concepts/live.md#dispatch-priority-and-overload-behavior) ใช้ message
channel แบบไม่จำกัดขนาด ลำดับความสำคัญของการ poll ไม่ได้สร้าง backpressure ต่อ producer หรือจำกัดความลึกของคิว
callback dispatcher ภายในบังคับใช้ขีดจำกัดของจำนวนที่ retain, storage ที่รู้จัก และ callback chain แยกต่างหาก

- **Implementation**: [Runner](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/src/runner.rs), `AsyncRunner::new` และ `recv`
  [Dispatch](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/common/src/actor/dispatch.rs): การนับ admission และ `drain`
- **การตรวจสอบตัวอย่าง**: `test_recv_processes_system_event_before_command` ใน runner
  module ตรวจสอบลำดับความสำคัญของ channel คู่นั้น `event_count_limit_latches_after_exact_capacity`
  และ `progress_limit_persists_between_bounded_drains` ใน dispatch module ตรวจสอบขีดจำกัดภายใน
- **ข้อจำกัด**: test ลำดับการ poll ไม่ได้พิสูจน์ว่า latency หรือความคืบหน้ามีขอบเขตภายใต้โหลดต่อเนื่อง
  ขีดจำกัดของ callback ภายในไม่ได้จำกัดคิวของ live runner หรือหน่วยความจำทั้งหมดของ process coverage ของ
  callback activation ใน production และความเป็นเจ้าของ live lifecycle ยังคงเป็นข้อกำหนดของ integration แม้จะ
  implement drain ใน running loop แล้วก็ตาม การ monitor คิวให้สัญญาณด้าน operation โดยไม่
  throttle feed หรือหยุดการเทรดโดยอัตโนมัติ
