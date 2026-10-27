---
title: "Markdown Style"
---

Standard revision: 1

เอกสารนี้กำหนด baseline ของ Markdown ที่ใช้ร่วมกันสำหรับ repository ที่นำสำเนาไปใช้ในเครื่อง สำเนาต้องเหมือนกับ
source ที่ดูแลอยู่ทุก byte ส่วนเพิ่มเติมเฉพาะ repository ให้แยกไว้ใน local guide อีกฉบับ

## ระดับของข้อกำหนด [#requirement-levels]

แต่ละกฎมีระดับอย่างใดอย่างหนึ่งจากสามระดับนี้:

- **Required**: ใช้กับทุกไฟล์ที่อยู่ในขอบเขต กฎที่เขียนเป็นคำสั่งโดยไม่มีเงื่อนไข เช่น "Use" และ "Do not" หรือใช้คำว่า
  "must" ถือเป็น Required
- **Preferred**: ระบุรูปแบบตั้งต้นเมื่อมีรูปแบบที่ถูกต้องมากกว่าหนึ่งแบบ กฎแบบ Preferred ใช้คำว่า "Prefer" และไม่มีผลต่อ
  conformance
- **Transitional**: ใช้กับ construct ที่ระบุไว้เมื่อถูกเพิ่มหรือแก้ไขอย่างมีนัยสำคัญ และมีป้ายกำกับ **Transitional** หรือ
  ถูกอธิบายว่าเป็นกฎช่วงเปลี่ยนผ่าน ของเดิมที่มีอยู่ยังคงไว้ได้จนกว่าจะมีการ migrate แยกต่างหาก

ข้อความที่ใช้ "may" หรือ "allowed" เป็นการอนุญาตแบบมีขอบเขต ไม่ใช่ข้อบังคับ เงื่อนไขใด ๆ ที่จำกัดการอนุญาตนั้นถือเป็น
Required repository อาจบันทึกข้อยกเว้นภายในที่แคบกว่าได้ ในกรณีที่ renderer, เนื้อหาที่ generate ขึ้น หรือเนื้อหาที่
นำเข้ามาจำเป็นต้องใช้

## ภาษาและ extension [#language-and-extensions]

