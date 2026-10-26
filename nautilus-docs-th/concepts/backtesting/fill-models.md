# Fill model

ข้อมูลย้อนหลังไม่สามารถบอกได้ว่า order จำลองจะมีปฏิสัมพันธ์กับผู้เล่นอื่นในตลาดอย่างไร **fill model**
จึงเป็นตัวควบคุมสมมติฐานที่ NautilusTrader ใช้ในเรื่องสิทธิ์การ fill ของ limit order, slippage หนึ่ง tick
และสภาพคล่องสังเคราะห์ (ถ้าเปิดใช้)

## พฤติกรรมตามประเภทของ book {#behavior-by-book-type}

เมื่อใช้ข้อมูล L2 หรือ L3, book ที่บันทึกไว้จะเป็นตัวให้ระดับราคาและขนาด matching engine จะไล่ไปตามระดับเหล่านั้น
และ `prob_fill_on_limit` ใช้จำลองได้ว่า limit order ที่ราคาถูกแตะจะ fill หรือไม่ ส่วน `prob_slippage` ไม่มีผล
เพราะตัว book เองเป็นตัวกำหนด price impact

เมื่อใช้ book แบบ L1 รวมถึง book ที่อัปเดตจาก quote, trade หรือ bar:

- `prob_fill_on_limit` ควบคุมว่า limit order จะ fill หรือไม่เมื่อราคาของมันถูกแตะ
- `prob_slippage` จะถูกประเมินทุกครั้งที่มี fill ไม่ว่าจะเป็น order ประเภทใดหรือฝั่ง liquidity ใด
- หากการสุ่ม slippage สำเร็จ ราคา fill จะขยับไปหนึ่ง tick ในทิศทางที่เสียเปรียบต่อ order
- Model อาจสร้าง L2 book สังเคราะห์เพื่อแทนสภาพคล่องที่อยู่ถัดจาก best bid และ best ask

ตัวอย่างเช่น เมื่อ `prob_slippage=0.5` fill ฝั่ง BUY แต่ละครั้งมีโอกาส 50% ที่ราคาจะสูงขึ้นหนึ่ง tick
ให้ตั้ง `random_seed` เมื่อต้องการให้ run ทำซ้ำผลการสุ่มของ model ได้เหมือนเดิม

หาก venue ไม่ได้ระบุ fill model จะใช้ `DefaultFillModel` ด้วยค่า:

- `prob_fill_on_limit=1.0`: limit order ที่ราคาถูกแตะมีสิทธิ์ fill
- `prob_slippage=0.0`: fill แบบ L1 จะไม่มี slippage หนึ่ง tick แบบสุ่ม

ค่าเริ่มต้นเหล่านี้ไม่ได้ปิดกฎ residual-fill ที่แยกต่างหากของ matching engine สำหรับ order แบบ market-style
ที่มีสิทธิ์ fill

