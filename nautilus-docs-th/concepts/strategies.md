# Strategies

strategy คือ component ที่ implement method ตามที่ logic ของมันต้องใช้ `Strategy` สร้างต่อยอดจาก `DataActor`
และเพิ่มความสามารถในการจัดการ order

ในฝั่ง Rust ผู้เขียน strategy จะ implement callback ของ `DataActor` ที่ต้องใช้ และใช้ `nautilus_strategy!` เพื่อสร้าง
implementation ของ `Strategy` จากนั้นเรียก facade method เช่น `clock()`, `cache()`, `order()` และ `portfolio()`
บน `self`

`DataActorNative` คือการเข้าถึง runtime wiring และ state ของ actor core แบบ native-only ส่วน `StrategyNative`
เปิดให้เข้าถึง state ของ strategy แบบ borrow เช่น order factory, order manager และ portfolio ให้ import trait
เหล่านี้เฉพาะสำหรับ performance path ภายใน binary เดียวกัน หรือ runtime wiring ภายในเท่านั้น

**ความสามารถ**:

- request ข้อมูลย้อนหลัง (historical data)
- subscribe live data feed
- ตั้ง time alert หรือ timer
- เข้าถึง cache
- เข้าถึง portfolio
- สร้างและจัดการ order และ position

:::tip
อ่านคู่มือ [Actors](actors.md) ก่อนพัฒนา strategy คู่มือนั้นอธิบายพฤติกรรมของการ subscribe, request และ
callback ที่ strategy ได้รับสืบทอดมา
:::

