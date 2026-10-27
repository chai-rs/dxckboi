---
title: "Data Testing Spec"
---

ใช้ test matrix นี้เพื่อตรวจสอบความสามารถด้านข้อมูลของ adapter ด้วย actor `DataTester` ของ Rust ซึ่งใน Rust
import ได้จาก `nautilus_testkit::testers` แต่ละ case ใช้ ID ที่มี prefix (เช่น TC-D01) และจัดกลุ่มตามความสามารถ

**adapter แต่ละตัวต้องผ่านชุดย่อยของ test ที่ตรงกับประเภทข้อมูลที่ตนรองรับ**

กลุ่มของ test เรียงจากข้อมูลที่ derive น้อยที่สุดไปหามากที่สุด: instrument และข้อมูล book ดิบก่อน
ตามด้วย quote, trade, bar และข้อมูลอนุพันธ์ adapter ที่ผ่านกลุ่ม 1-4 ถือว่าเป็นไปตามข้อกำหนดด้านข้อมูลขั้นพื้นฐาน
(baseline data compliant)

ให้บันทึกพฤติกรรมด้านข้อมูลเฉพาะของ adapter (channel เฉพาะ, throttling, ความหมายของ snapshot ฯลฯ)
ไว้ในคู่มือของ adapter นั้นเอง ไม่ใช่ที่นี่

## สิ่งที่ต้องเตรียม [#prerequisites]

ก่อนรัน data test:

- instrument เป้าหมายต้องมีอยู่และโหลดได้ผ่าน instrument provider
- ตั้ง API credential ผ่าน environment variable (`{VENUE}_API_KEY`, `{VENUE}_API_SECRET`) เมื่อ venue ต้องการ
  การยืนยันตัวตนสำหรับข้อมูลที่กำลังทดสอบ
- หาก venue มีโหมด demo/testnet ให้ใช้ credential ที่สร้างสำหรับ environment นั้น API key ของ demo และ production
  มักแยกกันและใช้แทนกันไม่ได้ การใช้ credential ผิดชุดจะทำให้เกิด error ด้านการยืนยันตัวตน (เช่น HTTP 401)

**การตั้งค่า node ฝั่ง Rust** (อ้างอิง: `crates/adapters/{adapter}/examples/node_data_tester.rs`):

```rust
use nautilus_testkit::testers::{DataTester, DataTesterConfig};

let tester_config = DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_quotes(true)
    .build()?;
let tester = DataTester::new(tester_config);
node.add_actor(tester)?;
node.run().await?;
```

## สเกลของ timestamp [#timestamp-scale]

Nautilus เก็บ `ts_event` และ `ts_init` เป็น Unix nanosecond (`UnixNanos`) ทุก data message ที่มีฟิลด์เหล่านี้
ต้องใช้สเกลนี้ ไม่ใช่ค่าวินาที, มิลลิวินาที หรือไมโครวินาทีที่หลงเหลือมา

- ค่าที่ต่ำกว่า `10^16` ไม่น่าจะเป็น timestamp แบบ Unix nanosecond ที่สมเหตุสมผล (`10^16` ns คือประมาณ
  116 วันหลังจาก 1970-01-01) และมักหมายความว่า adapter ไม่ได้แปลงสเกลจาก venue
- เวลาจาก venue ที่ละเอียดระดับวินาทีและแปลงถูกต้องแล้วจะลงท้ายด้วย `000000000` และยังผ่าน: นั่นคือความละเอียด
  ที่หยาบ ไม่ใช่ความผิดพลาดของสเกล
- `ts_event` ของ live stream ควรใกล้เคียงกับเวลาจริง (wall-clock) ของ session ผลลัพธ์จากการ request ข้อมูล
  ย้อนหลังอาจเก่ากว่าและยังถูกต้องหากสเกลเป็น nanosecond
- `ts_init` คือ clock ในเครื่อง ณ เวลาที่ Nautilus สร้าง object อาจมีความคลาดเล็กน้อยที่ `ts_event` >
  `ts_init` ได้เมื่อ clock ของ venue เดินเร็วกว่า

`DataTester` จะเตือนเมื่อ `ts_event` หรือ `ts_init` ไม่ผ่านการตรวจสเกลบนประเภทข้อมูลเหล่านี้ รวมถึงชุดข้อมูล
ย้อนหลังของแต่ละประเภท:

- instrument, instrument status และ instrument close
- book delta และ book depth
- quote, trade และ bar
- mark price, index price และ funding rate
- option greeks

มันไม่ได้ตรวจ book ที่สร้างขึ้นใหม่ใน `on_book` ให้ถือว่าคำเตือนเป็นความล้มเหลวของ case ที่สร้าง message นั้น

---

## ความสอดคล้องของการ sync order book [#order-book-sync-conformance]

adapter ที่ดูแล order book จาก stream ของ venue ต้องรักษา output ให้ถูกต้องตลอดเวลาแม้เกิดความผิดพลาดจาก venue
และเครือข่าย ชุด unit test และ integration test ไม่สามารถจำลองจังหวะเวลาของ venue ได้ ดังนั้นการเปลี่ยนแปลงกลไก
การ sync และการกู้คืนของ book ต้องผ่านทั้งการตรวจด้วย model แบบ deterministic และการตรวจสอบแบบ live กับ venue จริง

**การตรวจสอบแบบ live** ในที่นี้หมายถึงการสังเกตเฉพาะ market data: subscribe, request และฉีด fault ห้ามส่ง order
ระหว่างการตรวจสอบนี้ **dark book** คือ book ที่ subscribe แล้วแต่ไม่เคยได้รับข้อมูลเลย

### ข้อตกลงของ book stream [#book-stream-contract]

`BookStreamChecker` ใน `nautilus_live::book::conformance` ซึ่งเปิดใช้ด้วย feature `test-support` ของ
`nautilus-live` จะใช้ข้อตกลงนี้กับ batch `OrderBookDeltas` ทุกตัวที่ถูกส่งออกมา:

- batch ลงท้ายด้วย `F_LAST` แต่ละ `F_LAST` ปิดกลุ่มของ event หนึ่งกลุ่ม และ book ต้องผ่านการตรวจ integrity หลัง
  ทุกกลุ่ม เพราะผู้ใช้ข้อมูลจะเห็น book ที่ขอบเขตเหล่านั้น
- กลุ่ม snapshot คือ `Clear` ตามด้วย delta แบบ `Add` ซึ่งติด flag `F_SNAPSHOT` ทั้งหมด `Clear` ที่อยู่ลำพัง
  คือ snapshot ว่าง
- กลุ่ม incremental ไม่มีทั้ง `F_SNAPSHOT` และ `Clear` และตามหลัง snapshot
- sequence ของกลุ่ม incremental แต่ละกลุ่มต้องมากกว่ากลุ่มก่อนหน้า เมื่อ sequence ของ venue เพิ่มขึ้นแบบ monotonic
  ภายในช่วง snapshot เดียว `seqId` ของ OKX อาจ reset ได้ ส่วน book ของ Polymarket, Hyperliquid, Betfair และ
  AX Exchange ไม่มี sequence จาก venue ดังนั้น checker ของ venue เหล่านี้ทั้งหมดจะข้ามกฎนี้และอาศัย oracle แทน
- book จะไม่ส่งอะไรออกมาอีกหลังการ unsubscribe เสร็จสิ้น

### ระดับของการตรวจสอบ [#validation-levels]

เลือกระดับให้เหมาะกับส่วนที่เสี่ยงที่สุดของการเปลี่ยนแปลง ระดับที่สูงกว่ารวมเกณฑ์ของทุกระดับที่ต่ำกว่าด้วย

