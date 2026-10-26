# ราคา fill และการ matching

matching engine ของ backtest ถือว่าข้อมูล order book และ trade ที่บันทึกไว้เป็นข้อมูล **immutable** (แก้ไขไม่ได้):
fill จำลองจะไม่แก้ไข book ในอดีต วิธีนี้ทำให้ตลาดที่ replay ยังคงเหมือนต้นฉบับ แต่ก็ต้องกำหนดสมมติฐานให้ชัดเจนว่า
liquidity ที่แสดงอยู่ชุดเดียวกันสามารถ fill order จำลองได้มากกว่าหนึ่ง order หรือไม่

engine มีตัวควบคุมสองตัวสำหรับสมมติฐานเหล่านี้:

- `liquidity_consumption=True` ติดตามขนาดที่แสดงซึ่งถูกใช้ไปแล้วในแต่ละระดับราคา
- การกำหนด `random_seed` ของ fill model แบบคงที่ทำให้การตัดสินใจเชิงความน่าจะเป็นของ model นั้นทำซ้ำได้
  โดยไม่ได้กำหนดความสุ่มหรือลำดับการ execute นอกเหนือจาก model นั้น

## การกำหนดราคา fill {#fill-price-determination}

ราคา fill ขึ้นอยู่กับประเภท order, liquidity side, ประเภท book และสภาวะตลาดที่ทำให้เกิดการ match

### Book แบบ L2 และ L3 {#l2-and-l3-books}

เมื่อมีข้อมูล depth order แบบ market จะไล่กินระดับราคาใน book ที่ cross กัน order แบบ limit จะได้ราคาจาก book
ที่ cross เมื่อทำหน้าที่เป็น taker และใช้ราคา limit ของตัวเองเมื่อทำหน้าที่เป็น maker

| ประเภท order            | พฤติกรรมการ fill                                                |
| ---------------------- | ------------------------------------------------------------- |
| `MARKET`               | ไล่กินระดับราคาที่ cross ในฐานะ taker                              |
| `MARKET_TO_LIMIT`      | fill ระดับแรกที่ execute ได้ แล้วพักส่วนที่เหลือไว้ที่ราคานั้น               |
| `LIMIT`                | ใช้ระดับที่ cross เมื่อเป็น taker หรือใช้ราคา limit เมื่อเป็น maker        |
| `STOP_MARKET`          | ไล่กินระดับที่ cross หลังถูก trigger                                 |
| `STOP_LIMIT`           | ใช้กฎแบบ limit หลังถูก trigger                                     |
| `MARKET_IF_TOUCHED`    | ไล่กินระดับที่ cross หลังถูก trigger                                 |
| `LIMIT_IF_TOUCHED`     | ใช้กฎแบบ limit หลังถูก trigger                                     |
| `TRAILING_STOP_MARKET` | ไล่กินระดับที่ cross หลัง activate และถูก trigger                     |
| `TRAILING_STOP_LIMIT`  | ใช้กฎแบบ limit หลัง activate และถูก trigger                         |

order ที่ใช้ depth อาจ fill เพียงบางส่วนได้ เมื่อขนาดที่ cross อยู่น้อยกว่าปริมาณที่เหลือของ order

### Book แบบ L1 {#l1-books}

เมื่อใช้ book แบบ L1 ตลาดที่บันทึกไว้จะมีเพียง bid และ ask ที่ดีที่สุด:

| กลุ่ม order                                                               | พฤติกรรมการ fill                                                                   |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------------- |
| `MARKET`, `MARKET_IF_TOUCHED`, `STOP_MARKET` และ `TRAILING_STOP_MARKET`  | ใช้กฎราคาตลาดหรือราคา trigger แล้ว fill ส่วนที่เหลือที่ราคาแย่ลงหนึ่ง tick              |
| `MARKET_TO_LIMIT`                                                        | ใช้ quote ฝั่งตรงข้ามที่ดีที่สุด แล้วพักส่วนที่เหลือไว้ที่ราคา fill แรก                      |
| Limit-style taker                                                        | ใช้ quote ที่ cross ที่ดีที่สุด โดยไม่เกินราคา limit                                  |
| Limit-style maker                                                        | ใช้ราคา limit เมื่อถูก match จาก trade หรือการเคลื่อนของตลาด                         |

หลังจาก order แบบ market ที่เข้าเงื่อนไขใช้ขนาด L1 ที่แสดงอยู่หมดแล้ว engine จะ fill ส่วนที่เหลือที่ราคาแย่ลงหนึ่ง tick
เว้นแต่ราคานั้นจะ:

- ข้ามขอบเขต price protection ที่ตั้งไว้
- ข้าม `min_price` หรือ `max_price` ของ instrument
- เป็นศูนย์หรือติดลบ ในกรณีที่ instrument ไม่อนุญาต

**กฎส่วนที่เหลือแบบ deterministic** นี้แยกจาก slippage เชิงความน่าจะเป็นของ fill model

