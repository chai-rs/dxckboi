# Synthetics

**Synthetic instrument** คือ instrument ที่กำหนดขึ้นที่ฝั่ง local โดยราคาได้มาจาก instrument อื่น
สามารถรวม component จาก venue เดียวหรือหลาย venue แล้วนำผลลัพธ์มาแสดงเป็น
instrument มาตรฐานของ Nautilus ภายใต้ venue code สังเคราะห์ `SYNTH`

synthetic instrument มีประโยชน์สำหรับ:

- ให้ component ประเภท `DataActor` และ `Strategy` subscribe feed ของ quote หรือ trade ได้
- trigger emulated order จากราคาที่คำนวณขึ้น
- สร้าง bar จาก quote หรือ trade สังเคราะห์

:::info
synthetic instrument ซื้อขายโดยตรงไม่ได้ มันมีอยู่เฉพาะภายในแพลตฟอร์มฝั่ง local และใช้
เป็นเครื่องมือวิเคราะห์ ในอนาคต Nautilus อาจรองรับการซื้อขาย instrument ที่เป็น component โดยอิงจาก
พฤติกรรมของ synthetic instrument
:::

## ภาษาของ formula {#formula-language}

synthetic instrument แต่ละตัวกำหนด formula สำหรับคำนวณราคา Nautilus จะประเมิน formula นี้ด้วย
expression engine เชิงตัวเลขที่มีมาในตัว แล้วแปลงผลลัพธ์ตัวเลขสุดท้ายเป็น `Price`
ของ synthetic

### Syntax ที่รองรับ {#supported-syntax}

formula สามารถอ้างอิงค่า `InstrumentId` ของ component ได้โดยตรง รวมถึง ID ที่มี `/`
และ `-`

| Construct           | Example                                        | Notes                                                                       |
| ------------------- | ---------------------------------------------- | --------------------------------------------------------------------------- |
| Component reference | `BTCUSDT.BINANCE`                              | ใช้ข้อความ `InstrumentId` ดิบ                                                |
| Component reference | `AUD/USD.SIM`                                  | ID ที่มี `/` ใช้ได้                                                          |
| Component reference | `ETH-USDT-SWAP.OKX`                            | ID ที่มี `-` ใช้ได้                                                          |
| Numeric literal     | `1`, `0.5`, `1.2e-3`                           | ประเมินด้วย semantics ของ `f64`                                              |
| Boolean literal     | `true`, `false`                                | ใช้ในเงื่อนไขและ logical expression                                         |
| Parentheses         | `(a + b) / 2`                                  | ใช้วงเล็บเพื่อเปลี่ยนลำดับความสำคัญ                                           |
| Unary operators     | `-x`, `!flag`                                  | unary `-` กลับเครื่องหมายตัวเลข unary `!` กลับค่า boolean                     |
| Binary operators    | `+ - * / % ^`, `== !=`, `< <= > >=`, `&& \|\|` | arithmetic ทำงานกับตัวเลข logical operator ทำงานกับ boolean                  |
| Local assignment    | `spread = a - b; spread / 2`                   | statement ทำงานจากซ้ายไปขวา และ formula ต้องจบด้วยค่า                        |
| Comments            | `// line`, `/* block */`                       | comment จะถูกละเว้น                                                          |

:::note
formula ใหม่ควรใช้ค่า `InstrumentId` ดิบ เพื่อความเข้ากันได้ย้อนหลัง formula ที่แทน `-` ด้วย
`_` ใน component ID ยังคงใช้ได้
:::

### ลำดับความสำคัญของ operator {#operator-precedence}

expression engine ประเมิน operator ตามลำดับต่อไปนี้ จากลำดับความสำคัญสูงสุดไป
ต่ำสุด:

| Level   | Operators            | Notes                                                         |
| ------- | -------------------- | ------------------------------------------------------------- |
| Highest | `^`                  | ยกกำลัง เชื่อมจากขวา (right associative)                        |
|         | Unary `-`, unary `!` | `-2 ^ 2` ประเมินเป็น `-(2 ^ 2)`                                |
|         | `*`, `/`, `%`        | คูณ, หาร และ modulo                                            |
|         | `+`, `-`             | บวกและลบ                                                       |
|         | `<`, `<=`, `>`, `>=` | การเปรียบเทียบตัวเลข                                           |
|         | `==`, `!=`           | เท่ากันและไม่เท่ากัน ทั้งสองฝั่งต้องเป็น type เดียวกัน               |
| Lowest  | `&&`, `\|\|`         | operator แบบ boolean                                          |