| Level              | Trigger                                                                                          | วิธีการ                                                                                          | เกณฑ์การยอมรับ                                                                                                                          |
| ------------------ | ------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| L0 Model           | การเปลี่ยนแปลงใด ๆ ต่อ `BookSync`, `BookRecovery` หรือการที่ adapter ใช้งานสิ่งเหล่านั้น                        | property test ของ state machine ราย book เทียบกับ model อ้างอิง พร้อม regression ที่ฝังไว้                   | property test ผ่าน และการย้อนการแก้ไขที่รู้จัก เช่นกฎความเป็นเจ้าของ gap ทำให้ test ล้มเหลว                                                      |
| L1 Conformance     | การเปลี่ยนแปลงใด ๆ ต่อโค้ด sync/recovery ที่เคยตรวจสอบแล้ว                                                   | รัน stress harness หรือ oracle ที่ใช้อยู่ของ venue ซ้ำ                                                   | PASS ตามเกณฑ์ที่บันทึกไว้, ไม่มีการละเมิดจาก checker, ไม่มี dark book, ไม่มี error ที่อธิบายไม่ได้                                                  |
| L2 Edge probe      | การเปลี่ยนพฤติกรรมที่ขอบเขต (timeout, เส้นทางที่ปิดไว้, budget หมด, เพดานของการ retry)                          | scenario ที่ขอบเขตแบบเจาะจง รวมถึงค่าปรับแต่งสุดขั้วใหม่แต่ละค่า                                                | ทุก scenario ผ่าน เส้นทางที่ปิดไว้ยังเงียบ budget ที่หมดแล้วไปถึงเพดาน และ snapshot ที่มาช้ายังกู้ book ได้                                       |
| L3 Race probe      | การเปลี่ยนแปลงด้าน concurrency หรือลำดับ (gate, epoch, ปฏิสัมพันธ์กับ reconnect) หรือการแก้ race ที่พบจาก live | ฉีด fault พร้อม subscribe churn ภายใต้ oracle ที่เป็นอิสระ                                              | การกู้คืนที่บังคับให้เกิดหลายสิบครั้งเสร็จสิ้นโดยไม่มี dark book และ scenario ของ race ที่รายงานไว้ผ่านโดยไม่เกิดซ้ำ                                      |
| L4 Full validation | implementation ของ sync/recovery ใหม่                                                              | L1-L3 พร้อม soak test ที่มี churn และ fault จาก reconnect ต่อเนื่อง                                       | เกณฑ์ระดับล่างทั้งหมดผ่านตลอดการ soak และ latency ของการกู้คืนอยู่ในขอบเขต                                                                    |

property test ของ L0 คือ `schedule_keeps_sync_contract` ใน `crates/live/src/book/sync.rs`
ให้บันทึก level, venue, oracle และผลลัพธ์ไว้กับการเปลี่ยนแปลง การแก้ไขที่พบจากการตรวจสอบแบบ live
ต้องเริ่มใหม่ที่ระดับที่พบปัญหานั้น: การรันซ้ำต้องผ่านเกณฑ์เดิม ไม่ใช่เกณฑ์ที่เบากว่า

### แคตตาล็อกของ fault [#fault-catalog]

