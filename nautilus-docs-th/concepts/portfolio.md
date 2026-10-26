# Portfolio

Portfolio เก็บ state ที่ได้จาก account และ position ของ trading node หรือ backtest strategy ใช้ Portfolio
เพื่อ query ข้อมูล account, PnL, exposure, margin, equity และสถิติผลการดำเนินงาน

## การแปลงสกุลเงิน {#currency-conversion}

Portfolio สามารถแปลง PnL และ exposure จากสกุลเงินต้นทุน (cost currency) ดั้งเดิมไปเป็นสกุลเงินหลัก
(base currency) ของ account หรือสกุลเงินเป้าหมายที่ระบุไว้อย่างชัดเจน ทำให้รองรับ instrument ที่มี
cost currency ต่างกัน และ account ที่มี base currency ต่างกันได้

### การแปลงที่รองรับ {#supported-conversions}

การแปลงสกุลเงินใช้ได้กับ query ของ portfolio ต่อไปนี้:

- `realized_pnl()` และ `realized_pnls()` แปลง realized PnL
- `unrealized_pnl()` และ `unrealized_pnls()` แปลง unrealized PnL
- `total_pnl()` และ `total_pnls()` แปลง total PnL
- `net_exposure()` และ `net_exposures()` แปลง net exposure

ทั้งแปดเมธอดรับ `target_currency` (ไม่บังคับ) query ที่ระบุเป้าหมายและสำเร็จจะมีเฉพาะสกุลเงินนั้น
Portfolio แปลงค่าดั้งเดิมแต่ละค่าไปเป็นสกุลเงินเป้าหมายโดยตรง แม้ account จะมี base currency ต่างออกไปก็ตาม

### พฤติกรรมกรณี account เดียว {#single-account-behavior}

เมื่อ `PortfolioConfig.convert_to_account_base_currency=true` (ค่าเริ่มต้น) query สำหรับ account เดียวที่ไม่ระบุ
`target_currency` จะแปลงค่าเป็น base currency ของ account หาก account มี base currency มิฉะนั้นผลลัพธ์จะคงอยู่ใน
cost currency ดั้งเดิม

### พฤติกรรมกรณีหลาย account {#multi-account-behavior}

เมื่อ query หลาย account โดยไม่ระบุ `target_currency` ผลลัพธ์จะขึ้นกับว่าเมธอดคืนค่าเป็น map ของสกุลเงิน
หรือเป็นค่า `Money` ค่าเดียว:

- เมธอดแบบ collection อาจคืน dictionary ที่มีหนึ่ง entry ต่อสกุลเงินของผลลัพธ์ โดยใช้ base currency ของ account
  ในกรณีที่ตั้งค่าไว้ และใช้ cost currency ดั้งเดิมในกรณีอื่น ระบุ `target_currency` เพื่อรวมผลลัพธ์ทั้งหมด
  เป็นสกุลเงินเดียว
- เมธอดแบบค่าเดียวจะคืน `None` หากค่าจาก account ต่าง ๆ ไม่สามารถสรุปเป็นสกุลเงินของผลลัพธ์เดียวได้

`net_exposures()` จะคืน `None` เช่นกัน หาก instrument เดียวกระจายอยู่ในหลาย account ที่มี base currency ต่างกัน
เพราะ exposure ราย instrument ไม่สามารถสรุปเป็นสกุลเงินของผลลัพธ์เดียวได้

ตัวอย่างเช่น หลาย account ที่มี base currency เดียวกันจะได้ผลเป็น `{USD: Money(...)}`, account ที่มี base currency
ต่างกันและ instrument แยกกันชัดเจนจะได้ `{USD: Money(...), EUR: Money(...)}` และเมื่อระบุ
`target_currency=USD` จะได้ `{USD: Money(...)}`

### กรณีการคำนวณล้มเหลว {#calculation-failures}

