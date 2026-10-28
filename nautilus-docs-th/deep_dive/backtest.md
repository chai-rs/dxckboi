---
title: "Backtest: ประกอบ engine และอ่านผล"
description: "ตั้งค่า venue จำลอง, fill model, fee model, อ่าน BacktestResult, parameter sweep และการใช้ control market พิสูจน์ผล"
---

หน้านี้อธิบาย `src/backtest.rs` ซึ่งประกอบ `BacktestEngine` ทั้งตัวไว้ในฟังก์ชันเดียว ทั้ง binary และ test ใช้ฟังก์ชันนี้ร่วมกัน
จึงมั่นใจได้ว่าสิ่งที่ test ตรวจคือ setup เดียวกับที่ใช้รันจริง

## ประกอบ engine [#engine]

```rust
pub fn run(
    params: Params,
    market: Market,
    bars: usize,
    log_level: LevelFilter,
) -> anyhow::Result<BacktestResult> {
    let instrument = InstrumentAny::CurrencyPair(audusd_sim());
    let instrument_id = instrument.id();
    let bar_type = BarType::from("AUD/USD.SIM-1-MINUTE-LAST-EXTERNAL");

    let risk = RiskEngineConfig::builder()
        .max_order_submit(RateLimit::new(10, DurationNanos::from_secs(1)))
        .max_notional_per_order(AHashMap::from([(instrument_id, Decimal::from(2_000_000))]))
        .build()?;
    let config = BacktestEngineConfig::builder()
        .logging(LoggerConfig::builder().stdout_level(log_level).build()?)
        .risk_engine(risk)
        .build();
    let mut engine = BacktestEngine::new(config)?;

    engine.add_venue(
        SimulatedVenueConfig::builder()
            .venue(Venue::from("SIM"))
            .oms_type(OmsType::Netting)
            .account_type(AccountType::Margin)
            .book_type(BookType::L1_MBP)
            .starting_balances(vec![Money::from("100_000 USD")])
            // 0.2 bps maker / 0.5 bps taker.
            .fee_model(
                FeeModelAny::MakerTaker(MakerTakerFeeModel::new(
                    Decimal::new(2, 5),
                    Decimal::new(5, 5),
                ))
                .into(),
            )
            // Limit orders fill when touched; every fill has a 30% chance of slipping one tick.
            .fill_model(FillModelAny::Default(DefaultFillModel::new(1.0, 0.3, Some(42))?).into())
            .build()?,
    )?;
    engine.add_instrument(&instrument)?;
    // ... add_data, add_strategy ...
    engine.run(None, None, None, false)?;
    Ok(engine.get_result())
}
```

ลำดับที่ต้องเป็นแบบนี้: venue → instrument → data → strategy → run
`add_data` ที่เปิด validate จะตรวจว่า instrument ของข้อมูลถูกเพิ่มไว้แล้ว

### เลือกค่าของ venue [#venue]

| ค่า | ที่เลือก | เหตุผล |
| --- | --- | --- |
| `oms_type` | `Netting` | หนึ่ง position ต่อ instrument ตรงกับ perpetual ส่วนใหญ่บน exchange คริปโต ส่วน `Hedging` ให้ถือ long และ short แยกกันได้ |
| `account_type` | `Margin` | ใช้ leverage ได้ ถ้าใช้ `Cash` การ sell ต้องมีของอยู่ในมือก่อน |
| `book_type` | `L1_MBP` | ข้อมูลมีแค่ bar จึงใช้ top-of-book พอ ไม่มีความลึกของ order book |
| `starting_balances` | 100,000 USD | ใช้คำนวณ position sizing และ PnL |

### Fee model [#fee-model]

`fee_model` เป็นค่าที่ต้องกำหนดเสมอ ไม่มี default โปรเจกต์นี้ใช้ maker 0.2 bps และ taker 0.5 bps
(`Decimal::new(2, 5)` คือ 0.00002) ถ้าต้องการทดสอบ logic ล้วน ๆ ใช้ `MakerTakerFeeModel::zero()` ได้
แต่อย่าใช้ค่าธรรมเนียมเป็นศูนย์ตอนตัดสินว่า strategy มีกำไรหรือไม่ strategy ความถี่สูงจำนวนมากดูมีกำไรเพราะไม่ได้คิดค่าธรรมเนียม

### Fill model [#fill-model]

`DefaultFillModel::new(prob_fill_on_limit, prob_slippage, random_seed)`:

- `prob_fill_on_limit = 1.0`: LIMIT order fill ทันทีที่ราคาแตะ ค่าที่ต่ำกว่า 1 จำลองการต่อคิวที่ราคาเดียวกัน
- `prob_slippage = 0.3`: กับ book แบบ L1 ทุกการ fill (ไม่ว่าชนิดของ order) มีโอกาส 30% ที่ราคาจะขยับไปในทางเสียเปรียบหนึ่ง tick
  รวมถึง take profit ที่เป็น LIMIT ด้วย