assignment ไม่ใช่ expression operator ให้คั่น statement ด้วย `;` และให้ statement สุดท้าย
เป็นค่าที่ต้องการให้ synthetic สร้างออกมา

### ฟังก์ชันในตัว {#built-in-functions}

| Function | Signature                              | Notes                                                                                                 |
| -------- | -------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `abs`    | `abs(x)`                               | ค่าสัมบูรณ์                                                                                            |
| `ceil`   | `ceil(x)`                              | ปัดขึ้น                                                                                                |
| `floor`  | `floor(x)`                             | ปัดลง                                                                                                  |
| `round`  | `round(x)`                             | ปัดเป็นจำนวนเต็มที่ใกล้ที่สุดตามกฎ `f64` ของ Rust                                                        |
| `min`    | `min(x1, x2, ...)`                     | รับ argument ตัวเลขตั้งแต่หนึ่งตัวขึ้นไป                                                                  |
| `max`    | `max(x1, x2, ...)`                     | รับ argument ตัวเลขตั้งแต่หนึ่งตัวขึ้นไป                                                                  |
| `if`     | `if(condition, when_true, when_false)` | condition ต้องเป็น boolean ทั้งสอง branch ต้องเป็น type เดียวกัน และจะประเมินเฉพาะ branch ที่ถูกเลือก        |

### กฎของ type {#type-rules}

- input จาก component เป็นตัวเลข
- arithmetic operator ต้องการ operand เป็นตัวเลขและคืนผลเป็นตัวเลข
- `<`, `<=`, `>`, `>=` ต้องการ operand เป็นตัวเลขและคืนผลเป็น boolean
- `==` และ `!=` รับ type ใดก็ได้ที่ตรงกัน (ตัวเลขทั้งคู่หรือ boolean ทั้งคู่) และคืนผลเป็น
  boolean
- `&&`, `||` และ unary `!` ต้องการ operand เป็น boolean
- `&&` และ `||` เป็นแบบ short-circuit ฝั่งขวาจะถูกประเมินเฉพาะเมื่อจำเป็น
- local variable ต้องถูก assign ก่อนใช้งาน
- ชื่อ local variable ต้องขึ้นต้นด้วยตัวอักษร ASCII หรือ `_` และตามด้วยตัวอักษร ASCII, ตัวเลข
  หรือ `_`
- ผลลัพธ์สุดท้ายของ formula ต้องเป็นตัวเลข formula ที่จบด้วย assignment หรือให้ผลเป็น
  boolean ใช้กับ synthetic instrument ไม่ได้

### ข้อจำกัด {#limits}

expression engine บังคับใช้ข้อจำกัดตอน compile ต่อไปนี้ formula ที่เกินข้อจำกัดจะ
เกิด error ที่ชัดเจนตอนสร้าง

| Limit           | Value | Description                                                                                       |
| --------------- | ----- | ------------------------------------------------------------------------------------------------- |
| Stack depth     | 32    | จำนวนค่าระหว่างกลางสูงสุดบน evaluation stack                                                        |
| Local variables | 16    | จำนวนชื่อ local variable ที่ไม่ซ้ำกันสูงสุด                                                          |
| Nesting depth   | 128   | ความลึกสูงสุดของการซ้อน syntax และ expression tree โดย expression ระดับบนสุดนับเป็นหนึ่งระดับ          |

weighted sum ของ 8 component ใช้ stack depth สูงสุด 3 และไม่มี local เลย weighted sum ของ
N component จะสร้าง tree ที่ลึก N + 1 ดังนั้นข้อจำกัด nesting depth จึงจำกัด weighted sum ไว้ที่
127 component

### ตัวอย่าง {#examples}

- spread แบบง่าย: `BTCUSDT.BINANCE - ETHUSDT.BINANCE`
- ค่าเฉลี่ยของคู่ FX สองคู่: `(AUD/USD.SIM + NZD/USD.SIM) / 2`
- นำค่าระหว่างกลางมาใช้ซ้ำ: `spread = BTCUSDT.BINANCE - ETHUSDT.BINANCE; spread / 2`
- output แบบมีเงื่อนไข: `if(BTCUSDT.BINANCE > ETHUSDT.BINANCE, BTCUSDT.BINANCE, ETHUSDT.BINANCE)`