query ของ PnL และ exposure จะ fail closed เมื่อขาดราคา, xrate หรือการคำนวณแบบ exact ที่จำเป็นใด ๆ

query แบบ collection **ล้มเหลวทั้งชุด** จะไม่คืนผลลัพธ์บางส่วน หรือปนสกุลเงินเป้าหมายกับสกุลเงินต้นทาง
เช่น instrument เดียวที่ไม่มีราคาจะทำให้ผลลัพธ์ทั้งหมดของ `unrealized_pnls` หรือ `total_pnls` ใช้ไม่ได้
query `net_exposures()` แบบทุก scope ที่ถูกต้องจะคืน `{}` เมื่อ portfolio ไม่มี position

:::warning
ต้องมีข้อมูลอัตราแลกเปลี่ยนเมื่อใช้ `target_currency` เพื่อรวมค่าข้ามสกุลเงิน
:::

### ประเภทราคาที่ใช้ในการแปลง {#conversion-price-types}

- **การประเมินมูลค่า position**: ใช้ราคา `MARK` ปัจจุบันก่อนเมื่อเปิด `use_mark_prices` มิฉะนั้นจะใช้
  `BID` สำหรับ position ฝั่ง long และ `ASK` สำหรับ position ฝั่ง short ก่อนลองแหล่งราคาสำรองที่เหลือ
- **การแปลงสกุลเงิน**: ใช้ xrate แบบ `MID` ปัจจุบันจาก cache หากเปิด `use_mark_xrates` จะใช้ xrate แบบ
  `MARK` ปัจจุบันก่อน และใช้ `MID` เป็นตัวสำรอง query ที่ระบุสกุลเงินเป้าหมายอย่างชัดเจนจะไม่นำ xrate เก่า
  ที่ค้างไว้มาใช้ซ้ำ

### การรวม exposure {#exposure-aggregation}

`net_exposure()` ประเมินมูลค่าแต่ละ position ที่เปิดอยู่ของ instrument หนึ่ง แล้วบวก notional ฝั่ง long และลบ
notional ฝั่ง short ผลที่คืนคือขนาดของ notional สุทธิที่ประเมินแล้ว จึงไม่บอกทิศทาง `price` ที่ผู้เรียกส่งมาจะใช้
ประเมินมูลค่าทุก position ที่เลือกด้วยราคาเดียวกัน หากไม่มีการ override หรือราคา mark ร่วม ราคา bid และ ask
ที่ต่างกันตามฝั่งอาจทำให้เกิดมูลค่าคงค้างแม้ปริมาณสองฝั่งจะเท่ากัน

`net_exposures()` จัดกลุ่มและรวมขนาด exposure ราย instrument ที่ไม่เป็นศูนย์ตามสกุลเงินของผลลัพธ์ โดยไม่หักกลบ
exposure ตามทิศทางระหว่าง instrument ต่างตัวกัน

### การ override ราคา {#price-overrides}

เมธอด `unrealized_pnl`, `total_pnl` และ `net_exposure` รับ `price` (ไม่บังคับ) เมื่อระบุ Portfolio จะประเมินมูลค่า
instrument ที่เลือกด้วยราคานั้นแทนการอ่านราคาตลาดจาก cache การคำนวณนี้ทำใหม่ทุกครั้ง และไม่แทนที่ PnL,
exposure หรือราคาตลาดใน cache ที่ query ถัดไปจะใช้

## Equity และ mark-to-market {#equity-and-mark-to-market}

Portfolio มี query แบบ pull สำหรับการประเมินมูลค่า portfolio อย่างต่อเนื่องและ snapshot ที่บันทึกไว้
ผลลัพธ์รายสกุลเงินใช้ base currency ของ account ที่เกี่ยวข้อง หรือ cost currency ดั้งเดิม

