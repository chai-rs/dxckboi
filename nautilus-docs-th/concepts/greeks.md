# Greeks

Nautilus มีสองเส้นทางสำหรับทำงานกับ option Greeks ซึ่งเป็นค่าที่วัดว่าราคา option ตอบสนองต่อ
การเปลี่ยนแปลงของตัวแปรตลาดอย่างไร:

1. **Greeks จาก venue**: Greeks แบบ real-time ที่ stream มาจาก venue ที่รองรับผ่าน
   data type `OptionGreeks` และระบบรวม option chain (option chain aggregation)
1. **ตัวคำนวณ Greeks ในเครื่อง**: `GreeksCalculator` คำนวณ Black-Scholes Greeks จาก market data
   ใน cache รองรับการรวมระดับ portfolio, สถานการณ์ shock และ beta weighting

จะใช้เส้นทางใดเส้นทางหนึ่งอย่างอิสระหรือใช้ร่วมกันก็ได้ Greeks จาก venue มาถึงผ่านระบบ
subscription ของข้อมูลและไม่ต้องคำนวณในเครื่อง ส่วนตัวคำนวณในเครื่องครอบคลุม venue ที่
ไม่ได้ stream Greeks, การ backtest และการปรับแต่งเอง เช่น shock, beta weighting และ percent
Greeks

## Greeks จาก venue {#venue-provided-greeks}

### OptionGreeks {#optiongreeks}

type `OptionGreeks` แทนค่าความอ่อนไหว (sensitivity) ที่ venue ให้มาสำหรับสัญญา option หนึ่งสัญญา เป็น
type ที่ implement ใน Rust โดยตรง

| ฟิลด์               | คำอธิบาย                                               |
| ------------------ | ---------------------------------------------------- |
| `instrument_id`    | สัญญา option ที่ Greeks ชุดนี้ใช้ด้วย                       |
| `convention`       | convention ของ numeraire สำหรับ Greeks (`GreeksConvention`) |
| `delta`            | อัตราการเปลี่ยนแปลงของราคา option ต่อหนึ่งหน่วยของ underlying |
| `gamma`            | อัตราการเปลี่ยนแปลงของ delta ต่อหนึ่งหน่วยของ underlying      |
| `vega`             | vega ที่ venue รายงาน                                    |
| `theta`            | theta ที่ venue รายงาน                                   |
| `rho`              | rho ที่ venue รายงาน โดยมีค่าเริ่มต้นเป็นศูนย์                   |
| `mark_iv`          | mark implied volatility (อาจไม่มีค่า)                     |
| `bid_iv`           | bid implied volatility (อาจไม่มีค่า)                      |
| `ask_iv`           | ask implied volatility (อาจไม่มีค่า)                      |
| `underlying_price` | ราคาของ underlying ณ เวลาที่คำนวณ (อาจไม่มีค่า)              |
| `open_interest`    | open interest ของสัญญา (อาจไม่มีค่า)                       |
| `ts_event`         | UNIX timestamp (นาโนวินาที) ของ event                    |
| `ts_init`          | UNIX timestamp (นาโนวินาที) ตอน initialize               |

actor หรือ strategy สามารถ subscribe ด้วย `subscribe_option_greeks` และรับการอัปเดตผ่าน handler
`on_option_greeks`

ดู API การ subscribe ทั้งหมดได้ในคู่มือ [Options](options.md) ซึ่งรวมถึงการรวม option chain,
การกรองช่วง strike และโหมด snapshot

### การจัดเก็บและการ replay {#persistence-and-replay}

`OptionGreeks` เป็นสมาชิกโดยตรงของ enum `Data` ดังนั้นจึงถูกจัดเก็บลง data catalog และ replay ใน
backtest ในฐานะ market data แบบ built-in แทนที่จะเป็น custom data ให้ใช้ method ของ catalog ที่เฉพาะสำหรับ type นี้
ในการเขียนและ query

ระหว่าง replay Greeks ที่จัดเก็บไว้จะไปถึง actor หรือ strategy ที่ subscribe ผ่าน handler `on_option_greeks`
ตัวเดียวกับที่ใช้กับข้อมูล live และยังป้อนเข้าสู่การรวม option chain ด้วย

