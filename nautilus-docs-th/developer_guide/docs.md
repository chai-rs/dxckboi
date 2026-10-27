---
title: "Docs Style"
---

ใช้คู่มือนี้สำหรับสไตล์และแนวปฏิบัติในการเขียนเอกสารของ NautilusTrader

คู่มือ [Markdown Style](markdown_style.md) เป็นมาตรฐานพื้นฐานร่วมกันสำหรับไวยากรณ์และการจัดรูปแบบ Markdown
และ `.markdownlint.jsonc` บังคับใช้ส่วนที่ตรวจสอบได้แบบอัตโนมัติ คู่มือนี้ครอบคลุมเฉพาะสิ่งที่เป็นของเอกสาร
NautilusTrader โดยไม่ทวนมาตรฐานพื้นฐานนั้นซ้ำ

## หลักการทั่วไป [#general-principles]

- เลือกใช้โครงสร้างที่เรียบง่าย และเขียนให้กระชับอ่านง่าย
- ใช้ convention, สไตล์ และรูปแบบที่สม่ำเสมอ
- เขียนให้ผู้อ่านที่มีพื้นฐานทางเทคนิคต่างกันเข้าใจได้

## ประเภทของเอกสาร [#documentation-types]

หน้าส่วนใหญ่ควรจัดอยู่ในหนึ่งในสี่ประเภท
([Divio documentation system](https://docs.divio.com/documentation-system/))
การผสมหลายประเภทไว้ในหน้าเดียวทำให้อ่านยากและดูแลรักษายาก

| ประเภท           | จุดประสงค์                        | Section          |
| ---------------- | ------------------------------- | ---------------- |
| **Tutorial**     | สอนโดยพาทำงานหนึ่งอย่างไปทีละขั้น     | `tutorials/`     |
| **How-to guide** | แก้ปัญหาเฉพาะเรื่อง                 | `how_to/`        |
| **Explanation**  | อธิบายการออกแบบและสถาปัตยกรรม      | `concepts/`      |
| **Reference**    | อธิบายกลไกการทำงานโดยละเอียด        | `api_reference/` |

มีสอง section ที่เป็นข้อยกเว้น:

- `getting_started/` รวม walkthrough แบบ tutorial เข้ากับขั้นตอนการติดตั้งสำหรับผู้เริ่มต้น
- หน้าใน `integrations/` รวมเนื้อหาแบบ reference (ความสามารถ, symbology) เข้ากับเนื้อหาแบบ how-to
  (การติดตั้ง, การตั้งค่า) ทำให้แต่ละหน้าของ venue สมบูรณ์ในตัวเอง

เนื้อหา how-to ที่แยกเดี่ยวและไม่ได้เฉพาะกับ venue ใดให้อยู่ใน `how_to/`

### การเลือกประเภทที่เหมาะสม [#choosing-the-right-type]

- **หน้านี้พาผู้มาใหม่ผ่านประสบการณ์การเรียนรู้หรือไม่?** Tutorial
- **หน้านี้ตอบคำถาม "ทำ...อย่างไร?" ให้คนที่รู้จักระบบอยู่แล้วหรือไม่?** How-to guide
- **หน้านี้อธิบายว่าทำไมสิ่งหนึ่งจึงทำงานแบบนั้นหรือไม่?** Explanation
- **หน้านี้แสดงรายการ class, config field, enum หรือความสามารถหรือไม่?** Reference

tutorial บอกว่า "ทำสิ่งนี้ แล้วสิ่งนี้ แล้วสิ่งนี้" ผู้เขียนเป็นคนเลือกเส้นทาง
ส่วน how-to guide บอกว่า "นี่คือวิธีทำให้ได้ X" ผู้อ่านรู้อยู่แล้วว่าต้องการ X ให้แยกสองประเภทนี้ออกจากกัน:

- tutorial ไม่ควรสมมติว่าผู้อ่านมีความรู้มาก่อน
- how-to guide ไม่ควรสอนแนวคิดพื้นฐาน

เมื่อเอกสารประเภทหนึ่งต้องอ้างถึงอีกประเภทหนึ่ง ให้ใส่ link แทนการเขียนซ้ำในหน้า ตัวอย่างเช่น how-to guide
ที่ตั้งค่า `LiveNodeConfig` ควร link ไปยัง API reference สำหรับนิยามของ field แทนที่จะแสดงรายการซ้ำอีกครั้ง

## ภาษาและน้ำเสียง [#language-and-tone]

- ใช้ประโยค active voice เมื่อทำได้ ("Configure the adapter" แทน "The adapter should be configured")
- ใช้ present tense เมื่ออธิบายความสามารถที่มีอยู่ในปัจจุบัน
- ใช้ future tense เฉพาะกับฟีเจอร์ที่วางแผนไว้
- หลีกเลี่ยงศัพท์เฉพาะที่ไม่จำเป็น และนิยามศัพท์เทคนิคเมื่อใช้ครั้งแรก
- เขียนตรงไปตรงมาและกระชับ หลีกเลี่ยงคำฟุ่มเฟือยอย่าง "basically", "simply", "just"
- ใช้โครงสร้างคู่ขนานในรายการ รักษารูปแบบไวยากรณ์ให้สม่ำเสมอทุกข้อ

## ตาราง Markdown [#markdown-tables]

ไวยากรณ์ตาราง การจัดแนว pipe และการเว้นระยะของเส้นคั่นให้เป็นไปตามคู่มือ
[Markdown Style](markdown_style.md#tables)

### Notes และคำอธิบาย [#notes-and-descriptions]

- notes และคำอธิบายทั้งหมดควรจบด้วยจุด
- เขียน notes ให้กระชับแต่ได้ใจความ
- ใช้ sentence case (ตัวพิมพ์ใหญ่เฉพาะอักษรแรกและคำนามเฉพาะ)

### ตัวอย่าง [#example]

```markdown
| Order Type             | Spot | Margin | USDT Futures | Coin Futures | Notes                   |
| ---------------------- | ---- | ------ | ------------ | ------------ | ----------------------- |
| `MARKET`               | ✓    | ✓      | ✓            | ✓            |                         |
| `STOP_MARKET`          | -    | ✓      | ✓            | ✓            | Not supported for Spot. |
| `MARKET_IF_TOUCHED`    | -    | -      | ✓            | ✓            | Futures only.           |
```

### สัญลักษณ์แสดงการรองรับ [#support-indicators]

- ใช้ `✓` สำหรับฟีเจอร์ที่รองรับ
- ใช้ `-` สำหรับฟีเจอร์ที่ไม่รองรับ (ไม่ใช้ `✗` หรือสัญลักษณ์อื่น)
- เมื่อเพิ่ม notes สำหรับฟีเจอร์ที่ไม่รองรับ ให้เน้นด้วยตัวเอียง: `*Not supported*`
- เขียน notes ของสิ่งที่ไม่รองรับให้เจาะจงเมื่อเหตุผลมีความสำคัญ: ใช้ `*Not supported by <venue>*`
  สำหรับสิ่งที่ venue ไม่มี หรือ `*Not currently implemented*` สำหรับสิ่งที่ adapter ยังไม่ได้ทำ
- ปล่อย cell ว่างไว้เมื่อไม่มีเนื้อหาที่จำเป็น

## การอ้างอิงโค้ด [#code-references]

inline code และ fenced code block ให้เป็นไปตามคู่มือ [Markdown Style](markdown_style.md#code)

เมื่ออ้างอิงตำแหน่งในโค้ด ให้ใช้ `file_path::function_name` หรือ `file_path::ClassName` แทนเลขบรรทัด
ซึ่งจะล้าสมัยเมื่อโค้ดเปลี่ยน

## Headings [#headings]

สไตล์ของ heading, ตัวพิมพ์ และลำดับชั้นให้เป็นไปตามคู่มือ [Markdown Style](markdown_style.md#headings):
ใช้ title case สำหรับ heading ของหน้า และ sentence case สำหรับ heading ที่อยู่ใต้ลงไป

NautilusTrader มีข้อยกเว้นหนึ่งข้อสำหรับหน้า `index.md`: heading และ link นำทางที่ระบุชื่อเอกสารแต่ละหน้า
ให้ใช้ชื่อ H1 ของเอกสารปลายทางตรงตามต้นฉบับ รวมถึงตัวพิมพ์ใหญ่เล็ก ส่วน heading ของ section ทั่วไปยังคงใช้ sentence case

ให้ขึ้นต้นคำนามเฉพาะด้วยตัวพิมพ์ใหญ่เสมอไม่ว่า heading จะอยู่ระดับใด (ชื่อผลิตภัณฑ์ เทคโนโลยี บริษัท ตัวย่อ)

## Lists [#lists]

เครื่องหมายรายการ การเรียงลำดับ และการย่อหน้าให้เป็นไปตามคู่มือ [Markdown Style](markdown_style.md#lists)

จบรายการด้วยจุดเมื่อเป็นประโยคสมบูรณ์

## Links และการอ้างอิง [#links-and-references]

ข้อความของ link, สไตล์ของ link และรูปภาพให้เป็นไปตามคู่มือ
[Markdown Style](markdown_style.md#links-and-images)

อ้างอิงเอกสารภายนอกเมื่อเหมาะสม

## คำศัพท์ทางเทคนิค [#technical-terminology]

- สร้างตาราง capability โดยอิงตาม domain model ของ Nautilus ไม่ใช่ศัพท์เฉพาะของ exchange
- กล่าวถึงศัพท์เฉพาะของ exchange ในวงเล็บหรือ notes เมื่อจำเป็นต่อความชัดเจน
- ใช้คำศัพท์ให้สม่ำเสมอตลอดทั้งเอกสาร

## ตัวอย่างและ code sample [#examples-and-code-samples]

- ให้ตัวอย่างที่ใช้งานได้จริงและรันได้
- ใส่ import และบริบทที่จำเป็น
- ใช้ชื่อตัวแปรและค่าที่สมจริง
- เพิ่ม comment อธิบายส่วนที่ไม่ชัดเจนในตัวอย่าง

## Admonitions [#admonitions]

ใช้ admonition block เพื่อเน้นข้อมูลสำคัญ:

| Admonition   | จุดประสงค์                                                      |
| ------------ | ------------------------------------------------------------- |
| `:::note`    | บริบทเสริมที่ช่วยให้ชัดเจนขึ้นแต่ไม่ใช่สาระหลัก                         |
| `:::info`    | ข้อมูลสำคัญที่ผู้อ่านควรรู้                                          |
| `:::tip`     | คำแนะนำที่มีประโยชน์หรือ best practice                             |
| `:::warning` | ข้อผิดพลาดที่อาจเกิดขึ้นหรือข้อควรระวังสำคัญ                             |
| `:::danger`  | ปัญหาร้ายแรงที่อาจทำให้ข้อมูลสูญหายหรือระบบล้มเหลว                      |

อย่าใช้ admonition มากเกินไป เพราะยิ่งมากยิ่งลดความสำคัญลง

## MDX components [#mdx-components]

เว็บไซต์เอกสาร (fumadocs) มี MDX component ในตัวที่ใช้ได้ในทุกไฟล์ `.md` โดยไม่ต้อง import

### Tabs [#tabs]

ใช้ tabs สำหรับเนื้อหาที่แยกตามภาษาหรือตาม variant

สำหรับตัวอย่างโค้ด ให้เพิ่ม `tab="..."` ให้กับ fenced code block ที่อยู่ติดกัน

สำหรับตารางหรือเนื้อหาอื่น ให้ครอบแต่ละ variant ด้วย `<Tabs>` และ `<Tab>` ตาราง Fields ของ instrument ใช้วิธีนี้
เพื่อให้แต่ละภาษาแสดง column ของ type เพียง column เดียว แทนการแสดง column ของแต่ละภาษาเคียงกัน
เว้นบรรทัดว่างไว้ทั้งด้านบนและด้านล่างของเนื้อหาข้างใน เพื่อให้ Markdown render ได้

> หมายเหตุ: ตัวอย่าง Tabs ในต้นฉบับใช้คู่ Rust/Python จึงตัดออก

### Steps [#steps]

ใช้ `Steps` และ `Step` สำหรับขั้นตอนที่ต้องทำตามลำดับ

```markdown
<Steps>
<Step>
Configure the adapter.
</Step>
<Step>
Start the trading node.
</Step>
</Steps>
```

### Accordions [#accordions]

ใช้ `Accordions` และ `Accordion` สำหรับเนื้อหาที่ยุบ/ขยายได้

```markdown
<Accordions>
<Accordion title="Advanced configuration">
Content here.
</Accordion>
</Accordions>
```

### Files [#files]

ใช้ `Files`, `Folder` และ `File` สำหรับแสดงโครงสร้าง directory

```markdown
<Files>
<Folder name="src" defaultOpen>
<File name="main.rs" />
<File name="lib.rs" />
</Folder>
</Files>
```

### Cards [#cards]

ใช้ `Cards` และ `Card` สำหรับ grid ของเนื้อหาที่มี link

```markdown
<Cards>
<Card title="Getting started" href="/latest/getting_started" />
<Card title="Concepts" href="/latest/concepts" />
</Cards>
```

### TypeTable [#typetable]

ใช้ `TypeTable` สำหรับตารางเอกสารของ parameter หรือ type

## API documentation [#api-documentation]

- อธิบาย parameter และ return type ให้ชัดเจน
- ใส่ตัวอย่างการใช้งานสำหรับ API ที่ซับซ้อน
- อธิบาย side effect หรือพฤติกรรมสำคัญ
- เขียนคำอธิบาย parameter ให้กระชับแต่ครบถ้วน