- ใช้ [CommonMark](https://spec.commonmark.org/current/) เป็น specification พื้นฐาน
- ใช้ [GitHub Flavored Markdown](https://github.github.com/gfm/) สำหรับตาราง, task list, ขีดฆ่า (strikethrough)
  และ autolink
- ใช้ front matter, Markdown extension หรือ renderer component เพิ่มเติม เฉพาะเมื่อ repository บันทึกไว้และรองรับ
- อย่านำ syntax ที่ขึ้นกับ renderer extension ที่ไม่ได้บันทึกไว้เข้ามาใช้

specification ที่ลิงก์ไว้เป็นเอกสารอ้างอิง ไม่ใช่สิ่งที่ต้องอ่านเป็นประจำ ให้เปิดอ่านเฉพาะเมื่อ local guide, เนื้อหาโดยรอบ
และ markdownlint ยังไม่สามารถตอบความกำกวมที่เป็นรูปธรรมของ parser หรือ renderer ได้

## การบังคับใช้ [#enforcement]

- ปฏิบัติตาม `.markdownlint.jsonc` ของ repository หรือ config ภายในที่เทียบเท่า
- ถือว่ามาตรฐานนี้คือ style ที่ตั้งใจไว้ และ markdownlint คือการบังคับใช้แบบอัตโนมัติสำหรับส่วนที่เป็นกลไกของมาตรฐาน
- ถือว่าความขัดแย้งที่ไม่ได้บันทึกไว้ระหว่างมาตรฐานนี้กับ config ภายในคือ drift ที่ต้องแก้ไข ไม่ใช่ข้อยกเว้นโดยปริยาย
- ตรวจให้แน่ใจว่า Markdown ทั้งหมดที่เขียนขึ้นภายในขอบเขตการ lint ของ repository ผ่าน Markdown check ที่ตั้งค่าไว้
- อย่าแก้ไข Markdown ที่ถูก generate ขึ้นด้วยมือ ให้แก้ที่ source แล้วรัน generator

ไฟล์ที่ถูก generate, เอกสารจาก third-party ที่นำเข้ามา และ renderer fixture อาจใช้ข้อยกเว้นของ repository ที่บันทึกไว้

## Heading [#headings]

- ใช้ ATX heading (`#`, `##`, `###` และอื่น ๆ)
- ใช้ H1 หนึ่งตัวเป็น Markdown heading แรก
- ใช้ title case สำหรับ H1
- ใช้ sentence case สำหรับ H2 ลงไป
- รักษาลำดับชั้นให้สมเหตุสมผลโดยไม่ข้ามระดับของ heading
- เว้นบรรทัดว่างหนึ่งบรรทัดทั้งด้านบนและด้านล่างของแต่ละ heading

## ย่อหน้าและการตัดบรรทัด [#paragraphs-and-wrapping]

- คั่นย่อหน้าด้วยบรรทัดว่างหนึ่งบรรทัด
- อย่าใส่บรรทัดว่างติดกันหลายบรรทัด
- ควรตั้งเป้าความยาวบรรทัดของเนื้อความไว้ที่ [100, 120] ตัวอักษร
- ควรตัดบรรทัดตามจังหวะธรรมชาติและยอมให้บรรทัดยาวขึ้น แทนที่จะทิ้งคำหนึ่งถึงสามคำไว้ในบรรทัดถัดไป
- อนุญาตให้ code block, ตาราง และปลายทางของลิงก์ที่ยาวเกินเป้าหมายได้เมื่อจำเป็น

## รายการ [#lists]

- ใช้ `-` สำหรับรายการแบบไม่เรียงลำดับ
- ใช้รายการแบบเรียงลำดับเฉพาะเมื่อลำดับมีความสำคัญ
- **Transitional**: ใช้ `1.` สำหรับทุก item ใน source ของรายการแบบเรียงลำดับ เพื่อให้ renderer ใส่เลขที่แสดงผลเอง
- รักษาการย่อหน้าและการเว้นระยะของรายการให้สอดคล้องกับ config ของ markdownlint ภายใน
- เว้นบรรทัดว่างหนึ่งบรรทัดก่อนและหลังรายการ

ตัวอย่าง:

```markdown
- First item
- Second item

1. First step
1. Second step
```

## Admonition [#admonitions]

- ใช้เฉพาะ syntax ของ admonition ที่ repository บันทึกไว้และ render ได้
- เมื่อ repository บันทึกไว้และ render GitHub alert ได้ โดยไม่มี syntax ภายในที่เข้มงวดกว่า ให้ใช้รูปแบบ GitHub
  blockquote alert โดยใช้ `NOTE`, `TIP`, `IMPORTANT`, `WARNING` หรือ `CAUTION` อย่างใดอย่างหนึ่ง
- เมื่อไม่มีการรองรับ extension นั้นหรือยังไม่ยืนยัน ให้ใช้ blockquote แบบ portable พร้อมป้ายกำกับตัวหนา
- สำหรับ admonition ที่มีหลายย่อหน้า ให้ใส่ `>` หน้าทุกบรรทัดเนื้อหาและบรรทัดว่างที่ต่อเนื่อง บรรทัดว่างเปล่า ๆ จะเป็นการ
  จบ admonition
- คงป้ายกำกับประเภทไว้ทั้งใน source และผลลัพธ์ที่ render อย่าสื่อความหมายด้วยสีหรือไอคอนเพียงอย่างเดียว
- รักษา syntax ของ admonition ที่ใช้กันอยู่และรองรับแล้วไว้ระหว่างการแก้ไขเล็ก ๆ อย่าแปลงเพียงเพื่อบังคับใช้รูปแบบ
  fallback นี้

GitHub alert:

```markdown
> [!WARNING]
> Back up the database before running the migration.
```

Portable fallback:

```markdown
> **Warning:** Back up the database before running the migration.
```

## ตาราง [#tables]

- ใช้ GFM pipe table สำหรับเนื้อหาแบบตาราง
- ใส่ pipe ทั้งด้านหน้าและด้านท้าย
- จัด pipe ให้ตรงกันในแนวตั้ง
- เติมช่องว่างให้แต่ละคอลัมน์กว้างเท่ากับ cell ที่กว้างที่สุดบวกช่องว่างอีกด้านละหนึ่งช่อง ซึ่งตรงกับผลลัพธ์ตั้งต้นของ
  Prettier MD060 ตรวจการจัดแนวของ pipe แต่ไม่สนใจเนื้อหาใน cell จึงยอมรับการเติมช่องว่างที่กว้างกว่า cell ใด ๆ มาก
- เติมช่องว่างใน delimiter cell (`| ----- |` ไม่ใช่ `|-----|`) ซึ่ง MD060 ก็ไม่ได้ตรวจเช่นกัน
- pre-commit hook `normalize markdown table padding` จะเติมช่องว่างนี้ให้ ไม่จำเป็นต้องนับช่องว่างเอง
- ควรจัดคอลัมน์ข้อความชิดซ้าย และคอลัมน์ตัวเลขชิดขวาตามความเหมาะสม
- ทำให้ delimiter row สอดคล้องกับการจัดแนวที่ต้องการเมื่อ render
- หลีกเลี่ยงตาราง HTML เว้นแต่ Markdown ไม่สามารถแสดงโครงสร้างที่ต้องการได้ หรือ repository บันทึกความจำเป็นไว้

ตัวอย่าง:

```markdown
| Name  | Value |
| ----- | ----: |
| Alpha |    42 |
| Beta  |    17 |
```

## Code [#code]

- ใช้ code block แบบ backtick fence แทน code block แบบย่อหน้า
- **Transitional**: ระบุภาษาให้กับทุก fence เปิด ใช้ `text` สำหรับข้อความธรรมดาหรือ output ที่ไม่มี grammar เฉพาะกว่านี้
- ใช้ fence ด้านนอกที่ยาวกว่าเมื่อต้องการแสดงตัวอย่าง Markdown ที่มี fence
- ใช้ inline code สำหรับคำสั่ง, ชื่อไฟล์, function, type, environment variable, configuration key และ identifier

ตัวอย่าง:

````markdown
```rust
fn main() {
    println!("Hello");
}
```
````

รัน `cargo test` แล้วแก้ไข `.markdownlint.jsonc`

## การเน้นข้อความ [#emphasis]

- ใช้ `*italic*` สำหรับการเน้น
- ใช้ `**bold**` สำหรับการเน้นอย่างหนัก
- หลีกเลี่ยงการเน้นที่ไม่ได้เพิ่มความหมาย

## เส้นคั่น [#thematic-breaks]

ใช้ขีดสามตัว:

```markdown
---
```

## ลิงก์และรูปภาพ [#links-and-images]

- ใช้ข้อความลิงก์ที่สื่อความหมาย
- ควรใช้ inline link เมื่อปลายทางเดียวกันปรากฏมากกว่าหนึ่งครั้งในเอกสาร ควรใช้ลิงก์แบบ reference-style
- หลีกเลี่ยง URL เปล่า ๆ
- ใช้ลิงก์ภายในแบบ relative เมื่อ renderer ของ repository รองรับ
- ใส่ alternative text ที่มีประโยชน์ให้รูปภาพ ใช้ alternative text ว่างเฉพาะกับรูปที่ตั้งใจใช้เพื่อตกแต่งเท่านั้น

## HTML [#html]

- ควรใช้ syntax ของ Markdown ที่ portable มากกว่า raw HTML
- ใช้ raw HTML เฉพาะเมื่อ Markdown ไม่สามารถแสดงผลลัพธ์ที่ต้องการได้ หรือ repository บันทึก element หรือ component
  นั้นไว้
- เมื่อแก้ไขเนื้อหาโดยรอบ ให้คง front matter, MDX component, fence attribute และ repository extension อื่น ๆ ที่รองรับ
  ไว้

## ไฟล์ [#files]

- ใช้ encoding แบบ UTF-8
- ใช้ line ending แบบ LF
- จบแต่ละไฟล์ด้วย newline หนึ่งตัว
- อย่าทิ้ง trailing whitespace และอย่าใช้ช่องว่างท้ายบรรทัดเพื่อสร้าง hard line break ของ Markdown

## แนวทางการแก้ไข [#editing-guidance]

เมื่อสร้างหรือแก้ไข Markdown:

- สำหรับการแก้ไขเล็ก ๆ ให้รักษา style โดยรอบและใช้ markdownlint โดยไม่ต้องเปิด guide ฉบับเต็ม เว้นแต่ทั้งสองอย่าง
  ยังไม่สามารถตอบคำถามที่เป็นรูปธรรมได้
- ก่อนสร้างหรือปรับโครงสร้างเอกสารอย่างมีนัยสำคัญ ให้อ่านเฉพาะส่วนที่เกี่ยวข้องของ Markdown guide ภายใน repository
- รักษาโครงสร้างเอกสารเดิมไว้ เว้นแต่งานนั้นจำเป็นต้องเปลี่ยน
- รักษารูปแบบให้สอดคล้องกับเนื้อหาโดยรอบและ renderer extension ที่รองรับ
- อย่าจัดรูปแบบส่วนที่ไม่เกี่ยวข้องใหม่
- รัน Markdown check แบบเจาะจงของ repository เมื่อมีให้ใช้

## การนำมาใช้ในช่วงเปลี่ยนผ่าน [#transitional-adoption]

`MD003` บังคับใช้ ATX heading ส่วนการใช้ `1.` ซ้ำในรายการแบบเรียงลำดับ และการระบุภาษาใน fence เปิดของ code block
เป็นกฎช่วงเปลี่ยนผ่าน `MD029` และ `MD040` จะยังปิดอยู่จนกว่าเอกสารเดิมจะได้รับการ migrate เชิงกลไกแยกต่างหาก อย่าขยาย
การแก้ไขเอกสารเล็ก ๆ ออกไปเพียงเพื่อ migrate construct เดิมเหล่านั้น