- `random_seed = Some(42)`: ทำให้การสุ่มของ fill model ซ้ำได้ ถ้าไม่ใส่ seed ผลจะต่างกันทุกครั้งและ test ที่เทียบตัวเลขจะใช้ไม่ได้

### Bar execution [#bar-execution]

venue จำลองใช้ bar ขับเคลื่อนการ match order (`bar_execution` เป็น `true` โดย default) โดยเดินราคาภายในแท่งตามลำดับ
Open → High → Low → Close ข้อจำกัดที่ต้องรู้คือ เมื่อแท่งเดียวแตะทั้ง take profit และ stop loss ของ position เดียวกัน
engine ไม่รู้ว่าอะไรเกิดก่อนจริง:

- position long จะเจอ High ก่อน take profit จึง fill ก่อน **ผลออกมาดีเกินจริง**
- position short จะเจอ High ก่อนเช่นกัน แต่ High คือ stop loss ของ short **ผลออกมาแย่เกินจริง**

`bar_adaptive_high_low_ordering(true)` ใน `SimulatedVenueConfig` เปลี่ยนลำดับตามว่า High หรือ Low อยู่ใกล้ Open มากกว่า
ช่วยได้บางส่วน แต่ทางแก้ที่แท้จริงคือใช้ข้อมูลละเอียดกว่า bar (quote หรือ trade tick) เมื่อระยะ take profit และ stop
แคบใกล้เคียงกับช่วงราคาของหนึ่งแท่ง

## Logger เป็นของทั้ง process [#logging]

ปัญหาที่สามที่เจอจริง: binary เวอร์ชันแรกรัน backtest หนึ่งรอบแบบมี log แล้วรัน sweep ต่อในโปรแกรมเดียวกัน
โดยตั้ง `stdout_level` เป็น `Off` ผลคือ log ของ sweep ยังออกมาเต็มจอ (รวมกว่า 60 MB)

สาเหตุคือ logger ของ Nautilus เป็น global ของ process engine ตัวแรกที่ถูกสร้างเป็นตัวตั้งค่า logger
engine ตัวต่อ ๆ มาเปลี่ยนระดับ log ไม่ได้ โปรเจกต์นี้จึงแยกเป็นสองคำสั่ง แต่ละคำสั่งใช้ระดับ log เดียวตลอด process:

```bash
cargo run --release -- run      # หนึ่งรอบ, log ระดับ Info
cargo run --release -- sweep    # หลายรอบ, log ปิด
```

## อ่านผลลัพธ์ [#results]

`engine.get_result()` คืน `BacktestResult`:

| ฟิลด์ | ความหมาย |
| --- | --- |
| `total_orders`, `total_positions` | จำนวน order และ position ทั้งหมด |
| `stats_pnls` | สถิติ PnL แยกตามสกุลเงิน เช่น `stats_pnls["USD"]["PnL (total)"]` |
| `stats_returns` | สถิติของผลตอบแทน (Sharpe, Sortino, volatility ฯลฯ) |
| `stats_general` | สถิติทั่วไปอื่น ๆ |
| `returns_series` | ผลตอบแทนตามเวลา (`BTreeMap<UnixNanos, f64>`) |
| `elapsed_time_secs` | **ช่วงเวลาของข้อมูลที่จำลอง** (เวลาสิ้นสุด − เวลาเริ่มของ backtest) ไม่ใช่เวลาที่ใช้รัน |

ชื่อ `elapsed_time_secs` ชวนให้เข้าใจผิด ใน backtest 30 วันค่านี้คือราว 2.59 ล้านวินาที
ถ้าต้องการเวลาที่ใช้รันจริงให้วัดเองด้วย `std::time::Instant` อย่างที่ `main.rs` ทำ

นอกจากนี้ engine จะ log สรุปผลเต็มรูปแบบหลังรันเสร็จเอง (เปิดอยู่โดย default ผ่าน `run_analysis`)

## ผลการรัน [#run-output]

```bash
$ cargo run --release -- run
orders=6012 positions=2004 pnl_total=802602.31 simulated_days=30.0 wall_time=8.80s
```

ข้อมูล 30 วัน (43,200 bar) รันเสร็จในไม่ถึง 10 วินาทีบนเครื่อง 4 core สถิติจาก log:

| สถิติ | ค่า |
| --- | --- |
| Win rate | 0.65 |
| Avg winner | 1,021.16 USD |
| Avg loser | −755.70 USD |
| Max loser | −1,003.37 USD |

กำไร 800,000 USD จากทุน 100,000 USD ใน 30 วัน **ไม่สมจริง** และไม่ควรเชื่อ เหตุผลมีสองข้อ:

1. ข้อมูลถูกสร้างให้มี mean reversion ซึ่งเป็นสิ่งที่ strategy นี้หากิน ตลาดจริงไม่ได้เป็นแบบนี้ตลอด
2. position sizing ตาม equity ทำให้กำไรทบต้น ขนาด position โตขึ้นตามทุน