เมื่อ strategy subscribe `OptionChainSlice` data engine ของ backtest จะรวม `OptionGreeks` ที่ replay
เข้ากับการอัปเดต BBO ของ `QuoteTick` ที่ replay สำหรับแต่ละ option instrument ฟิลด์ `underlying_price` ใช้เป็น
ตัวตั้งต้นในการเลือก ATM และ `delta` รองรับการเลือก strike ตาม delta ผ่าน `StrikeRange.delta(target, tolerance)`

### Core schema เทียบกับ custom data {#core-schema-versus-custom-data}

ฟิลด์ดั้งเดิมของ `OptionGreeks` ประกอบเป็น core schema ได้แก่ Greeks มาตรฐานห้าตัว (`delta`, `gamma`,
`vega`, `theta` และ `rho`) รวมถึง implied volatility, ราคา underlying, open interest และ
convention ชื่อฟิลด์เหล่านี้คงที่

ไม่มี schema เดียวที่ครอบคลุมทุกกรณีการใช้งาน Greeks ค่าเฉพาะของ venue หรือเฉพาะของโมเดล เช่น `vanna`,
`volga`, `charm`, input สำหรับ calibration หรือ metadata ของ surface ให้เก็บใน [custom data](custom_data.md) ไม่ใช่ใน type
ดั้งเดิม ฟิลด์ optional ของ venue เป็นแบบ nullable ส่วน `convention` ไม่เป็น nullable

### Type ใน Rust ที่อยู่เบื้องหลัง {#underlying-rust-types}

implementation หลักใน Rust อยู่ใน `crates/model/src/data/greeks.rs` และ
`crates/model/src/data/option_chain.rs`:

- `OptionGreekValues`: struct ธรรมดาที่มีฟิลด์ `delta`, `gamma`, `vega`, `theta` และ `rho`
  implement `Add` และ `Mul<f64>` สำหรับการรวมค่า
- `OptionGreeks`: ห่อ `OptionGreekValues` ไว้พร้อมกับ `instrument_id`, `convention`, ฟิลด์ implied volatility
  และ timestamp implement `Deref<Target = OptionGreekValues>` เพื่อให้ผู้เรียกฝั่ง Rust เข้าถึง
  ฟิลด์ Greek ได้โดยตรง
- trait `HasGreeks`: มี method `greeks()` ที่คืนค่า `OptionGreekValues`
  implement โดย `OptionGreeks`, `GreeksData`, `PortfolioGreeks` และ
  `BlackScholesGreeksResult`

### ฟังก์ชัน Black-Scholes {#black-scholes-functions}

ฟังก์ชันคำนวณราคาระดับล่างอยู่ใน `crates/model/src/data/greeks.rs` ได้แก่ `black_scholes_greeks`
(คำนวณ Greeks เมื่อทราบ volatility), `imply_vol`, `imply_vol_and_greeks` (หา implied volatility จากราคาตลาด
แล้วคำนวณ Greeks) และ `refine_vol_and_greeks` (ปรับ volatility จากค่าประมาณตั้งต้น โดยมี full solver เป็นทางสำรอง)

`refine_vol_and_greeks()` จะทำขั้นตอน Halley refinement หนึ่งครั้งก่อน และยอมรับผลลัพธ์ก็ต่อเมื่อ
volatility ยังอยู่ภายในขอบเขตของ refinement และราคาจากโมเดลตรงกับ premium ภายใน
0.01 เปอร์เซ็นต์ มิฉะนั้นจะแก้หา implied volatility แบบเต็ม