| Method                                         | สิ่งที่คืน                                                |
| ---------------------------------------------- | ------------------------------------------------------ |
| `mark_values(venue, account_id)`               | ยอดรวม MTM แบบมีเครื่องหมายของ position ที่เปิดอยู่          |
| `equity(venue, account_id)`                    | equity รวมจาก balance และมูลค่าของ position               |
| `build_snapshot(account_id)`                   | ยอดรวม MTM ทั้ง account และ metadata ของการประเมินมูลค่า     |
| `snapshots(account_id)`                        | snapshot ของ account ที่บันทึกไว้ เรียงตามลำดับการ emit       |
| `missing_price_instruments(venue, account_id)` | instrument ที่ถูก flag ว่าประเมินราคาไม่ได้                   |

position ฝั่ง long นับ notional เป็นบวก ฝั่ง short นับเป็นลบ position ที่ flat จะถูกข้าม

query `equity()` แบบระบุ account จะคืน `{}` สำหรับ account ที่ไม่รู้จัก สำหรับ account ที่รู้จัก หากการประเมิน
มูลค่า snapshot แบบ exact ล้มเหลว จะคืน error แทนการแสดงผลเป็น equity ว่าง

### สูตรของ equity {#equity-formula}

equity เริ่มจาก `balances_total` แล้วบวกมูลค่าของ position ที่เปิดอยู่รายสกุลเงิน:

| ประเภท account                           | ค่าที่บวกเข้ากับ balance                                    |
| --------------------------------------- | -------------------------------------------------------- |
| Cash ที่ไม่มี base currency หรือ Wallet    | mark ของ position โดยมีข้อยกเว้นเรื่อง balance ด้านล่าง       |
| Cash ที่มี base currency หรือ betting     | mark ของทุก position ที่เปิดอยู่                             |
| Calculated margin                       | mark ของ premium แบบมีเครื่องหมาย + unrealized PnL ของ position อื่น |
| Reported margin                         | unrealized PnL ของทุก position ที่เปิดอยู่                    |

#### การประเมินมูลค่าแบบ Cash และ Wallet {#cash-and-wallet-valuation}

สำหรับ position ที่เป็นของ Cash account ที่ไม่มี base currency หรือของ Wallet account จะไม่นับ mark ของ base asset
แบบ non-inverse เมื่อเข้าเงื่อนไขสองข้อ คือ balance ต้องถือ asset นั้นอยู่แล้ว และ cost currency ของ instrument
ต้องต่างจาก base currency ของมัน instrument แบบ inverse และ asset ที่ยังไม่ถูก credit จะยังคงนับ mark

`mark_values()` คืนมูลค่ารวม (gross) ของ position ที่เปิดอยู่เสมอ รวมถึง asset ที่ถูก credit แล้ว ส่วน `equity()` และ
snapshot จะนับ base asset แบบ non-inverse แต่ละตัวเพียงครั้งเดียว ผ่าน balance หรือผ่าน mark

#### การประเมินมูลค่าแบบ margin {#margin-valuation}

margin account แบบ calculated (`calculate_account_state=true`) อัปเดตเงินสดจาก fill ส่วน account แบบ reported ใช้
balance จาก venue query ของ margin ที่ระบุ venue จะใช้กฎของเจ้าของ position แต่ละราย ดู
[accounting](accounting.md) สำหรับการอัปเดต balance

instrument ที่อิง premium รวมถึง option และ binary option จะแลกเปลี่ยนเงินสดตาม notional เมื่อเกิด fill
mark แบบมีเครื่องหมายของ instrument เหล่านี้แทนสินทรัพย์หรือหนี้สิน ส่วน future และ perpetual ไม่มีการแลกเปลี่ยน
เงินสดตาม notional และใช้ pipeline ของ unrealized PnL

:::note
ยอดรวมจาก venue อาจแทนเงินสด หรือรวมมูลค่า position ไว้ด้วย Portfolio ใช้ค่าเหล่านี้โดยไม่ normalize
ดังนั้น equity ของ reported account อาจต่างจาก equity ที่ venue แสดง
:::