การ matching ที่ขับเคลื่อนด้วย trade มีกฎเพิ่มอีกหนึ่งข้อ หาก trade เป็นหลักฐานการ fill ที่ราคาซึ่งไม่มีอยู่ใน book
engine จะ fill ที่ราคา limit ของ order และจำกัดปริมาณของ fill นั้นไม่ให้เกินขนาดของ trade ดู
[การ execute จาก trade](trade-execution.md#trade-driven-matching)

fill model สามารถเปลี่ยนราคาหรือสร้าง depth book สังเคราะห์ได้ ดู [fill models](fill-models.md)

### การ fill ของ market order ที่ถูก trigger เมื่อใช้ bar {#triggered-market-order-fills-with-bars}

การ execute ด้วย bar จะแยกแยะระหว่าง gap กับการเคลื่อนไหวภายใน bar สำหรับ order แบบ `STOP_MARKET`,
`MARKET_IF_TOUCHED` และ `TRAILING_STOP_MARKET`

หาก bar เปิดเลยราคา trigger ไปแล้ว order จะ fill ที่ราคาตลาดจำลอง ตัวอย่างเช่น SELL stop ที่ 100
อาจ fill ที่ 90 เมื่อ bar ถัดไปเปิดที่ 90

หาก bar เปิดก่อนถึงราคา trigger แล้ว high, low หรือ close ภายหลังเคลื่อนผ่านราคานั้น engine จะใช้ราคา trigger
ตัวอย่างเช่น:

1. SELL stop มีราคา trigger ที่ 100
1. bar เปิดที่ 102
1. low ลงไปถึง 98
1. order fill ที่ 100

กฎนี้สมมติว่าราคาเคลื่อนผ่าน trigger อย่างต่อเนื่อง หาก strategy ต้องการพฤติกรรมของ gap และเส้นทางราคาที่แม่นยำกว่านี้
ให้ใช้ข้อมูล quote, trade หรือ order book

## Price protection {#price-protection}

**Price protection** จำกัดว่า order แบบ `MARKET` และ `STOP_MARKET` จะไล่กิน book ไปได้ไกลแค่ไหน
กำหนด offset เป็นจำนวน price increment ของ instrument ผ่าน option `price_protection_points` ใน config ของ backtest venue

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง config แบบ Python จึงตัดออก

engine คำนวณขอบเขต ณ เวลาที่ fill:

- BUY: `ask + (points * price_increment)`
- SELL: `bid - (points * price_increment)`

สำหรับ instrument ที่มี price increment 0.01 ค่า 100 points ทำให้ BUY fill ได้สูงสุดไม่เกิน 1.00 เหนือ ask ปัจจุบัน
ระดับราคาที่เกินขอบเขตจะถูกตัดออก ดังนั้น order อาจค้างอยู่ในสถานะ fill บางส่วน

- market order ได้ขอบเขตเมื่อถูกประมวลผล
- stop-market order ได้ขอบเขตเมื่อถูก trigger โดยใช้ bid หรือ ask ณ เวลานั้น

ตั้ง `price_protection_points=0` เพื่อปิด protection

## Order book immutability {#order-book-immutability}

fill จำลองจะไม่ลดขนาดใน book ในอดีต โดยค่าเริ่มต้น การ matching แต่ละรอบสามารถใช้ขนาดที่บันทึกไว้ได้เต็มจำนวน
หากต้องการติดตามการใช้ liquidity ให้เปิด option `liquidity_consumption` ใน config ของ backtest venue

เมื่อตั้ง `liquidity_consumption=True` engine จะบันทึกขนาดเดิมและขนาดที่ถูกใช้ไปของแต่ละระดับราคา
ขนาดที่ใช้ได้คือ `original_size - consumed` เมื่อมี book update ใหม่ที่ระดับนั้น ข้อมูลที่บันทึกจะถูก reset
เป็นขนาดที่แสดงใหม่

### Passive fill บน L1 {#l1-passive-fills}

เมื่อตลาด L1 เคลื่อนผ่าน passive limit:

| `liquidity_consumption` | พฤติกรรมของปริมาณที่เหลือ                                               |
| ----------------------- | --------------------------------------------------------------------- |
| `False`                 | fill order ที่เหลือทั้งหมดที่ราคา limit                                    |
| `True`                  | fill เฉพาะขนาดที่แสดงซึ่งยังไม่ถูกใช้ และปล่อยส่วนที่เหลือเปิดไว้               |

ตัวอย่างเช่น:

1. ask อยู่ที่ 100.10 จำนวน 50 หน่วย
1. BUY LIMIT จำนวน 1,000 หน่วยพักอยู่ที่ 100.05
1. ask ถัดไปคือ 100.00 จำนวน 30 หน่วย
1. เมื่อเปิด consumption จะ fill 30 หน่วยและเหลือเปิดอยู่ 970 หน่วย
1. update ในภายหลังอาจให้ขนาดใหม่เพิ่มเติมได้

### Trade consumption {#trade-consumption}

trade อาจให้ขนาดที่ execute ได้ที่ราคาซึ่งไม่มีใน book ปัจจุบัน เมื่อเปิด consumption fill ที่ขับเคลื่อนด้วย trade
จะแบ่งใช้ขนาดนั้นร่วมกัน แทนที่จะให้แต่ละ order ใช้ trade ได้เต็มจำนวน

สำหรับ book แบบ L2 และ L3 trade ที่เป็นตัว trigger อาจใช้ depth ที่แสดงอยู่ไปแล้ว engine จะหัก volume นั้นออก
ก่อนให้ order ที่ถูก trigger ใช้ระดับราคาที่เหลือ และจะข้ามการปรับนี้เมื่อมี book update ที่ใหม่กว่าซึ่งสะท้อน trade
นั้นอยู่แล้ว

book สังเคราะห์ที่ fill model คืนมาไม่ใช้การติดตาม consumption รายระดับราคา model ต้องแทนสมมติฐานเรื่อง
liquidity ของตัวเองเอง

### ข้อจำกัด {#limitations}

การติดตาม consumption เป็นการประมาณ **ขนาดที่ใช้ได้** ไม่ใช่ **ลำดับความสำคัญของ order** ตั้ง `queue_position=True`
พร้อมข้อมูล book และ trade เพื่อติดตามคิวที่แสดงอยู่ หรือใช้ `prob_fill_on_limit` เพื่อประมาณแบบความน่าจะเป็น

fill ที่ขับเคลื่อนด้วย trade ยังเป็นแบบฉวยโอกาสด้วย: trade print พิสูจน์ได้เพียงว่ามี liquidity อยู่ชั่วขณะ
ไม่ได้พิสูจน์ว่า liquidity นั้นยังคงอยู่หลังจาก trade ที่บันทึกไว้

## ข้อกำหนดเรื่อง precision {#precision-requirements}

เมื่อ precision ที่กำหนดของ instrument ไม่เกิน `FIXED_PRECISION` ของ build matching engine จะยอมรับ precision
ที่ต่ำกว่าหรือเท่ากันใน order ใหม่และการอัปเดต order ตัวอย่างเช่น quantity `1` ใช้ได้กับ size precision 3
และ price `100` ใช้ได้กับ price precision 2

สำหรับ instrument เหล่านี้ precision ที่สูงกว่าจะถูก reject แม้ทศนิยมส่วนเกินจะเป็นศูนย์ก็ตาม:
`1.0000` ใช้ไม่ได้กับ size precision 3

fill แบบเศษส่วนและการลดปริมาณจะคงค่าที่แน่นอนไว้ order ที่มี quantity `1` ซึ่ง fill บางส่วน `0.500`
จะรายงานว่า fill แล้ว `0.500` และเหลือ `0.500`

build สำหรับ DeFi ใช้ precision ของ instrument ที่สูงกว่า `FIXED_PRECISION` ได้ สำหรับ instrument เหล่านี้
field ของ order ที่ถูกตรวจสอบตามรายการด้านล่างต้องมี **precision ตรงพอดี**: ทั้ง precision 17 และ precision 0
จะถูก reject เมื่อ instrument ต้องการ 18 ส่วน field ของ market data ที่ระบุไว้ต้องตรงกับ `price_precision`
และ `size_precision` ที่กำหนดไว้ของ instrument พอดี

ผลของ precision ที่ไม่ถูกต้องขึ้นอยู่กับว่าเข้ามาที่ matching engine ทางใด:

| Input            | Field ที่ตรวจสอบ                                       | ผลเมื่อ precision ไม่ถูกต้อง                                      |
| ---------------- | ---------------------------------------------------- | ------------------------------------------------------------ |
| `QuoteTick`      | ราคาและขนาดของ bid และ ask                              | log warning แล้วข้าม tick นั้น                                   |
| `TradeTick`      | ราคาและขนาด                                           | log warning แล้วข้าม tick นั้น                                   |
| `Bar` ที่ execute ได้ | open, high, low, close และ volume                     | log warning แล้วข้าม bar นั้น                                    |
| order ใหม่         | quantity, display quantity, price และ trigger price   | reject order                                                 |
| การอัปเดต order    | quantity, price และ trigger price                     | reject การแก้ไข                                               |
| fill ที่สร้างขึ้น      | ราคาและปริมาณของ fill                                   | normalize เมื่อเข้ากันได้ มิฉะนั้น warn แล้วข้าม fill นั้น              |

engine จะ log error หลังจาก **precision ของ market data ไม่ตรงติดต่อกัน 20 ครั้ง** เมื่อตั้ง
`shutdown_on_error=True` error นั้นสามารถร้องขอให้ปิด backtest ตามปกติได้ quote, trade หรือ bar ที่ execute ได้
ซึ่งถูกต้องจะ reset ตัวนับการไม่ตรงติดต่อกัน

**Bar volume**: `Bar.volume` ต้องใช้หน่วยปริมาณและ size precision ของ instrument ให้แปลง field quote-volume
เฉพาะของผู้ให้บริการข้อมูลก่อนสร้าง bar

ใช้ factory method ของ instrument (`make_price` และ `make_qty`) เพื่อสร้างค่าที่เข้ากันได้

นอกจากนี้ให้ตรวจสอบว่านิยาม instrument ตรงกับแหล่งข้อมูล และ loader ที่เขียนเองยังคง precision ของแหล่งข้อมูลไว้
