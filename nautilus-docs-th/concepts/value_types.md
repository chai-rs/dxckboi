# Value Types

NautilusTrader มี value type เฉพาะสำหรับแทนแนวคิดหลักในการเทรด ได้แก่
`Price`, `Quantity` และ `Money` type เหล่านี้ใช้เลขคณิตแบบ fixed-point ภายใน
เพื่อให้การคำนวณมีประสิทธิภาพและให้ผลลัพธ์ที่แน่นอน (deterministic) เหมือนกันในทุกแพลตฟอร์ม
และทุก environment

## ภาพรวม {#overview}

| Type       | วัตถุประสงค์                                       | มีเครื่องหมาย | Currency |
| ---------- | ----------------------------------------------- | ---------- | -------- |
| `Quantity` | ขนาดของ trade, จำนวนของ order, position          | ไม่          | -        |
| `Price`    | ราคาตลาด, quote, ระดับราคา                       | ใช่          | -        |
| `Money`    | จำนวนเงิน, P&L, ยอดเงินคงเหลือใน account            | ใช่          | ใช่        |

## การดำเนินการทางคณิตศาสตร์ {#arithmetic-operations}

value type รองรับตัวดำเนินการทางคณิตศาสตร์มาตรฐาน type ของผลลัพธ์ขึ้นอยู่กับตัวดำเนินการ
และ type ของ operand

### การดำเนินการแบบ binary ระหว่าง type เดียวกัน {#same-type-binary-operations}

การบวกและการลบระหว่าง value type เดียวกันจะคืนค่าเป็น type นั้น ซึ่งคงความหมายทางโดเมนไว้
(ราคาบวกราคาก็ยังเป็นราคา):

| การดำเนินการ          | ผลลัพธ์     |
| --------------------- | ---------- |
| `Quantity + Quantity` | `Quantity` |
| `Quantity - Quantity` | `Quantity` |
| `Price + Price`       | `Price`    |
| `Price - Price`       | `Price`    |
| `Money + Money`       | `Money`    |
| `Money - Money`       | `Money`    |

การคูณ, การหาร, การหารปัดเศษลง (floor division) และการหาเศษ (modulo) ระหว่างค่าสองค่าที่เป็น type
เดียวกันจะคืนค่าเป็น `Decimal`:

| การดำเนินการ     | ผลลัพธ์    |
| ---------------- | --------- |
| `Price * Price`  | `Decimal` |
| `Price / Price`  | `Decimal` |
| `Price // Price` | `Decimal` |
| `Price % Price`  | `Decimal` |

รูปแบบเดียวกันนี้ใช้กับ `Quantity` และ `Money` ด้วย

การดำเนินการเหล่านี้คืนค่าเป็น `Decimal` เพราะผลลัพธ์มีความหมายเชิงมิติ (dimension) ต่างออกไป:

- การคูณราคากับราคาได้ "ราคายกกำลังสอง" ไม่ใช่ราคา
- การหาร quantity ด้วย quantity ได้อัตราส่วนที่ไม่มีหน่วย ไม่ใช่ quantity

การคืนค่าเป็น `Decimal` ทำให้การเปลี่ยนหน่วยชัดเจน และป้องกันการตีความผลลัพธ์ผิดว่าเป็นค่าที่มีหน่วยเดิม

> หมายเหตุ: ต้นฉบับมีส่วนเรื่องตัวดำเนินการแบบ unary และการดำเนินการข้าม type ที่อธิบายตามกฎของ Python
> จึงตัดออก

## การจัดการ precision {#precision-handling}

`Price` และ `Quantity` เก็บฟิลด์ `precision` แบบชัดเจนและเปลี่ยนแปลงไม่ได้ ซึ่งระบุจำนวนตำแหน่งทศนิยม
`Money` ใช้ precision ของสกุลเงินสำหรับการปัดเศษเริ่มต้นและเป็น precision ขั้นต่ำในการแสดงผล ส่วน
constructor แบบ exact ของมันสามารถเก็บจำนวนที่ละเอียดกว่านั้นได้โดยไม่ต้องเปลี่ยนสกุลเงิน type เหล่านี้ไม่มี
precision แบบ "ไม่ระบุ"

### การแทนค่าแบบ fixed-point {#fixed-point-representation}

value type ใช้จำนวนเต็มที่ถูก scale แทนเลขทศนิยมแบบ floating-point โดย scale ส่วนกลางคือ 10^9 ในโหมดมาตรฐาน
และ 10^16 ในโหมด high-precision ค่า DeFi ที่รองรับ precision สูงกว่านั้นจะใช้ precision ของตัวเองเป็น scale
สำหรับ `Price` และ `Quantity` ค่า precision ที่ระบุจะควบคุมรูปแบบการแสดงผลและการ serialize
`Money` แสดงผลอย่างน้อยตาม precision ของสกุลเงินและคงตัวเลขที่ละเอียดกว่าซึ่งเก็บไว้