:::warning
ข้อมูล order book ย้อนหลังจะไม่เปลี่ยนแปลงหลังเกิด fill เมื่อ `liquidity_consumption=False` ขนาดที่แสดงอยู่
เดียวกันอาจรองรับ order จำลองได้มากกว่าหนึ่งตัวในรอบเดียวกัน ให้ตั้ง `liquidity_consumption=True` เพื่อติดตาม
ขนาดที่ถูกใช้ไปในแต่ละระดับจนกว่าจะมีข้อมูลใหม่เข้ามา ดู
[order book immutability](fill-prices-and-matching.md#order-book-immutability)
:::

## Model ที่มีให้ใช้ {#available-models}

| Model                        | พฤติกรรมด้าน liquidity                                      |
| ---------------------------- | --------------------------------------------------------- |
| `DefaultFillModel`           | ใช้ book ที่บันทึกไว้ของ matching engine                      |
| `BestPriceFillModel`         | ให้ขนาดไม่จำกัดที่ best bid และ best ask                       |
| `OneTickSlippageFillModel`   | ให้ขนาดไม่จำกัดที่ราคาถัดจาก best price ไปหนึ่ง tick              |
| `ProbabilisticFillModel`     | เลือก best price หรือแย่กว่าหนึ่ง tick                         |
| `TwoTierFillModel`           | วาง 10 หน่วยที่ best แล้ววางส่วนที่เหลือที่ราคาแย่กว่าหนึ่ง tick    |
| `ThreeTierFillModel`         | วาง 50, 30 และ 20 หน่วยกระจายไปสามระดับ                     |
| `LimitOrderPartialFillModel` | วาง 5 หน่วยที่ best แล้ววางส่วนที่เหลือที่ราคาแย่กว่าหนึ่ง tick     |
| `SizeAwareFillModel`         | เปลี่ยนรูปทรงของ book เมื่อขนาด order ถึง 10 หน่วย              |
| `CompetitionAwareFillModel`  | เปิดให้เห็นสัดส่วน (ปรับได้) ของ 1,000 หน่วยที่ best            |
| `VolumeSensitiveFillModel`   | วาง 25% ของ volume ภายในของ model ไว้ที่ best                 |
| `MarketHoursFillModel`       | ใช้ spread สังเคราะห์แบบปกติ หรือกว้างขึ้นหนึ่ง tick              |

`BestPriceFillModel` ยัง fill limit order ที่อยู่ที่หรือภายใน spread ด้วยราคาของ order เองด้วย

**ขนาดตาม instrument**: ขนาดของแต่ละ tier เป็นค่าคงที่ของ model ในหน่วยปริมาณของ instrument
ควรตรวจสอบว่าเหมาะกับสเกลของ instrument ก่อนใช้ model แบบ tier

Book สังเคราะห์จะไม่รวมระดับใดที่อยู่นอก `min_price` และ `max_price` ของ instrument และจะไม่รวมราคาที่เป็นศูนย์
หรือติดลบเมื่อ instrument ไม่อนุญาตราคาเหล่านั้น

`CompetitionAwareFillModel` รับค่า `liquidity_factor` ในช่วง `[0.0, 1.0]` ค่าเริ่มต้นคือ `0.3`
และจะ clamp ขนาดที่คำนวณได้ให้มีอย่างน้อยหนึ่งหน่วยปริมาณของ instrument

## Configuration {#configuration}

กำหนด fill model ให้ venue ผ่าน option `fill_model` ของ config ของ backtest venue โดย model แบบ synthetic book
ใช้พารามิเตอร์ constructor ชุดเดียวกัน (`prob_fill_on_limit`, `prob_slippage`, `random_seed`)

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## พารามิเตอร์เชิงความน่าจะเป็น {#probabilistic-parameters}

### `prob_fill_on_limit` (ค่าเริ่มต้น: `1.0`) {#prob_fill_on_limit-default-10}

ค่านี้ควบคุมว่า limit order จะ fill หรือไม่เมื่อตลาดแตะแต่ไม่ข้ามราคาของมัน:

- `0.0`: ไม่ fill เมื่อแตะเลย
- `0.5`: fill โดยเฉลี่ยครึ่งหนึ่งของการแตะที่มีสิทธิ์
- `1.0`: fill ทุกครั้งที่แตะ

การที่ราคาข้าม limit price เป็นเงื่อนไข matching อีกข้อหนึ่งที่แยกต่างหาก หากต้องการติดตาม volume ในคิวอย่างชัดเจน
ดู [queue position tracking](trade-execution.md#queue-position-tracking)

### `prob_slippage` (ค่าเริ่มต้น: `0.0`) {#prob_slippage-default-00}

สำหรับ book แบบ L1 ค่านี้ควบคุมการขยับราคาไปในทางเสียเปรียบหนึ่ง tick ในแต่ละ fill:

- `0.0`: ไม่เพิ่ม slippage จาก model เลย
- `0.5`: เพิ่มหนึ่ง tick โดยเฉลี่ยครึ่งหนึ่งของ fill ทั้งหมด
- `1.0`: เพิ่มหนึ่ง tick ในทุก fill

การสุ่มนี้มีผลทั้งกับ fill ฝั่ง maker และ taker แต่ไม่มีผลกับ book แบบ L2 หรือ L3 หาก slippage จะทำให้ราคาข้าม
`min_price` หรือ `max_price` ของ instrument จะคงราคา fill เดิมไว้ และจะคงราคาเดิมไว้เช่นกันหากจะทำให้ราคาเป็นศูนย์
หรือติดลบซึ่ง instrument ไม่อนุญาต

## Order book สังเคราะห์ {#synthetic-order-books}

ก่อนตัดสินผลการ fill, matching engine จะขอ order book สังเคราะห์ (ถ้ามี) จาก model หาก model คืน book มา
engine จะ fill กับระดับราคาใน book นั้น หากคืน `None` engine จะใช้ book ที่บันทึกไว้

:::warning[การใช้ liquidity ของ book สังเคราะห์]
การติดตาม `liquidity_consumption` รายระดับไม่มีผลกับ book ที่ model สังเคราะห์ขึ้น custom model ต้องจัดการ
พฤติกรรมการใช้ liquidity ที่ต้องการเองใน book ที่มันคืนกลับมา
:::