ถ้าอย่างนั้น ตัวเลขนี้บอกอะไรได้บ้าง

## Control market: พิสูจน์ว่ากำไรมาจากไหน [#control-market]

backtest ที่ทำกำไรสูงผิดปกติมีสาเหตุได้สองแบบ แบบแรกคือข้อมูลมีรูปแบบที่ strategy ใช้ประโยชน์ได้จริง
แบบที่สองคือ backtest มีบั๊ก เช่น look-ahead bias, fill ที่ราคาดีเกินจริง หรือค่าธรรมเนียมไม่ถูกคิด

วิธีแยกสองแบบนี้คือรัน strategy เดิมบนตลาดที่ **รู้แน่นอนว่าไม่มี edge** ในโปรเจกต์นี้คือ random walk ล้วน
(`Market::RandomWalk` ตั้ง `reversion` เป็น 0) ในตลาดแบบนี้ strategy ที่ไม่มีบั๊กต้องขาดทุนเท่ากับต้นทุนการเทรดโดยประมาณ:

```bash
$ cargo run --release -- run random
orders=7602 positions=2534 pnl_total=-54102.99 simulated_days=30.0 wall_time=11.28s
```

ขาดทุน 54,000 USD จากค่าธรรมเนียมและ slippage ตามที่คาด ถ้าผลตรงนี้ออกมาเป็นกำไร ก็แปลว่า backtest มีบั๊กที่สร้างเงินขึ้นมาเอง
และกำไรในตลาดแรกก็เชื่อไม่ได้เช่นกัน

## Parameter sweep [#parameter-sweep]

```bash
$ cargo run --release -- sweep
market         bb_period  bb_k  atr_stop  positions   pnl_total
MeanReverting         20     2         1       2420   748558.59
MeanReverting         20     2         2       1770   784212.48
MeanReverting         20   2.5         1        724   288851.18
MeanReverting         20   2.5         2        629   311022.43
MeanReverting         40     2         1       1907   729213.90
MeanReverting         40     2         2       1315   721854.71
MeanReverting         40   2.5         1        808   402454.37
MeanReverting         40   2.5         2        625   430139.15
RandomWalk            20     2         1       3126   -74761.39
RandomWalk            20     2         2       2187   -33910.27
RandomWalk            20   2.5         1        932   -34602.98
RandomWalk            20   2.5         2        779   -11580.10
RandomWalk            40     2         1       2636   -69636.60
RandomWalk            40     2         2       1694   -35761.27
RandomWalk            40   2.5         1       1080   -41527.48
RandomWalk            40   2.5         2        800   -27126.22
```

วิธีอ่านตาราง:

- **ผลไปทางเดียวกันทุกชุดพารามิเตอร์** ตลาด mean reversion ได้กำไรทุกแถว random walk ขาดทุนทุกแถว
  แปลว่าผลขึ้นกับคุณสมบัติของตลาด ไม่ได้ขึ้นกับการเลือกพารามิเตอร์ที่บังเอิญดี
- **`bb_k` สูงขึ้น เทรดน้อยลง** (band กว้างขึ้น ราคาหลุดยากขึ้น) ทั้งกำไรในตลาดแรกและขาดทุนในตลาดที่สองลดลง
- **ขาดทุนใน random walk แปรตามจำนวนเทรด** แถวที่เทรดมากขาดทุนมาก ยืนยันว่าขาดทุนมาจากต้นทุนการเทรด

ข้อควรระวังเมื่อทำ sweep กับข้อมูลจริง:

- อย่าเลือกพารามิเตอร์จากแถวที่ดีที่สุดแถวเดียว ให้มองหาบริเวณที่ผลดีต่อเนื่องกันหลายแถว
  ถ้าผลดีเฉพาะจุดเดียวแต่แถวข้างเคียงแย่ แปลว่า overfit
- แบ่งข้อมูลเป็นช่วงสำหรับเลือกพารามิเตอร์ และช่วงที่ไม่เคยแตะสำหรับยืนยันผล (out-of-sample)
- ยิ่งลองหลายชุด โอกาสเจอชุดที่ดีโดยบังเอิญยิ่งสูง ผลของชุดที่ดีที่สุดจึงมักดีเกินจริง

## ต่อยอด [#next]

- ใช้ข้อมูลจริงผ่าน `BacktestNode` และ Parquet catalog: [รัน backtest ด้วย Rust](../how_to/run_rust_backtest.md)
- ทำความเข้าใจการ match order ใน backtest: [Fill models](../concepts/backtesting/fill-models.md),
  [Bar execution](../concepts/backtesting/bar-execution.md)
- ใส่สิ่งที่หน้านี้ตรวจด้วยตาให้เป็น test อัตโนมัติ: [Testing](testing.md)