ฟังก์ชันคำนวณราคาจะแจ้ง error เมื่อ input หรือผลลัพธ์ไม่ถูกต้อง ฟังก์ชัน IV ยังปฏิเสธ
premium ที่ไม่มี implied volatility ที่เป็นบวกและจำกัด (finite) `imply_vol()` คืนคำตอบแบบ scalar
โดยไม่คำนวณราคาซ้ำ ส่วนฟังก์ชันที่แก้หา IV และคืนค่า Greeks ต้องคำนวณราคาซ้ำให้ตรงภายใน
0.01 เปอร์เซ็นต์ และจะใช้ implementation แบบ exact เมื่อ precision ของ fast kernel ไม่เพียงพอ
การคำนวณราคาเมื่อทราบ volatility ก็ใช้ implementation แบบ exact เช่นกันเมื่อผลลัพธ์แบบเร็วไม่จำกัด (nonfinite) หรือ
ไม่สอดคล้องกัน หากผลลัพธ์แบบ exact ยังไม่ผ่านการตรวจสอบ ฟังก์ชันจะแจ้ง error

`BlackScholesGreeksResult` ที่ฟังก์ชันเหล่านี้คืนมาประกอบด้วย: `price`, `vol`,
`delta`, `gamma`, `vega`, `theta`, `itm_prob` และ `rho`

Convention:

- Vega ถูก scale ด้วย 0.01 (ความอ่อนไหวต่อการเปลี่ยนแปลงของ vol 1 จุดเปอร์เซ็นต์)
- Theta ถูก scale ด้วย 1/365.25 (การลดลงของมูลค่ารายวัน)
- Rho ถูก scale ด้วย 0.01 (ความอ่อนไหวต่อการเปลี่ยนแปลงของอัตราดอกเบี้ย 1 จุดเปอร์เซ็นต์)
- option แบบ American จะถูกคำนวณราคาเหมือนแบบ European สำหรับการคำนวณ Greeks

## ตัวคำนวณ Greeks ในเครื่อง {#local-greeks-calculator}

### GreeksCalculator {#greekscalculator}

`GreeksCalculator` คำนวณ Black-Scholes Greeks จาก market data ใน cache โดยใช้ cache และ clock
และเข้าถึงได้จาก actor และ strategy

#### Greeks ของ instrument {#instrument-greeks}

`instrument_greeks()` คำนวณ Greeks ของ instrument เดียว (option หรือ underlying) โดยใช้ quantity เท่ากับ 1
และใช้ `flat_interest_rate` เมื่อไม่มี yield curve ใน cache ผลลัพธ์คือ `GreeksData` หรือไม่มีค่าระหว่างที่
market data ยังไม่พร้อม (warm-up)

สำหรับ option instrument ตัวคำนวณจะทำตามขั้นตอนต่อไปนี้:

1. ค้นหา instrument และปฏิเสธ option ที่หมดอายุแล้วหรือ option spread ที่ไม่รองรับก่อนจะใช้ Greeks ที่อยู่ใน cache
1. resolve underlying เมื่อการประเมินมูลค่าใหม่หรือแบบ shock ต้องใช้
1. ดึงราคาจาก cache instrument มาตรฐานจะเลือก `MID` ก่อนและถอยกลับไปใช้ `LAST` ส่วน instrument
   ที่เป็น index จริงจะเลือกราคา index ที่อยู่ใน cache ก่อน
1. ค้นหา yield curve จาก cache โดยถอยกลับไปใช้ `flat_interest_rate`
1. แก้หา implied volatility จาก premium ตลาดที่ถูกต้อง แล้วคำนวณ Greeks ที่ผ่านการตรวจสอบ
1. คืนค่า object `GreeksData` ที่มีค่าที่คำนวณได้

กรณีไม่มีราคาจะคืนค่าว่าง (`None`) ทำให้ strategy มองช่วง warm-up เป็นเส้นทางปกติที่ไม่ต้องทำอะไร
ส่วนกรณีไม่มีนิยามของ instrument, option หมดอายุ, option spread, input หรือผลลัพธ์แบบ shock ที่ไม่ถูกต้อง
และการแก้หา implied volatility ที่ล้มเหลวจะแจ้ง error การหมดอายุจะถูกตรวจสอบก่อนคืนค่า Greeks
ของ option ที่อยู่ใน cache

ตัวคำนวณจะหา carry เป็น `r - q` เมื่อมี dividend yield หรือ curve และกำหนด carry คงที่ที่
ศูนย์ในกรณีอื่น การประเมินมูลค่าแบบ shock จากค่าใน cache จะคง input ตัวเลขที่อยู่ใน cache ไว้ และเลือกความอ่อนไหวของ
carry ต่ออัตราดอกเบี้ยจาก dividend yield หรือ dividend curve ที่มีของการเรียกครั้งปัจจุบัน