- snapshot รายงาน unrealized PnL แยกต่างหากสำหรับทุก position รวมถึง option
- spread combo ไม่สร้าง position ใน portfolio และไม่อัปเดต balance แบบ calculated มูลค่าอยู่ที่ leg แต่ละตัว

### ราคาสำรอง {#price-fallback}

การประเมินมูลค่าจะขอราคาจาก `Cache` ตามลำดับนี้ และหยุดที่รายการแรกที่พบ:

1. ราคา mark หาก `use_mark_prices=true` (ค่าเริ่มต้น) ใน `PortfolioConfig` และมีราคา mark อยู่ใน cache
2. quote ตามฝั่ง: `BID` สำหรับ long, `ASK` สำหรับ short
3. ราคา trade ล่าสุด
4. ราคาปิดของ bar ล่าสุดใน cache (มีค่าเมื่อ `bar_updates=true`)

ตั้ง `use_mark_prices=false` เพื่อข้ามขั้น mark และเริ่มจาก quote ตามฝั่ง

ราคาที่น้อยกว่าหรือเท่ากับศูนย์จะนับเป็นราคาปัจจุบันเฉพาะ instrument ที่อนุญาตราคาติดลบ (future, option,
futures spread, option spread และ spot commodity) ซึ่งไม่รวม instrument แบบ inverse ที่ไม่ใช่ premium เพราะ notional
ของมันหารด้วยราคา quote ฝั่งที่มีราคาน้อยกว่าหรือเท่ากับศูนย์ต้องมีขนาดไม่เป็นศูนย์ด้วย เพราะฝั่งที่ว่างอาจมาในรูป
ราคาศูนย์และขนาดศูนย์ มิฉะนั้น Portfolio จะข้ามราคานั้นและไปยังแหล่งถัดไป

หากทั้งสี่แหล่งไม่ให้ราคาปัจจุบัน Portfolio จะใช้ราคาที่ถูกต้องล่าสุดของ instrument และฝั่ง position นั้นต่อไป
และ snapshot ถัดไปจะระบุ instrument ไว้ใน `stale_instruments` หาก position ไม่เคยมีราคาที่ถูกต้องเลย จะถูกใส่ใน
missing-price tracker, ระบุไว้ใน `unpriced_instruments` และไม่นำมารวมในผลรวม

### การแปลงเป็น base currency {#base-currency-conversion}

เมื่อ `convert_to_account_base_currency=true` (ค่าเริ่มต้น) และ account มีการตั้ง `base_currency` ค่าใน cost currency
จะถูกแปลงเป็น base currency โดยใช้ xrate แบบ `MID` จาก `Cache.get_xrate()` เมื่อ `use_mark_xrates=true` จะใช้ mark
xrate ใน cache จาก `Cache.get_mark_xrate()` ก่อน และ fallback ไปใช้ `MID` หากไม่มี dictionary ผลลัพธ์จะมี key เดียว
ตรงกับ base currency

เมื่อ `convert_to_account_base_currency=false` หรือ account ไม่มี `base_currency` ผลลัพธ์จะใช้ cost currency ดั้งเดิม
ของแต่ละ position เป็น key และไม่มีการแปลง xrate

หากไม่มี xrate ปัจจุบันสำหรับการแปลงที่จำเป็น Portfolio จะใช้อัตราที่ถูกต้องล่าสุดต่อไป และระบุสกุลเงินต้นทางไว้ใน
`stale_currencies` หากไม่เคยมีอัตราที่ถูกต้องเลย position จะถือว่าประเมินราคาไม่ได้และถูก flag ผ่าน
missing-price tracker โดยจะไม่ถูกประเมินมูลค่าด้วยอัตรา 1.0 อย่างเงียบ ๆ

### Metadata การประเมินมูลค่าของ snapshot {#snapshot-valuation-metadata}