**Precision และ identity**: ราคาสองค่าที่มีค่าทศนิยมเท่ากันแต่ precision ต่างกันถือว่าเท่ากัน
precision ควบคุมการแสดงผล ไม่ใช่ identity ฟิลด์ `precision` กำหนดรูปแบบของ string และจำนวน
ตำแหน่งทศนิยมที่แสดง ส่วนความเท่ากันอิงจากค่าตัวเลขที่อยู่เบื้องหลัง

**การ serialize market data**: เมื่อเขียนข้อมูลตลาด (เช่น quote, trade และ order book delta) ลง
Parquet หรือ Arrow metadata ของไฟล์จะเก็บ precision ไว้ ทำให้ฝั่งที่อ่านสามารถ decode ค่าได้
ถูกต้อง ค่า market data ทั้งหมดในไฟล์เดียวกันต้องมี precision เดียวกัน

:::warning
หาก venue เปลี่ยน tick size ของ instrument (ซึ่งทำให้ precision เปลี่ยนไปด้วย) ไฟล์ข้อมูลที่เขียนก่อนและหลัง
การเปลี่ยนแปลงจะมี metadata ของ precision ต่างกัน อย่ารวมไฟล์เหล่านั้นเป็นไฟล์เดียว
:::

ดูว่า precision ระดับ instrument จำกัดราคาและ quantity ที่ใช้ได้อย่างไรในส่วน
[Precision](instruments/index.md#precision) ของคู่มือ Instruments

### Precision ในการคำนวณ {#arithmetic-precision}

เมื่อคำนวณระหว่างค่าที่มี precision ต่างกัน ผลลัพธ์จะใช้ precision สูงสุดของ operand

## ข้อจำกัดเฉพาะของแต่ละ type {#type-specific-constraints}

### Quantity {#quantity}

`Quantity` แทนจำนวนที่ไม่ติดลบ การพยายามสร้าง quantity ที่ติดลบ
หรือลบ quantity ที่มากกว่าออกจาก quantity ที่น้อยกว่าจะทำให้เกิด error

### Money {#money}

ค่า `Money` มีสกุลเงินกำกับอยู่ การบวกและการลบระหว่างค่า `Money`
ต้องใช้ **สกุลเงินเดียวกัน** มิฉะนั้นจะเกิด error

ใช้ `Money.from_decimal_exact` หรือ `Money.from_str_exact` เพื่อเก็บจำนวนที่ละเอียดกว่า precision ของสกุลเงิน

constructor แบบ exact จะปฏิเสธการสูญเสีย precision และจำนวนที่อยู่นอกช่วง USD รองรับทศนิยม 9 ตำแหน่งในโหมด
มาตรฐาน และ 16 ตำแหน่งในโหมด high-precision สกุลเงิน DeFi จะใช้ scale ของสกุลเงินนั้นเมื่อสูงกว่า
ส่วน constructor ปกติ, `Money.from_str` และ `Money.from_decimal` จะปัดเศษให้เท่ากับ precision ของสกุลเงิน
การ serialize เป็น string จะคงตัวเลขที่ละเอียดกว่าซึ่งเก็บไว้ และการ deserialize จะคืนค่าเหล่านั้นโดยไม่ปัดตาม
สกุลเงิน

## รูปแบบการใช้งานทั่วไป {#common-patterns}

### การแปลงเป็น type อื่น {#converting-to-other-types}

value type มี method สำหรับแปลงค่า เช่น `as_decimal()` (คง precision ไว้) และ `as_double()` (แปลงเป็น float)

การแปลงเป็น float รองรับ precision สูงสุด 16 สำหรับ precision 17 หรือ 18 จะเกิด error
แทนที่จะปัดตัวเลขส่วนเกินทิ้ง การแปลงที่ precision 16 หรือต่ำกว่ายังอาจมีการปัดเศษได้ เพราะ float เก็บตัวเลข
นัยสำคัญได้ประมาณ 16 หลัก ใช้ `as_decimal()` เพื่อหลีกเลี่ยงการแปลงผ่าน binary float การแปลงนี้ใช้ `Decimal`
ของ Rust ซึ่งมี mantissa ขนาด 96 บิต จึงอาจสูญเสียตัวเลขหลังจุดทศนิยมได้เมื่อค่ามีตัวเลขนัยสำคัญมากเกินไป
การ serialize เป็น string จะคงจำนวน `Money` ที่เก็บไว้ครบถ้วน

### การสร้างจาก string {#creating-from-strings}

value type สามารถ parse จากรูปแบบ string ได้ เช่น `"100.5"` สำหรับ quantity, `"99.95"` สำหรับราคา
และ `"1000.00 USD"` สำหรับ money