สำหรับ instrument ที่ไม่ใช่ option เช่น futures และหุ้น ตัวคำนวณจะคืน `GreeksData` ที่มี `delta=1` หรือ
delta แบบ beta-weighted และมี gamma, vega, theta และ rho เป็นศูนย์ ฟิลด์เฉพาะของ option จะคงค่าเริ่มต้นไว้

#### สถานการณ์ shock {#shock-scenarios}

ใช้สมมติฐานการเปลี่ยนแปลงของ spot, volatility หรือเวลา ผ่านพารามิเตอร์ `spot_shock` (เช่น +10 จุดบน
underlying), `vol_shock` (เช่น +2 จุดเปอร์เซ็นต์ของ volatility) และ `time_to_expiry_shock` (เช่น `1 / 365.25`
เพื่อเลื่อนไปข้างหน้าหนึ่งวันปฏิทิน)

#### การอัปเดต volatility {#volatility-update}

ปรับ implied volatility จากจุดตั้งต้นที่อยู่ใน cache ด้วย `update_vol` (ใช้ vol ใน cache เป็นจุดตั้งต้น)
และ `cache_greeks` (เก็บผลลัพธ์ไว้สำหรับรอบถัดไป)

เมื่อมี Greeks อยู่ใน cache `update_vol=True` จะใช้ volatility ใน cache เป็นจุดตั้งต้นของ refinement โดยมี
การตรวจสอบและ full solver เป็นทางสำรองตามที่อธิบายไว้ข้างต้น หาก cache ไม่มี Greeks ก่อนหน้าของ
instrument นั้น ตัวคำนวณจะแก้หา implied volatility แบบเต็ม

#### Greeks แบบ beta-weighted {#beta-weighted-greeks}

แสดง delta และ gamma ในรูปของ index ผ่าน `index_instrument_id`, `beta_weights` และ `percent_greeks`

#### Vega แบบถ่วงน้ำหนักตามเวลา {#time-weighted-vega}

ทำให้ vega ของวันหมดอายุที่ต่างกันเทียบกันได้ ด้วย `vega_time_weight_base` (เช่น `30` เพื่อ normalize
เป็น vega 30 วัน)

#### Greeks ของ portfolio {#portfolio-greeks}

`portfolio_greeks()` รวม Greeks ของ position ที่เปิดอยู่ทั้งหมดที่ตรงกับเงื่อนไขการกรอง และคืนค่าเป็น
`PortfolioGreeks` นอกจากตัวกรองด้านล่างแล้วยังรับ `flat_interest_rate`, `index_instrument_id`,
`beta_weights` และ `percent_greeks`

ตัวกรอง:

- `underlyings`: รายการ prefix ของ symbol ตัวอย่างเช่น `["AAPL"]` จะตรงกับหุ้น AAPL และ option ของ AAPL
  ทั้งหมด
- `venue`: จำกัดเฉพาะ venue เดียว
- `instrument_id`: จำกัดเฉพาะ instrument เดียว
- `strategy_id`: จำกัดเฉพาะ strategy เดียว
- `side`: กรองตามฝั่งของ position เช่น `LONG` หรือ `SHORT`
- `greeks_filter`: callable ที่รับ `GreeksData` ของแต่ละ position หลังจาก `pnl`, `price` และค่า Greek
  ถูก scale ด้วย contract multiplier แล้วตามด้วย quantity ของ position แบบมีเครื่องหมาย คืนค่า `True` เพื่อรวม position นั้น
  ข้อมูลบริบทของ instrument รวมถึง `multiplier` และ `quantity` จะไม่เปลี่ยนแปลง ให้ใช้ค่าเหล่านี้โดยตรง
  เพราะ `to_portfolio_greeks()` จะคูณ multiplier ซ้ำอีกครั้ง

การประเมินมูลค่า instrument ที่ล้มเหลวหรือผลรวมที่ไม่จำกัด (nonfinite) จะทำให้เกิด error การคำนวณจะไม่คืน
ผลลัพธ์ portfolio เพียงบางส่วน

