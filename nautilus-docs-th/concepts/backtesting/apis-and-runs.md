---
title: "Backtest API และการรันซ้ำ"
---

NautilusTrader มี API ระดับ **low-level** คือ `BacktestEngine` สำหรับควบคุมโดยตรง และ API ระดับ **high-level**
คือ `BacktestNode` สำหรับการรันที่อิง catalog และกำหนดค่าผ่าน config ได้

## การเลือกระดับ API [#choosing-an-api-level]

ใช้ low-level API เมื่อ:

- ข้อมูลใส่ใน memory ได้พอ หรือคุณจะ stream เป็น batch เอง
- ต้องการโหลดข้อมูลจาก format อื่นที่ไม่ใช่ Parquet catalog ของ Nautilus
- ต้องการควบคุม venue, instrument, actor, strategy หรือ execution algorithm โดยตรง
- ต้องการรันข้อมูลชุดเดิมที่โหลดไว้ซ้ำ โดยเปลี่ยนเฉพาะบาง component

ใช้ high-level API เมื่อ:

- ข้อมูลอยู่ใน `ParquetDataCatalog`
- ข้อมูลต้องโหลดเป็น chunk โดยอัตโนมัติ
- ต้องการให้ config object เดียวอธิบายและระบุตัวตนของการรันที่อิง catalog
- ต้องการให้การรันที่เป็นอิสระต่อกันแต่ละครั้งใช้ engine ใหม่

## Low-level API [#low-level-api]

Low-level API มี `BacktestEngine` เป็นศูนย์กลาง สร้าง engine ด้วย `BacktestEngineConfig` แล้วเพิ่ม venue,
instrument, component และข้อมูล (`add_venue`, `add_instrument`, `add_strategy`, `add_data`) ก่อนเรียก `run()`

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก ดูตัวอย่างฝั่ง Rust ได้ที่[การรัน backtest ด้วย Rust](../../how_to/run_rust_backtest.md)

Instrument ID แต่ละตัวลงทะเบียนได้ครั้งเดียว การเพิ่มนิยามเดิมซ้ำจะไม่มีผลอะไร (no-op) และคง matching engine
เดิมไว้ ดังนั้น resting order, สถานะ book และ ID ที่สร้างไว้จะยังอยู่ แต่การเพิ่มนิยามที่ต่างออกไปให้กับ ID
ที่ลงทะเบียนแล้วจะเกิด error และ engine จะไม่เปลี่ยนแปลง การเปรียบเทียบนี้ไม่สนใจ `ts_event` และ `ts_init`

### การโหลดข้อมูล [#loading-data]

การเรียก `add_data()` แต่ละครั้งจะคัดลอก input ไปเป็น stream อิสระหนึ่งชุด engine จะเรียงแต่ละ stream ตาม
replay timestamp แล้ว merge ทุก stream ตามลำดับเวลาระหว่างรัน การเพิ่มข้อมูลทีละ batch ต่อ instrument
จึงไม่ทำให้ต้อง sort list สะสมซ้ำไปซ้ำมา

**ความพร้อมในการรัน**: ให้คงค่าเริ่มต้น `sort=True` ไว้ เว้นแต่ workflow แบบ batch จะจัดการความพร้อมเองโดยตั้งใจ
การเรียกด้วย `sort=False` จะทำให้ engine อยู่ในสถานะยังไม่พร้อมรัน ต้องเรียก `sort_data()` ก่อน `run()`
เว้นแต่จะมีการเรียก `add_data(..., sort=True)` ภายหลังซึ่งคืนสถานะพร้อมให้แล้ว

`sort_data()` จะทำเครื่องหมายว่า stream ที่เรียงลำดับแยกกันเหล่านั้นพร้อมสำหรับ replay และเรียกซ้ำได้อย่างปลอดภัย

**ความเป็นเจ้าของ input**: การล้างหรือแก้ไข collection ต้นฉบับหลังเรียก `add_data()` ไม่ทำให้ stream ที่โหลดแล้วเปลี่ยน

### การ stream batch เอง [#streaming-batches-manually]

