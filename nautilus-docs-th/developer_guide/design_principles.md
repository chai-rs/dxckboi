# หลักการออกแบบ

หน้านี้กำหนดหลักการ นโยบาย และ trade-off ที่ใช้เป็นแนวทางในการออกแบบ การ implement และการ review
NautilusTrader ส่วน [Architecture](../concepts/architecture.md) อธิบาย component และโครงสร้างขณะ runtime

นโยบายเหล่านี้ไม่ได้ยืนยันว่าทุก path ที่มีอยู่ในปัจจุบันเป็นไปตามนโยบายแล้ว คู่มือเฉพาะเรื่องจะอธิบายพฤติกรรมและข้อจำกัด
ปัจจุบัน ส่วน [runtime conformance contract](runtime_conformance.md) จะจับคู่ข้อกำหนดที่เลือกไว้กับ source code,
การตรวจสอบตัวแทน และข้อจำกัดของการ implement

## ลำดับความสำคัญในการออกแบบ {#design-priorities}

การตัดสินใจด้านการออกแบบจะชั่งน้ำหนักคุณลักษณะด้านคุณภาพเหล่านี้โดยประมาณตามลำดับดังนี้:

- Reliability (ความน่าเชื่อถือ)
- Performance (ประสิทธิภาพ)
- Modularity (ความเป็นโมดูล)
- Testability (ความสามารถในการทดสอบ)
- Maintainability (ความสามารถในการดูแลรักษา)
- Deployability (ความสามารถในการ deploy)

การปรับปรุง performance ต้องรักษา invariant ที่สำคัญและการตรวจสอบที่จำเป็นของ invariant นั้นไว้ testability และ
maintainability ช่วยค้ำจุน reliability ส่วน deployability มีผลต่อการ rollout, การตั้งค่า และการกู้คืนอย่างปลอดภัย
ลำดับความสำคัญนี้ไม่ได้ทำให้คุณลักษณะใดเป็นเรื่องที่ไม่จำเป็น

## ความถูกต้องของข้อมูลและความล้มเหลว {#data-integrity-and-failure}

NautilusTrader ให้ความสำคัญกับความถูกต้องของข้อมูล (data integrity) มากกว่าความพร้อมใช้งาน (availability) สำหรับ
การดำเนินการด้านการเทรด ขอบเขตของการคำนวณทางคณิตศาสตร์และการจัดการข้อมูลจะคืนค่า error หรือ panic แทนที่จะ
ยอมรับค่าที่ไม่ถูกต้องแบบเงียบ ๆ ซึ่งอาจส่งผลต่อการตัดสินใจเทรด

### การดำเนินการที่ไม่ถูกต้อง {#invalid-operations}

ระบบจะ fail fast โดยคืนค่า error หรือ panic ตาม API contract ในกรณีต่อไปนี้:

- arithmetic overflow หรือ underflow ในการคำนวณกับ timestamp, ราคา หรือ quantity ที่เกินช่วงที่ถูกต้อง
- ข้อมูลไม่ถูกต้องระหว่าง deserialization รวมถึง NaN, infinity หรือค่านอกช่วงใน market data หรือ config
- การแปลงชนิดข้อมูลล้มเหลว เช่น ค่าติดลบในที่ที่รับได้เฉพาะค่าบวก (timestamp, quantity)
- การ parse input ที่ผิดรูปแบบสำหรับราคา, timestamp หรือค่า precision

ในระบบเทรด ราคา timestamp หรือ quantity ที่ผิดเพียงค่าเดียวอาจลุกลามไปเป็น:

- การคำนวณขนาด position หรือความเสี่ยงที่ผิดพลาด
- order ที่ถูกส่งที่ราคาผิด
- backtest ที่ให้ผลลัพธ์ชวนเข้าใจผิด
- การขาดทุนทางการเงินแบบเงียบ ๆ

