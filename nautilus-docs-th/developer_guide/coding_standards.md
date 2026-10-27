---
title: "มาตรฐานการเขียนโค้ด"
---

## รูปแบบโค้ด [#code-style]

ใช้ codebase เป็นแนวทางสำหรับข้อตกลงด้านการจัดรูปแบบ โดยเสริมด้วยกฎด้านล่าง

### กฎการจัดรูปแบบที่ใช้กับทุกไฟล์ [#universal-formatting-rules]

กฎต่อไปนี้ใช้กับไฟล์ source **ทุกไฟล์** (Rust, shell ฯลฯ):

- ใช้ **space เท่านั้น** ห้ามใช้ hard tab
- โดยทั่วไปแต่ละบรรทัดควรสั้นกว่า **100 ตัวอักษร** ตัดบรรทัดอย่างพิจารณาเมื่อจำเป็น
- ใช้การสะกดแบบ American English (`color`, `serialize`, `behavior`) ซึ่งบังคับด้วย `.typos.toml` คงการสะกดของ API
  ภายนอกไว้ด้วยการยกเว้นเป็นราย identifier แบบตรงตัว และไม่รวมข้อมูลที่คัดลอกมาตรงตัวและไฟล์ที่ generate ขึ้น รัน
  `prek run typos --all-files` เพื่อตรวจทั้ง repository

### Shell script [#shell-scripts]

Bash เป็น shell เริ่มต้นสำหรับ script ใน repository ใช้ POSIX `sh` เฉพาะเมื่อผู้เรียกไม่สามารถคาดหวังได้ว่ามี Bash
ติดตั้งอยู่ ดู [Shell](shell.md) สำหรับข้อกำหนดด้านการเลือก script, นามสกุลไฟล์, portability, โครงสร้าง, การทดสอบ,
การจัดรูปแบบ และ linting

### ข้อตกลงด้าน comment [#comment-conventions]

1. โดยทั่วไปให้เว้น **หนึ่งบรรทัดว่างด้านบน** ของทุก comment block หรือ docstring เพื่อแยกออกจากโค้ดให้เห็นชัด
2. ใช้ *sentence case* - ขึ้นต้นด้วยตัวพิมพ์ใหญ่ ที่เหลือเป็นตัวพิมพ์เล็ก ยกเว้นคำนามเฉพาะหรือตัวย่อ
3. ห้ามเว้นสอง space หลังจุด
4. **Comment บรรทัดเดียว** *ต้องไม่* ลงท้ายด้วยจุด ยกเว้นเมื่อลงท้ายด้วย URL หรือ inline Markdown link ในกรณีนั้น
   ให้คงเครื่องหมายวรรคตอนที่ link ต้องใช้ไว้
5. **Comment หลายบรรทัด** ควรคั่นประโยคด้วย comma (ไม่ใช่จุดทุกบรรทัด) บรรทัดสุดท้าย *ควร* ลงท้ายด้วยจุด
6. เขียน comment ให้กระชับ เน้นความชัดเจน และอธิบายเฉพาะสิ่งที่ไม่ชัดเจนในตัว - *less is more*
7. หลีกเลี่ยงการใช้ emoji ในข้อความ

### รูปประโยคของ doc comment [#doc-comment-mood]

เขียน Rust doc comment ใน **indicative mood** เช่น *"Returns a cached client."* ซึ่งตรงกับสไตล์ที่แพร่หลายใน ecosystem
ของ Rust และทำให้เอกสารที่ generate ออกมาอ่านเป็นธรรมชาติสำหรับผู้ใช้

### คำศัพท์และการใช้ถ้อยคำ [#terminology-and-phrasing]

1. **Error message**: หลีกเลี่ยงการใช้ ", got" ใน error message ให้ใช้คำที่สื่อความหมายมากกว่า เช่น ", was", ", received"
   หรือ ", found" ตามบริบท
   - ไม่ดี: `"Expected string, got {type(value)}"`
   - ดี: `"Expected string, was {type(value)}"`