`PortfolioSnapshot.total_equity` ให้รายละเอียด MTM รายสกุลเงินเสมอ เมื่อเปิดการแปลงเป็น base currency และ
account มี base currency `base_currency_equity` จะให้ค่าสเกลาร์หลักในสกุลเงินนั้น และเป็น `None` เมื่อปิดการแปลง
หรือ account ไม่มี base currency

`is_stale` เป็น true เมื่อ snapshot ใช้ราคาหรือ xrate ที่ค้างไว้ หรือไม่รวม position ที่ไม่เคยมีข้อมูลสำหรับประเมินมูลค่า
ครบถ้วน ฟิลด์ที่เกี่ยวข้องจะระบุสาเหตุ:

- `stale_instruments`: instrument ที่ประเมินมูลค่าด้วยราคาที่ค้างไว้
- `stale_currencies`: สกุลเงินต้นทางที่แปลงด้วย xrate ที่ค้างไว้
- `unpriced_instruments`: instrument ที่ไม่ถูกนับรวม เพราะไม่เคยมีการประเมินมูลค่าที่ถูกต้องครบถ้วนเลย

เรียก `build_snapshot(account_id)` เพื่อเก็บตัวอย่างตามต้องการ และเรียก `snapshots(account_id)` เพื่ออ่านลำดับ snapshot
ที่บันทึกไว้ (มีขอบเขตจำกัด) เมธอดเหล่านี้ใช้ได้จาก Portfolio และ Strategy API ของ Rust การ build snapshot ไม่ได้เพิ่ม
snapshot นั้นลงในลำดับที่บันทึกไว้ มีเพียงการ emit ตาม lifecycle ที่ตั้งค่าไว้เท่านั้นที่บันทึก snapshot

### Equity curve อัตโนมัติ {#automatic-equity-curve}

`PortfolioConfig.equity_curve=true` (ค่าเริ่มต้น) จะบันทึกและ publish snapshot แบบ mark-to-market:

- เมื่อแต่ละ account ลงทะเบียน
- ทุกเที่ยงคืนตามเวลา UTC แม้ account จะไม่มี position
- เมื่อ backtest หรือ live node shut down

ตั้ง `equity_curve=false` สำหรับ workload ที่ไม่ใช้ equity curve เช่นการรัน optimizer การคำนวณ `equity()` และ
`build_snapshot()` ตามต้องการยังคงใช้ได้

ค่า `snapshot_interval_ms` ที่แยกต่างหากยังเป็นแบบ opt-in เมื่อตั้งค่าไว้ จะเพิ่ม snapshot แบบละเอียดเฉพาะช่วงที่
account มี position เปิดอยู่

### การติดตามราคาที่ขาดหาย {#missing-price-tracking}

tracker เก็บชุด instrument ที่ขาดราคาล่าสุดของแต่ละ scope ของ query ที่กรองตาม account และ scope ของ venue
ที่ไม่ได้กรอง `missing_price_instruments(venue)` คืน union ของทั้งหมดในระดับ venue ส่ง `account_id` เพื่อคืนเฉพาะ
ชุดปัจจุบันของ account นั้น

ผลการสังเกตแต่ละครั้งถือเป็นข้อมูลที่เชื่อถือได้จนกว่า scope เดียวกันจะรันอีกครั้ง ผลที่กรองแล้วไม่ได้ประกาศว่า
ผลแบบไม่กรองก่อนหน้าได้รับการแก้ไขแล้ว tracker มีพฤติกรรมที่สังเกตได้สองแบบ:

- log แบบ warning จะออกหนึ่งครั้งต่อ instrument เมื่อมีอย่างน้อยหนึ่ง scope รายงาน instrument นั้นเป็นครั้งแรก
  หลังจากที่ไม่มี scope ใดรายงาน warning จะไม่ซ้ำในการเรียกครั้งถัด ๆ ไป เมื่อทุก scope ที่รายงานสังเกตเห็นการ
  กลับมาเป็นปกติแล้ว หากราคาหายไปอีกในอนาคตจะ warning ใหม่