การล้มเหลวทันทีที่จุดของการดำเนินการที่ไม่ถูกต้องให้ประโยชน์ดังนี้:

- **ไม่มีความเสียหายแบบเงียบ**: input ที่ผ่านการตรวจสอบจะล้มเหลวก่อนที่ค่าที่ไม่ถูกต้องจะลุกลาม
- **ได้ feedback ทันที**: ผู้เรียกจะได้รับ error หรือ process จะหยุดทำงาน ณ จุดที่ contract ถูกละเมิด
- **มีบริบทสำหรับวินิจฉัย**: ข้อความ error และ panic ระบุการดำเนินการหรือค่าที่ถูกปฏิเสธ
- **พฤติกรรมเป็น deterministic**: เมื่อลำดับและ config เป็น deterministic input ที่ไม่ถูกต้องแบบเดียวกันจะให้ความล้มเหลว
  แบบเดียวกัน แต่ input ที่ไม่ deterministic ยังอาจทำให้ผลลัพธ์แตกต่างกันได้

ความล้มเหลวที่คาดการณ์ได้ เช่น ด้าน network, storage, business validation และ user input มีช่องทาง error ที่ชัดเจน
ส่วนการละเมิด invariant ที่กู้คืนไม่ได้จะหยุดการดำเนินการหรือ process ก่อนที่ state ที่ไม่ถูกต้องจะลุกลาม
[Rust error contracts](rust.md#failure-contract-examples) กำหนดพฤติกรรมของ panic และ API ที่อาจล้มเหลว (fallible)

### การจำกัดขอบเขตความล้มเหลว {#failure-containment}

ให้หยุดขอบเขตที่เล็กที่สุดซึ่งไม่สามารถยืนยันความถูกต้องได้อีกต่อไป หากส่วนที่เหลือของระบบยังทำงานต่อได้อย่างปลอดภัย
input ที่ถูกปฏิเสธหรือการดำเนินการที่ล้มเหลวไม่จำเป็นต้องหยุด component ที่ไม่เกี่ยวข้อง เมื่อพิสูจน์ได้ว่ามีการแยกส่วน
(isolation) แล้ว shared state ที่ไม่น่าเชื่อถืออาจต้องหยุดทั้ง node การจำกัดขอบเขตต้องเป็นไปตาม failure contract
ของ API และขอบเขตการแยกส่วนที่พิสูจน์แล้ว และต้องไม่สันนิษฐานว่า component จะกู้คืนจาก panic ใด ๆ ก็ได้

การหยุด process ไม่ได้ยกเลิก order ที่ยังทำงานอยู่บน venue หรือลด exposure การกู้คืนต้องยืนยัน state ของ venue
ให้ได้ก่อน จึงจะตัดสินใจได้ว่าการกระทำต่อไปใดที่ปลอดภัย

## Invariant ที่ตรวจสอบได้ด้วยการรัน {#executable-invariants}

NautilusTrader นำแนวปฏิบัติแบบ high-assurance มาใช้กับ critical path ทีละขั้น executable invariant ช่วยตรวจสอบว่า
พฤติกรรมตรงกับ business requirement:

- ระบุ component ที่มีผลกระทบสูง รวมถึง core domain type และ flow ด้าน risk และ execution แล้วเขียน invariant
  ของสิ่งเหล่านั้นเป็นภาษาธรรมดา
- แปลง invariant เหล่านั้นเป็นการตรวจสอบที่รันได้ (unit test, property test, fuzzer และ static assertion) ที่รันใน CI
- บังคับใช้ invariant ด้าน ownership และ state ผ่าน type และ failure contract ที่ชัดเจน และเพิ่มเครื่องมือแบบ formal
  ในจุดที่ประโยชน์ด้าน assurance คุ้มกับต้นทุน
- กำหนดให้ integration ต้องรักษา invariant ของ critical path ที่มีอยู่ และเพิ่ม executable coverage สำหรับ invariant
  ที่ integration นั้นสร้างขึ้นใหม่หรือเปลี่ยนแปลง

แนวทางนี้ทำให้ flow ที่มีความเสี่ยงสูงได้รับการตรวจสอบเข้มข้นขึ้น โดยไม่ต้องแบกต้นทุนด้าน assurance เท่ากันในทุก path

อ่านเพิ่มเติม: [High Assurance Rust](https://highassurance.rs/)

## Message immutability {#message-immutability}

message (request, response, event และ command) เป็น immutable หลังจากถูกสร้าง ฟิลด์ต่าง ๆ จะไม่เปลี่ยนแปลงตลอด
อายุของ message ดูกฎด้าน ownership ที่เป็นผลมาจาก invariant นี้ได้ที่
[Message Bus: message integrity](../concepts/message_bus.md#message-integrity)

invariant นี้ปกป้องคุณสมบัติหลายอย่างที่ระบบต้องพึ่งพา:

- **input ที่คงที่**: consumer ทุกตัวเห็น payload ของ message เหมือนกัน การ replay ลำดับ message จะคง logical input
  ดั้งเดิมไว้สำหรับ backtest, การสร้างเหตุการณ์ incident ขึ้นใหม่ และ regression test
- **ความถูกต้องเชิงเวลา**: message เก็บสิ่งที่ผู้สร้างรายงาน สังเกต ร้องขอ หรืออนุมานไว้ ณ เวลาที่สร้าง ให้รักษา
  provenance (ที่มาของข้อมูล) ที่มีอยู่ไว้ immutability ไม่ได้ยืนยันว่าข้อมูลของผู้สร้างเป็นความจริง การแก้ไขต้องทำผ่าน
  record ใหม่อย่างชัดเจน
- **concurrency ที่ปลอดภัยขึ้น**: ผู้อ่านไม่ต้องประสานงานกันเพื่อป้องกัน payload ของ message จากการถูกเขียนทับภายหลัง
  ช่วยตัดแหล่งที่มาของ race condition ที่พบบ่อยรอบ ๆ shared state
- **การ debug และการตรวจสอบย้อนหลัง**: log, trace, เครื่องมือ replay และการตรวจสอบ dead-letter เก็บ payload ดั้งเดิมไว้
  ทำให้สืบค้นได้ว่าระบบได้รับหรือสร้างอะไร เกิดขึ้นเมื่อใด และระบบทำอะไรกับข้อมูลนั้น
- **ขอบเขต ownership ที่ชัดเจน**: component ถือว่า message ที่เข้ามาเป็น input หาก component ต้องการรูปแบบอื่น
  ก็ต้องสร้าง local state ใหม่หรือ message ใหม่อย่างชัดเจน
- **การกระจายที่ทนทานขึ้น**: message ที่ serialize แล้วข้ามขอบเขตของ process และ service ในรูปของสำเนาอยู่แล้ว
  กฎ ownership เดียวกันจึงทำให้ in-memory model สอดคล้องกับความเป็นจริงนี้

## หลักฐานและแหล่งอำนาจ {#evidence-and-authority}

แยกแยะระหว่างรายงานจากภายนอก การสังเกตภายในระบบ การอนุมาน และการตัดสินใจเชิงนโยบาย รักษาแหล่งที่มาและความไม่แน่นอน
ของสิ่งเหล่านี้ไว้เมื่อนำไปสร้าง state ค่าตัวเลขที่ถูกต้องหรือการเปลี่ยน state ที่ถูกต้อง ไม่ได้ยืนยันข้อเท็จจริงฝั่ง venue
ด้วยตัวมันเอง หลักฐานที่ขาดหายต้องไม่กลายเป็นผลลัพธ์ที่ยืนยันแล้ว เพียงเพราะ timeout หรือ retry limit หมดลง

[Execution policies](../concepts/execution/policies.md) นำกฎนี้ไปใช้กับผลลัพธ์ของ command, reconciliation และ
ข้อจำกัดของประวัติที่เก็บไว้

## การควบคุม nondeterminism {#controlled-nondeterminism}

ทำให้เวลา ความสุ่ม ลำดับของ input และผลกระทบภายนอกชัดเจน ณ ขอบเขตที่ใช้สิ่งเหล่านั้น test ที่ทำซ้ำได้ต้องควบคุมแหล่ง
ที่มีผลต่อ assertion และเก็บ input, seed และ config ที่จำเป็นสำหรับสืบสวนความล้มเหลวไว้ ระบุเงื่อนไขด้าน binary,
platform และ input ของการรับประกัน determinism ใด ๆ ให้ชัดเจน

[DST contract](../concepts/dst.md) กำหนดขอบเขตที่รองรับของการ execute แบบควบคุมด้วย seed พฤติกรรมของ live venue
และ input ภายนอกที่เป็นอิสระยังคงอยู่นอกการรับประกันนั้น

## การใช้ทรัพยากรแบบมีขอบเขต {#bounded-resource-use}

ออกแบบ queue, retry, ประวัติที่เก็บไว้ และ callback chain ให้มี resource budget และพฤติกรรมเมื่อทรัพยากรหมดที่ชัดเจน
หากการใช้หน่วยความจำแปรผัน ให้คำนึงถึงขนาด payload ด้วยนอกเหนือจากจำนวน item จำกัดทั้งปริมาณงานและพื้นที่จัดเก็บ
เพื่อไม่ให้ queue ที่ถูกเติมอยู่เรื่อย ๆ ผูกขาดการ execute

เลือกระหว่าง backpressure, การปฏิเสธ หรือการหยุดทำงานอย่างปลอดภัย ตาม contract ที่ได้รับผลกระทบ ภาวะ overload ต้องไม่
ทำให้การเปลี่ยน state ที่จำเป็นสูญหายแบบเงียบ ๆ หรือทิ้งการดำเนินการที่ทำไปเพียงบางส่วนไว้โดยแสดงว่าเสร็จสมบูรณ์แล้ว
ทั้งหมดนี้เป็นข้อกำหนดด้านการออกแบบ path ที่มีอยู่ยังอาจไม่มีขอบเขตได้ ตามที่บันทึกไว้ใน [live dispatch and overload
behavior](../concepts/live.md#dispatch-priority-and-overload-behavior)

## การกู้คืนหลังความล้มเหลว {#recovery-after-failure}

fault ที่กู้คืนไม่ได้ต้องไม่ปล่อยให้ระบบทำงานต่อบน state ที่อาจไม่ถูกต้อง การ startup ตามปกติรวมการกู้คืน state ที่
ตั้งค่าไว้ ดังนั้นการ restart จึงใช้ initialization path เดียวกับการ startup ทั่วไป การกู้คืนขึ้นอยู่กับ state ที่เก็บไว้และ
durability ของ backing store

supervisor ภายนอกเป็นผู้รับผิดชอบการ restart หลังจาก process ล้มเหลว การกู้คืนมุ่งลด downtime ให้น้อยที่สุด ระยะเวลา
ขึ้นอยู่กับ state ที่ต้องกู้คืนและ backing store การกู้คืนด้าน execution ต้อง reconcile state ของ venue แทนที่จะ retry
command ไปยัง venue แบบไม่ไตร่ตรอง การทำงานปกติยังคงรองรับ graceful shutdown ส่วน fault ที่กู้คืนไม่ได้อาจทำให้การ
cleanup ไม่ปลอดภัย

[Runtime failure and recovery](../concepts/architecture.md#crash-only-design) อธิบายขอบเขตของ process และ
persistence

## ขอบเขตของ domain และ integration {#domain-and-integration-boundaries}

domain type และ contract กำหนดพฤติกรรมด้านการเทรด component สื่อสารกันผ่าน interface ที่ชัดเจนและ message ที่เป็น
immutable แนวทาง ports and adapters ทำให้ implementation ของ venue transport และ backing store อยู่นอก trading core
ที่ใช้ร่วมกัน ดังนั้น custom integration จึงรักษา domain contract เดียวกันไว้ได้

## พฤติกรรมของ backtest และ live {#backtest-and-live-behavior}

สภาพแวดล้อม backtest, sandbox และ live ใช้ core trading component และ behavioral contract ร่วมกัน code ของ strategy
และ execution algorithm ชุดเดียวกันสามารถรันได้ในทุกสภาพแวดล้อมเหล่านี้ live execution ยังนำพฤติกรรมด้าน venue,
transport, จังหวะเวลา, persistence, กิจกรรมภายนอก และ reconciliation เข้ามาด้วย ซึ่ง simulation อาจจำลองได้ไม่ครบ
การใช้ code ร่วมกันไม่ได้หมายความว่าจะได้ผลลัพธ์เหมือนกันเมื่อ input หรือสภาพการทำงานต่างกัน

[Common core](../concepts/architecture.md#common-core) จัดเตรียม engine และ interface ส่วน
[Behavioral models](../concepts/behavioral_models.md) กำหนดวิธีที่ implementation ของ model เข้าสู่ runtime

## ข้อกำหนดของ queued callback dispatch {#queued-callback-dispatch-requirements}

การส่งมอบ callback ให้ actor และ strategy ตามแบบแผนต้องรักษาลำดับการ publish, การเข้าถึง component แบบ exclusive และ
คุณสมบัติตาม lifecycle ไว้ พร้อมความคืบหน้าที่มีขอบเขต reentrancy ต้องไม่เปลี่ยนกฎของลำดับ การส่งมอบตามลำดับไม่ได้
หมายความว่าจะได้ cache snapshot ณ เวลาของ event

runtime เป็นผู้รับผิดชอบการจัดตาราง drain ณ ขอบเขต ownership ที่ชัดเจนจำนวนไม่มาก หลังจาก mutable borrow ที่ครอบอยู่
สิ้นสุดลง method สำหรับ dispatch, flush และ lifecycle แต่ละตัวต้องไม่มี draining policy ของตัวเอง ให้การจัดการความล้มเหลว
และการ cleanup อยู่ภายใต้ lifecycle owner ที่มีอยู่ แชร์ code สำหรับการจัดตารางเฉพาะในจุดที่ช่วยลดความซับซ้อน โดยยังรักษา
กฎ settlement ของ backtest และกฎการ yield ของ live ไว้

ลำดับ callback ที่เป็น deterministic ใน synchronous core และใน backtest เป็นเกณฑ์ที่จำเป็นสำหรับการเปิดใช้งาน ให้ตรวจสอบ
ลำดับที่แน่นอนผ่าน native component จริง รวมถึงการ publish ซ้อนกัน, fan-out, command ที่ callback สร้างขึ้น, งานที่มี
timestamp เดียวกัน และการทำงานต่อเนื่องข้ามรอบ drain ที่มีขอบเขต จำนวนและยอดคงเหลือสุดท้ายเพียงอย่างเดียวไม่สามารถ
ยืนยันลำดับได้

สิ่งเหล่านี้เป็นข้อกำหนดสำหรับ queued dispatch ไม่ใช่การรับประกันของ synchronous path ที่มีอยู่
[Callback dispatch contract](callback_dispatch.md) ระบุจังหวะของงาน maintenance, ขอบเขต ownership, การทำให้
lifecycle เป็นโมฆะ และพฤติกรรมการ drain ส่วน
[runtime conformance contract](runtime_conformance.md#callback-ordering-and-ownership) บันทึก coverage ของการ
implement และข้อกำหนดสำหรับการเปิดใช้งานที่ยังค้างอยู่