2. **การสะกด**: ใช้ "hardcoded" (คำเดียว) แทน "hard-coded" หรือ "hard coded" ซึ่งเป็นการสะกดที่ทันสมัยและเป็นที่ยอมรับกว่า

3. **การตั้งชื่อตัวแปร error**: ใช้ตัวอักษรเดียว `e` สำหรับ error ที่จับได้:
   - Rust: ใช้ `Err(e)` ไม่ใช่ `Err(err)` หรือ `Err(error)` และใช้ `|e|` ไม่ใช่ `|err|` ใน closure

### ข้อตกลงด้านการตั้งชื่อ [#naming-conventions]

1. **Field ภายใน**: ใช้ตัวย่อได้สำหรับ field แบบ private/internal (เช่น `_price_prec`, `_size_prec`) เพื่อให้โค้ดใน hot path กระชับ

2. **API ที่ผู้ใช้เห็น**: ใช้ชื่อเต็มที่สื่อความหมายสำหรับ public property, พารามิเตอร์ของฟังก์ชัน, return type และชื่อ/label
   ของ metric (เช่น `price_precision`, `size_precision`) เพื่อป้องกันไม่ให้คำย่อหลุดไปอยู่ใน dashboard หรือ alert

3. **Error message และ log**: ใช้คำเต็มเพื่อความชัดเจน (เช่น "price precision" ไม่ใช่ "price prec") ผู้ใช้ไม่ควรเห็นคำย่อเลย

4. **คำศัพท์ด้าน execution**: ใช้ `Execution` ในชื่อ type แบบ PascalCase ที่เป็น public และเป็นของโปรเจกต์เอง เช่น
   `BinanceExecutionClientConfig` ส่วน type ที่เป็น implementation ภายในอาจคงชื่อ `Exec` ที่ใช้กันมาอยู่แล้วได้ และสงวน
   `Exec` ไว้สำหรับตระกูล `ExecAlgorithmId` และ `ExecTester`, ชื่อ `exec_*` ที่ใช้กันมาแล้ว และคำศัพท์ของ venue หรือ
   protocol เช่น `OkxExecType` ตั้งชื่อ wire model เฉพาะ protocol ตามแนวคิดของ venue เช่น `HyperliquidExchangeAction`
   คงชื่อ public ที่ใช้กันมาแล้ว, รายการ release ในอดีต และชื่อต้นทางในตาราง migration ไว้

5. **คำขยายด้าน runtime**: ใช้ `Live` เมื่อ type นั้นเลือกหรือกำหนด semantics ของ runtime แบบ real-time เช่น `LiveNode`
   เทียบกับ `BacktestNode` และ `LiveClock` เทียบกับ `VirtualClock` ตระกูล `LiveDataEngineConfig`, `LiveRiskEngineConfig`
   และ `LiveExecutionEngineConfig` ก็ต่างจาก engine config แกนกลางที่นำกลับมาใช้ได้ในลักษณะเดียวกัน

   ไม่ต้องใส่ `Live` ในตระกูล adapter client ทั่วไป เพราะ client ที่เชื่อมต่อจริงเป็นค่าเริ่มต้นอยู่แล้ว ให้ขยายชื่อ implementation
   ทางเลือกตามพฤติกรรมของมัน เช่น `SandboxExecutionClient` หรือ `DatabentoHistoricalClient` คู่ protocol แบบ live/historical
   ที่ระบุชัดเจนอาจคง `Live` ไว้เพื่อแยกสอง implementation ออกจากกัน

6. **Config ของ adapter factory**: ตั้งชื่อ input ด้าน data และ execution เป็น `<Venue>DataClientConfig` และ
   `<Venue>ExecutionClientConfig` factory ใช้ client config เหล่านี้โดยตรงแทนที่จะมี factory config ห่อแยกอีกชั้น
   `LiveNodeConfig` เป็นเจ้าของ `trader_id` ส่วนค่า `account_id` เฉพาะ venue ให้อยู่ใน execution client config

#### API สำหรับโหลดข้อมูล [#data-loading-apis]