- เมื่อ venue ไม่มี position (flat) entry ของ venue ใน tracker จะถูกล้าง เพื่อไม่ให้ instrument เก่ายังคงถูก flag อยู่

`build_snapshot(account_id)` อัปเดต scope ของ account นั้น ส่วน snapshot ที่บันทึกโดย equity curve หรือ
`snapshot_interval_ms` จะไม่อัปเดต tracker แต่จะรายงาน position ที่ไม่มีราคาไว้ใน `unpriced_instruments`

เรียก `missing_price_instruments(venue)` เพื่อตรวจสอบชุดปัจจุบัน

:::tip
หาก `equity()` ต่ำกว่าที่คาดไว้ ให้ตรวจสอบ `missing_price_instruments(venue)` ก่อนไปตรวจสอบสูตรคำนวณ
instrument ที่ไม่มีราคา mark, quote, trade หรือ bar ที่ใช้ได้จะไม่ถูกนับรวมในยอดรวม และจะปรากฏใน
missing-price tracker
:::

### Scope ของ venue และ account {#venue-and-account-scope}

query แบบ collection รับ scope `venue` และ `account_id` (ไม่บังคับ) หากระบุทั้งสองค่า account ต้องออกภายใต้ venue
นั้นหรือถือ position อยู่ที่ venue นั้น มิฉะนั้น query จะเกิด error เมื่อไม่ระบุ `account_id` query ระดับ venue จะรวม
ค่าจากทุก account บน venue นั้น

`balances_locked`, `instrument_initial_margins` และ `instrument_maintenance_margins` อ่านจาก account เดียวแทน
หากไม่ระบุ `account_id` จะหาค่าได้เฉพาะเมื่อมี account ที่ออกภายใต้ venue นั้นเพียงหนึ่งเดียว และคืน `None` ในกรณีอื่น

การประเมินมูลค่าที่กรองตาม account จะ reconcile เฉพาะผลการสังเกตของ account นั้น flag ที่ account อื่นบน venue
เดียวกันตั้งไว้จึงยังคงอยู่

## สถิติของ portfolio {#portfolio-statistics}

`Portfolio.statistics()` คำนวณค่า `PortfolioStatistics` ใหม่จากทุก account, position ใน cache, position snapshot,
PnL ณ เวลาปิดที่บันทึกไว้ และ portfolio snapshot โดยคำนวณใหม่ทุกครั้งที่เรียก จึงควรเรียกให้น้อยที่สุดใน hot path

ผลลัพธ์ประกอบด้วย:

- สถิติ PnL ของแต่ละสกุลเงิน
- สถิติผลตอบแทนจาก return series ที่เลือกใช้ตามที่อธิบายด้านล่าง
- สถิติทั่วไปที่ได้จาก position