### GreeksData {#greeksdata}

`GreeksData` เก็บบริบทของการคำนวณ Greeks ของ instrument เดียว การ cache และการ publish ที่ร้องขอ
จะใช้การประเมินมูลค่า option แบบไม่มี shock หลังจากผ่านการตรวจสอบแล้วเท่านั้น การประเมินมูลค่าจาก cache
จะข้ามขั้นตอนเหล่านั้น รวมถึงกรณีที่ใช้ shock ด้วย `GreeksCalculator` ใน Rust สามารถ publish ไปยัง topic
`data.GreeksData.underlying={underlying}.instrument_id={instrument_id}`
ซึ่งใช้ symbol ของ underlying และ instrument ID เต็มของ option เป็น key

| ฟิลด์               | คำอธิบาย                                                                   |
| ------------------ | ------------------------------------------------------------------------- |
| `instrument_id`    | instrument ที่ใช้คำนวณ                                                      |
| `is_call`          | `True` สำหรับ call หรือผลลัพธ์ที่ไม่ใช่ option, `False` สำหรับ put             |
| `strike`           | ราคา strike                                                               |
| `expiry`           | วันหมดอายุในรูปจำนวนเต็ม `YYYYMMDD`                                          |
| `expiry_in_days`   | จำนวนวันจนถึงวันหมดอายุ                                                     |
| `expiry_in_years`  | จำนวนปีจนถึงวันหมดอายุ (`expiry_in_days / 365.25`)                           |
| `multiplier`       | contract multiplier                                                       |
| `quantity`         | quantity ซึ่ง `instrument_greeks()` ตั้งเป็น 1                              |
| `underlying_price` | ราคา underlying ที่ใช้ในการคำนวณ                                             |
| `interest_rate`    | อัตราดอกเบี้ยที่ใช้ในการคำนวณ                                                 |
| `cost_of_carry`    | cost of carry (`r - dividend yield` เมื่อมีการระบุ หรือศูนย์หากไม่มี)              |
| `vol`              | implied volatility                                                        |
| `pnl`              | PnL เทียบกับจุดเข้าของ position เมื่อมีการระบุ position                         |
| `price`            | ราคาจากโมเดลของ option หรือ PnL ของ position ที่ไม่ใช่ option เมื่อมีการระบุ       |
| `delta`            | Delta                                                                     |
| `gamma`            | Gamma                                                                     |
| `vega`             | Vega ต่อ volatility หนึ่งจุดเปอร์เซ็นต์                                      |
| `theta`            | Theta รายวัน                                                              |
| `rho`              | Rho ต่ออัตราดอกเบี้ยหนึ่งจุดเปอร์เซ็นต์                                        |
| `itm_prob`         | ความน่าจะเป็นที่จะเป็น in-the-money                                          |
| `ts_event`         | timestamp ของ event หน่วยนาโนวินาที                                         |
| `ts_init`          | timestamp ตอน initialize หน่วยนาโนวินาที                                    |

`GreeksData.to_portfolio_greeks()` คูณ contract multiplier หนึ่งครั้งกับ `pnl`, `price` และค่า Greek
ทั้งห้าตัว รวมถึง `rho` แล้วคืนค่า `PortfolioGreeks` ที่มี timestamp เดิม
โดยปล่อยการ scale ด้วย quantity ของ position ให้เป็นหน้าที่ของผู้เรียก และคง `GreeksData` แบบต่อหน่วยไว้

`portfolio_greeks()` ใช้การ scale แบบ multiplier ก่อนแล้วจึง quantity นี้ก่อนการกรองและการรวมค่า ฟิลด์
`quantity` ระหว่างทางจะยังเป็น `1` และไม่ได้เป็นส่วนหนึ่งของ `PortfolioGreeks` ผู้เรียกฝั่ง Rust ใช้
`quantity * &greeks_data.to_portfolio_greeks()` เพื่อให้ได้ผลลัพธ์เดียวกัน ส่วน expression ใน Rust
`quantity * &greeks_data` จะ scale ด้วย quantity เท่านั้นและคืนค่าเป็น `GreeksData`