เพิ่ม strategy เข้าระบบ Nautilus ได้ใน [environment context](architecture.md#environment-contexts) ใดก็ได้
strategy จะเริ่มส่ง command และรับ event ตาม logic ของมันทันทีที่ระบบเริ่มทำงาน การรับข้อมูล การจัดการ event
และการจัดการ order รองรับ strategy ทุกประเภท ทั้ง directional, momentum, re-balancing, pairs และ market making

trading strategy ของ Nautilus มีสองส่วนหลัก:

- implementation ของ strategy เอง
- config ของ strategy ซึ่งเป็น *optional*

:::note
source ของ strategy ชุดเดียวกันรันได้ทั้งใน environment แบบ backtest และ live แต่ live execution ยังมีพฤติกรรม
ด้าน venue, transport, timing, persistence, กิจกรรมจากภายนอก และ reconciliation ที่ simulation อาจจำลอง
ไม่ได้ ดู [ความแตกต่างระหว่าง backtest และ live](live.md#backtest-and-live-differences)
:::

ดู method ทั้งหมดที่มีได้ที่ [Rust API reference](https://nautilustrader.io/docs/core-latest/) และ
[Rust](rust.md#strategies)

## Implementation ของ strategy {#strategy-implementation}

จากนั้นคุณ implement handler ตามที่ต้องใช้ เพื่อทำงานตามการเปลี่ยน state และ event

:::warning
ให้ initialize state ธรรมดาตอนสร้าง strategy และทำงานที่ต้องใช้ระบบ (clock, cache, portfolio, order factory)
ใน `on_start()` เพราะส่วนเหล่านี้พร้อมใช้หลังจาก strategy ลงทะเบียนกับ trader แล้วเท่านั้น
:::

### Handler {#handlers}

handler คือ method ของ strategy ที่ทำงานตาม event หรือการเปลี่ยน state method เหล่านี้ขึ้นต้นด้วย `on_*`
implement บางตัวหรือทั้งหมดก็ได้ตามที่ strategy ต้องการ

มี handler หลายตัวสำหรับ event ประเภทใกล้เคียงกัน เพื่อให้คุณควบคุมระดับความละเอียดได้ คุณจะตอบสนอง event
เฉพาะด้วย handler เฉพาะ หรือใช้ handler ทั่วไปสำหรับ event ที่เกี่ยวข้องกันหลายแบบ (ใช้ logic แบบ switch ทั่วไป)
ก็ได้ ระบบจะเรียก handler ตามลำดับจาก **เฉพาะเจาะจงที่สุดไปทั่วไปที่สุด**

handler ของ data, order และ position ที่ subscribe ไว้จะถูก dispatch เฉพาะเมื่อ strategy อยู่ในสถานะ `RUNNING`
message ที่มาถึงในสถานะอื่นจะถูก log แต่ไม่ส่งต่อให้ handler ส่วน response ของ request ไม่ถูกจำกัดแบบนี้: handler
`on_historical_*` ยังทำงานแม้ response จะมาถึงหลังจาก strategy หยุดแล้ว

#### Action ตาม state {#stateful-actions}

การเปลี่ยน state ใน lifecycle จะ trigger handler กลุ่มนี้ คำแนะนำ:

- ใช้ method `on_start` เพื่อ initialize strategy (เช่น ดึง instrument, subscribe ข้อมูล)
- ใช้ method `on_stop` สำหรับงาน cleanup (เช่น cancel order ที่เปิดอยู่, ปิด position ที่เปิดอยู่, unsubscribe ข้อมูล)

#### การจัดการข้อมูล {#data-handling}

handler กลุ่มนี้รับการอัปเดตข้อมูล ทั้ง market data ที่มีในตัวและ custom data ที่ผู้ใช้นิยามเอง ดูรายการ handler
ฝั่ง Rust ได้ที่ [Rust: handler methods](rust.md#handler-methods)

update ที่ subscribe ไว้และ response ของ request จะไปถึง handler ต่างกัน ดู
[Actors: callback handlers](actors.md#callback-handlers) สำหรับการจับคู่ระหว่าง operation กับ handler

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

#### การจัดการ order {#order-management}

handler กลุ่มนี้รับ event ที่เกี่ยวกับ order message ประเภท `OrderEvent` จะถูกส่งให้ handler ตามลำดับต่อไปนี้:

1. handler เฉพาะ (เช่น `on_order_accepted`, `on_order_rejected` เป็นต้น)
2. `on_order_event(...)`

#### การจัดการ position {#position-management}

handler กลุ่มนี้รับ event ที่เกี่ยวกับ position message ประเภท `PositionEvent` จะถูกส่งให้ handler ตามลำดับต่อไปนี้:

1. handler เฉพาะ (เช่น `on_position_opened`, `on_position_changed` เป็นต้น)
2. `on_position_event(...)`

ใช้ `on_time_event()` สำหรับ timer event, `on_order_event()` สำหรับ order event แบบรวม และ
`on_position_event()` สำหรับ position event แบบรวม

#### ตัวอย่าง handler {#handler-example}

`on_start` ของ strategy ตัวอย่าง EMA cross แสดงสิ่งต่อไปนี้:

- การลงทะเบียน indicator เพื่อรับ bar update
- การ request ข้อมูลย้อนหลัง (เพื่อเติมค่าให้ indicator)
- การ subscribe live data

การตรวจ cache สำคัญใน live trading เพราะ direct subscription สมมติว่า instrument ถูกโหลดไว้แล้วโดย config ของ
instrument provider หรือโดย instrument request ก่อนหน้า

indicator ที่ลงทะเบียนไว้จะได้รับ bar จาก response ของ request ก่อนที่ `on_historical_bars()` จะทำงาน ดังนั้น request
จะเติมค่าให้ indicator ได้ ให้ตรวจ `indicators_initialized()` ก่อนใช้ค่าของ indicator

ดูตัวอย่าง Rust ที่สมบูรณ์ได้ที่ [Write a Strategy (Rust)](../how_to/write_rust_strategy.md)

### Clock และ timer {#clock-and-timers}

strategy ใช้ `Clock` เพื่อสร้าง timestamp และตั้ง time alert หรือ timer ที่ trigger `TimeEvent`

#### Timestamp ปัจจุบัน {#current-timestamps}

clock ให้ timestamp UTC ปัจจุบันได้ทั้งในรูปแบบวันเวลา และเป็นจำนวนนาโนวินาทีนับจาก UNIX epoch

#### Time alert {#time-alerts}

time alert จะ dispatch `TimeEvent` ไปยัง handler `on_time_event` ณ เวลา alert ที่กำหนด ใน live trading การ
schedule และงานที่รออยู่ในคิวอาจทำให้การส่งล่าช้า เวลา alert ไม่ใช่การรับประกันเรื่อง latency

#### Timer {#timers}

timer แบบต่อเนื่องสร้าง `TimeEvent` เป็นช่วงเวลาสม่ำเสมอจนกว่าจะหมดอายุหรือถูก cancel

timer จะเริ่มทันที และ event แรกจะ fire หลังจากนั้นหนึ่ง interval ส่ง `fire_immediately=True` หากต้องการให้ fire
ที่เวลาเริ่มต้นแทน

timer แบบช่วงเวลาคงที่ใน live จะย้าย event แรกที่เลยกำหนดมาไว้ที่เวลาปัจจุบัน ยกเว้นที่ขอบ stop แบบ inclusive
จากนั้นจะไล่ tick ที่ถูก runtime หน่วงไว้ให้ทัน

ส่ง `callback` เพื่อ route object `TimeEvent` ไปยัง method ของคุณเองแทน `on_time_event` ชื่อ timer ใช้ namespace
เดียวกันทั้ง clock ดังนั้นให้ใช้ชื่อที่ไม่ซ้ำภายใน component ดู
[Actors: timers and alerts](actors.md#timers-and-alerts)

#### Calendar timer {#calendar-timers}

ใช้ `set_timer_zoned()` สำหรับเวลาท้องถิ่นที่เกิดซ้ำ ส่ง span string ที่เป็นบวก เช่น `"1d"` หรือ `"P1M"` และเวลาเริ่มต้น
ที่ระบุ timezone ตัวอย่างเช่นการ fire ทุกวันเวลา 09:30 ตามเวลา New York เริ่มตั้งแต่พรุ่งนี้

event แรกโดย default คือหนึ่ง interval หลัง `start_time` ส่วน `fire_immediately=True` จะรวมเวลาเริ่มต้นด้วย
เมื่อ `allow_past=False` event แรกต้องไม่อยู่ในอดีต เวลา stop เป็นแบบ inclusive การ cancel จะหยุด event ที่ตามมา

วัน สัปดาห์ เดือน และปี เป็นไปตามปฏิทินท้องถิ่น ส่วนหน่วยชั่วโมงและสั้นกว่าวัดเป็นเวลาที่ผ่านไปจริง ที่เวลา 09:30 ใน
New York event แบบ `"1d"` จะห่างกัน 23 ชั่วโมงเมื่อเริ่ม DST และ 25 ชั่วโมงเมื่อสิ้นสุด DST ถ้าใช้ `"24h"` event
จะห่างกัน 24 ชั่วโมงเสมอ และเวลาท้องถิ่นจะเลื่อนไปหนึ่งชั่วโมง

ให้เลือกเวลาเริ่มต้นท้องถิ่นที่มีอยู่จริง ระบบไม่ตรวจสอบ DST gap ที่เวลาเริ่มต้น การเกิดซ้ำยึดเวลาเริ่มต้นนั้นเป็นหลัก
ดังนั้นการปัดวันสิ้นเดือนและการปรับ DST จะไม่ทำให้ deadline ถัดไปเลื่อน gap ที่เกิดภายหลังจะเลื่อนไปข้างหน้าเท่าขนาด
ของ gap ส่วน fold จะใช้ occurrence แรก

ใน live trading `set_timer_zoned()` จะ fire occurrence ที่ค้างอยู่ถัดไปทันทีหากเลยกำหนด โดยใช้ timestamp ที่ schedule
ไว้ จากนั้นจะทำต่อที่ deadline แรกหลังเวลาปัจจุบัน virtual clock จะปล่อยทุก occurrence ที่ถึงกำหนดระหว่างการเลื่อน
เวลา timestamp ของ event ยังคงเป็น UTC UNIX nanoseconds

ใน Rust strategy ให้ใช้ `ClockApi::set_timer_zoned()` ถ้าใช้ `Clock` โดยตรง ให้ส่ง
`Arc::new(CalendarSchedule::new(&start_time, span)?)` ให้ `set_timer_schedule()` schedule นี้ implement
`TimerSchedule` และให้ deadline เป็น UTC โดย clock และ timer เป็นผู้ดูแลการส่งและการ cancel

### การเข้าถึง cache {#cache-access}

`Cache` ส่วนกลางของ trader เก็บข้อมูลและ execution object (order, position ฯลฯ) มี method จำนวนมากที่รองรับ
การกรอง ด้านล่างเป็นกรณีใช้งานพื้นฐาน

#### การดึงข้อมูล {#fetching-data}

ดึง quote, trade และ bar ล่าสุดจาก cache ได้ตาม instrument ID หรือ bar type method เหล่านี้คืนค่าว่าง (`None`)
ถ้าไม่มีข้อมูลที่ร้องขอ

#### การดึง execution object {#fetching-execution-objects}

ดึง order และ position แต่ละตัวจาก cache ได้ด้วย client order ID และ position ID

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

### การเข้าถึง portfolio {#portfolio-access}

`Portfolio` ส่วนกลางของ trader ให้ข้อมูลเกี่ยวกับ account และ position

#### ข้อมูล account และ position {#account-and-positional-information}

portfolio มี query สำหรับ account, balance ที่ถูก lock, initial margin และ maintenance margin ราย instrument,
unrealized/realized/total PnL, net exposure, net position และการตรวจว่า net long, net short หรือ flat ทั้งแบบ
รวมตาม venue/account และแบบราย instrument

ถ้าส่งทั้ง `venue` และ `account_id` ทั้งสองต้อง resolve ไปเป็น account เดียวกัน มิฉะนั้น query จะ error
Portfolio เปิด query ให้ strategy ใช้ ส่วน command ของ engine ยังอยู่ภายใน Rust runtime

#### Report และการวิเคราะห์ {#reports-and-analysis}

ดูคู่มือ [Portfolio statistics](portfolio.md#portfolio-statistics) สำหรับการวิเคราะห์ performance คู่มือ
[Portfolio](portfolio.md) ยังอธิบาย equity, การประเมินมูลค่าแบบ mark-to-market และขอบเขตของ query แบบหลาย account

### Trading command {#trading-commands}

trading command ต่อไปนี้ใช้สำหรับการจัดการ order ดูคู่มือ [Execution](execution/) สำหรับ flow ทั้งหมดในระบบ

#### การ submit order {#submitting-orders}

strategy แต่ละตัวมี `OrderFactory` เพื่อลด boilerplate ในการสร้าง object `Order` ใน Rust ให้ใช้ `OrderApi`
ผ่าน `self.order()` (ดู [Rust: order management](rust.md#order-management))

command `SubmitOrder` หรือ `SubmitOrderList` จะถูก route ครั้งแรกดังนี้:

- ถ้าระบุ `emulation_trigger` command จะถูกส่งไปที่ `OrderEmulator` *เป็นอันดับแรก*
- ถ้าระบุ `exec_algorithm_id` (และไม่มี `emulation_trigger`) command จะถูกส่งไปที่ `ExecutionAlgorithm` ที่เกี่ยวข้อง
  *เป็นอันดับแรก*
- นอกเหนือจากนั้น command จะถูกส่งไปที่ `RiskEngine` *เป็นอันดับแรก*

ดู [Emulated Orders](orders/emulated.md) สำหรับการ submit order เพื่อ emulate

:::info
คุณระบุทั้ง order emulation และ execution algorithm พร้อมกันได้ ในกรณีนี้ order จะถูกส่งไปที่ `OrderEmulator` ก่อน
และเมื่อถูก release แล้วจึง route ต่อไปยัง `ExecutionAlgorithm`
:::

ดู [Execution algorithms](execution/algorithms.md) สำหรับกฎพารามิเตอร์ของ TWAP และพฤติกรรมของ order ที่ถูก spawn

#### การ cancel order {#canceling-orders}

cancel order ได้ทีละตัว เป็น batch หรือทั้งหมดของ instrument หนึ่ง (กรองตามฝั่งได้)

ถ้า order *ปิด* ไปแล้วหรือกำลังรอ cancel อยู่แล้ว จะมีการ log warning

ถ้า order *เปิด* อยู่ สถานะจะกลายเป็น `PENDING_CANCEL`

การ route ขึ้นกับ command และ state ของแต่ละ order:

- สำหรับ emulated order `cancel_order(...)` จะไปที่ `OrderEmulator` ก่อน สำหรับ order ที่ไม่ได้ emulate จะไปที่
  `ExecutionAlgorithm` ที่เกี่ยวข้องถ้า order มี `exec_algorithm_id` และยัง active อยู่ในระบบภายใน มิฉะนั้นจะไปที่
  `ExecutionEngine`
- `cancel_all_orders(...)` โดย default จะ cancel ทุก order ที่ตรงเงื่อนไขและผูกกับ strategy แต่ละ order ใช้การ route
  แบบเดียวกับ `cancel_order(...)` ถ้าการ cancel บางตัวล้มเหลว method จะพยายาม cancel order ที่เหลือต่อ แล้วจึงคืน
  error ที่รวม error ของการ cancel ทั้งหมด
- `cancel_orders(...)` จะไปที่ `ExecutionEngine` เสมอในรูป command `BatchCancelOrders` ตัวเดียว

:::info
GTD timer ที่ถูก manage ไว้จะถูก cancel ด้วยหลังจาก command ออกจาก strategy ไปแล้ว
:::

สำหรับการ cancel แบบ batch ทุก order ใน batch ต้องเป็น instrument เดียวกัน และ batch ต้องไม่มี emulated order หรือ
local order

:::warning
ส่ง `strategy_only=False` เพื่อใช้ path การ cancel แบบกว้าง strategy จะส่ง command `CancelAllOrders` หนึ่งตัวแม้ใน
cache จะไม่มี order ที่ตรงเงื่อนไขเลย execution engine จะ resolve execution client และ account หนึ่งตัว แล้ว route การ
cancel ของ venue, emulated และ execution algorithm ภายในขอบเขตนั้น

order ที่ตรงเงื่อนไขซึ่งผูกกับ strategy อื่นอาจถูก cancel ด้วย แต่ order ที่ assign ให้ execution client อื่นจะไม่ถูกแตะ
ใช้โหมดกว้างเฉพาะเมื่อตั้งใจให้มีขอบเขตข้าม strategy เท่านั้น ดู
[Cancel-all routing](execution/index.md#cancel-all-routing) สำหรับ flow ทั้งหมดและกฎการเลือก client
:::

#### การแก้ไข order {#modifying-orders}

order แก้ไขได้ทีละตัวเมื่อเป็น emulated order หรือ *เปิด* อยู่บน venue (ถ้า venue รองรับ)

สำหรับการแก้ไขที่ถูกต้อง order ที่ *ปิด* ไปแล้วหรือกำลังรอ cancel จะ log warning ถ้า order *เปิด* อยู่ สถานะจะกลายเป็น
`PENDING_UPDATE`

:::warning
`modify_order(...)` จะ error เมื่อค่าที่ส่งมาทั้งหมดว่างหรือไม่เปลี่ยนแปลง strategy สามารถส่ง quantity เดิมซ้ำได้ขณะที่
order อยู่ในสถานะ `PENDING_UPDATE` การตรวจเหล่านี้ทำก่อนการตรวจ order ที่ปิดแล้วและการรอ cancel
:::

command `ModifyOrder` จะถูก route ครั้งแรกดังนี้:

- ถ้า order เป็น emulated order strategy จะส่ง command ไปที่ `OrderEmulator` ก่อน
- มิฉะนั้น ถ้า order มี `exec_algorithm_id` และยัง active อยู่ในระบบภายใน command จะถูกส่งไปที่ `ExecutionAlgorithm`
  ที่เกี่ยวข้อง
- มิฉะนั้น order จะถูกส่งไปที่ `RiskEngine` *เป็นอันดับแรก*

:::info
`ExecutionAlgorithm` จะปฏิเสธ `ModifyOrder` ของ primary order ที่ยัง active อยู่ภายใน และ log warning โดยไม่ปล่อย
event ใด ๆ เพราะ schedule ของ algorithm ผูกกับ quantity ที่มันคำนวณไว้ การ apply การแก้ไขโดยตรงจะทำให้ state นั้นเสีย
primary order จึงไม่ถูกเปลี่ยนแปลง
:::

:::info
price และ trigger price ก็แก้ไขได้เช่นกัน (เมื่อเป็น emulated order หรือ venue รองรับ)
:::

ใช้ `modify_orders(...)` เพื่อส่งการแก้ไขหลายรายการเป็น command `BatchModifyOrders` ตัวเดียวไปที่ `RiskEngine`
เช่นเดียวกับ batch cancel ทุก order ต้องเป็น instrument เดียวกัน และ batch ต้องไม่มี emulated order หรือ local order

#### Market exit {#market-exit}

method `market_exit()` เป็นวิธีออกจากทุก position และ cancel ทุก order ของ strategy อย่างนุ่มนวล strategy จะยังทำงาน
ต่อหลัง exit เสร็จ คุณจึงกลับเข้า position ใหม่ภายหลังได้หากต้องการ

การเรียกจะ log warning และกลับออกมาโดยไม่มีผล ถ้า strategy ไม่ได้อยู่ในสถานะ `RUNNING` หรือมีการ exit ดำเนินอยู่แล้ว

ขั้นตอนของ market exit:

1. เรียก `on_market_exit()`
2. cancel order ที่เปิดอยู่และ in-flight ทั้งหมดของ strategy
3. ปิด position ที่เปิดอยู่ทั้งหมดด้วย market order ที่ติด tag `MARKET_EXIT`
4. ตรวจเป็นระยะ (ทุก `market_exit_interval_ms`) จนกว่า order ทั้งหมดจะจบและ position ทั้งหมดถูกปิด เมื่อจำนวน
   working order เป็นศูนย์ จะ submit order ปิดใหม่สำหรับ position ที่ยังเปิดอยู่
5. เรียก `post_market_exit()` เมื่อ flat แล้ว หรือเมื่อถึง `market_exit_max_attempts` โดย log order และ position ที่ยัง
   ค้างอยู่

มี hook สองตัวสำหรับ logic ที่กำหนดเอง:

- `on_market_exit()`: ถูกเรียกเมื่อขั้นตอน exit เริ่มต้น
- `post_market_exit()`: ถูกเรียกเมื่อขั้นตอน exit เสร็จสิ้น

ระหว่าง market exit order ที่ไม่ใช่ reduce-only จะถูก deny อัตโนมัติด้วยเหตุผล `MARKET_EXIT_IN_PROGRESS` ส่วน order
ปิดของการ exit เองผ่านได้เพราะติด tag `MARKET_EXIT` สำหรับ order list ถ้ามี order ใดใน list ที่ไม่ใช่ reduce-only
ทั้ง list จะถูก deny เพื่อรักษาความหมายของ list (เช่น bracket order ที่มีความเกี่ยวพันกัน)

ใช้ `is_exiting()` เพื่อตรวจว่ากำลัง exit อยู่หรือไม่ (เช่น เพื่อข้าม logic การ submit order)

ตั้ง `manage_stop=True` ใน config เพื่อให้ทำ market exit เมื่อ strategy หยุด

เมื่อเปิดตัวเลือกนี้ การเรียก `stop()` จะทำ market exit ก่อน แล้วจึงหยุด strategy เมื่อ flat แล้ว หรือเมื่อถึง
`market_exit_max_attempts` การถึงขีดจำกัดจำนวนครั้งอาจทำให้ยังมี order หรือ position ค้างอยู่

:::warning
ใน backtest การ stop แบบ managed ที่ร้องขอตอนจบการรันจะทำไม่เสร็จ `market_exit()` จะ schedule การตรวจว่าเสร็จครั้งแรก
ที่ `market_exit_interval_ms` หลังเวลาปัจจุบัน ซึ่งเลยเวลาจบที่ร้องขอไป และ engine ได้ flush timer ครั้งสุดท้ายไปแล้ว
จึงไม่มี callback ใดทำงานเพื่อยืนยันว่า exit เสร็จ ผลคือ strategy จบการรันโดยยังอยู่ในสถานะ `RUNNING` และถูกรายงาน
ที่ระดับ `ERROR` แม้ว่าจะ flat แล้วก็ตาม
:::

ตัวเลือก config ใน `StrategyConfig`:

- `manage_stop` (default: `False`): ถ้าเป็น `True` `stop()` จะทำ market exit ก่อนหยุด
- `market_exit_interval_ms` (default: `100`): ช่วงเวลาระหว่างการตรวจว่า exit เสร็จหรือยัง
- `market_exit_max_attempts` (default: `100`): จำนวนครั้งสูงสุดของการตรวจก่อนจบการ exit
- `market_exit_time_in_force` (default: `GTC`): time in force ของ market order ที่ใช้ปิด
- `market_exit_reduce_only` (default: `True`): market order ที่ใช้ปิดควรเป็น reduce only หรือไม่

#### การปิด position {#closing-positions}

ใช้ `close_position(...)` และ `close_all_positions(...)` เพื่อทำให้ flat โดยไม่ต้องผ่านขั้นตอน market exit ทั้งหมด
ทั้งสองตัว submit market order สำหรับปิด และ strategy ยัง submit order ใหม่ได้ตามปกติ ดูคู่มือ
[Execution](execution/)

## Config ของ strategy {#strategy-configuration}

การแยก config ออกมาให้ความยืดหยุ่นเต็มที่ว่าจะสร้าง strategy ที่ไหนและอย่างไร config สามารถ serialize ส่งข้ามเครือข่าย
ได้ ทำให้รองรับ backtest แบบกระจายและ live trading จากระยะไกล

สิ่งนี้เป็นทางเลือก (opt-in) คุณข้าม config ไปและส่งพารามิเตอร์ให้ constructor ของ strategy โดยตรงก็ได้ ถ้าต้องการ
backtest แบบกระจายหรือ live trading จากระยะไกล ให้นิยาม config

ให้แยกข้อมูลสองประเภทออกจากกัน:

- ข้อมูล config: เก็บค่าตั้งต้นที่กำหนดว่า strategy ทำงานอย่างไร เช่น trade size และ instrument ID
- ตัวแปร state ของ strategy: ติดตาม state ที่กำหนดเองของ strategy เช่น เวลาที่เริ่มทำงานหรือจำนวน bar ที่ประมวลผลแล้ว

การแยกค่าตั้งต้นออกจาก state ที่เปลี่ยนแปลงทำให้โค้ดเข้าใจและดูแลง่ายขึ้น

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

:::note
strategy มักเทรด instrument เดียว แต่จำนวน instrument ที่ทำงานด้วยได้ถูกจำกัดด้วยทรัพยากรของเครื่องเท่านั้น
:::

### การ claim external order {#claiming-external-orders}

live adapter อาจรายงาน order บน venue ที่ strategy ไม่ได้ submit หรือที่ NautilusTrader ยังไม่เคยเห็นในเครื่อง
NautilusTrader จะสร้าง order เหล่านี้เป็น external order และ assign ให้ strategy `EXTERNAL` เว้นแต่จะมี claim ที่ active
ระบุว่า strategy อื่นเป็นเจ้าของ

ตั้ง `external_order_instrument_ids` เพื่อระบุรายการ instrument ที่ strategy ตั้งใจจะ claim สำหรับ external order,
fill และกิจกรรม reconciliation ที่ถูกสร้างขึ้น

`external_order_instrument_ids` คือเจตนาใน config ที่ serialize ได้ เมื่อ live node ลงทะเบียน strategy runtime จะ
แปลงเจตนานั้นเป็น external order claim ที่ active ใน cache ที่ใช้ร่วมกัน claim แต่ละตัว assign `InstrumentId` หนึ่งตัว
ให้ `StrategyId` หนึ่งตัว

การลงทะเบียนจะล้มเหลวโดยไม่เปลี่ยน claim ที่ active ถ้า:

- cache ที่ใช้ร่วมกันถูก borrow อยู่แล้ว
- รายการมี instrument ซ้ำ
- instrument ใดในรายการมี claim อยู่แล้ว รวมถึง claim ของ strategy เดียวกัน

หลังลงทะเบียน ให้เรียก `set_external_order_instrument_ids(...)` เพื่อแทนที่ชุด claim ที่ active ทั้งชุดของ strategy
method นี้:

- คง claim ในรายการที่ strategy เป็นเจ้าของอยู่แล้วไว้
- ปล่อย claim ที่ไม่อยู่ในรายการ
- ได้ claim ของ instrument ในรายการที่ยังไม่มีเจ้าของ

การส่งรายการว่างจะปล่อยทุก claim ที่ strategy เป็นเจ้าของ ถ้า instrument ปรากฏมากกว่าหนึ่งครั้งหรือเป็นของ strategy อื่น
การเรียกจะล้มเหลวและชุด claim ที่ active จะไม่เปลี่ยน

การเปลี่ยนแปลงมีผลกับการตัดสินความเป็นเจ้าของหลังการอัปเดตเท่านั้น ไม่ได้ assign external order ที่อยู่ใน cache แล้วใหม่

strategy ต้องลงทะเบียนก่อนจึงจะอัปเดต claim ที่ active ได้ และการอัปเดตจะล้มเหลวถ้า cache ที่ใช้ร่วมกันถูก borrow อยู่แล้ว

การหยุด strategy จะคง claim ที่ active ไว้ เพื่อให้การ route คงที่หาก strategy เริ่มใหม่ การนำ strategy ออกจาก trader
จะปล่อย claim การ reset cache ก็คง claim ไว้เช่นกัน

:::warning
การโอน instrument ระหว่าง strategy ต้องเรียกสองครั้ง: เจ้าของเดิมปล่อย instrument ก่อน แล้วเจ้าของใหม่จึง claim
การโอนนี้ไม่ atomic ข้าม strategy report จากภายนอกที่ถูกประมวลผลระหว่างการเรียกสองครั้งนั้นจะไม่มี claim ที่ active
และจะถูก assign ให้ strategy `EXTERNAL`
:::

ดู [External order creation](execution/reconciliation.md#external-order-creation) ว่า execution และ reconciliation
ใช้ claim ที่ active อย่างไร

### Managed GTD expiry {#managed-gtd-expiry}

strategy สามารถจัดการการหมดอายุของ order แบบ GTD (*Good 'till Date*) ได้เอง ใช้การจัดการในเครื่องเมื่อ exchange หรือ
broker ไม่รองรับ time in force นั้น หรือเมื่อคุณต้องการจัดการการหมดอายุในเครื่องเอง

เปิดใช้โดยตั้ง `manage_gtd_expiry=True` ใน `StrategyConfig` เมื่อ submit order ที่มี time in force แบบ GTD strategy จะ
เริ่ม time alert ภายใน และเมื่อ alert นั้น fire strategy จะ cancel order ถ้ายังไม่ *ปิด*

ตอนเริ่มทำงาน strategy จะตั้ง alert กลับคืนสำหรับ GTD order ที่เปิดอยู่ของตนใน cache และ cancel order ที่เลยเวลาหมดอายุ
ไปแล้ว

เมื่อ cancel request ถูกปฏิเสธ strategy ที่กำลังทำงานจะคง alert ที่มีอยู่ไว้ สำหรับ GTD order ที่เปิดอยู่หรือ inflight ซึ่ง
ไม่มี expiry alert:

- ถ้าเลยเวลาหมดอายุแล้ว จะ retry การ cancel ทันที
- มิฉะนั้น จะตั้ง expiry alert กลับคืน

ขั้นตอนนี้ทำก่อน `on_order_cancel_rejected` ดังนั้นการ retry ทันทีอาจทำให้ order กลับไปเป็น `PENDING_CANCEL` ก่อนที่
callback จะทำงาน strategy ที่หยุดแล้วจะไม่ตั้ง alert คืนหรือ retry การ cancel เมื่อถูกปฏิเสธ

บาง venue (เช่น Binance Futures) รองรับ time in force แบบ GTD ดังนั้นเพื่อเลี่ยงความขัดแย้งเมื่อใช้
`manage_gtd_expiry` ควรตั้ง `use_gtd=False` ใน config ของ execution client

### หลาย strategy {#multiple-strategies}

สำหรับ strategy เดียวกันหลาย instance ที่มี config ต่างกัน (เช่น เทรดคนละ instrument) ให้แต่ละตัวมี **strategy ID และ
order ID tag ที่ไม่ซ้ำกัน**

ระบบต้องระบุได้ว่า strategy ใดเป็นเจ้าของแต่ละ command และ event order ID tag ยังทำให้ client order ID ที่สร้างขึ้นไม่ซ้ำกัน
ข้าม strategy ภายใต้ trader เดียวกัน

#### Order ID tag {#order-id-tag}

ตั้ง `strategy_id` ในแต่ละ config runtime จะนำ order ID tag มาจากส่วนสุดท้ายที่คั่นด้วยขีด (hyphen) ของ strategy ID
ดังนั้น `MyStrategy-001` และ `MyStrategy-002` จะได้ tag เป็น `001` และ `002`

ถ้าส่ง `order_id_tag` ด้วย tag จะถูกต่อท้าย strategy ID ของ runtime เว้นแต่ ID จะลงท้ายด้วย tag นั้นอยู่แล้ว เช่น
strategy ID `MyStrategy-PRIMARY` กับ `order_id_tag` เป็น `ABC` จะลงทะเบียนเป็น `MyStrategy-PRIMARY-ABC`

strategy ที่ลงทะเบียนโดยไม่มี `strategy_id` จะใช้ชื่อ type ของ strategy เป็น base ID และ `order_id_tag` จะกลายเป็น suffix
ดังนั้น `MyStrategy` กับ `order_id_tag` เป็น `ABC` จะลงทะเบียนเป็น `MyStrategy-ABC` ถ้าไม่มี tag การลงทะเบียนจะ
กำหนด tag ตัวเลขถัดไปให้ เริ่มจาก `000`

เนื่องจาก runtime อ่าน tag กลับจากส่วนสุดท้ายที่คั่นด้วยขีดของ strategy ID `order_id_tag` จึงมีขีดไม่ได้ เช่น
`order_id_tag` เป็น `A-B` จะทำให้เกิด error

trader ID มี tag ในลักษณะเดียวกัน และส่วนสุดท้ายที่คั่นด้วยขีดของมันจะไปอยู่ใน ID ที่สร้างขึ้นชุดเดียวกัน ดู
[Configure a live trading node](../how_to/configure_live_trading.md) สำหรับข้อกำหนดที่ว่า trader ID ต้องไม่ซ้ำกันข้าม node

:::note
แพลตฟอร์มมีมาตรการความปลอดภัยในตัว strategy ID ที่ซ้ำกันจะทำให้เกิด error ที่บอกว่า strategy ID นั้นลงทะเบียนไว้แล้ว
strategy ID สองตัวที่ต่างกันแต่ใช้ order ID tag เดียวกันจะทำให้เกิด error ที่รายงานว่า tag ขัดแย้งกัน
:::

Rust ถือว่า `StrategyConfig` เป็น input สำหรับการสร้างที่เปลี่ยนแปลงไม่ได้ (immutable) `StrategyId` ของ runtime เป็นตัวที่
ถือ order ID tag ทำให้การลงทะเบียน actor, การสร้าง client order ID, การสร้าง order list ID และการสร้าง position ID
สอดคล้องกันผ่าน `strategy_id.get_tag()`

ดูรายละเอียดเพิ่มเติมของ `StrategyId` ได้ที่ [Rust API reference](https://nautilustrader.io/docs/core-latest/)

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Actors](actors.md) - พื้นฐานที่ strategy ขยายต่อ
- [Events](events/) - ประเภทของ event และการ dispatch handler
- [Orders](orders/) - ประเภทของ order และการจัดการจาก strategy
- [Backtesting](backtesting/) - ทดสอบ strategy ด้วยข้อมูลย้อนหลัง