ชุดเริ่มต้นประกอบด้วย `WinRate`, `ProfitFactor`, `SharpeRatio` และ `LongRatio` ดูประเภทสถิติในตัวทั้งหมดได้ที่
[Rust API docs](https://nautilustrader.io/docs/core-latest/)

ส่งประเภทในตัวอื่น ๆ เช่น `MaxDrawdown` ให้ `Portfolio.register_statistic()` เพื่อเพิ่มลงใน `Portfolio.statistics()`,
ผลลัพธ์ของ backtest และ log หลังการรัน `PortfolioAnalyzer` ที่สร้างแยกต่างหากจะเก็บการลงทะเบียนของตัวเอง
ซึ่งไม่ส่งผลไปถึง Portfolio

หลัง backtest `engine.get_result()` จะเปิดเผยหมวดเหล่านี้ผ่าน `stats_pnls`, `stats_returns` และ `stats_general`
พร้อม `returns_series` ที่เลือกใช้ เมื่อ `run_analysis=true` engine จะ log สถิติทั้งสามหมวดภายใต้
`PORTFOLIO PERFORMANCE` หลังการรันด้วย

### สถิติแบบกำหนดเอง {#custom-statistics}

สร้างสถิติของตัวเองบนพื้นฐาน `PortfolioStatistic` และ override เมธอดคำนวณสำหรับหมวด input ที่สถิตินั้นรองรับ
แต่ละหมวดได้รับข้อมูลชุดเดียวกับที่สถิติในตัวใช้ และสถิติจะให้ค่าเฉพาะในหมวดที่ override เมธอดที่ตรงกัน
`Portfolio.statistics()` ป้อนข้อมูลให้สามหมวดแรก ผลลัพธ์ที่เทียบกับ benchmark มาจาก
`PortfolioAnalyzer.get_performance_stats_returns_vs_benchmark()` ซึ่งรับ benchmark series จากผู้เรียก

| Method                                  | Input                                        | หมวดของผลลัพธ์             |
| --------------------------------------- | -------------------------------------------- | ------------------------ |
| `calculate_from_returns`                | ผลตอบแทนที่ใช้ UNIX nanoseconds เป็น key       | `returns`                |
| `calculate_from_realized_pnls`          | realized PnL ของสกุลเงินเดียว เรียงจากเก่าไปใหม่  | `pnls`                   |
| `calculate_from_positions`              | position และ position snapshot               | `general`                |
| `calculate_from_returns_with_benchmark` | ผลตอบแทนของ strategy และผลตอบแทนของ benchmark | `PortfolioAnalyzer` เท่านั้น |

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

ชื่อเริ่มต้นคือชื่อ class ที่แยกตามขอบเขตคำ เช่น `TradeCount` จะลงทะเบียนเป็น "Trade Count" override property
`name` เพื่อกำหนดชื่อเอง การลงทะเบียนสถิติที่มีชื่อซ้ำกับตัวที่มีอยู่จะแทนที่ตัวเดิม และ
`Portfolio.deregister_statistic()` ใช้ลบสถิติตามชื่อ

การลงทะเบียนจะคงอยู่ข้ามการเรียก `Portfolio.statistics()` และการ reset state ของ analyzer จึงส่งผลถึงผลลัพธ์ของ
backtest และ log หลังการรันตลอดทั้งการรัน ให้ลงทะเบียนก่อนการรันเพื่อให้ครอบคลุมทุก query

คืนค่าเป็นตัวเลขทศนิยม หรือ `None` เมื่อ metric ไม่มีนิยามสำหรับข้อมูลที่ได้รับ กำหนดผลลัพธ์สำหรับข้อมูลว่างหรือ
ไม่เพียงพอให้ชัดเจน: คืน `None` เมื่อไม่ทราบค่า metric หรือใช้ค่าเริ่มต้นที่เหมาะกับโดเมน เช่น `0.0`
สถิติที่เกิด error หรือคืนค่าที่ไม่ใช่ตัวเลขจะ log error และไม่ให้ค่าสำหรับหมวดนั้น สถิติที่เหลือยังคงคำนวณต่อไป

ทุกหมวดจะถูกเรียกสำหรับสถิติที่ลงทะเบียนไว้ รวมถึงการรันที่ไม่มีการปิด trade เลย ซึ่งหมวด PnL จะได้รับ list ว่าง

:::warning
เมธอดคำนวณทำงานในขณะที่ Portfolio กำลัง borrow state ภายในอยู่ ดังนั้นการเรียกเมธอดของ Portfolio ที่แก้ไข state
จากภายในเมธอดคำนวณจะทำให้เกิด panic ให้สถิติเป็น pure function ของข้อมูลที่ได้รับเท่านั้น
:::

## ผลตอบแทน: position เทียบกับ portfolio {#returns-position-vs-portfolio}

analyzer ติดตาม return series สองชุดที่แตกต่างกัน:

- **Position returns** (`analyzer.position_returns()`) วัด realized return ต่อ position เป็นผลตอบแทนจากราคา
  ที่คำนึงถึงฝั่ง เทียบกับราคาเปิดเฉลี่ย สะท้อนการเคลื่อนไหวของราคา instrument ระหว่างเข้าและออก โดยไม่ขึ้นกับ
  ขนาด account หรือ leverage
- **Portfolio returns** (`analyzer.portfolio_returns()`) วัดการเปลี่ยนแปลงรายวันเป็นเปอร์เซ็นต์ของ equity ของ
  account แบบ mark-to-market กำไร $900 บน account ขนาด $100,000 จะรายงานประมาณ 0.9% สำหรับวันนั้น

เมื่อ portfolio snapshot ที่สมบูรณ์ครอบคลุม **อย่างน้อยสองวันที่ตาม UTC ที่ต่างกัน** analyzer จะใช้ snapshot สุดท้าย
ของแต่ละวันและคำนวณ portfolio returns โดยอัตโนมัติ และใช้เป็น series หลักสำหรับสถิติ, tearsheet และ heatmap
ผลตอบแทนรายเดือน

การสุ่มตัวอย่างรายวันเป็นไปตามกฎเหล่านี้:

- snapshot ที่ emit ตรงเที่ยงคืน UTC จะปิดวันก่อนหน้า ทำให้ระดับรายวันสอดคล้องกับ sample แบบละเอียด
- sample ที่ถูกต้องตัวแรกตอนลงทะเบียนจะเป็นจุดยึดค่าเปิดสำหรับวันแรกที่ไม่ครบวัน
- วันที่ account ขาดหายหรือไม่มีราคาจะถูก forward-fill หลังจากแต่ละ account มี sample ที่ถูกต้องตัวแรกแล้ว
- snapshot หลายตัวในวันเดียวกันนับเป็นหนึ่งวัน ดังนั้นการเทรดภายในวันอย่างเดียวจะไม่ทำให้เกิด portfolio returns

เมื่อไม่มี portfolio returns analyzer จะ fallback ไปใช้ position returns

accessor `analyzer.returns()` ช่วยเลือกตามลำดับความสำคัญนี้ให้: ใช้ portfolio returns เมื่อมี มิฉะนั้นใช้
position returns

### Account หลายสกุลเงิน {#multi-currency-accounts}

portfolio returns ต้องการให้ equity ใน snapshot ของแต่ละ account สรุปเป็นสกุลเงินร่วมสกุลเดียวได้ โดยปกติการแปลงเป็น
base currency จะให้ค่าสเกลาร์นั้น เมื่อ snapshot มีหลายสกุลเงิน หรือ account สรุปเป็นสกุลเงินต่างกัน analyzer จะ
fallback ไปใช้ position returns การระบุสกุลเงินของ tearsheet อย่างชัดเจนสามารถเลือก equity รายสกุลเงินที่ตรงกันได้
เมื่อมีข้อมูล

สำหรับ portfolio returns แบบหลายสกุลเงิน ให้แปลง balance เป็นสกุลเงินร่วมก่อน แล้วคำนวณการเปลี่ยนแปลงเป็น
เปอร์เซ็นต์ภายนอก

### การคำนวณแบบหลาย account {#multi-account-calculation}

การวิเคราะห์ backtest จะรวมทุก account ใน cache หลังจากแปลงเป็นสกุลเงินร่วมแล้ว tearsheet ใช้กฎการรวมทั้ง account
แบบเดียวกันสำหรับ backtest หลาย venue

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Positions](positions.md): การติดตาม position ภายใน portfolio
- [Reports](reports.md): การสร้างรายงานวิเคราะห์ portfolio
- [Visualization](visualization.md): การแสดงผลประสิทธิภาพของ portfolio
