# การทดสอบ

test อัตโนมัติทำหน้าที่เป็นข้อกำหนดที่รันได้ (executable specification) ของแพลตฟอร์มเทรด ช่วยบันทึกพฤติกรรม
ที่ตั้งใจไว้ ทำให้ refactor ได้อย่างมั่นใจ และจับ regression ได้ก่อนขึ้น production นอกจากนี้ test ยังเป็นตัวอย่าง
ของ flow ที่ซับซ้อน และให้ feedback จาก CI ได้เร็ว ปัญหาจึงถูกพบตั้งแต่เนิ่นๆ

ชุด test ครอบคลุมหมวดหมู่ต่อไปนี้:

- Unit test
- Integration test
- Acceptance test
- Performance test
- Property-based test
- Fuzzing
- Memory leak test

## นโยบายการทดสอบ {#testing-policy}

test และ runtime contract เป็นระบบการออกแบบเดียวกัน ลำดับขั้นของ
[Design by contract](rust.md#design-by-contract) พยายามผลัก invariant เข้าไปไว้ใน type system ให้มากที่สุด
ส่วนลำดับขั้นของการทดสอบด้านล่างจะไล่จัดการสิ่งที่ยังไม่รู้ที่เหลืออยู่ด้วยพื้นที่ input ที่ใหญ่ขึ้นและโมเดลการ execute
ที่ละเอียดขึ้น แต่ละชั้นขยายความครอบคลุมไปยัง input หรือสถานะการ execute ที่ชั้นล่างเข้าไม่ถึง

ไม่ใช่ทุก module ที่ต้องใช้ทุกเทคนิค ใช้หัวข้อนี้ตัดสินว่าชั้นใดเหมาะสมก่อนเพิ่ม test หรือ `debug_assert!`

### ลำดับขั้นของกลไก {#mechanism-ladder}

[คู่มือ Rust](rust.md#design-by-contract) อธิบายเรื่อง runtime contract ไว้ ลำดับความสำคัญคือ:

1. type system
1. `check_*` จาก `nautilus_core::correctness` ที่ขอบเขตของ API
1. `debug_assert!` สำหรับ invariant ภายใน
1. `assert!` สำหรับการตรวจสอบที่สำคัญต่อ soundness หรือที่ต้องเปิดตลอดเวลา

ชั้นของ test ไล่ระดับขึ้นในแบบคู่ขนานกัน เริ่มจากชั้นต่ำสุดที่พิสูจน์สิ่งที่สำคัญได้ และขยับขึ้นเฉพาะเมื่อชั้นนั้น
ตรวจจับ regression ไม่ได้อีก หรือพื้นที่ input ใหญ่เกินกว่าจะเลือก case ด้วยมือ

| ชั้น                      | เงื่อนไขที่ควรใช้                                                                     |
| ------------------------ | -------------------------------------------------------------------------------- |
| Unit test                | function หรือ transition เดียวที่มีชุด case เล็กและนับได้                                 |
| Parametrized test        | รูปแบบเดียวกันซ้ำกันใน input แบบไม่ต่อเนื่อง (order side, status, instrument)                |
| Property-based test      | invariant ต้องเป็นจริงสำหรับ input ทั้งกลุ่มที่ไล่นับด้วยตัวเองไม่ไหว                          |
| Integration test         | หลาย module ทำงานร่วมกันผ่าน engine หรือ runtime จริง (ไม่ใช่ mock)                     |
| Fuzz test                | byte ที่ไม่น่าเชื่อถือหรือเป็นปฏิปักษ์ผ่าน parser, decoder หรือ handler ของ wire format        |
| Spec acceptance test     | พฤติกรรมขึ้นกับ contract ของ venue จริง (ดู `spec_exec_testing.md`)                     |
| Deterministic simulation | ความถูกต้องขึ้นกับการจัดตาราง task, timeout หรือลำดับตาม wall-clock                     |
| Formal verification      | pure function ที่มี invariant ชัดเจน และมีพื้นที่ input จำกัดที่คุ้มค่าจะพิสูจน์                |

ขั้น formal verification ยังเป็นเพียงเป้าหมาย: ยังไม่มี harness ของ Kani หรือ Prusti เข้ามาใน workspace
แถวนี้บันทึกเงื่อนไขของการไล่ระดับไว้สำหรับเมื่อมีการนำ verifier มาใช้ ไม่ใช่ข้อผูกมัดในปัจจุบัน

### กฎการเลือกชั้นตามรูปแบบ module {#projection-rule}

รูปแบบของ module เป็นตัวกำหนดว่าชั้นใดคุ้มค่า บาง module ไม่ต้องใช้ลำดับขั้นครบทั้งหมด ให้ใช้กฎนี้กับแต่ละ
module ไม่ใช่แต่ละ crate เช่น crate ของ adapter มีทั้ง parser แบบ pure และ client loop ที่ผูกกับ I/O
แต่ละแถวจึงใช้กับคนละส่วน

| รูปแบบ module                         | ชั้นที่ใช้                                         | ตัวอย่าง                                 |
| ----------------------------------- | --------------------------------------------- | -------------------------------------- |
| Pure function, invariant ชัดเจน       | Unit, parametrized, property, fuzz            | Reconciliation kernel, portfolio math  |
| Pure function, ไม่มี invariant ที่ระบุ  | Unit, parametrized, property, fuzz            | Codec, adapter parser, formatter       |
| Stateful, synchronous               | Unit, parametrized, property over transitions | Cache, order book                      |
| Stateful, async                     | Unit, integration, deterministic simulation   | Live engine, execution manager         |
| ผูกกับ I/O, contract ของ venue        | Integration, spec acceptance, boundary fuzz   | Adapter client loop                    |

### เมื่อไหร่ไม่ควรเพิ่ม coverage {#when-not-to-add-coverage}

- เพิ่ม `debug_assert!` เฉพาะจุดที่ test เข้าถึงได้ release build ตัดการตรวจสอบนี้ทิ้ง assertion ที่ไม่ถูก
  execute จึงไม่ให้สัญญาณใดๆ unit test ที่เจาะจงนับเป็น harness ได้ ส่วน proptest หรือ fuzz harness
  ช่วยขยายสัญญาณ
- เลือก proptest มากกว่า edge-case test ที่เขียนด้วยมือเมื่อ invariant ครอบคลุม input ทั้งกลุ่ม unit test
  แบบเจาะจงยังใช้ได้สำหรับพฤติกรรมผิดปกติของ venue ที่รู้จัก และเป็นตัวทำซ้ำ regression จาก counterexample
  ที่ถูก shrink แล้ว
- อย่าทำ live spec acceptance card ซ้ำเป็น integration test ให้ลิงก์ไปหามันแทน
- อย่าเพิ่ม coverage ด้วย test ที่ assert สิ่งที่ภาษาหรือ framework รับประกันอยู่แล้ว
  (`Option::is_some` หลัง `Some(..)`, `Vec::len` หลัง `push`)

### ความพร้อมสำหรับ DST {#dst-readiness}

Deterministic simulation testing (DST) ต้องการ runtime ที่ปลอดจาก non-determinism แฝง ก่อนยก module
ให้รันภายใต้ DST ให้ตรวจสอบสิ่งต่อไปนี้:

- primitive ด้านเวลา, task, runtime และ signal ต้องผ่าน `nautilus_common::live::dst`
  แทนการเรียก `tokio` โดยตรง การอ่าน wall-clock ต้องผ่าน seam ใน `nautilus_core::time`
  แทนการเรียก `SystemTime::now()` ตรงจุดที่ใช้
- map ของ state ที่การวนลูปขึ้นกับลำดับต้องใช้ `IndexMap` หรือ `IndexSet` ไม่ใช่ hash collection ปกติ
- ทุก `tokio::select!` บนเส้นทาง control-plane ต้องตั้ง `biased` เพื่อให้ลำดับการ poll คงที่
- ห้ามมีการเรียก `Instant::now()`, `SystemTime::now()`, `tokio::signal::ctrl_c`,
  `std::thread::spawn` หรือ `tokio::task::spawn_blocking` หลุดออกนอก seam primitive ของ blocking thread
  และ OS thread ทำลาย determinism ของ madsim ได้แบบเดียวกับการอ่านนาฬิกาแบบแฝง
- ID ที่ไวต่อการ replay (`trade_id`, `venue_order_id`) ต้องเป็น pure function ของ input ดู
  `crates/execution/src/reconciliation/ids.rs` ส่วน UUID ของ event ชั่วคราวบนเส้นทาง reconciliation อื่น
  ไม่จำเป็นต้อง deterministic

probe `surface` ใน `crates/common/src/live/dst.rs` ตรึงไว้แค่รูปแบบของการ re-export ไม่ได้ตรวจว่าผู้เรียก
ใช้ seam จริงหรือไม่ การบังคับใช้ทำผ่านการ review ให้รันการ audit ทุกครั้งที่มี async module ใหม่เข้ามาใน
workspace หรือ module เดิมมีการจัดตาราง control-plane ใหม่

## Behavior-driven development {#behavior-driven-development}

Behavior-driven development (BDD) ใช้ตัวอย่างที่เป็นรูปธรรมเพื่อตกลงพฤติกรรมที่ต้องการก่อน implement
Gherkin แสดงตัวอย่างเหล่านั้นในรูปแบบที่อ่านง่ายและผูกเข้ากับโค้ด test **ใช้ Gherkin สำหรับนโยบายที่ครอบคลุม
หลาย component และมีผลลัพธ์ที่แตกต่างกันไม่กี่แบบ** ซึ่ง reviewer ต้องเข้าใจ feature file อธิบายเงื่อนไขที่ระบบ
ตัดสินใจและผลลัพธ์ที่ผู้เรียกสังเกตได้ ข้อความของ step ผูกกับ Rust ตอน runtime แต่ละ feature จึงเพิ่มชั้นที่
compiler ไม่ได้ตรวจ ให้เก็บการคำนวณ, permutation ที่ละเอียด และการตรวจ wiring ไว้ใน Rust

**BDD อธิบายว่าพฤติกรรมถูกกำหนดอย่างไร ส่วน integration อธิบายว่า test ทดสอบ component ใดบ้าง**
scenario ของ startup reconciliation เป็น integration test ที่เขียนด้วย Gherkin โดยรัน node, execution manager,
cache และ portfolio จริงร่วมกับ execution client ที่ script ไว้ Gherkin ไม่ได้เพิ่มชั้นบังคับให้กับ
[นโยบายการทดสอบ](#testing-policy) และ response ของ venue ที่ script ไว้ไม่ถือเป็น
[live adapter acceptance](spec_exec_testing.md)

### การเลือกกฎและตัวอย่าง {#selecting-rules-and-examples}

เริ่มจาก contract ที่มีเอกสาร, การเปลี่ยนพฤติกรรมที่ตกลงกันแล้ว หรือ defect ที่ยืนยันแล้ว ใช้โค้ดตรวจว่า
พฤติกรรมถูก implement อย่างไร ถ้ากฎที่ตั้งใจไว้กับ implementation ไม่ตรงกัน ให้แก้ความขัดแย้งนั้นก่อนจะถือ
ผลลัพธ์ปัจจุบันเป็นข้อกำหนด

สำหรับแต่ละกฎ:

1. ระบุสิ่งที่รับประกันและขอบเขต รวมถึง configuration และ input ที่กฎนั้นขึ้นอยู่ด้วย
1. ระบุ input ที่ให้ผลลัพธ์ต่างกันอย่างมีนัยสำคัญ รวมถึงกรณีล้มเหลวและการกู้คืน
1. เลือกตัวอย่างที่เป็นรูปธรรมสำหรับความแตกต่างแต่ละแบบที่ผู้อ่านต้องเข้าใจเพื่อเข้าใจกฎ
1. ระบุสิ่งที่สังเกตได้ที่พิสูจน์ผลลัพธ์แต่ละแบบ และขอบเขตการ execute ที่เล็กที่สุดที่ทำให้เกิดผลนั้น ถ้ามีคำถาม
   เรื่องพฤติกรรมที่ยังไม่ได้ข้อสรุป ให้ยกขึ้นมาพร้อมการเปลี่ยนแปลงหรือใน issue ก่อนจะเขียนเป็นความคาดหวัง

ตัวอย่างเช่น [reconciliation contract](../concepts/execution/reconciliation.md) กำหนดว่า position report
ที่อยู่ในขอบเขตแต่ยังไม่ได้ resolve ต้องกันไม่ให้ actor และ strategy เริ่มทำงานเมื่อเปิด reconciliation ไว้
การกู้คืนที่มีราคา, การล้มเหลวเพราะไม่มีราคา และการปิดการสร้าง report แสดงผลลัพธ์ที่แตกต่างกันภายใต้กฎนั้น
ให้แยกความแตกต่างเหล่านั้นเป็น scenario ต่างหากเพื่อให้เห็นชัด

### ความเป็นเจ้าของ test และความซ้ำซ้อน {#test-ownership-and-overlap}

**ให้ตัวอย่างที่เป็นรูปธรรมแต่ละตัวมี test หลักหนึ่งตัว** ใช้ Gherkin สำหรับ behavior contract ที่เป็นตัวแทน
และใช้ Rust test แบบเจาะจงสำหรับ permutation ที่ละเอียดหรือการทำงานร่วมกันของ component ที่ไม่ต้องการ
คำอธิบายเชิง domain แยกต่างหาก สำหรับ startup reconciliation การแบ่งที่พบบ่อยคือ:

| ประเด็น                                    | ความครอบคลุมหลัก                 | สิ่งที่พิสูจน์                                                    |
| ---------------------------------------- | ------------------------------- | -------------------------------------------------------------- |
| กู้คืน exposure ก่อนเริ่มเทรด                  | Gherkin integration scenario    | strategy เห็น exposure ที่กู้คืนแล้วตอน startup                     |
| exposure ที่กู้คืนไม่ได้ทำให้ startup ไม่ผ่าน       | Gherkin integration scenario    | startup ล้มเหลวก่อนเริ่มเทรดและเก็บกวาด connection                   |
| price precision และขอบเขตทางคณิตศาสตร์        | Rust unit หรือ parametrized test  | ค่าที่แน่นอนถูกต้องข้ามขอบเขตของตัวเลข                              |
| ตัวตนของ client และการ route report           | Rust integration test           | report ไปถึง account และ execution client ที่ถูกต้อง                |
| ลำดับของ task และ race ของการ cancel           | Deterministic simulation test   | ผลลัพธ์ที่ต้องการเป็นจริงภายใต้ schedule ที่ควบคุมได้                     |

เส้นทางใน production ที่ทับซ้อนกันเป็นเรื่องปกติ ให้คง test ไว้ที่ขอบเขตต่างๆ เมื่อแต่ละตัวพิสูจน์คุณสมบัติต่างกัน
เช่น การคำนวณ reconciliation กับการที่ผลนั้นมองเห็นได้ใน callback ของ strategy ต้องใช้การสังเกตคนละแบบ
ส่วน startup test เต็มรูปแบบตัวที่สองที่มี setup, action และ assertion เหมือนกันต้องมีจุดประสงค์ที่ต่างออกไป

#### การแทนที่ test เดิม {#replacing-an-existing-test}

ก่อนแทนที่ test เดิมด้วย Gherkin scenario:

- เปรียบเทียบ input, assertion, entry point, เส้นทางของ lifecycle และสภาพแวดล้อมการ execute
- คงการตรวจสอบเฉพาะด้านตัวตน, precision, การกรอง, lifecycle, การเก็บกวาด และ simulation ไว้
- ยืนยันว่าตัวที่มาแทนรันอยู่ในชุด test ปกติและพิสูจน์ contract เดียวกัน

scenario ที่ขอ stop จาก start callback ของ strategy ไม่ได้พิสูจน์ว่า node เข้าสู่สถานะ `Running`
ให้คง test ที่ตรวจ transition นั้นไว้ แชร์การสร้าง fixture และ client ที่ script ไว้ได้ แต่อย่าเรียก function
ของ test เดิมจาก step definition หรือเขียน logic ของ reconciliation ซ้ำใน fixture

### การเขียน Gherkin {#writing-gherkin}

จัดแต่ละ feature รอบความสามารถที่สอดคล้องกัน เช่น startup reconciliation โดยใช้โครงสร้างต่อไปนี้:

- **`Feature`** ตั้งชื่อความสามารถและอธิบายว่าทำไมพฤติกรรมนั้นสำคัญ
- **`Rule`** (ถ้ามีประโยชน์) รวมตัวอย่างของการรับประกันหรือนโยบายหนึ่งเรื่อง
- **`Scenario`** ตั้งชื่อพฤติกรรมที่สังเกตได้หนึ่งอย่าง และให้บริบทที่จำเป็นต่อการตีความ
- **`Scenario Outline`** เปลี่ยน input สำหรับพฤติกรรมเดียวกัน เช่น exposure ฝั่ง long และ short ให้แยก
  นโยบายที่ต่างกันเป็นคนละ scenario แทนการรวมไว้ใน matrix แบบมีเงื่อนไขเดียว

#### บริบทและผลลัพธ์ {#context-and-outcomes}

เขียน step `Given` สำหรับเงื่อนไขเริ่มต้น `When` สำหรับ action และ `Then` สำหรับผลลัพธ์ที่สังเกตได้
ใช้คำศัพท์เชิง domain เช่น position report, recovered exposure และ strategy startup ระบุชื่อ method หรือ
entry point ของ Rust เมื่อ public contract ของมันเป็นหัวข้อของตัวอย่าง มิฉะนั้นให้เก็บรายละเอียดการสร้างและ
wiring ไว้ใน step definition

ใช้ `Background` สำหรับบริบทเริ่มต้นที่สั้นและใช้ร่วมกัน ให้เงื่อนไขที่อธิบายผลลัพธ์ที่แตกต่างของ scenario
อยู่ใน scenario นั้น การ override อย่างชัดเจน เช่น การปิด startup reconciliation ต้องมองเห็นได้ ระบุ configuration
ที่เกี่ยวข้องกับกฎแม้จะตรงกับค่า default ใส่หน่วยและค่าทางการเงินที่แน่นอนในตัวอย่าง และให้ scenario
โฟกัสพอที่ผู้อ่านจะอธิบายกฎได้โดยไม่ต้องอ่าน step ใน Rust

**ระบุว่าผลลัพธ์เป็นจริงเมื่อใด ถ้าลำดับมีความสำคัญ** ใน startup feature นั้น
`the strategy observes a long position of 5.000 ETH at 2500.25 USDT when it starts` ต้องการการสังเกต
จาก start callback การ assert cache ตอนท้ายพิสูจน์ลำดับนั้นไม่ได้ scenario ที่เป็นกรณีล้มเหลวต้อง assert
เหตุผลของความล้มเหลวที่เกี่ยวข้องและผลที่ห้ามเกิด เช่น actor หรือ strategy เริ่มทำงานหลังจาก reconciliation ล้มเหลว

ใช้ tag เพื่อเลือกนโยบาย เช่น `@recovery` การค้นหา test ตามปกติต้องรวมชุด test นี้ด้วยโดยไม่ต้องใช้ filter แบบ opt-in

### Step และ fixture ใน Rust {#rust-steps-and-fixtures}

#### โครงสร้างและการนำกลับมาใช้ {#layout-and-reuse}

ใช้ development dependency `cucumber` ของ workspace สำหรับชุด Gherkin ใน Rust ทำตามโครงสร้างของ live crate
คือ runner `tests/spec/main.rs` และ `tests/spec/features/<capability>.feature` ซึ่งเป็นการเลือกโครงสร้างของ
repository และ runner เป็นผู้กำหนด path ของ feature เมื่อเพิ่มความสามารถใหม่ ให้มี `World` และ runner test
ของตัวเองพร้อม path ของ feature ที่ระบุชัดเจน เพื่อไม่ให้ชุด test ไปอ่าน feature ของกันและกัน เก็บ step ไว้กับ
workflow ของมัน และแยก module ของ Rust เมื่อ workflow ที่ต่างกันต้องการ setup หรือการสังเกตแยกกัน

- **Step**: นำ step กลับมาใช้ซ้ำเฉพาะเมื่อข้อความมีความหมาย, setup และ assertion contract เหมือนกันในทั้งสองที่
  หลีกเลี่ยง pattern กว้างๆ ที่ match พฤติกรรมที่ไม่เกี่ยวข้อง หรือเลือกผลลัพธ์ที่คาดหวังจาก state ที่ซ่อนอยู่
- **Fixture**: ใส่ส่วนการสร้างที่ test target อิสระหลายตัวใช้ร่วมกันไว้ใน `tests/common/` เช่น
  [startup client fixture](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/tests/common/startup.rs) ส่วน fixture ที่ใช้ในชุดเดียวให้เก็บไว้ในชุดนั้น
- **World**: กำหนดขอบเขตให้ตรงกับ workflow ที่ทดสอบ อย่าขยาย `World` ที่ใช้ร่วมกันหรือสร้าง fixture framework
  ทั่วไปเผื่อ feature ในอนาคตที่ยังไม่มีจริง

#### พฤติกรรมของ step {#step-behavior}

step definition แปลง input เชิง domain เป็นการเรียกโค้ด production และเปรียบเทียบสิ่งที่สังเกตได้กับความคาดหวัง
ที่ระบุชัดเจน ใช้ domain type หรือ `Decimal` สำหรับค่าทางการเงิน และ assert ฟิลด์ที่กำหนด contract รวมถึง
ตัวตนเมื่อเกี่ยวข้อง script response ที่ขอบเขตของ client ภายนอก โดยให้พฤติกรรมที่ทดสอบเป็นของจริง
**เก็บตัวควบคุมการทดสอบไว้ในโค้ด test**: อย่าเพิ่ม flag, configuration, hook หรือ accessor ใน production
เพียงเพื่อขับ scenario

#### การแยกส่วนและจังหวะเวลา {#isolation-and-timing}

Cucumber สร้าง `World` ใหม่สำหรับแต่ละ scenario แต่ scenario ในชุดเดียวกันใช้ process และ runtime ร่วมกัน
การแยก process ต่อ test ของ nextest ไม่ได้แยก scenario เหล่านี้ออกจากกัน state ที่อยู่นอก `World`
รวมถึง callback และ message bus แบบ thread-local ต้องเก็บกวาดอย่างชัดเจน

- รันชุด test ที่ใช้ state แบบ thread-local ของ live node ตามลำดับบน runtime thread เดียว
- ปล่อย client, task, subscription และ state ที่ใช้ร่วมกันแม้ step จะล้มเหลว จัดการเก็บกวาดให้ assertion หรือ
  timeout ข้ามขั้นตอน dispose ไม่ได้
- บันทึกการสังเกตที่ขอบเขต lifecycle ที่ contract ระบุ เช่น `on_start` ของ strategy การขอ stop ที่จุดนั้น
  อาจกันไม่ให้เข้าสู่ `Running` ได้ จึงควรเลือกจุดสังเกตที่ช้ากว่าเมื่อจำเป็น
- จำกัดเวลาของ action ด้วย timeout และทำตาม[ข้อตกลงการรอแบบ async](#waiting-for-asynchronous-effects)
  เมื่อรอผลลัพธ์ scenario ที่มี timeout ตามปกติไม่ถือเป็นความครอบคลุมแบบ deterministic simulation

### การดูแล feature file {#maintaining-feature-files}

อัปเดตกฎ, ตัวอย่าง และ implementation ไปพร้อมกันเมื่อพฤติกรรมที่ต้องการเปลี่ยน อย่าเปลี่ยนผลลัพธ์ที่คาดหวัง
ให้ตรงกับ defect ให้แก้ defect ในการเปลี่ยนแปลงเดียวกัน ถ้าทำไม่ได้ ให้บันทึกไว้ใน issue และเพิ่มตัวอย่าง
regression พร้อมกับการแก้ไข การ refactor โค้ดที่คงพฤติกรรมเดิมไว้มักต้องแก้เฉพาะ binding หรือ fixture ใน Rust
เพิ่มตัวอย่างเมื่อมันนำเสนอความแตกต่างที่มีความหมาย และใช้ parametrized test หรือ property test แบบเจาะจง
สำหรับการผสมเพิ่มเติมที่แสดงกฎเดียวกัน

#### ข้อกำหนดของ runner {#runner-requirements}

รัน scenario ที่เปลี่ยนแปลงและ test เดิมที่ได้รับผลกระทบ ใช้ Rust test harness มาตรฐานที่มี
`#[tokio::test]` หนึ่งตัวต่อชุด เพื่อให้ nextest ค้นพบได้ ทำตาม
[live runner](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/tests/spec/main.rs):

- ใช้ `fail_on_skipped()` เพื่อให้ step ที่ไม่ match ล้มเหลว
- assert ว่า `execution_has_failed()` เป็น false และมี step ผ่านอย่างน้อยหนึ่ง step assertion ที่ล้มเหลว,
  binding ที่กำกวม, parse error และการเลือกที่ว่างเปล่าต้องทำให้คำสั่งล้มเหลว
- parse option ของ Cucumber แยกจาก argument ของ libtest โดยใช้ `CUCUMBER_FILTER_TAGS` สำหรับการกรอง

สำหรับ runner ใหม่หรือที่เปลี่ยนแปลง ให้ตรวจสอบการค้นพบ test, การกรอง และการรายงานความล้มเหลวของ scenario/step
ลองใช้ความคาดหวังที่ผิด, step ที่ไม่ match และ filter ที่ว่างชั่วคราว เพื่อยืนยันว่าแต่ละกรณีจบด้วย exit code
ที่ไม่ใช่ศูนย์ แล้วคืน scenario และการตั้งค่า filter ที่ถูกต้องหลังจากนั้น นับจำนวน scenario จาก summary ของ
Cucumber เพราะ nextest รายงานชุด Gherkin แต่ละชุดเป็น test เดียว ดู
[การรัน startup scenario](#gherkin-startup-reconciliation-scenarios) สำหรับคำสั่งและการกรองด้วย tag

[คู่มือ example mapping ของ Cucumber](https://cucumber.io/docs/bdd/example-mapping/) อธิบายวิธีพัฒนากฎ
และตัวอย่าง ส่วน [Gherkin reference](https://cucumber.io/docs/gherkin/reference/) กำหนด syntax ที่ใช้ในที่นี้

## Property-based testing {#property-based-testing}

property testing ตรวจสอบว่า logic เป็นจริงสำหรับ input ที่ถูกต้อง*ทั้งหมด* ไม่ใช่เฉพาะตัวอย่างที่เลือกมา
โปรเจกต์ใช้ [`proptest`](https://altsysrq.github.io/proptest-book/intro.html) ใน Rust เพื่อบังคับ invariant

- **กรณีการใช้งาน**: domain type หลัก (`Price`, `Quantity`, `UnixNanos`), accounting engine, matching engine และ state machine
- **ตัวอย่าง invariant**:
  - Round-trip serialization: `parse(to_string(value)) == value`
  - Inverse operation: `(A + B) - B == A`
  - Transitivity: `If A < B and B < C, then A < C`

## Fuzzing {#fuzzing}

fuzzing ป้อนข้อมูลที่ไม่มีโครงสร้างหรือข้อมูลที่เป็นอันตรายเข้าสู่ระบบ เพื่อตรวจว่าระบบล้มเหลวอย่างสง่างาม

- **กรณีการใช้งาน**: ขอบเขตของ network, parser ข้อมูลจาก exchange (JSON, FIX, WebSocket feed) และ state machine ที่ซับซ้อน
- **เป้าหมาย**: ระบบคืน `Result::Err` และไม่ panic, ค้าง หรือ memory รั่วเมื่อเจอข้อมูลที่ผิดรูปแบบ

fuzz binary ของ adapter ลงทะเบียนอยู่ใน package ของแต่ละ adapter หลัง feature `fuzz` รัน target ที่ลงทะเบียนไว้
ทั้งหมดของ adapter หนึ่งตัวจาก root ของ repository:

```bash
scripts/fuzz-adapter.sh derive
```

workspace pin `libfuzzer-sys` ไว้ และ `nautilus-live` เป็นเจ้าของ integration กับ libFuzzer ที่ใช้ร่วมกัน
package แยกที่มี `publish = false` สงวนไว้เฉพาะ fuzz target ที่ dependency ต้องไม่เข้าไปอยู่ใน dependency graph
ของ adapter ที่ publish แล้ว
oracle แบบ differential ของ Pornin ที่ pin ด้วย git ของ Lighter เป็นหนึ่งใน dependency ลักษณะนั้น

เมื่อสร้างหรือแก้ไข type หลัก ให้เขียน property test ครอบคลุมขอบเขตทางคณิตศาสตร์

performance test ช่วยพัฒนา component ที่ประสิทธิภาพมีความสำคัญ

## การรัน test {#running-tests}

CI รัน `scripts/ci/check_test_network.py` ผ่าน `make test-scripts` เพื่อตรวจหาการเรียก network โดยตรงไปยัง
address ที่เป็น literal ที่ไม่ใช่ local, switch ที่เปิด live test อย่างชัดเจน และ option ของ fork-RPC โดยตรวจ
directory ของ test และไฟล์ Rust ตั้งแต่ test module ที่ตั้งชื่อไว้เป็นต้นไป address แบบ loopback และ domain
ของ fixture ที่สงวนไว้ใช้ได้

**ข้อจำกัดของการตรวจ**: heuristic นี้ไม่ resolve ปลายทางที่เป็นตัวแปร ไม่ตามการเรียกเข้าไปในโค้ดอื่น และไม่ได้
บังคับการแยก network

สำหรับชุด benchmark ของ Rust ที่ลงทะเบียนไว้:

```bash
make cargo-ci-benches
```

ดู [คู่มือ Benchmarking](benchmarking.md) สำหรับคำสั่ง Criterion และ iai แบบเจาะจง, การ profile และนโยบาย
การวัดผล รัน benchmark แยกจาก unit test เพื่อไม่ให้รบกวนกัน

### Rust test {#rust-tests}

ก่อนรันชุด test ทั้งหมด ให้[เตรียม test fixture ขนาดใหญ่](test_datasets.md#dataset-categories)

```bash
make cargo-test
# or
cargo nextest run --workspace --features "$(bash scripts/cargo-features.bash)" --cargo-profile nextest --lib --tests
```

:::info
`cargo nextest` เป็น runner ที่รองรับสำหรับชุด unit และ integration test ของ Rust ทั้งหมด ชุด test พึ่งการแยก
process ต่อ test สำหรับ state แบบ process-global และ thread-local ซึ่งรวมถึง logging, message bus และ
state ของ test แบบ deterministic

`cargo test --workspace` แบบธรรมดารัน case ของ test binary ใน process เดียวกัน จึงไม่ใช่ด่านตรวจของทั้งชุด
ที่รองรับ และไม่รับประกันว่าจะผ่าน `cargo test` แบบธรรมดายังเหมาะกับ doctest และ test แบบเจาะจงที่รู้ว่า
ทำงานได้กับ runner ของ libtest
:::

#### Gherkin startup reconciliation scenario {#gherkin-startup-reconciliation-scenarios}

[feature file](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/tests/spec/features/startup_reconciliation.feature) ของ live node
อธิบาย startup reconciliation ด้วย step แบบ Given/When/Then
ส่วน [Rust step definition](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/live/tests/spec/main.rs) รัน node จริงพร้อม response ของ venue
ที่ script ไว้ และสังเกต exposure ที่กู้คืนแล้วภายใน start callback ของ strategy

หากต้องการแสดง scenario ทั้งหมดพร้อม step ให้รัน:

```bash
CARGO_BUILD_JOBS=4 cargo test --locked -p nautilus-live --test spec --profile nextest \
  startup_reconciliation -- --nocapture
```

หากต้องการเลือกเฉพาะ scenario ด้าน recovery ให้ใช้ tag filter ของ Cucumber:

```bash
CUCUMBER_FILTER_TAGS='@recovery' CARGO_BUILD_JOBS=4 \
  cargo test --locked -p nautilus-live --test spec --profile nextest startup_reconciliation -- --nocapture
```

nextest ค้นพบชุด feature เป็น test เดียวชื่อ `startup_reconciliation` target `spec` ยังมี test แยกสำหรับการ
dispose หลัง panic คำสั่งต่อไปนี้จึงเลือกเฉพาะชุด feature:

```bash
CARGO_BUILD_JOBS=4 NEXTEST_TEST_THREADS=2 \
  cargo nextest run --locked -p nautilus-live --test spec --cargo-profile nextest \
  -E 'test(=startup_reconciliation)' --no-capture
```

scenario รันตามลำดับบน runtime thread เดียว โดยใช้ node ใหม่ต่อ scenario test guard จะ dispose node เมื่อ
step ของ startup return หรือ unwind หลัง panic scenario ที่สำเร็จจะขอ stop จาก start callback ของ strategy
จึงไม่ได้พิสูจน์การเข้าสู่ `Running` ทั้ง `start()` และ `run()` ใช้ execution manager, cache และ portfolio จริง
step binding ที่หายไปหรือกำกวม และ assertion ที่ล้มเหลวจะทำให้ชุด test ล้มเหลว filter ที่ไม่เลือก scenario
ใดเลยก็ล้มเหลวเช่นกัน ชุดนี้ใช้ runtime ปกติ จึงไม่ถือเป็นความครอบคลุมแบบ deterministic simulation

#### Rust doctest {#rust-doctests}

`cargo nextest` execute doctest ไม่ได้ จึงต้องรันผ่าน target แยก:

```bash
make cargo-test-doc
# or
cargo test --doc --workspace --features "$(bash scripts/cargo-features.bash)" --profile nextest
```

ตัวอย่างในเอกสารถือเป็นส่วนของ test ที่ต้องดูแล workflow `nightly-tests` ที่ตั้งเวลาไว้จะรัน target นี้
ดู[คู่มือ Rust](rust.md#doc-examples) สำหรับวิธี annotate code fence ให้ compile ได้

#### การทดสอบกับ optional feature {#testing-with-optional-features}

ใช้ `EXTRA_FEATURES` เพื่อรวม optional feature เช่น `sbe` หรือ `hypersync`:

```bash
# Test with SBE support
make cargo-test EXTRA_FEATURES="nautilus-common/sbe"

# Test with multiple features
make cargo-test EXTRA_FEATURES="nautilus-common/sbe hypersync"

# Legacy shorthand for hypersync
make cargo-test HYPERSYNC=true

# Test specific crate with features
make cargo-test-crate-nautilus-serialization FEATURES="sbe"
```

## รูปแบบการเขียน test {#test-style}

### ทั่วไป {#general}

- ตั้งชื่อ function ของ test ตามสิ่งที่ทดสอบ ไม่จำเป็นต้องใส่ assertion ที่คาดหวังลงในชื่อ
- เพิ่มคำอธิบาย (doc comment) เมื่อช่วยให้ setup, scenario หรือความคาดหวังชัดเจนขึ้น
- **รวม assertion ไว้ด้วยกัน** เมื่อทำได้: ทำขั้นตอน setup/act ทั้งหมดก่อน แล้วค่อย assert พร้อมกัน
  เพื่อหลีกเลี่ยงรูปแบบที่ไม่ดีแบบ act-assert-act
- ใช้ `unwrap`, `expect` หรือเรียก `panic!`/`assert` โดยตรงใน test ได้ ความชัดเจนและความกระชับสำคัญกว่า
  การจัดการ error แบบป้องกันในที่นี้
- สำหรับ behavior test ให้ assert ค่าที่ return, การเปลี่ยน state หรือ side effect แทนการ assert log message
  การจับ log ใช้ global state ขึ้นกับลำดับการรัน test และอาจพังเมื่อถ้อยคำเปลี่ยน
- จับ log เฉพาะเมื่อ logging เองคือ contract เช่น การไม่ให้ secret หลุด และการคงบริบทการวินิจฉัยที่เป็น public
  ให้จับ log แบบ serialize และ join task ที่ปล่อย log ก่อนปล่อยการจับ assert เฉพาะ record ที่เกี่ยวข้องและ
  การไม่มี secret หลีกเลี่ยงการ assert ทั้ง transcript

### Rust {#rust}

สำหรับข้อตกลงการเขียน test เฉพาะของ Rust (โครงสร้าง module, `#[rstest]`, การทำ parameterization)
ดู[คู่มือ Rust](rust.md#testing-conventions)

## การรอผลแบบ asynchronous {#waiting-for-asynchronous-effects}

ใน Rust test ให้ใช้ notification channel หรือ event อื่นที่ test เป็นเจ้าของ แทนการประเมินเงื่อนไขซ้ำๆ:

1. subscribe ก่อนอ่าน state ที่เป็นแหล่งข้อมูลจริง
1. ตรวจซ้ำหลังทุก notification เพื่อไม่ให้พลาด transition ที่เกิดระหว่างการอ่านกับการ await

เมื่อไม่มีสัญญาณที่เหมาะสม ให้ใช้ `wait_until_async(...)` จาก `nautilus_common::testing` ซึ่งหยุดทันทีที่เงื่อนไข
สำเร็จและมี timeout จำกัด ใช้ sleep แบบคงที่เฉพาะเมื่อช่วงเวลานั้นเองคือสิ่งที่ทดสอบ

## Mock {#mocks}

เลือกใช้ stub ที่เขียนเองซึ่งคืนค่าคงที่มากกว่า mocking framework ใช้ mock เฉพาะเพื่อ assert จำนวนครั้งที่เรียก
หรือ argument หรือเพื่อจำลองการเปลี่ยน state ที่ซับซ้อน หลีกเลี่ยงการ mock object ที่กำลังทดสอบ

## Code coverage {#code-coverage}

ตั้งเป้า coverage สูงโดยไม่ลดทอนการจัดการ error หรือเปลี่ยนสถาปัตยกรรมเพียงเพื่อ test บาง branch ยังทดสอบไม่ได้
หากไม่แก้พฤติกรรมของ production เช่น เงื่อนไขสุดท้ายในบล็อก if-else แบบป้องกันอาจถูกเรียกเฉพาะกับค่าที่ไม่คาดคิด
ให้คงการตรวจสอบเหล่านี้ไว้ เพื่อให้การเปลี่ยนแปลงในอนาคตทดสอบได้หากจำเป็น

ข้อยกเว้นที่ออกแบบไว้ตั้งแต่แรกก็อาจทดสอบได้ยากในทางปฏิบัติ เป้าหมายจึงไม่ใช่ coverage 100%

## การ debug Rust test {#debugging-rust-tests}

ใช้ test configuration ค่าเริ่มต้นเพื่อ debug Rust test

หากต้องการรันทั้งชุดพร้อม debug symbol ไว้ใช้ภายหลัง ให้รัน `make cargo-test-debug` แทน `make cargo-test`

ใน IntelliJ IDEA ให้ปรับ run configuration ของ case `#[rstest]` แบบ parametrised ให้อ่านเป็น `test --package nautilus-model --lib data::bar::tests::test_get_time_bar_start::case_1`
(ลบ `-- --exact` และต่อท้ายด้วย `::case_n` โดย `n` เริ่มที่ 1) วิธีแก้นี้สอดคล้องกับพฤติกรรมที่อธิบายไว้ใน [rust-analyzer issue 8964](https://github.com/rust-lang/rust-analyzer/issues/8964#issuecomment-871592851)

ใน VS Code สามารถเลือก test case ที่ต้องการ debug ได้โดยตรง

## การทดสอบ data type {#data-type-testing}

data type แต่ละตัวไหลผ่านหลายชั้นของแพลตฟอร์ม ตารางด้านล่างแสดงว่า type ที่มีอยู่ถูกทดสอบที่ใดบ้าง เพื่อให้
type ใหม่ทำตามรูปแบบเดียวกันได้

### Matrix ของชั้น test {#test-layer-matrix}

| ชั้น                    | ตำแหน่ง                                       | สิ่งที่ครอบคลุม                                                 |
| ---------------------- | ------------------------------------------- | ------------------------------------------------------------ |
| DataEngine subscribe   | `crates/data/tests/integration/engine/`     | engine ประมวลผลคำสั่ง subscribe/unsubscribe ได้ถูกต้อง            |
| DataEngine publish     | `crates/data/tests/integration/engine/`     | engine route ข้อมูลที่ publish ไปยัง message bus                   |
| DataActor subscribe    | `crates/common/src/actor/tests.rs`          | actor subscribe และรับข้อมูลผ่าน typed publish                   |
| DataActor unsubscribe  | `crates/common/src/actor/tests.rs`          | actor หยุดรับข้อมูลหลัง unsubscribe                              |
| Adapter live tests     | `docs/developer_guide/spec_data_testing.md` | live data acceptance test (DataTester)                        |

### ความครอบคลุมต่อ data type {#coverage-per-data-type}

ตารางต่อไปนี้แสดงว่าแต่ละ data type มี test ครอบคลุมในชั้นใดบ้าง ใช้เป็น checklist เมื่อเพิ่ม type ใหม่

| Data type           | Engine | Actor (Rust) | Adapter spec |
| ------------------- | ------ | ------------ | ------------ |
| `InstrumentAny`     | ✓      | ✓            | ✓            |
| `OrderBookDeltas`   | ✓      | ✓            | ✓            |
| `OrderBook`         | ✓      | ✓            | ✓            |
| `QuoteTick`         | ✓      | ✓            | ✓            |
| `TradeTick`         | ✓      | ✓            | ✓            |
| `Bar`               | ✓      | ✓            | ✓            |
| `MarkPriceUpdate`   | ✓      | ✓            | ✓            |
| `IndexPriceUpdate`  | ✓      | ✓            | ✓            |
| `FundingRateUpdate` | ✓      | ✓            | ✓            |
| `InstrumentStatus`  | ✓      | ✓            | ✓            |
| `InstrumentClose`   | ✓      | ✓            | ✓            |
| `OptionGreeks`      | ✓      | ✓            | ✓            |
| `OptionChainSlice`  | -      | ✓            | ✓            |
| `CustomData`        | ✓      | ✓            | -            |

`OptionChainSlice` ถูกประกอบโดย `OptionChainManager` ของ DataEngine จาก subscription ของ greeks และ quote
ราย instrument จึงไม่มีคำสั่ง subscribe ของ engine เป็นของตัวเอง

### การเพิ่ม data type ใหม่ {#adding-a-new-data-type}

เมื่อเพิ่ม data type ใหม่ ให้เพิ่ม test ในแต่ละชั้น:

1. **DataEngine** (`crates/data/tests/integration/engine/`): เพิ่ม test `test_execute_subscribe_<type>` และ
   `test_execute_unsubscribe_<type>` ตามรูปแบบของ subscribe test ที่มีอยู่: ลงทะเบียน client, สร้างคำสั่ง,
   เรียก `engine.execute` แล้ว assert รายการ subscription

1. **DataActor Rust** (`crates/common/src/actor/tests.rs`):
   - เพิ่มฟิลด์ `received_<type>: Vec<Type>` ใน `TestDataActor`
   - implement handler `on_<type>` ใน impl ของ trait `DataActor`
   - เพิ่ม test `test_subscribe_and_receive_<type>` และ `test_unsubscribe_<type>`
   - ใช้ typed publish function (`msgbus::publish_<type>`) แทน `publish_any`
     สำหรับ type ที่ใช้การ route แบบ `TypedHandler`

1. **เอกสาร**: เพิ่มรายการในตาราง callback ของ `actors.md`, signature ของ handler ใน `strategies.md`,
   stub ของ method subscribe ใน `adapters.md` และ test card ใน `spec_data_testing.md`

:::tip
ค้นหา type ที่มีอยู่แล้ว เช่น `instrument_close` หรือ `funding_rate` ในทุกชั้น เพื่อดูตัวอย่างจริงของรูปแบบที่อธิบายไว้ข้างต้น
:::