ใช้ free function สำหรับการนำเข้าข้อมูลแบบ stateless ใช้ class เฉพาะเมื่อ instance ต้องเก็บ configuration ที่นำกลับมาใช้ได้,
cache, worker, สถานะการ iterate หรือ resource ที่เปิดค้างไว้ข้ามการเรียกหลายครั้ง อย่าใช้ class ที่ไม่มีสถานะเพียงเพื่อจัดกลุ่ม
static method หรือ class method

ปฏิบัติตามความแตกต่างเชิงความหมายใน [Polars I/O API](https://docs.pola.rs/api/python/stable/reference/io.html)
โดยปรับให้เข้ากับคำศัพท์ `load_*` ที่ Nautilus ใช้อยู่:

- `load_<source>_<data>` อ่าน, normalize และสร้างผลลัพธ์ที่สมบูรณ์ทันที (eager)
- `scan_<source>_<data>` สร้าง lazy query หรือแผนการ execute แบบเลื่อนเวลา
- `stream_<source>_<data>` ทยอยส่ง record หรือ batch ออกมาทีละส่วน
- `write_<format>` เขียนผลลัพธ์ที่อยู่ในหน่วยความจำทันที (eager)
- `sink_<format>` เขียนผ่านเส้นทางการ execute แบบ lazy หรือ streaming

สำหรับการนำเข้าข้อมูล ให้เรียงส่วนของชื่อจากทั่วไปไปเฉพาะเจาะจง: verb, source, ข้อมูลเชิงตรรกะ แล้วตามด้วยรูปแบบการแทนข้อมูล
(optional) ใส่รูปแบบการแทนข้อมูลเฉพาะเมื่อมี format พี่น้องอยู่จริง เช่น `load_binance_order_book_deltas` ดีกว่า class แบบ
stateless อย่าง `BinanceOrderBookDeltaDataLoader.load` หรือฟังก์ชันแบบกว้าง ๆ อย่าง `load_binance_data`

### การจัดรูปแบบ [#formatting]

1. สำหรับบรรทัดโค้ดที่ยาวและการเรียกที่มี argument มากกว่าสองสามตัว layout ที่แนะนำคือใช้ indent ตรรกะระดับถัดไป
   ขึ้นบรรทัดใหม่ที่ indent นั้น แทนการจัดแนวแบบ hanging 'vanity' ตามวงเล็บเปิด วิธีนี้ช่วยประหยัดพื้นที่ทางขวาและทำให้
   โค้ดสำคัญอยู่ตรงกลางสายตามากขึ้น layout นี้ยังคงสภาพได้แม้ชื่อฟังก์ชันหรือ method จะเปลี่ยน

2. วงเล็บปิดควรอยู่บนบรรทัดใหม่ โดยจัดแนวที่ indent ตรรกะ

3. พารามิเตอร์หรือ argument แบบ hanging หลายตัวควรลงท้ายด้วย trailing comma

## Commit message [#commit-messages]

Commit message ใช้ subject ที่ขึ้นต้นด้วยตัวพิมพ์ใหญ่ในรูปคำสั่ง (imperative) ที่ระบุส่วนที่ได้รับผลกระทบ และอาจตามด้วย body
ที่อธิบายการเปลี่ยนแปลง

### บรรทัด subject [#subject-line]

- ขึ้นต้นด้วยกริยารูปคำสั่งที่ขึ้นต้นด้วยตัวพิมพ์ใหญ่ เพื่อให้ subject บอกว่า commit ทำอะไรเมื่อถูก apply
  `Add`, `Fix`, `Improve`, `Refine`, `Update`, `Remove`, `Refactor` และ `Standardize` ครอบคลุมประวัติส่วนใหญ่
- ระบุส่วนที่ได้รับผลกระทบ (crate, adapter, subsystem หรือ type) เพื่อให้ไล่ดู log ได้ง่าย
- ให้ subject ยาวอย่างน้อย 10 ตัวอักษร เพื่อให้ระบุส่วนที่ได้รับผลกระทบได้ชัดเจน
- พยายามให้ยาวไม่เกิน 60 ตัวอักษร เพื่อให้แสดงผลบน GitHub ได้ชัดเจนและข้อความกระชับ commit-message hook
  จะเตือนโดยไม่ทำให้ล้มเหลวเมื่อ subject ยาวเกินเป้านี้ และโปรเจกต์มีแผนจะบังคับใช้ขีดจำกัดนี้ในอนาคต
- อย่าลงท้าย subject ด้วยจุด
- อย่าใส่หมายเลข issue หรือ pull request ใน subject GitHub จะต่อท้ายหมายเลข pull request ให้เองตอน squash merge และ
  การอ้างอิงอื่น ๆ ควรอยู่ใน body commit-message hook จะปฏิเสธ subject ที่มี `#<number>` ไม่ว่าตำแหน่งใด

```text
Add Decimal constructors to Instrument trait
Fix non-atomic order event application
Refine cross-platform wheel validation
Remove stale security audit exceptions
```

หลีกเลี่ยงรูปแบบเหล่านี้:

```text
feat(bybit): add due_post_only flag        # Conventional Commits type and scope
fix: bug                                   # lowercase, unspecific, too short
Fixed the Bybit post-only rejection flag.  # past tense, trailing period
Update stuff                               # says nothing about the surface
Fix the post-only flag (#4544)             # pull request number added by hand
Fix PR #4544 review feedback               # issue or pull request number in the subject
```

### Conventional Commits [#conventional-commits]

ห้ามใช้ syntax ของ [Conventional Commits](https://www.conventionalcommits.org/) สำหรับ commit message หรือชื่อ
pull request editor และ AI assistant หลายตัวสร้างรูปแบบนี้เป็นค่าเริ่มต้น แต่ไม่มี commit ใดในประวัติของ repository นี้ใช้เลย
type และ scope เป็นการซ้ำข้อมูลที่ subject บอกไว้แล้ว ชื่อ pull request ก็สำคัญเช่นกัน เพราะ squash merge จะเปลี่ยนชื่อ PR
เป็น subject ของ commit

### Body [#body]

Body เป็น optional แต่การเปลี่ยนแปลงใดที่มากกว่าเรื่องเล็กน้อยควรอธิบายว่าทำไมจึงเปลี่ยน แทนที่จะเล่าซ้ำสิ่งที่อยู่ใน diff

- คั่น body กับ subject ด้วยบรรทัดว่างหนึ่งบรรทัด
- ให้บรรทัดใน body ยาวไม่เกิน 79 ตัวอักษร เพื่อให้สอดคล้องกับ PEP 8 และเครื่องมือ Git แบบดั้งเดิม
- ใช้ย่อหน้าหรือ bullet point ตามที่เหมาะกับการเปลี่ยนแปลง bullet อาจใช้รูปคำสั่งแบบเดียวกับ subject และไม่จำเป็นต้องลงท้ายด้วยจุด
- ใส่ hyperlink ที่ให้ข้อมูลเมื่อช่วยผู้อ่านในอนาคตได้

### การอ้างอิง issue [#issue-references]

- อ้างอิง issue ใน body โดยทั่วไปที่บรรทัดสุดท้าย: `Resolves #4534` เมื่อ commit ปิด issue นั้น หรือ `Related to #4547`
  เมื่อเป็นงานเพียงบางส่วน
- GitHub จะต่อท้ายหมายเลข pull request ให้ subject ตอน squash merge ทำให้ได้ subject เช่น
  `Fix TWAP child-order sizing and interval validation (#4544)` อย่าเติม suffix นี้เอง และอย่าอ้างอิง pull request หรือ issue
  ที่ตำแหน่งอื่นใดใน subject subject ไม่มีที่สำหรับรายละเอียดที่ body อธิบายได้ดีกว่า และหมายเลขที่เขียนเองจะซ้ำซ้อนหรือ
  ขัดแย้งกับหมายเลขที่ถูกต่อท้าย
- พยายามให้ชื่อ pull request สั้นพอที่เมื่อต่อ suffix แล้ว subject หลัง squash merge ยังยาวไม่เกิน 60 ตัวอักษร