### PortfolioGreeks {#portfoliogreeks}

`PortfolioGreeks` เป็นผลลัพธ์ของการแปลงและการรวมค่าระดับ portfolio

รองรับการบวกผลลัพธ์ portfolio เข้าด้วยกัน โดยคง timestamp ของ operand ด้านซ้ายไว้
การคูณด้วย scalar ทางซ้ายจะ scale `pnl`, `price` และค่า Greek ทั้งห้าตัว โดยคง timestamp ไว้
ใน Rust ให้ใช้ `scalar * &portfolio_greeks`

| ฟิลด์       | คำอธิบาย                                                           |
| ---------- | ----------------------------------------------------------------- |
| `pnl`      | PnL รวมหลังการ scale ด้วย multiplier และ quantity แบบมีเครื่องหมาย       |
| `price`    | มูลค่าจากโมเดลรวมหลังการ scale ด้วย multiplier และ quantity แบบมีเครื่องหมาย |
| `delta`    | delta ของ portfolio                                               |
| `gamma`    | gamma ของ portfolio                                               |
| `vega`     | vega ของ portfolio                                                |
| `theta`    | theta ของ portfolio                                               |
| `rho`      | rho ของ portfolio ต่ออัตราดอกเบี้ยหนึ่งจุดเปอร์เซ็นต์                      |
| `ts_event` | timestamp ของ event หน่วยนาโนวินาที                                 |
| `ts_init`  | timestamp ตอน initialize หน่วยนาโนวินาที                            |

### Yield curve {#yield-curves}

`YieldCurve` เก็บอัตราดอกเบี้ยหรือ dividend yield ตาม tenor หน่วยเป็นปี curve ที่มีอัตราเดียวจะเป็นแบบ
flat และ curve ที่มีตั้งแต่สามจุดขึ้นไปจะถูก interpolate ตัวคำนวณค้นหา curve ใน cache ด้วย
`curve_name`: ใช้รหัส quote currency ของ option (เช่น `USD`) สำหรับอัตราดอกเบี้ย และใช้
instrument ID ของ underlying สำหรับ dividend yield หากไม่มี curve ที่ตรงกัน จะใช้
`flat_interest_rate` และ `flat_dividend_yield`

`YieldCurve` เป็น custom data ให้ห่อ curve แต่ละตัวด้วย `CustomData` เพื่อเขียนลง catalog, replay
ใน backtest หรือ publish ไปยัง actor ที่ subscribe ไว้ data engine ไม่ได้ cache curve ให้ ดังนั้น actor
ที่ได้รับ curve ต้องเรียก `cache.add_yield_curve()` ก่อนที่ตัวคำนวณจะใช้งานได้

curve จะไม่ถูกต้องเมื่อ `tenors` และ `interest_rates` มีความยาวไม่เท่ากัน, มีสองจุดหรือไม่มีเลย, หรือ
มีค่าที่ไม่จำกัด (non-finite) และเมื่อ `tenors` ไม่ได้เพิ่มขึ้นอย่างเคร่งครัดโดยแต่ละขั้นกว้างพอที่จะ
interpolate ได้ `cache.add_yield_curve()` จะปฏิเสธ curve เหล่านี้ รวมถึง curve ที่ decode มาจาก catalog
หรือ JSON ตัวคำนวณจึงไม่มีวันประเมินค่า curve เหล่านั้น
ใน Rust ให้เรียก
`nautilus_serialization::ensure_custom_data_registered::<YieldCurve>()` ก่อนการเขียนและ query
ของ catalog หรือ streaming

### การบันทึกค่าที่คำนวณได้ {#recording-calculated-values}

`GreeksData` และ `PortfolioGreeks` รองรับการบันทึกแบบ Arrow โดยตรงภายใต้ selector
`NautilusRecordType` ที่ตรงกัน streaming writer ใน Rust รับ record เหล่านี้ได้โดยตรง
writer ที่ subscribe ไว้ยังจับการ publish `GreeksData` แบบมี type ของตัวคำนวณใน Rust ได้ด้วย