adapter ทุกตัวต้องให้ผลลัพธ์เหล่านี้ ไม่ว่าจะอยู่ใน
[recovery family](adapters.md#order-book-recovery-ownership) ใด:

| Fault                       | ผลลัพธ์ที่ต้องการ                                                                                                                                          |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Sequence gap                | output หยุดที่ gap และกลับมาส่งต่อเมื่อ snapshot ใหม่แทนที่ book แล้วเท่านั้น                                                                                          |
| Snapshot หายหรือมาช้า          | deadline ของ snapshot จะเริ่มหรือ retry การกู้คืน snapshot ที่ยอมรับระหว่างความพยายามแต่ละครั้งจะยุติการกู้คืน                                                              |
| เริ่มการกู้คืนไม่ได้                | book ยังไม่มีเจ้าของและจะขอการกู้คืนอีกครั้งใน frame ถัดไป task ที่ถูกปฏิเสธจะยกเลิกช่วงที่ตนอ้างสิทธิ์ไว้                                                                  |
| การแทนที่ถูกปฏิเสธ              | การกู้คืน retry error ที่ classifier ระบุว่าถาวรจะข้าม budget และ retry ที่เพดาน                                                                                    |
| Retry budget หมด             | log error หนึ่งครั้ง แล้ว retry ที่เพดานจนกว่าจะยอมรับ snapshot ได้                                                                                                  |
| Reconnect ระหว่างการกู้คืน       | การกู้คืนที่กำลังรันอยู่คง budget, ความเป็นเจ้าของ และการเขียนที่ค้างอยู่ไว้ และการรอเพดานครั้งถัดไปจะจบทันที book อื่นจะ resync จาก snapshot ใหม่                                 |
| Unsubscribe ระหว่างการกู้คืน     | การกู้คืนและการเขียนที่ค้างอยู่หยุดลง และ book ไม่ส่งอะไรออกมาอีก                                                                                                   |

### เทคนิคการบังคับให้เกิดสถานการณ์ [#forcing-techniques]

ให้ความสำคัญกับลำดับเหตุการณ์ที่แตกต่างมากกว่าปริมาณดิบ probe จะคุ้มค่าก็ต่อเมื่อมันบังคับลำดับที่ชุด test ทำให้เกิด
ไม่ได้ (reconnect ระหว่างการกู้คืน, snapshot ที่แข่งกับการเปิด gate, unsubscribe ที่แข่งกับ subscribe ที่ยังค้างอยู่)
จำนวน message เพียงอย่างเดียวไม่ใช่เหตุผลที่เพียงพอสำหรับ probe

| เทคนิค                                                                       | สิ่งที่ทดสอบ                                                            | ตัวเลขที่พิสูจน์แล้วว่าได้ผล                                                                   | สิ่งที่จับได้จริง                                                              |
| --------------------------------------------------------------------------- | -------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| Subscribe churn (หมุน unsubscribe/resubscribe พร้อม flap เต็มรูปแบบเป็นระยะ)      | การเริ่มกู้คืน, การเปลี่ยนรอบ gate/epoch, race ของการยกเลิกที่ค้างอยู่             | tick ทุก 20 วินาทีตลอดการรัน 10-15 นาที การกู้คืนที่บังคับหลายสิบครั้ง (40+) โดยไม่มี dark book           | ข้อบกพร่องของ subscribe ซ้ำที่กู้คืนไม่ได้ (นำไปสู่การทบทวน design)                  |
| Traffic freeze (STOP tunnel ~40 วินาที)                                      | การตรวจจับ connection ที่ตาย, replay หลัง reconnect, การกู้คืนหลัง reconnect | freeze 2-3 ครั้งต่อการรัน เว้นระยะห่างกันหลายนาที                                                 | พิสูจน์การกู้คืนหลัง reconnect ภายใต้ packet loss ทั้งหมด ยังไม่พบข้อบกพร่อง             |
| Proxy fault injection (drop/hold/cut frame ตามกฎ)                           | การจัดการ gap, การปล่อย frame ที่ถูกถือไว้, ความสอดคล้องกับ oracle               | oracle batch หลายพันชุดต่อการรัน (6k+), นับ gap ต่อรอบ                                        | race ของ harness ที่ขึ้นกับ timeout (หน้าต่างสังเกตคงที่เทียบกับค่าเริ่มต้นใหม่)            |
| ค่าปรับแต่งสุดขั้ว (0 และค่าสั้นที่ไม่ใช่ค่าเริ่มต้น)                                        | branch ที่ปิด deadline, การส่งพารามิเตอร์ต่อตลอดเส้นทาง                       | รันสั้นหนึ่งครั้งต่อค่าสุดขั้ว (4-5 นาที) ขณะมี churn                                               | ยืนยันการแก้ไข zero-timeout ที่พบจาก review บน live และพิสูจน์การส่งพารามิเตอร์         |
| Reconnect ที่ client สั่ง (คำสั่ง reconnect สาธารณะ แล้วทดสอบต่อ)                  | การกู้คืนหลัง reconnect โดยไม่แตะเครือข่ายของเครื่อง host                       | ผ่าน reconnect/reconcile ติดต่อกัน 5+ ครั้ง                                                    | พิสูจน์การกู้คืนโดยไม่มี fault จาก host ยังไม่พบข้อบกพร่อง                          |
| รัน scenario ของ race ซ้ำต่อเนื่อง                                               | ความไวต่อ scheduler                                                    | ผ่าน live ติดต่อกัน 5+ ครั้ง และรันซ้ำ 100 เท่าสำหรับ harness แบบ deterministic                      | ความไม่เสถียร (flake) ที่ผ่านครั้งหนึ่งแต่ล้มเหลวนาน ๆ ครั้ง                           |

ให้ route แต่ละ venue ผ่านตำแหน่งเครือข่ายที่ venue นั้นให้บริการ Polymarket จำกัดการเข้าถึงตามภูมิภาค ส่วน OKX,
Lighter, Binance, Hyperliquid และ AX Exchange ตรวจสอบแบบเชื่อมต่อตรงได้ ยืนยันว่าเส้นทางนั้นได้รับข้อมูลจาก venue
ก่อนการรันที่ยาว: socket อาจเชื่อมต่อได้ขณะที่ venue ยังเงียบ branch ที่ venue ไม่เคยสร้างขึ้นใน live ควรอยู่ใน harness
แบบ deterministic ที่ใช้ข้อมูล wire ที่บันทึกไว้ ไม่ใช่ในการรันแบบ live

### Oracle [#oracles]

oracle คือการสร้างความจริงของ venue ขึ้นใหม่อย่างอิสระ แล้วเปรียบเทียบกับ book ที่ถูกส่งออกมาผ่าน
`BookStreamChecker::verify`:

- สร้างจาก connection แยกต่างหากหรือ REST snapshot ห้ามสร้างจาก state ของ adapter เอง
- เปรียบเทียบที่ sequence ของ venue ที่ตรงกัน ข้าม sample ที่จัดให้ตรงกันไม่ได้ และไม่นับเป็นการผ่าน
- นับว่าช่วง snapshot หนึ่งได้รับการตรวจสอบแล้วเมื่อการเปรียบเทียบหลัง snapshot นั้นสำเร็จ harness ที่ใช้
  `Coverage::Episodes` เช่น OKX จะตรวจแต่ละ batch เมื่อมาถึง และทำให้ session ล้มเหลวหากมีช่วงใดไม่ได้รับการตรวจ
  ส่วน harness ที่ใช้ `Coverage::Samples` เช่น Binance จะจับคู่ sample ของ oracle ตาม update ID ภายหลัง จึงรายงาน
  จำนวนการตรวจของ oracle และ sample ที่จับคู่ไม่ได้ ไม่ใช่ความครอบคลุมรายช่วง

### Stress harness [#stress-harnesses]

stress harness เป็นเครื่องมือพัฒนาสำหรับการเปลี่ยนแปลงโค้ด sync และการกู้คืนของ book ไม่ได้เป็นส่วนหนึ่งของ crate
ที่ publish และไม่ได้รันใน CI `BookStreamChecker` ที่มันใช้มาพร้อมกับ `nautilus-live` ภายใต้ feature `test-support`
test อื่นจึงใช้ข้อตกลงเดียวกันได้

adapter ที่ใช้กลไก book ร่วมจะเก็บ live harness ไว้ที่ `crates/adapters/<venue>/tests/stress/book_stress.rs`
และลงทะเบียนเป็น test target ชื่อ `<venue>-book-stress`:

```toml
[[test]]
name = "okx-book-stress"
path = "tests/stress/book_stress.rs"
harness = false
test = false
required-features = ["examples"]
```

`harness = false` ให้ target ควบคุม runtime และอาร์กิวเมนต์ของตัวเอง และ `test = false` ทำให้มันไม่ถูกรันใน
`cargo test` และ nextest ตามค่าเริ่มต้น ให้เพิ่ม `nautilus-live` พร้อม feature `test-support` ใน dev-dependencies
ของ crate

กลไกร่วมเป็นซอร์สของ test อยู่ที่ `crates/live/tests/book/stress/` ซึ่งแต่ละ harness compile รวมเข้ามาด้วย path include:

```rust
#[path = "../../../../live/tests/book/stress/mod.rs"]
mod stress;
```

module ร่วมเป็นผู้รัน harness ส่วน venue จัดเตรียมเฉพาะชิ้นส่วนของตัวเองด้วยการ implement `StressVenue`:

- `WireCodec` ที่จัดประเภทแต่ละ frame ของ venue ว่าเป็น book snapshot, book update หรือการตอบรับ unsubscribe
  โดยบันทึก frame ลงใน oracle ก่อนที่ fault ใด ๆ จะมีผล และยังเขียน frame ใหม่เพื่อฝัง sequence gap หรือความไม่ตรงกัน
  แบบ in-band ได้ รวมถึงตอบ subscribe ของ adapter ด้วยการปฏิเสธจาก venue ได้ด้วย
- route ของ proxy, configuration ของ data client และ route ของ proxy เพิ่มเติม เช่น proxy สำหรับ REST snapshot
- การเปรียบเทียบกับ oracle สำหรับแต่ละ batch ที่ถูกส่งออกมา, เงื่อนไขของ book ที่สมบูรณ์ และการตรวจตัวเองตอนเริ่มต้น
- scenario ต่าง ๆ ที่เขียนโดยใช้ `Session`

`FaultProxy` ส่งต่อ traffic WebSocket ของ adapter ไปยัง venue สำหรับ route ที่ URL ต้นทางไม่ใช่ WebSocket URL
มันจะส่งต่อเป็นบรรทัดที่คั่นด้วย CRLF ผ่าน raw TCP route แบบบรรทัดให้บริการเฉพาะที่อยู่ของ proxy และ
`WireCodec::connect` ของ venue จะเปิด connection ต้นทางเอง เช่น ผ่าน TLS

proxy ใช้กฎ `Fault` ราย book (drop snapshot หรือ update, ทำให้เสียหาย, hold, ทำให้เงียบ, ตัดเมื่อ unsubscribe,
ปฏิเสธ subscribe) และการตัดหรือ freeze ทั้ง connection `Session` ส่งแต่ละ batch ที่ถูกส่งออกมาผ่าน `BookStreamChecker`
และ oracle รอให้ book กลับมาสมบูรณ์ และเมื่อ shutdown จะตรวจว่า socket และ handle ของ reconnect ทั้งหมดถูกปล่อยแล้ว

harness ทุกตัวรับ flag ชุดเดียวกัน และ venue อาจเพิ่ม flag ของตัวเอง `--help` จะแสดงรายการ flag:

| Flag              | ความหมาย                                                         | Default       |
| ----------------- | -------------------------------------------------------------- | ------------- |
| `--scenario NAME` | scenario ที่จะรัน                                                | `churn`       |
| `--timeout SECS`  | `book_snapshot_timeout_secs` ค่า `0` ปิด deadline ของ snapshot     | `10`          |
| `--rounds N`      | จำนวนรอบของ stress                                              | ค่าเริ่มต้นของ venue |

รัน harness อย่างชัดเจนโดยตัด environment variable ของ adapter ออก:

```bash
CARGO_BUILD_JOBS=16 bash scripts/strip-adapter-env.bash \
  cargo test -p nautilus-okx --features examples --test okx-book-stress -- --timeout 10 --rounds 18
```

Betfair stream market data ให้เฉพาะ account ที่ login แล้ว harness ของมันจึงรันโดยตั้ง credential ของ Betfair ไว้ ดู
[Live recovery validation](../integrations/betfair.md#live-recovery-validation) market data ของ AX Exchange
ก็ต้องการการยืนยันตัวตนเช่นกัน harness ของมันจึงอ่าน sandbox credential จาก environment และรันโดยไม่ใช้ wrapper

harness จะเขียนหนึ่งบรรทัดต่อ event ไปยัง stderr แต่ละบรรทัดขึ้นต้นด้วยคำที่กำหนดตายตัว:

| บรรทัด       | ความหมาย                                                                 |
| ---------- | ----------------------------------------------------------------------- |
| `START`    | venue และอาร์กิวเมนต์                                                     |
| `CHECK`    | การตรวจตัวเองตอนเริ่มต้นหรือ probe ของ scenario ผ่าน                          |
| `ROUND`    | stress รอบหนึ่งเสร็จสิ้น พร้อมตัวนับ                                          |
| `SHUTDOWN` | session หยุดอย่างเรียบร้อย พร้อมตัวนับและความครอบคลุมของ oracle                |
| `PASS`     | การรันเสร็จสิ้น เป็นบรรทัดสุดท้ายของการรันที่ผ่านเสมอ                              |
| `FAIL`     | การตรวจล้มเหลวหรือ thread ใด panic โปรเซสจะจบด้วย status 1                    |

ความล้มเหลวจาก deadline จะรายงาน frame ของ venue ที่แต่ละ route ของ proxy ได้รับ เพื่อแยก route ที่เงียบออกจาก
ความล้มเหลวของ adapter

**การตรวจตัวเอง**: target ที่ตั้ง `harness = false` ไม่สามารถรันฟังก์ชัน `#[test]` ได้ venue จึงพิสูจน์การ parse wire
และ oracle ของตนใน `StressVenue::self_check` ก่อนจะมี traffic ใด ๆ จาก venue ส่วน proxy ร่วม, การ parse อาร์กิวเมนต์
และ wire book มี unit test อยู่ใน test target `book` ของ `nautilus-live` รันได้ด้วย
`cargo nextest run -p nautilus-live --features test-support --test book`

**เอกสารของ harness**: เพิ่มหัวข้อ `Live recovery validation` ในคู่มือ integration ของ adapter โดยระบุการตรวจ,
fault ที่ฉีด, คำสั่งรัน, scenario, flag และ endpoint ที่ต้องใช้ OKX, Binance, Lighter, Polymarket, Hyperliquid, Bybit,
Betfair และ AX Exchange มี harness ให้แล้ว

### การตรวจสอบแบบ in-band [#in-band-verification]

เมื่อ venue เผยแพร่ checksum หรือ hash ของ book ให้ตรวจ state ที่สมบูรณ์ซึ่ง checksum นั้นครอบคลุม และถือว่าความ
ไม่ตรงกันเป็น gap วิธีนี้จับความเสียหายที่การตรวจ sequence พลาดได้โดยไม่ต้องใช้ oracle ภายนอก

- **Kraken**: ตรวจ CRC32 checksum บนแต่ละ message ของ Spot L2 `book` และ L3 เมื่อเปิด `validate_l2_checksum`
  และ `validate_l3_checksum` ซึ่งเปิดเป็นค่าเริ่มต้นทั้งคู่
- **Polymarket**: ปฏิเสธ snapshot ที่ hash ไม่ตรงเมื่อมี input ของ hash ครบ snapshot ที่ input ไม่ครบและ update แบบ
  incremental ยังไม่ได้รับการตรวจและไม่ trigger การกู้คืนจาก hash ความลึกที่ขาดหายอาจไม่ถูกตรวจพบจนกว่าจะมี
  snapshot ใหม่ ดู [book snapshot validation](../integrations/polymarket.md#book-snapshot-validation)
  และ [price change bursts](../integrations/polymarket.md#price-change-bursts)
- **OKX**: frame ของ `books` มี `checksum` เป็นศูนย์ การตรวจสอบจึงอาศัย oracle แทน

---

แต่ละกลุ่มด้านล่างเริ่มด้วยตารางสรุป ตามด้วย test card แบบละเอียด
Test ID ใช้หมายเลขแบบเว้นช่วงเพื่อให้แทรกเพิ่มได้โดยไม่ต้องเรียงหมายเลขใหม่

---

## กลุ่ม 1: Instrument [#group-1-instruments]

ตรวจสอบการโหลดและการ subscribe instrument ก่อนทดสอบ stream ของ market data

| TC     | ชื่อ                       | คำอธิบาย                             | ข้ามเมื่อ              |
| ------ | ------------------------ | --------------------------------- | ------------------ |
| TC-D01 | Request instruments      | โหลด instrument ทั้งหมดของ venue      | ไม่ข้าม              |
| TC-D02 | Subscribe instrument     | subscribe การอัปเดตของ instrument    | ไม่มี instrument sub |
| TC-D03 | Load specific instrument | โหลด instrument ตัวเดียวตาม ID         | ไม่ข้าม              |

### TC-D01: Request instruments [#tc-d01-request-instruments]

| Field              | Value                                                                                         |
| ------------------ | --------------------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว                                                                            |
| **Action**         | DataTester request instrument ทั้งหมดของ venue ตอน start                                         |
| **Event sequence** | callback `on_instruments` ได้รับรายการ instrument                                                |
| **Pass criteria**  | ได้รับ instrument อย่างน้อยหนึ่งตัว แต่ละตัวมี symbol, price precision และ size increment ที่ถูกต้อง        |
| **Skip when**      | ไม่ข้าม                                                                                          |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .request_instruments(true)
    .build()?
```

### TC-D02: Subscribe instrument [#tc-d02-subscribe-instrument]

| Field              | Value                                                           |
| ------------------ | --------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                         |
| **Action**         | DataTester subscribe การอัปเดตของ instrument                       |
| **Event sequence** | callback `on_instrument` ได้รับ instrument                         |
| **Pass criteria**  | ได้รับ instrument ที่มี `instrument_id` ถูกต้องและฟิลด์ถูกต้อง            |
| **Skip when**      | adapter ไม่รองรับการ subscribe instrument                          |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_instrument(true)
    .build()?
```

### TC-D03: Load specific instrument [#tc-d03-load-specific-instrument]

| Field              | Value                                                                                  |
| ------------------ | -------------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว                                                                     |
| **Action**         | โหลด instrument ตัวหนึ่งตาม `InstrumentId` ผ่าน instrument provider                         |
| **Event sequence** | instrument มีอยู่ใน cache หลังโหลด                                                         |
| **Pass criteria**  | โหลด instrument ได้พร้อม ID, price precision, size increment และกฎการซื้อขายที่ถูกต้อง           |
| **Skip when**      | ไม่ข้าม                                                                                   |

**ข้อควรพิจารณา**:

- case นี้ทดสอบ method `load` / `load_async` ของ instrument provider โดยตรง
- ตรวจว่า instrument ถูกเก็บใน cache และเรียกดูจาก cache ด้วย `instrument_id` ได้

---

## กลุ่ม 2: Order book [#group-2-order-book]

ทดสอบโหมดการ subscribe order book และการ request snapshot

| TC     | ชื่อ                         | คำอธิบาย                               | ข้ามเมื่อ           |
| ------ | -------------------------- | ----------------------------------- | ----------------- |
| TC-D10 | Subscribe book deltas      | stream การอัปเดต `OrderBookDeltas`     | ไม่รองรับ book      |
| TC-D11 | Subscribe book at interval | snapshot ของ `OrderBook` เป็นระยะ       | ไม่รองรับ book      |
| TC-D12 | Subscribe book depth       | snapshot ของ `OrderBookDepth`          | ไม่รองรับ book depth |
| TC-D13 | Request book snapshot      | request book snapshot ครั้งเดียว          | ไม่รองรับ book snapshot |
| TC-D14 | Managed book from deltas   | สร้าง book ใน local จาก stream ของ delta | ไม่รองรับ book      |

scenario เหล่านี้ใช้ `BookType::L2_MBP` ทั้งนี้ builder ฝั่ง Rust สามารถ override `book_type` ได้เมื่อ adapter
ต้องการรูปแบบ book ที่ต่างออกไป

### TC-D10: Subscribe book deltas [#tc-d10-subscribe-book-deltas]

| Field              | Value                                                                                  |
| ------------------ | -------------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                                                |
| **Action**         | DataTester subscribe order book delta                                                   |
| **Event sequence** | ได้รับ event `OrderBookDeltas` ใน `on_book_deltas`                                         |
| **Pass criteria**  | ได้รับ delta ที่มี instrument ID ถูกต้อง และมี delta อย่างน้อยหนึ่งตัวที่มีการอัปเดต bid/ask          |
| **Skip when**      | adapter ไม่รองรับข้อมูล order book                                                          |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_book_deltas(true)
    .book_type(BookType::L2_MBP)
    .build()?
```

### TC-D11: Subscribe book at interval [#tc-d11-subscribe-book-at-interval]

| Field              | Value                                                                                                 |
| ------------------ | ----------------------------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                                                               |
| **Action**         | DataTester subscribe snapshot ของ order book เป็นระยะ                                                     |
| **Event sequence** | ได้รับ event `OrderBook` ใน `on_book` ตามช่วงเวลาที่ตั้งไว้                                                     |
| **Pass criteria**  | ได้รับ book snapshot ที่มี level ของ bid/ask และการอัปเดตมาถึงตามช่วงเวลาที่ตั้งไว้โดยประมาณ                              |
| **Skip when**      | adapter ไม่รองรับข้อมูล order book                                                                         |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_book_at_interval(true)
    .book_type(BookType::L2_MBP)
    .book_depth(10)
    .book_interval_ms(1000)
    .build()?
```

### TC-D12: Subscribe book depth [#tc-d12-subscribe-book-depth]

| Field              | Value                                                                            |
| ------------------ | -------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                                          |
| **Action**         | DataTester subscribe snapshot ของ `OrderBookDepth`                                   |
| **Event sequence** | ได้รับ event `OrderBookDepth` ใน `on_book_depth`                                     |
| **Pass criteria**  | depth snapshot เคารพขีดจำกัดจำนวน level ที่ร้องขอ และราคาเรียงลำดับถูกต้อง                      |
| **Skip when**      | adapter ไม่รองรับการ subscribe book depth                                             |

เลือก depth ที่ venue รองรับ ดูขีดจำกัดได้ในคู่มือของ adapter `book_depth` มีผลกับการ subscribe book ทุกแบบที่เปิดไว้
และกับการ request book snapshot หากไม่ระบุจะใช้ค่าเริ่มต้นของ adapter

- เมื่อรัน depth ควบคู่กับ delta หรือ interval book DataTester จะ subscribe depth โดยตั้ง `managed` เป็น false
  เพื่อไม่ให้เขียนทับ book ที่จัดการจาก delta
- เมื่อเปิดเฉพาะ depth ค่า managed จะเป็นไปตาม `manage_book`

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_book_depth(true)
    .book_type(BookType::L2_MBP)
    .build()?
```

### TC-D13: Request book snapshot [#tc-d13-request-book-snapshot]

| Field              | Value                                                         |
| ------------------ | ------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                       |
| **Action**         | DataTester request snapshot ของ order book ครั้งเดียว              |
| **Event sequence** | ได้รับ book snapshot ผ่าน callback ของข้อมูลย้อนหลัง                  |
| **Pass criteria**  | snapshot มี level ของ bid/ask ที่มีราคาและขนาดถูกต้อง                 |
| **Skip when**      | adapter ไม่รองรับการ request book snapshot                        |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .request_book_snapshot(true)
    .book_depth(10)
    .build()?
```

### TC-D14: Managed book from deltas [#tc-d14-managed-book-from-deltas]

| Field              | Value                                                                                                                     |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว และมี book delta stream เข้ามา                                                         |
| **Action**         | DataTester subscribe delta โดยตั้ง `manage_book` เป็น true และสร้าง order book ใน local จาก stream ของ delta                      |
| **Event sequence** | `OrderBookDeltas` ถูกนำไปใช้กับ `OrderBook` ใน local และ log book ตาม depth ที่ตั้งไว้                                              |
| **Pass criteria**  | book ใน local สร้างได้ถูกต้องจาก delta level ของ bid เรียงลง level ของ ask เรียงขึ้น และ book ไม่ว่างหลัง snapshot แรก                    |
| **Skip when**      | adapter ไม่รองรับข้อมูล order book                                                                                             |

**ข้อควรพิจารณา**:

- managed book นำแต่ละ delta ไปใช้กับ instance ของ `OrderBook` ที่ actor ดูแลอยู่
- ใช้ `book_levels_to_print` เพื่อควบคุมความละเอียดของ log

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_book_deltas(true)
    .manage_book(true)
    .book_type(BookType::L2_MBP)
    .build()?
```

`DataTesterConfig` มี `request_book_deltas` ให้ใช้ แต่ `DataTester` ไม่ได้ส่ง request ข้อมูลย้อนหลังนั้นจริง ให้ทดสอบ
การรองรับ book delta ย้อนหลังของ adapter ผ่าน actor แบบกำหนดเองไปก่อน จนกว่า tester จะ implement เส้นทาง request นี้

---

## กลุ่ม 3: Quote [#group-3-quotes]

ทดสอบการ subscribe quote tick และการ request ข้อมูลย้อนหลัง

| TC     | ชื่อ                        | คำอธิบาย                                       | ข้ามเมื่อ               |
| ------ | ------------------------- | ------------------------------------------- | --------------------- |
| TC-D20 | Subscribe quotes          | ตรวจว่า event `QuoteTick` ไหลเข้ามาหลัง start    | ไม่ข้าม                 |
| TC-D21 | Request historical quotes | request quote tick ย้อนหลัง                    | ไม่มี quote ย้อนหลัง       |

### TC-D20: Subscribe quotes [#tc-d20-subscribe-quotes]

| Field              | Value                                                                             |
| ------------------ | --------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                                           |
| **Action**         | DataTester subscribe quote ตอน start                                                |
| **Event sequence** | ได้รับ event `QuoteTick` ใน `on_quote`                                                 |
| **Pass criteria**  | ได้รับ `QuoteTick` อย่างน้อยหนึ่งตัวที่มีราคาและขนาดของ bid/ask ถูกต้อง และ bid < ask            |
| **Skip when**      | ไม่ข้าม                                                                              |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_quotes(true)
    .build()?
```

### TC-D21: Request historical quotes [#tc-d21-request-historical-quotes]

| Field              | Value                                                             |
| ------------------ | ----------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                           |
| **Action**         | DataTester request quote tick ย้อนหลัง                               |
| **Event sequence** | ได้รับ batch ของ quote ย้อนหลังผ่าน `on_historical_quotes`               |
| **Pass criteria**  | ได้รับ quote ที่มี timestamp, ราคา bid/ask และขนาดถูกต้อง                   |
| **Skip when**      | adapter ไม่รองรับการ request quote ย้อนหลัง                            |

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง config แบบ Python (`request_quotes`) จึงตัดออก

---

## กลุ่ม 4: Trade [#group-4-trades]

ทดสอบการ subscribe trade tick และการ request ข้อมูลย้อนหลัง

| TC     | ชื่อ                        | คำอธิบาย                                       | ข้ามเมื่อ               |
| ------ | ------------------------- | ------------------------------------------- | --------------------- |
| TC-D30 | Subscribe trades          | ตรวจว่า event `TradeTick` ไหลเข้ามาหลัง start    | ไม่ข้าม                 |
| TC-D31 | Request historical trades | request trade tick ย้อนหลัง                    | ไม่มี trade ย้อนหลัง       |

### TC-D30: Subscribe trades [#tc-d30-subscribe-trades]

| Field              | Value                                                                         |
| ------------------ | ----------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                                       |
| **Action**         | DataTester subscribe trade ตอน start                                            |
| **Event sequence** | ได้รับ event `TradeTick` ใน `on_trade`                                             |
| **Pass criteria**  | ได้รับ `TradeTick` อย่างน้อยหนึ่งตัวที่มีราคา, ขนาด และ aggressor side ถูกต้อง             |
| **Skip when**      | ไม่ข้าม                                                                          |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_trades(true)
    .build()?
```

### TC-D31: Request historical trades [#tc-d31-request-historical-trades]

| Field              | Value                                                                |
| ------------------ | -------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                              |
| **Action**         | DataTester request trade tick ย้อนหลัง                                  |
| **Event sequence** | ได้รับ batch ของ trade ย้อนหลังผ่าน `on_historical_trades`                  |
| **Pass criteria**  | ได้รับ trade ที่มี timestamp, ราคา, ขนาด และ trade ID ถูกต้อง                  |
| **Skip when**      | adapter ไม่รองรับการ request trade ย้อนหลัง                               |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .request_trades(true)
    .build()?
```

---

## กลุ่ม 5: Bar [#group-5-bars]

ทดสอบการ subscribe bar และการ request ข้อมูลย้อนหลัง

| TC     | ชื่อ                      | คำอธิบาย                                 | ข้ามเมื่อ             |
| ------ | ----------------------- | ------------------------------------- | ------------------- |
| TC-D40 | Subscribe bars          | ตรวจว่า event `Bar` ไหลเข้ามาหลัง start    | ไม่รองรับ bar          |
| TC-D41 | Request historical bars | request bar แบบ OHLCV ย้อนหลัง           | ไม่มี bar ย้อนหลัง       |

### TC-D40: Subscribe bars [#tc-d40-subscribe-bars]

| Field              | Value                                                                                          |
| ------------------ | ---------------------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว และตั้งค่า bar type แล้ว                                   |
| **Action**         | DataTester subscribe bar สำหรับ `BarType` ที่ตั้งค่าไว้                                              |
| **Event sequence** | ได้รับ event `Bar` ใน `on_bar`                                                                     |
| **Pass criteria**  | ได้รับ `Bar` อย่างน้อยหนึ่งตัวที่มีค่า OHLCV ถูกต้อง โดย high >= low, high >= open, high >= close        |
| **Skip when**      | adapter ไม่รองรับการ subscribe bar                                                                 |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .bar_types(vec![bar_type])
    .subscribe_bars(true)
    .build()?
```

### TC-D41: Request historical bars [#tc-d41-request-historical-bars]

| Field              | Value                                                           |
| ------------------ | --------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว และตั้งค่า bar type แล้ว    |
| **Action**         | DataTester request bar ย้อนหลังสำหรับ `BarType` ที่ตั้งค่าไว้             |
| **Event sequence** | ได้รับ bar ย้อนหลังผ่าน callback                                     |
| **Pass criteria**  | ได้รับ bar ที่มีค่า OHLCV ถูกต้องและ timestamp เรียงจากน้อยไปมาก            |
| **Skip when**      | adapter ไม่รองรับการ request bar ย้อนหลัง                            |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .bar_types(vec![bar_type])
    .request_bars(true)
    .build()?
```

---

## กลุ่ม 6: ข้อมูลอนุพันธ์ [#group-6-derivatives-data]

ทดสอบ data stream เฉพาะของอนุพันธ์: mark price, index price และ funding rate

| TC     | ชื่อ                               | คำอธิบาย                        | ข้ามเมื่อ           |
| ------ | -------------------------------- | ----------------------------- | ----------------- |
| TC-D50 | Subscribe mark prices            | event `MarkPriceUpdate`       | ไม่ใช่อนุพันธ์        |
| TC-D51 | Subscribe index prices           | event `IndexPriceUpdate`      | ไม่ใช่อนุพันธ์        |
| TC-D52 | Subscribe funding rates          | event `FundingRateUpdate`     | ไม่ใช่ perpetual     |
| TC-D53 | Request historical funding rates | ข้อมูล funding rate ย้อนหลัง      | ไม่ใช่ perpetual     |

### TC-D50: Subscribe mark prices [#tc-d50-subscribe-mark-prices]

| Field              | Value                                                                            |
| ------------------ | -------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument ประเภทอนุพันธ์แล้ว                                |
| **Action**         | DataTester subscribe การอัปเดต mark price                                           |
| **Event sequence** | ได้รับ event `MarkPriceUpdate` ใน `on_mark_price`                                     |
| **Pass criteria**  | ได้รับ `MarkPriceUpdate` อย่างน้อยหนึ่งตัวที่มี instrument ID และ mark price ถูกต้อง           |
| **Skip when**      | instrument ไม่ใช่อนุพันธ์ หรือ adapter ไม่ให้ข้อมูล mark price                                |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_mark_prices(true)
    .build()?
```

### TC-D51: Subscribe index prices [#tc-d51-subscribe-index-prices]

| Field              | Value                                                                              |
| ------------------ | ---------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument ประเภทอนุพันธ์แล้ว                                  |
| **Action**         | DataTester subscribe การอัปเดต index price                                            |
| **Event sequence** | ได้รับ event `IndexPriceUpdate` ใน `on_index_price`                                     |
| **Pass criteria**  | ได้รับ `IndexPriceUpdate` อย่างน้อยหนึ่งตัวที่มี instrument ID และ index price ถูกต้อง           |
| **Skip when**      | instrument ไม่ใช่อนุพันธ์ หรือ adapter ไม่ให้ข้อมูล index price                                 |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_index_prices(true)
    .build()?
```

### TC-D52: Subscribe funding rates [#tc-d52-subscribe-funding-rates]

| Field              | Value                                                                        |
| ------------------ | ---------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument ประเภท perpetual แล้ว                       |
| **Action**         | DataTester subscribe การอัปเดต funding rate                                     |
| **Event sequence** | ได้รับ event `FundingRateUpdate` ใน `on_funding_rate`                             |
| **Pass criteria**  | ได้รับ `FundingRateUpdate` อย่างน้อยหนึ่งตัวที่มี instrument ID และ rate ถูกต้อง             |
| **Skip when**      | instrument ไม่ใช่ perpetual หรือ adapter ไม่ให้ข้อมูล funding rate                      |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_funding_rates(true)
    .build()?
```

### TC-D53: Request historical funding rates [#tc-d53-request-historical-funding-rates]

| Field              | Value                                                                                        |
| ------------------ | -------------------------------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument ประเภท perpetual แล้ว                                       |
| **Action**         | DataTester request funding rate ย้อนหลัง (ค่าเริ่มต้นย้อนหลัง 7 วัน)                                  |
| **Event sequence** | ได้รับ funding rate ย้อนหลังผ่าน callback                                                         |
| **Pass criteria**  | ได้รับ funding rate ที่มี timestamp และค่า rate ถูกต้อง                                              |
| **Skip when**      | instrument ไม่ใช่ perpetual หรือ adapter ไม่รองรับการ request funding rate ย้อนหลัง                     |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .request_funding_rates(true)
    .build()?
```

---

## กลุ่ม 7: Instrument status [#group-7-instrument-status]

ทดสอบการ subscribe event ของ instrument status และ instrument close

| TC     | ชื่อ                          | คำอธิบาย                   | ข้ามเมื่อ            |
| ------ | --------------------------- | ------------------------ | ------------------ |
| TC-D60 | Subscribe instrument status | event `InstrumentStatus` | ไม่รองรับ status     |
| TC-D61 | Subscribe instrument close  | event `InstrumentClose`  | ไม่รองรับ close      |

### TC-D60: Subscribe instrument status [#tc-d60-subscribe-instrument-status]

| Field              | Value                                                                    |
| ------------------ | ------------------------------------------------------------------------ |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                                  |
| **Action**         | DataTester subscribe การอัปเดต instrument status                            |
| **Event sequence** | ได้รับ event `InstrumentStatus` ใน `on_instrument_status`                    |
| **Pass criteria**  | ได้รับ status event ที่มี `MarketStatusAction` ถูกต้อง (เช่น `Trading`)            |
| **Skip when**      | adapter ไม่รองรับการ subscribe instrument status                             |

**ข้อควรพิจารณา**:

- status event อาจเกิดขึ้นเฉพาะเมื่อ state เปลี่ยน (เช่น หยุดการซื้อขาย -> กลับมาซื้อขาย)
- ในช่วงเวลาซื้อขายปกติ อาจได้รับ status `Trading` ตอน subscribe

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_instrument_status(true)
    .build()?
```

### TC-D61: Subscribe instrument close [#tc-d61-subscribe-instrument-close]

| Field              | Value                                                       |
| ------------------ | ----------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument แล้ว                     |
| **Action**         | DataTester subscribe event ของ instrument close                |
| **Event sequence** | ได้รับ event `InstrumentClose` ใน `on_instrument_close`         |
| **Pass criteria**  | ได้รับ close event ที่มีราคาปิดและประเภทการปิดถูกต้อง                   |
| **Skip when**      | adapter ไม่รองรับการ subscribe instrument close                  |

**ข้อควรพิจารณา**:

- close event มักเกิดตอนจบ session สำหรับตลาดแบบดั้งเดิม
- อาจไม่เกิดขึ้นสำหรับ venue คริปโตที่เปิด 24/7 เว้นแต่ adapter จะสังเคราะห์ราคาปิดรายวันขึ้นมา

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_instrument_close(true)
    .build()?
```

---

## กลุ่ม 8: Option greeks [#group-8-option-greeks]

ทดสอบการ subscribe option greeks และ option chain

| TC     | ชื่อ                      | คำอธิบาย                                       | ข้ามเมื่อ            |
| ------ | ----------------------- | -------------------------------------------- | ------------------ |
| TC-D62 | Subscribe option greeks | ข้อมูล `OptionGreeks` ของ instrument ตัวเดียว     | ไม่รองรับ greeks     |
| TC-D63 | Subscribe option chain  | snapshot ของ `OptionChainSlice` สำหรับ series    | ไม่รองรับ chain      |

### TC-D62: Subscribe option greeks [#tc-d62-subscribe-option-greeks]

| Field              | Value                                                        |
| ------------------ | ------------------------------------------------------------ |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument ประเภท option แล้ว          |
| **Action**         | DataTester subscribe การอัปเดต option greeks                    |
| **Event sequence** | ได้รับ event `OptionGreeks` ใน `on_option_greeks`                |
| **Pass criteria**  | ได้รับ greeks ที่มีค่า delta, gamma, vega, theta ถูกต้อง            |
| **Skip when**      | adapter ไม่รองรับการ subscribe option greeks                     |

**ข้อควรพิจารณา**:

- greeks มีเฉพาะสำหรับ instrument ประเภท option
- ค่าขึ้นกับ pricing model ของ venue และอาจอัปเดตทุกครั้งที่ quote เปลี่ยน
- บาง venue (Bybit, Deribit) subscribe ราย instrument ส่วน OKX subscribe ราย instrument family แล้วกรองเหลือเฉพาะ
  instrument ที่ร้องขอ
- `rho` อาจเป็นศูนย์เมื่อ venue ไม่ได้ให้มา (Bybit, OKX)
- `underlying_price` และ `open_interest` อาจเป็น `None` ขึ้นกับ channel ของ venue

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_option_greeks(true)
    .build()?
```

### TC-D63: Subscribe option chain [#tc-d63-subscribe-option-chain]

| Field              | Value                                                               |
| ------------------ | ------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และโหลด instrument ประเภท option ของ series แล้ว        |
| **Action**         | DataTester subscribe snapshot ของ option chain สำหรับ series              |
| **Event sequence** | ได้รับ snapshot ของ `OptionChainSlice` ใน `on_option_chain`              |
| **Pass criteria**  | chain snapshot มี greeks ของ instrument ที่ตรงกับ series                   |
| **Skip when**      | adapter ไม่รองรับการ subscribe option chain                             |

**ข้อควรพิจารณา**:

- การ subscribe option chain ถูกจัดการโดย DataEngine ซึ่งสร้างการ subscribe quote และ greeks ราย instrument ภายใน
- ช่วง strike แบบ dynamic ต้องมีราคา ATM ก่อนเริ่มการ subscribe instrument โดย DataEngine จะ request ราคาอ้างอิง
  เริ่มต้น หรือไม่เช่นนั้นจะรอ option greeks จาก live
- ยังตั้งค่าผ่าน `DataTesterConfig` ไม่ได้ ต้องตั้งค่า actor เองด้วย `subscribe_option_chain` และ `OptionSeriesId`

---

## กลุ่ม 9: Lifecycle [#group-9-lifecycle]

ทดสอบพฤติกรรมใน lifecycle ของ actor: การจัดการ unsubscribe, การเก็บกวาดเมื่อ retire และพารามิเตอร์แบบกำหนดเอง

| TC     | ชื่อ                      | คำอธิบาย                                           | ข้ามเมื่อ           |
| ------ | ----------------------- | ------------------------------------------------ | ----------------- |
| TC-D70 | Unsubscribe on stop     | unsubscribe จาก data feed เมื่อ actor หยุด           | ไม่รองรับ unsub     |
| TC-D71 | Custom subscribe params | พารามิเตอร์การ subscribe เฉพาะของ adapter            | N/A               |
| TC-D72 | Custom request params   | พารามิเตอร์การ request เฉพาะของ adapter              | N/A               |
| TC-D73 | Retirement cleanup      | ปล่อย data subscription ที่ actor ถือไว้               | N/A               |
| TC-D74 | DeFi shared pool demand | คง feed ของ pool ที่ใช้ร่วมกันไว้จนถึงเจ้าของรายสุดท้าย      | ไม่รองรับ DeFi      |
| TC-D75 | DeFi bootstrap cancel   | ทิ้ง snapshot ของ pool bootstrap ที่ถูกยกเลิก           | ไม่รองรับ DeFi      |

### TC-D70: Unsubscribe on stop [#tc-d70-unsubscribe-on-stop]

| Field              | Value                                                            |
| ------------------ | ---------------------------------------------------------------- |
| **Prerequisite**   | มี data subscription ที่ active อยู่ (quote, trade, book)               |
| **Action**         | หยุด actor โดยตั้ง `can_unsubscribe` เป็น true (ค่าเริ่มต้น)               |
| **Event sequence** | data subscription ถูกลบออก และไม่ได้รับ data event อีก                 |
| **Pass criteria**  | unsubscribe อย่างเรียบร้อย ไม่มี error ใน log และไม่มี data event หลังหยุด  |
| **Skip when**      | adapter ไม่รองรับ unsubscribe                                       |

**Rust config**:

```rust
DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_quotes(true)
    .subscribe_trades(true)
    .can_unsubscribe(true)
    .build()?
```

### TC-D71: Custom subscribe params [#tc-d71-custom-subscribe-params]

| Field              | Value                                                                  |
| ------------------ | ---------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และ adapter รับพารามิเตอร์การ subscribe เพิ่มเติมได้          |
| **Action**         | subscribe ด้วย `subscribe_params` เฉพาะของ adapter                         |
| **Event sequence** | การ subscribe ถูกสร้างขึ้นโดยใช้พารามิเตอร์ที่กำหนด                                |
| **Pass criteria**  | ข้อมูลไหลเข้ามาโดยพารามิเตอร์เฉพาะของ adapter มีผล                               |
| **Skip when**      | N/A (เฉพาะของ adapter)                                                   |

**Rust config**:

```rust
use nautilus_core::Params;
use serde_json::json;

let subscribe_params = Params::new([("key".to_string(), json!("value"))].into());

DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .subscribe_quotes(true)
    .subscribe_params(subscribe_params)
    .build()?
```

**ข้อควรพิจารณา**:

- `subscribe_params` เป็นค่าทึบ (opaque) สำหรับ DataTester และถูกส่งผ่านไปยัง adapter โดยตรง
- `subscribe_params` เป็นฟิลด์ที่มีเฉพาะใน Rust
- ดูพารามิเตอร์ที่รองรับได้ในคู่มือของ adapter

### TC-D72: Custom request params [#tc-d72-custom-request-params]

| Field              | Value                                                                |
| ------------------ | -------------------------------------------------------------------- |
| **Prerequisite**   | adapter เชื่อมต่อแล้ว และ adapter รับพารามิเตอร์การ request เพิ่มเติมได้          |
| **Action**         | request ข้อมูลด้วย `request_params` เฉพาะของ adapter                       |
| **Event sequence** | request ได้รับการตอบสนองโดยใช้พารามิเตอร์ที่กำหนด                               |
| **Pass criteria**  | ได้รับข้อมูลย้อนหลังโดยพารามิเตอร์เฉพาะของ adapter มีผล                           |
| **Skip when**      | N/A (เฉพาะของ adapter)                                                 |

**Rust config**:

```rust
use nautilus_core::Params;
use serde_json::json;

let request_params = Params::new([("key".to_string(), json!("value"))].into());

DataTesterConfig::builder()
    .client_id(client_id)
    .instrument_ids(vec![instrument_id])
    .request_quotes(true)
    .request_params(request_params)
    .build()?
```

**ข้อควรพิจารณา**:

- `request_params` เป็นค่าทึบ (opaque) สำหรับ DataTester และถูกส่งผ่านไปยัง adapter โดยตรง
- `request_params` เป็นฟิลด์ที่มีเฉพาะใน Rust
- ดูพารามิเตอร์ที่รองรับได้ในคู่มือของ adapter

### TC-D73: Retirement cleanup [#tc-d73-retirement-cleanup]

| Field              | Value                                                                                                                                 |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------- |
| **Prerequisite**   | actor มี subscription ที่อิงกับ venue และมี actor สองตัวใช้ bar ที่ aggregate ภายในร่วมกัน                                                       |
| **Action**         | retire actor ตัวแรก แล้ว retire actor ตัวที่สองผ่าน trader                                                                                 |
| **Event sequence** | `on_dispose` ทำงานเสร็จ, command สำหรับ unsubscribe ถูกส่งออกไป และ actor ถูกยกเลิกการลงทะเบียน                                                |
| **Pass criteria**  | การ retire ครั้งแรกยังคงข้อมูลที่ใช้ร่วมกันไว้ การ retire ครั้งสุดท้ายปล่อย route ที่ถือไว้ และไม่เหลือ handler ของ actor ที่ retire แล้ว                       |
| **Skip when**      | N/A                                                                                                                                   |

bar ที่ใช้ร่วมกันต้องยัง active หลัง actor ตัวแรก retire และหยุดหลัง actor ตัวสุดท้าย retire

**ข้อควรพิจารณา**:

- `DataTesterConfig` ไม่ครอบคลุมการ retire หลาย actor ให้สร้าง actor สองตัวเอง แล้วลบออกผ่าน `Trader::remove_actor`
- หาก `on_dispose` ล้มเหลว actor ต้องยังลงทะเบียนอยู่พร้อม subscription ครบถ้วน เพื่อให้การ retire ครั้งต่อไปปล่อย
  subscription ได้โดยไม่เรียก hook ที่ล้มเหลวซ้ำ
- `on_stop` หรือ `on_fault` ที่ล้มเหลวต้องไม่ขัดขวางการ retire: การ dispose และการยกเลิกการลงทะเบียนต้องยังทำงานจนเสร็จ
  จาก state ระหว่างทางที่สอดคล้องกัน

### TC-D74: DeFi shared pool demand [#tc-d74-defi-shared-pool-demand]

| Field              | Value                                                                                                                      |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| **Prerequisite**   | DeFi data client หนึ่งตัว และ actor สองตัว subscribe pool เดียวกันผ่าน subscription type เดียวกันหรือที่ซ้อนทับกัน                             |
| **Action**         | retire หรือ unsubscribe actor ตัวหนึ่ง แล้วอีกตัว โดยทดสอบทั้งสองลำดับ                                                                 |
| **Event sequence** | การปล่อยครั้งแรกไม่ส่ง unsubscribe ไปยัง client สำหรับ type ที่ใช้ร่วมกัน การปล่อยครั้งสุดท้ายจะ unsubscribe และหยุด pool updater                 |
| **Pass criteria**  | event ของ pool ยังไปถึง actor ที่เหลือและ profiler จนถึงการปล่อยครั้งสุดท้าย และไม่มีอะไรไหลเข้ามาหลังจากนั้น                                      |
| **Skip when**      | adapter ไม่ให้บริการการ subscribe DeFi pool                                                                                    |

ครอบคลุมการซ้อนทับเหล่านี้:

- actor สองตัวที่ใช้ subscription type เดียวกัน retire ในลำดับใดก็ได้
- `SubscribePool` คู่กับ type ที่แคบกว่าแต่ละแบบ (swap, การอัปเดต liquidity, การเก็บค่าธรรมเนียม, flash event)
  unsubscribe ทั้งสองลำดับ ตัวกรอง event ที่แคบกว่าต้องยัง active ตราบที่ยังมี subscription ใดเหลืออยู่
- pool เดียวกันบน data client สองตัว client แต่ละตัวเก็บ demand ของตัวเอง และ pool updater ยัง active จนกว่าทั้งสอง
  จะปล่อย

**ข้อควรพิจารณา**:

- `DataTesterConfig` ไม่ครอบคลุมการ subscribe DeFi pool ให้สร้าง actor เอง
- การ unsubscribe ซ้ำจาก actor ตัวหนึ่งต้องไม่ปล่อย demand ของ actor อีกตัว

### TC-D75: DeFi bootstrap cancel [#tc-d75-defi-bootstrap-cancel]

| Field              | Value                                                                                                 |
| ------------------ | ----------------------------------------------------------------------------------------------------- |
| **Prerequisite**   | DeFi data client หนึ่งตัว และ pool ยังไม่อยู่ใน cache ดังนั้นการ subscribe จะ request snapshot ของ pool          |
| **Action**         | subscribe, ปล่อยเจ้าของรายสุดท้ายก่อนที่นิยามของ pool จะมาถึง แล้ว subscribe อีกครั้ง                            |
| **Event sequence** | snapshot request สองครั้งถูกส่งออกไป และ response ของครั้งแรกมาถึงหลังการ subscribe ครั้งที่สอง                      |
| **Pass criteria**  | engine ทิ้ง response แรก และมีเพียง response ของ request ปัจจุบันเท่านั้นที่ติดตั้ง profiler                         |
| **Skip when**      | adapter ไม่ให้ snapshot ของ pool                                                                       |

**ข้อควรพิจารณา**:

- การ subscribe ครั้งที่สองจะ request snapshot ใหม่ก็ต่อเมื่อ pool ยังไม่อยู่ใน cache เท่านั้น เมื่อนิยามของ pool จาก
  request แรกมาถึงแล้ว การ subscribe ครั้งถัดไปจะสร้าง profiler จาก pool ใน cache แทน case นี้จึงต้องการ response
  ที่มาช้า
- การ reset engine หรือการ disconnect ก็ยกเลิก bootstrap ที่ค้างอยู่เช่นกัน response ที่มาถึงหลังจากนั้นต้องไม่ติดตั้ง profiler

---

## เอกสารอ้างอิง configuration ของ DataTester [#datatester-configuration-reference]

`DataTesterConfig` รับพารามิเตอร์ด้านล่าง ค่าเริ่มต้นคือค่าที่ resolve แล้วหลังการสร้าง
การ request quote, trade และ bar ย้อนหลังใช้ช่วงย้อนหลังหนึ่งชั่วโมง ส่วนการ request funding rate ใช้เจ็ดวัน
ช่วงย้อนหลังนี้ตั้งค่าผ่าน `DataTesterConfig` ไม่ได้

| Parameter                     | Default | กลุ่มที่เกี่ยวข้อง    |
| ----------------------------- | ------- | --------------- |
| `actor_id`                    | `None`  | ทั้งหมด           |
| `client_id`                   | `None`  | ทั้งหมด           |
| `instrument_ids`              | `[]`    | ทั้งหมด           |
| `bar_types`                   | `None`  | 5               |
| `subscribe_book_deltas`       | `false` | 2               |
| `subscribe_book_depth`        | `false` | 2               |
| `subscribe_book_at_interval`  | `false` | 2               |
| `subscribe_quotes`            | `false` | 3               |
| `subscribe_trades`            | `false` | 4               |
| `subscribe_mark_prices`       | `false` | 6               |
| `subscribe_index_prices`      | `false` | 6               |
| `subscribe_funding_rates`     | `false` | 6               |
| `subscribe_bars`              | `false` | 5               |
| `subscribe_instrument`        | `false` | 1               |
| `subscribe_instrument_status` | `false` | 7               |
| `subscribe_instrument_close`  | `false` | 7               |
| `subscribe_option_greeks`     | `false` | 8               |
| `can_unsubscribe`             | `true`  | 9               |
| `request_instruments`         | `false` | 1               |
| `request_book_snapshot`       | `false` | 2               |
| `request_book_deltas`         | `false` | ยังไม่ได้ implement |
| `request_quotes`              | `false` | 3               |
| `request_trades`              | `false` | 4               |
| `request_bars`                | `false` | 5               |
| `request_funding_rates`       | `false` | 6               |
| `book_depth`                  | `None`  | 2               |
| `book_interval_ms`            | `1000`  | 2               |
| `book_levels_to_print`        | `10`    | 2               |
| `manage_book`                 | `true`  | 2               |
| `log_data`                    | `true`  | ทั้งหมด           |
| `stats_interval_secs`         | `5`     | ทั้งหมด           |
| `log_events`                  | `true`  | ทั้งหมด           |
| `log_commands`                | `true`  | ทั้งหมด           |

builder ของ Rust ยังมีพารามิเตอร์เหล่านี้ด้วย:

| Parameter          | Type       | Default  | กลุ่มที่เกี่ยวข้อง |
| ------------------ | ---------- | -------- | -------------- |
| `book_type`        | `BookType` | `L2_MBP` | 2              |
| `subscribe_params` | `Params?`  | `None`   | 9              |
| `request_params`   | `Params?`  | `None`   | 9              |