ใช้ **streaming mode** เมื่อชุดข้อมูลทั้งหมดใส่ใน memory ไม่พอ: เพิ่ม strategy ก่อน จากนั้นวนลูปแต่ละ batch โดยเรียก
`add_data(batch)` ตามด้วย `run(streaming=True)` และ `clear_data()` แล้วเรียก `end()` เมื่อจบ

`run(streaming=True)` จะหยุดพักเมื่อข้อมูลปัจจุบันหมด โดยไม่ finalize trader และไม่เลื่อน timer เกิน batch นั้น
ให้เรียก `end()` หลัง batch สุดท้าย ซึ่งจะ flush timer ไปถึงขอบเขตของการรันครั้งสุดท้าย เรียก stop handler
และสร้างผลลัพธ์สุดท้าย

Low-level API ไม่มี method `add_data_iterator()` แบบ generator ส่วน `BacktestNode` มีการแบ่ง chunk จาก catalog
ให้อัตโนมัติ ผู้ที่ใช้ engine โดยตรงให้ stream ด้วยลูปข้างต้น

## High-level API [#high-level-api]

High-level API มี `BacktestNode` เป็นศูนย์กลาง `BacktestRunConfig` แต่ละตัวประกอบด้วย:

- `BacktestVenueConfig` อย่างน้อยหนึ่งตัว
- `BacktestDataConfig` อย่างน้อยหนึ่งตัว
- `BacktestEngineConfig` (ไม่บังคับ)
- ค่าตั้งเพิ่มเติม (ไม่บังคับ) เช่น chunk size, ขอบเขตเวลา, การจัดการ exception และการ dispose

ต้อง build node ก่อน แล้วจึงเพิ่ม strategy ผ่าน method เฉพาะของแต่ละการรัน (เช่น `node.build()` ตามด้วย
`add_strategy_from_config(config.id, strategy_config)` แล้วจึง `run()`)

Fee เป็นส่วนหนึ่งของ config ของ venue ไม่ใช่ของ instrument venue ทุกตัวต้องมี `fee_model` ที่ระบุชัดเจน
รวมถึงกรณีต้องการ baseline ที่ไม่มีค่าธรรมเนียมก็ต้องระบุ model แบบ rate เป็นศูนย์อย่างชัดเจน (เช่น
`MakerTakerFeeModel` ที่ maker/taker rate เป็น 0) สอง account สามารถเทรด instrument เดียวกันภายใต้ตาราง
ค่าธรรมเนียมต่างกันได้โดยไม่ต้องเปลี่ยนนิยาม instrument

ตั้ง `chunk_size` เป็นค่าในช่วง `[1, 1_000_000]` เพื่อเปิด streaming หรือปล่อยเป็น `None` เพื่อโหลดข้อมูลทั้งหมดในครั้งเดียว

`BacktestNode` ยังมี method สำหรับเพิ่ม actor และ strategy ที่มีมาในตัวเข้าไปในการรันที่ build แล้วด้วย

### การ replay order book delta [#order-book-delta-replay]

โดยค่าเริ่มต้น `BacktestNode` จะ replay ข้อมูล `OrderBookDelta` จาก catalog เป็น batch แบบ `OrderBookDeltas`
ทั้งการรันแบบครั้งเดียวและแบบ streaming โดยจัดกลุ่ม delta ของแต่ละ instrument ตามลำดับใน catalog และปิด batch
ที่ delta ที่มี `F_LAST` เพื่อให้ batch หนึ่งเก็บ book event ที่สมบูรณ์หนึ่ง event

แต่ละ batch ใช้ flags, sequence และ timestamp จาก delta ตัวปิด และ replay ที่ `ts_init` ของ delta ตัวนั้น

การทำ batch เปลี่ยนวิธีที่ backtest ประมวลผล book event ดังนี้:

- **Matching**: matching engine จะ apply event ทั้งก้อนก่อนจับคู่ order ดังนั้น order จะไม่ fill กับสถานะ book
  ที่มีอยู่เพียงระหว่างกลาง event ผลการ fill จึงอาจต่างจากการ replay ทีละ delta
- **Callback**: strategy จะได้รับแต่ละ event ใน callback `on_book_deltas` ครั้งเดียว แม้ `buffer_deltas`
  ของ data engine จะปิดอยู่ก็ตาม