อ่านค่าที่บันทึกไว้ด้วย `query_record_arrow_bytes()` หรือ `query_record_arrow_stream()`
query เหล่านี้คืนข้อมูลแบบ Arrow record ที่คำนวณได้จะไม่เข้าร่วมใน market-data replay ของ `OptionGreeks`
และไม่เติมข้อมูลลง cache ของตัวคำนวณ ส่วน yield curve เป็น custom data ไม่ใช่ record
ดู [Yield curve](#yield-curves)

## การเลือกระหว่างสองเส้นทาง {#choosing-between-the-two-paths}

| เกณฑ์                  | จาก venue (`OptionGreeks`)                                | ตัวคำนวณในเครื่อง (`GreeksCalculator`)                  |
| --------------------- | --------------------------------------------------------- | --------------------------------------------------- |
| การคำนวณ              | venue หรือ broker เป็นผู้คำนวณ                                | Black-Scholes ในเครื่อง                               |
| Latency               | มาพร้อมกับ market data                                     | คำนวณเมื่อเรียกใช้                                      |
| Venue                 | Bybit, Deribit, Derive, Interactive Brokers และ OKX        | option ใดก็ได้ใน cache ที่มีราคาที่จำเป็น                     |
| สถานการณ์ shock        | ไม่รองรับ                                                  | shock ของ spot, vol และเวลา                           |
| การรวมระดับ portfolio  | ทำเอง เช่น วนลูปผ่าน `OptionChainSlice`                       | มีในตัวผ่าน `portfolio_greeks()`                        |
| Beta weighting        | ไม่รองรับ                                                  | มีในตัว                                               |
| การรองรับ backtest      | จากข้อมูล `OptionGreeks` ที่บันทึกไว้                          | จากราคาใน cache ณ จุดเวลาใดก็ได้                         |
| ค่าที่ได้                | delta, gamma, vega, theta, rho, IV และ open interest       | delta, gamma, vega, theta, rho, itm_prob และ vol    |
| Data type             | `OptionGreeks`                                            | `GreeksData` และ `PortfolioGreeks`                    |

## นิยามของ Greek {#greek-definitions}

คำศัพท์เหล่านี้ปรากฏในทั้งสองเส้นทาง ฟังก์ชัน Black-Scholes ในเครื่องจะ scale vega และ theta ตามที่อธิบาย
ไว้ข้างต้น ส่วน `OptionGreeks` จะคงค่าที่ venue หรือ broker แต่ละรายรายงานมา และบันทึก `convention` ของค่านั้นไว้

| Greek    | ฟิลด์       | นิยาม                                                                                                        |
| -------- | ---------- | ------------------------------------------------------------------------------------------------------------ |
| Delta    | `delta`    | อนุพันธ์อันดับหนึ่งของราคา option เทียบกับราคา underlying (`dV/dS`)                                                |
| Gamma    | `gamma`    | อนุพันธ์อันดับสองของราคา option เทียบกับราคา underlying (`d²V/dS²`)                                               |
| Vega     | `vega`     | ความอ่อนไหวต่อการเปลี่ยนแปลงของ implied volatility (`dV/dVol`)                                                    |
| Theta    | `theta`    | ความอ่อนไหวต่อเวลาที่ผ่านไป (`dV/dt`)                                                                            |
| Rho      | `rho`      | ความอ่อนไหวต่อการเปลี่ยนแปลงของอัตราดอกเบี้ยปลอดความเสี่ยง (`dV/dr`)                                                   |
| ITM prob | `itm_prob` | ความน่าจะเป็นที่ option จะจบแบบ in the money: `P(ϕS_T > ϕK)` โดยที่ `ϕ = 1` สำหรับ call และ `ϕ = -1` สำหรับ put |

## ตัวอย่าง {#examples}

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python (การ subscribe Greeks จาก Bybit, Deribit และ OKX) จึงตัดออก

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Options](options.md): option instrument, การ subscribe chain และการกรอง strike
- [Data](data/): data type แบบ built-in, custom data และโมเดลการ subscribe
- [Actors](actors.md): ข้อมูลอ้างอิงของการ subscribe และ handler
- [Strategies](strategies.md): การ implement strategy และ handler method
