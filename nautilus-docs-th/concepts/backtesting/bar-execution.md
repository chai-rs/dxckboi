---
title: "การ execute ด้วย bar (Bar-Based Execution)"
---

ข้อมูล bar บันทึกราคา open, high, low, close และ volume ของช่วงเวลาหนึ่ง แต่ไม่ได้บันทึกว่าแต่ละราคาเกิดขึ้นเมื่อใดภายในช่วงนั้น
หรือ high เกิดก่อน low หรือไม่ ดังนั้นการ execute ด้วย bar จึงเป็นการจำลองเส้นทางราคาภายใน bar ที่สมเหตุสมผล
ไม่ใช่การสร้าง trade จริงขึ้นมาใหม่

NautilusTrader แปลง execution bar แต่ละอันให้เป็น market update สังเคราะห์สำหรับ order book ระดับ L1
order ที่ค้างอยู่ (resting) จะถูก match ไปตามลำดับ update เหล่านั้นภายใน bar

## ข้อกำหนดเรื่อง timestamp ของ bar [#bar-timestamp-convention]

:::warning
สำหรับการจำลอง execution ค่า timestamp ตอน initialize (`ts_init`) ของแต่ละ bar ต้องแทน
**เวลาปิดของช่วง** เพื่อป้องกันไม่ให้ bar ทั้งแท่งมองเห็นได้ก่อนที่มันจะก่อตัวเสร็จ
:::

timestamp ของ event (`ts_event`) อาจแทนเวลาเปิดหรือเวลาปิดก็ได้ ขึ้นอยู่กับแหล่งข้อมูล:

- สำหรับ bar ที่ประทับเวลาตอนปิด ให้ตั้ง `ts_init` เป็นค่าเดียวกัน
- สำหรับ bar ที่ประทับเวลาตอนเปิด ให้ตั้ง `ts_init = ts_event + interval_ns` เช่น บวก
  `60_000_000_000` นาโนวินาทีสำหรับ bar หนึ่งนาที

หาก adapter มี setting อย่าง `bars_timestamp_on_close=True` ควรใช้ setting นั้น เพื่อให้ข้อมูลที่จัดเก็บ
เป็นไปตามข้อกำหนดที่คาดหวัง

สำหรับ custom data ให้กำหนดค่า `ts_event` และ `ts_init` ให้เรียบร้อยก่อนสร้าง object `Bar`, encode Arrow
record batch, เขียนลง catalog หรือเรียก `add_data()` และควรตรวจผลลัพธ์กับตัวอย่างเล็ก ๆ
ก่อนรัน backtest

## การประมวลผลข้อมูล bar [#processing-bar-data]

การ execute ด้วย bar จะทำงานเฉพาะเมื่อ:

- venue ตั้งค่า `bar_execution=True`
- venue ใช้ `BookType.L1_MBP`
- bar มีแหล่ง aggregation ภายนอก (external)

bar ที่ aggregate ภายใน และ bar ที่ส่งไปยัง venue แบบ L2 หรือ L3 ยังคงถูกส่งถึง strategy ที่ subscribe ไว้ แต่ bar เหล่านี้
จะไม่อัปเดต book ของ matching engine และไม่กระตุ้นการ match

สำหรับแต่ละ bar ที่เข้าเงื่อนไข engine จะ:

1. เลือก bar type ที่ละเอียดที่สุดที่ตั้งค่าไว้สำหรับ instrument นั้น
1. แบ่ง volume ของ bar ออกเป็น update สังเคราะห์สี่ชุด
1. ประมวลผล open, high และ low ตามลำดับที่ตั้งค่าไว้ แล้วตามด้วย close
1. match order หลัง update สังเคราะห์แต่ละครั้ง
1. ส่ง bar ที่สมบูรณ์ไปยัง actor และ strategy

ดังนั้น order ที่ค้างอยู่ตั้งแต่ต้น bar จึงสามารถ fill ที่จุด OHLC ระหว่างทางได้ ส่วน order ที่ submit
จาก `on_bar` จะมาถึงหลังจากประมวลผลครบทั้งสี่จุดของ bar นั้นแล้วเท่านั้น

## การจำลองราคา OHLC [#ohlc-price-simulation]

engine แบ่ง volume ของ bar เท่า ๆ กันให้กับราคาทั้งสี่จุด เศษที่เหลือจะยกให้จุด close เพื่อให้ update สังเคราะห์
รวมกันได้ volume ทั้งหมดเท่าเดิม หาก volume หนึ่งในสี่ต่ำกว่า `size_increment` ขั้นต่ำของ instrument
แต่ละจุดจะใช้ค่า increment ขั้นต่ำแทน

option `bar_adaptive_high_low_ordering` ของ venue ควบคุมเส้นทางราคาภายใน bar:

- เมื่อเป็น `False` (ค่าเริ่มต้น) ทุก bar ใช้ลำดับ `Open -> High -> Low -> Close`
- เมื่อเป็น `True` engine จะไปที่ราคาสุดขั้วที่ใกล้ราคา open มากกว่าก่อน:
  - ถ้า open ใกล้ high มากกว่า จะใช้ `Open -> High -> Low -> Close`
  - ถ้า open ใกล้ low มากกว่า จะใช้ `Open -> Low -> High -> Close`

เส้นทางแบบ adaptive เป็น **heuristic แบบ deterministic** ไม่ใช่การสร้างลำดับ trade จริงขึ้นมาใหม่
ความแม่นยำขึ้นกับตลาด ช่วงเวลา และแหล่งข้อมูล
[การวิเคราะห์เชิงสำรวจของ EUR/USD](https://gist.github.com/stefansimik/d387e1d9ff784a8973feca0cde51e363)
เป็นที่มาของ heuristic ตามระยะห่างนี้ แต่ไม่ได้ยืนยันอัตราความแม่นยำโดยทั่วไป

ระดับราคาที่ไปถึงก่อนเป็นตัวกำหนดว่า order ใดจะ fill ก่อน ดังนั้นเส้นทางราคาจึงมีผลเมื่อ protective stop
และ profit target อยู่ภายใน bar เดียวกัน

การตั้งค่า adaptive ordering ทำที่ระดับ venue

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## จังหวะเวลาการ submit order [#order-submission-timing]

ลำดับ OHLC ของ bar N ทำงานก่อน `on_bar(N)` หากไม่มี latency model order ที่ submit จาก
`on_bar` จะ settle ทันทีกับ book ที่เหลืออยู่ ณ close ของ bar N

latency model จะหน่วงเวลาที่ order มาถึงจริง เมื่อคำสั่งถึง timestamp ที่มาถึงแล้ว
engine สามารถปล่อยคำสั่งออกจาก latency queue ของ venue ได้สองทาง:

- **Market data ของ instrument**: ข้อมูลที่ route ผ่าน exchange สำหรับ instrument ของ order นั้นจะปล่อยคำสั่งออกมา หากมี
  ข้อมูลเป็น bar อย่างเดียวและไม่มี timer event คั่นกลาง bar แรกที่ถึงหรือเลยเวลามาถึงจะ sweep OHLC จนครบ
  ก่อนที่ order จะ settle ดังนั้น order จะเห็น close ของ bar นั้น ส่วน quote หรือ trade tick สามารถปล่อยคำสั่งได้เร็วกว่า
  โดย settle กับสถานะ book ที่ tick เหล่านั้นสร้างขึ้น
- **การ settle แบบไม่มีเงื่อนไข**: จุดต่าง ๆ เช่น timer ของ actor หรือ strategy, การ settle funding rate
  หรือการ drain ตอน shutdown จะปล่อยคำสั่งทั้งหมดที่ถึงกำหนด ณ เวลานั้น

market data ของ instrument อื่นจะไม่ปล่อยคำสั่งที่ถูกหน่วงไว้ให้ไป settle กับสถานะ book ที่เก่าแล้ว
timer สำหรับ snapshot ของ portfolio เพื่อทำ equity curve รายวัน และการ sample ตาม `PortfolioConfig.snapshot_interval_ms`
ก็จะไม่ปล่อยคำสั่งที่ถูกหน่วงไว้ก่อนหน้า

:::warning[การ fill ที่ open ของ bar ถัดไปและ look-ahead]
engine ไม่มีโหมด fill ที่ open ของ bar ถัดไปในตัว strategy สามารถสร้างสัญญาณจาก bar ก่อนหน้าที่ปิดแล้ว
ได้โดยไม่เกิด look-ahead แต่ engine จะประมวลผล open ของ bar ถัดไปก่อนส่ง bar นั้นออกไป หาก
strategy ใช้ open ของ bar ปัจจุบันจาก callback `on_bar` ก็จะเกิด look-ahead เมื่อใช้ข้อมูล bar อย่างเดียว
latency มักทำให้ settle กับสถานะ book ที่เกิดภายหลัง ไม่ใช่ open ถัดไป
:::

## จังหวะเวลาการ aggregate bar ภายใน [#internal-bar-aggregation-timing]

เมื่อ data engine aggregate time bar จาก tick จะมี timer ปิดแต่ละ bar ที่ขอบของช่วงเวลา
ข้อมูลที่มี timestamp ตรงกันพอดีอาจถูกประมวลผลหลัง timer ปิด bar นั้น

ตั้งค่า `time_bars_build_delay` ใน `DataEngineConfig` เพื่อหน่วง timer

ค่านี้มีหน่วยเป็นไมโครวินาที การหน่วงเล็กน้อย เช่น หนึ่งไมโครวินาที ช่วยให้ข้อมูลที่ขอบช่วงมาถึง
ก่อน bar ปิด ค่านี้มีผล **เฉพาะ bar ที่ aggregate ภายใน** เท่านั้น