- **Chunk size**: ในการรันแบบ streaming `chunk_size` นับแต่ละ batch เป็น replay item หนึ่งชิ้น ดังนั้น chunk
  หนึ่งอาจมีจำนวนแถวมากกว่า `chunk_size`

ทุก event ต้องจบด้วย delta ที่มี `F_LAST` หาก query จบลงโดยที่ delta ของ instrument ใดยังรอ delta ตัวปิดอยู่
การรันจะล้มเหลวพร้อม error ที่ระบุชื่อ instrument นั้น

ขอบเขตเวลาและ `filter_expr` คัดเลือกเป็นรายแถว จึงควรเลือกให้ event ยังสมบูรณ์ เพราะขอบเขตที่ตัดส่วนต้นของ
event ออกไปจะไม่ถูกตรวจพบ

ตั้ง `batch_deltas=False` ใน `BacktestDataConfig` เพื่อ replay ทีละ delta

## Shutdown เมื่อเกิด error [#shutdown-on-error]

ตั้ง `BacktestEngineConfig.shutdown_on_error=True` เพื่อขอให้ shutdown ตามปกติเมื่อ logger ของ Rust ปล่อย
error record ออกมา

Backtest loop จะรับรู้คำขอ หยุด trader และ engine แล้วคืนผลลัพธ์ที่เก็บได้จนถึงจุดนั้น โดยไม่ abort process

เมื่อใช้ `shutdown_on_error=True`:

- error record จาก logger ของ Rust ยังคงขอ shutdown แม้ component filter หรือ `bypass_logging=True`
  จะกดไม่ให้แสดงผลก็ตาม

Trigger นี้จะ reset เมื่อเริ่มการรัน kernel ครั้งใหม่ สำหรับพฤติกรรมของ `on_stop` ช่วงสุดท้ายและการรอคำสั่งให้เสร็จ
ดู [shutdown semantics](execution-flow.md#shutdown-semantics)

## การรันซ้ำ [#repeated-runs]

`BacktestEngine.reset()` คืนสถานะการเทรดและสถานะของ component ที่โหลดไว้กลับเป็นค่าเริ่มต้น

**สถานะที่ถูกล้าง**:

- Order, position และยอดเงินใน account
- สถานะ runtime ของ component
- ตัวนับและ timestamp ของ engine

**ข้อมูลและการลงทะเบียนที่ยังคงอยู่**:

- ข้อมูลที่เพิ่มผ่าน `add_data()`
- Instrument และ venue ที่ลงทะเบียนไว้ รวมถึง config ของ venue
- Actor, strategy และ execution algorithm ที่ลงทะเบียนไว้

Instrument ยังคงโหลดอยู่ เพราะ config ของ cache สำหรับ backtest โดยค่าเริ่มต้นตั้ง `drop_instruments_on_reset=False`

### ใช้ node ใหม่สำหรับการรันที่เป็นอิสระต่อกัน [#use-fresh-nodes-for-independent-runs]

`BacktestNode` รับ `BacktestRunConfig` หนึ่งตัว หากต้องการรัน config ที่เป็นอิสระต่อกัน ให้สร้างและ dispose
node ทีละตัว: สร้าง node ด้วย config แต่ละตัว เรียก `run()` เก็บผลลัพธ์ แล้วเรียก `dispose()` เสมอแม้จะเกิด error

Build แต่ละ node และลงทะเบียน strategy ก่อน `run()` ในกรณีที่ไม่ได้ส่ง strategy มาจาก controller

### ใช้ข้อมูลที่โหลดไว้ซ้ำสำหรับการรันหลายพารามิเตอร์ [#reuse-loaded-data-for-parameter-runs]

ใช้ `reset()` เมื่อต้องการให้การรันหลายครั้งใช้ข้อมูลและการตั้งค่า venue ที่โหลดไว้ร่วมกัน: เพิ่ม venue, instrument
และข้อมูลครั้งเดียว แล้วสลับ `run()` กับ `reset()` และหากต้องการเปลี่ยน strategy ให้เรียก `clear_strategies()`
แล้ว `add_strategy()` ตัวใหม่ก่อน `run()`

เรียก `clear_strategies()` ก่อนเปลี่ยน strategy instance และใช้ `clear_data()` เฉพาะเมื่อการรันครั้งถัดไปต้องโหลด
ชุดข้อมูลที่ต่างออกไป