## การสร้าง synthetic instrument {#creating-a-synthetic-instrument}

ก่อนสร้าง synthetic instrument ต้องแน่ใจว่า instrument ที่เป็น component ทั้งหมดมีอยู่ใน cache แล้ว ให้ subscribe
feed ของ quote หรือ trade ของแต่ละ component รวมถึงของ synthetic ด้วย quote สังเคราะห์ได้มาจาก quote
ของ component เท่านั้น และ trade สังเคราะห์ได้มาจาก trade ของ component เท่านั้น

เมื่อ tick ของ component มาถึง engine จะรวมเข้ากับราคาล่าสุดใน cache ของ component อื่น เพื่อ
คำนวณราคาสังเคราะห์ จนกว่า **ทุก component จะมี tick อย่างน้อยหนึ่งครั้ง** synthetic
จะไม่ publish อะไรออกมาเลย

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python (สร้าง synthetic แทน spread ระหว่าง `BTCUSDT.BINANCE` และ `ETHUSDT.BINANCE` จาก actor หรือ strategy) จึงตัดออก

:::note
`instrument_id` ของ synthetic อยู่ในรูป `{symbol}.SYNTH` เช่น symbol `BTC-ETH:BINANCE` จะได้
`BTC-ETH:BINANCE.SYNTH`
:::

## การอัปเดต formula {#updating-formulas}

สามารถอัปเดต formula ของ synthetic ได้ทุกเมื่อ

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## Trigger instrument ID {#trigger-instrument-ids}

สามารถ trigger emulated order จากราคาสังเคราะห์ได้ โดย synthetic
instrument จะปล่อย emulated order ออกไปเมื่อราคาสังเคราะห์ถึงเงื่อนไข trigger (กำหนดผ่าน
`emulation_trigger` และ `trigger_instrument_id` ของ order)

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## ประสิทธิภาพ {#performance}

formula ถูก compile ครั้งเดียวตอนสร้าง และถูกประเมินทุกครั้งที่ tick ราคาของ component เข้ามา expression
engine ใช้ stack ของ `f64` ที่ไม่มีการ allocate หน่วยความจำ (zero-allocation) ในการประเมิน จึงเพิ่ม overhead น้อยมากต่อเส้นทาง
การประมวลผล tick

วัดบน Apple M4 Pro, rustc 1.94.1, release profile (opt-level 3):

### การประเมิน (hot path) {#evaluation-hot-path}

| Formula pattern                         | Time  |
| --------------------------------------- | ----- |
| `(A + B) / 2.0`                         | 12 ns |
| `A * 0.4 + B * 0.3 + C * 0.2 + D * 0.1` | 18 ns |
| `if(A > B, A - B, B - A)`               | 12 ns |
| `spread = A - B; mid = ...; mid + ...`  | 19 ns |
| `max(min(A, B * 20), abs(A - B))`       | 15 ns |

### การ scale ของการประเมิน (weighted sum) {#evaluation-scaling-weighted-sum}

| Components | Time  |
| ---------- | ----- |
| 2          | 14 ns |
| 4          | 18 ns |
| 8          | 28 ns |

### การ compile (cold path) {#compilation-cold-path}

| Formula pattern  | Time   |
| ---------------- | ------ |
| Simple average   | 675 ns |
| 4-input weighted | 1.4 us |
| Conditional      | 1.0 us |
| With locals      | 1.3 us |
| Hyphenated IDs   | 755 ns |

## การจัดการ error {#error-handling}

Nautilus validate synthetic instrument ในทุกขอบเขต การ compile formula จะปฏิเสธ symbol ที่ไม่รู้จัก, error
ด้าน type และการใช้ capacity เกิน ส่วนการประเมินจะปฏิเสธจำนวน input ที่ไม่ถูกต้องและราคาที่ไม่ใช่ค่าจำกัด (NaN, Infinity)
ก่อนที่ค่าเหล่านั้นจะไปถึง formula

ดูข้อกำหนดของ input และ error ได้ที่ [Rust API reference](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Instruments](instruments/) - นิยามของ instrument และ instrument type เฉพาะของแต่ละ venue
- [Data](data/) - market data type ที่อ้างอิงถึง instrument
- [Orders](orders/) - order สามารถใช้ synthetic instrument ID เป็น emulation trigger ได้
