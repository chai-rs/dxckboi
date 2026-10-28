---
title: "Testing"
description: "unit test สำหรับกฎการเทรด, integration test ผ่าน engine จริง, การตรวจว่าผลซ้ำได้ และกับดัก floating point"
---

test ของระบบเทรดมีสองหน้าที่: ยืนยันว่า logic ทำตามที่ออกแบบ และจับการเปลี่ยนแปลงพฤติกรรมเมื่ออัปเกรด Nautilus หรือแก้โค้ด
โปรเจกต์นี้แบ่ง test เป็นสองชั้นตามการวางโมดูลใน [ตั้งค่าโปรเจกต์](project_setup.md#module-design)

```bash
$ cargo test --release
test signal::tests::close_below_lower_band_goes_long_toward_middle ... ok
test signal::tests::close_above_upper_band_goes_short ... ok
test signal::tests::close_inside_bands_does_nothing ... ok
test signal::tests::too_little_reward_is_skipped ... ok
test sizing::tests::risks_the_requested_fraction ... ok
test sizing::tests::notional_cap_wins_when_smaller ... ok
test sizing::tests::venue_maximum_is_respected ... ok
test sizing::tests::rounds_down_to_the_size_increment ... ok
test sizing::tests::below_venue_minimum_is_not_traded ... ok
test sizing::tests::zero_stop_distance_is_not_sized ... ok
test result: ok. 10 passed; 0 failed

test every_entry_is_a_full_bracket ... ok
test no_profit_without_an_edge ... ok
test same_inputs_give_same_results ... ok
test result: ok. 3 passed; 0 failed

test exporter_receives_every_fill ... ok
test split_design_trades_exactly_like_the_single_strategy ... ok
test writer_thread_finishes_after_the_run ... ok
test result: ok. 3 passed; 0 failed
```

## ชั้นที่ 1: unit test ของกฎ [#unit-tests]

กฎการเทรดและ position sizing เป็น pure function จึง test ด้วยตัวเลขที่คำนวณด้วยมือได้ทุกกรณี:

```rust
const BANDS: Bands = Bands {
    upper: 1.0100,
    middle: 1.0000,
    lower: 0.9900,
    atr: 0.0020,
};

#[test]
fn close_below_lower_band_goes_long_toward_middle() {
    let signal = evaluate(0.9890, BANDS, 1.5, 1.0);
    assert_eq!(
        signal,
        Signal::Long {
            take_profit: 1.0000,
            stop_loss: 0.9890 - 0.0030
        }
    );
}

#[test]
fn too_little_reward_is_skipped() {
    let narrow = Bands {
        upper: 1.0010,
        middle: 1.0000,
        lower: 0.9990,
        atr: 0.0020,
    };
```

```rust
const OPEN: SizeLimits = SizeLimits {
    increment: 1.0,
    min_units: 0.0,
    max_units: f64::INFINITY,
    max_notional: 1e9,
};

#[test]
fn venue_maximum_is_respected() {
    let limits = SizeLimits {
        max_units: 150_000.0,
        ..OPEN
    };

#[test]
fn zero_stop_distance_is_not_sized() {
    assert_eq!(units_for_risk(100_000.0, 0.01, 0.7, 0.7, OPEN), 0.0);
}
```

test ชั้นนี้ควรครอบคลุม **กรณีขอบ** มากกว่ากรณีปกติ เช่น ระยะ stop เป็นศูนย์, ขนาดต่ำกว่าขั้นต่ำของ venue
หรือเกินขั้นสูง กรณีเหล่านี้แทบไม่เกิดในข้อมูลทดสอบทั่วไป แต่เกิดใน live ในวันที่ตลาดผันผวน
`venue_maximum_is_respected` เพิ่มเข้ามาหลังจากเจอปัญหา `QUANTITY_EXCEEDS_MAXIMUM`
([Order และความเสี่ยง](orders_and_risk.md#instrument-limits)) บั๊กที่เจอแล้วทุกตัวควรมี test กันไม่ให้กลับมาอีก

## กับดัก floating point [#floating-point]

เวอร์ชันแรกของ `units_for_risk` ปัดเศษแบบนี้:

```rust
let units = (units / limits.increment).floor() * limits.increment;
```

แล้ว test ที่ตรงไปตรงมาที่สุดกลับพัง:

```text
test sizing::tests::risks_the_requested_fraction ... FAILED
```

1% ของ 100,000 คือ 1,000 หารด้วยระยะ stop 0.7000 − 0.6950 = 0.0050 ต้องได้ 200,000 พอดี
แต่ใน `f64` นั้น `0.7000 - 0.6950` ได้ `0.0050000000000000044` ผลหารจึงเป็น 199,999.99… และ `floor` ปัดลงเป็น 199,999

ในกรณีนี้ขาดไปหนึ่ง unit จึงไม่เสียหาย แต่ถ้า increment เป็น 1,000 units ขนาดจะหายไปทั้งก้อน และถ้าขนาดอยู่ใกล้ขั้นต่ำของ venue
order อาจหายไปทั้ง order ทางแก้ที่โปรเจกต์นี้ใช้คือเพิ่ม epsilon เล็ก ๆ ก่อน floor:

```rust
let units = (units / limits.increment + 1e-9).floor() * limits.increment;
```

ทางที่แม่นยำกว่าคือคำนวณด้วย `rust_decimal::Decimal` ตลอดทาง แล้วใช้ `instrument.make_qty_from_decimal`
ซึ่งเหมาะเมื่อต้องการให้ผลตรงกับ exchange ทุกหลักทศนิยม ส่วนกรณีทั่วไปใช้ `f64` กับ epsilon ก็เพียงพอ
**ไม่ว่าเลือกแบบไหน ต้องมี test ที่ใช้ตัวเลขซึ่งรู้คำตอบแน่นอน** เพราะบั๊กแบบนี้ไม่มี error เตือน

## ชั้นที่ 2: integration test ผ่าน engine [#integration-tests]

unit test บอกไม่ได้ว่า strategy ต่อกับ engine ถูกต้อง เช่น subscribe ถูก bar type, bracket ผ่าน risk engine,
หรือ hook ถูกเรียกจริง integration test จึงรัน backtest จริงด้วยฟังก์ชัน `run` ตัวเดียวกับที่ binary ใช้:

```rust
const PARAMS: Params = Params {
    bb_period: 20,
    bb_k: 2.0,
    atr_stop: 1.5,
};
const DAYS: usize = 24 * 60;

fn pnl(result: &BacktestResult) -> f64 {
    result.stats_pnls["USD"]["PnL (total)"]
}

#[test]
fn every_entry_is_a_full_bracket() {
    let result = run(PARAMS, Market::MeanReverting, 5 * DAYS, LevelFilter::Off).unwrap();
    assert!(result.total_positions > 0, "expected the strategy to trade");
    // Entry + take profit + stop loss, and nothing submitted twice.
    assert_eq!(result.total_orders, 3 * result.total_positions);
}

#[test]
fn same_inputs_give_same_results() {
    let a = run(PARAMS, Market::MeanReverting, 5 * DAYS, LevelFilter::Off).unwrap();
    let b = run(PARAMS, Market::MeanReverting, 5 * DAYS, LevelFilter::Off).unwrap();
    assert_eq!(a.total_orders, b.total_orders);
    assert_eq!(pnl(&a), pnl(&b));
}

#[test]
fn no_profit_without_an_edge() {
    // A pure random walk has no mean reversion to exploit, so costs must win.
    let result = run(PARAMS, Market::RandomWalk, 5 * DAYS, LevelFilter::Off).unwrap();
    assert!(result.total_positions > 0);
    assert!(
        pnl(&result) < 0.0,
        "profit on a random walk: {}",
        pnl(&result)
    );
}
```

แต่ละ assertion ตรวจคนละเรื่อง:

| Assertion | จับปัญหาอะไร |
| --- | --- |
| `total_positions > 0` | strategy ไม่เทรดเลย เช่น subscribe ผิด bar type หรือ order ถูก deny ทั้งหมด |
| `total_orders == 3 * total_positions` | order ถูกส่งซ้อน หรือ bracket ไม่ครบชุด |
| รันซ้ำได้ผลเท่าเดิม | มีความไม่แน่นอนแอบอยู่ เช่น fill model ไม่ได้ใส่ seed หรือใช้เวลาจริงในโค้ด |
| ขาดทุนบน random walk | backtest สร้างกำไรจากบั๊ก ([control market](backtest.md#control-market)) |

ข้อสังเกตเรื่องการเขียน:

- **ใช้ข้อมูลสั้น** 5 วัน (7,200 bar) พอให้ strategy เทรดหลายร้อยครั้ง test ทั้งไฟล์รันราว 1 วินาทีใน release mode
- **หนึ่ง test ต่อหนึ่งคำถาม** แต่ละ test function สร้าง engine ของตัวเอง cargo รัน test เหล่านี้ขนานกันได้
  (ทดลองแล้วว่า engine หลายตัวที่รันพร้อมกันคนละ thread ไม่รบกวนกัน) เมื่อ test พัง ชื่อ test ก็บอกทันทีว่าคุณสมบัติข้อไหนเสีย
- **ปิด log** ด้วย `LevelFilter::Off` ไม่อย่างนั้น output ของ test จะยาวหลายพันบรรทัด
- **ไม่ assert ตัวเลข PnL ตายตัว** ตัวเลขเปลี่ยนได้เมื่ออัปเกรด Nautilus (เช่น fill model ปรับวิธีคำนวณ)
  test ที่ assert เป็นความสัมพันธ์ (ซ้ำได้, ติดลบ, สัดส่วนของ order) ทนต่อการอัปเกรดกว่า และยังจับบั๊กจริงได้

## Test ของการ integrate [#integration-level-tests]

`tests/integration.rs` ตรวจสิ่งที่เกิดขึ้นเมื่อแยก component และต่อระบบภายนอก ซึ่งอธิบายละเอียดในหน้า
[Integration](integration.md):

| Test | ยืนยันว่า |
| --- | --- |
| `split_design_trades_exactly_like_the_single_strategy` | แยก logic เป็น actor + executor แล้วผลเท่ากับ strategy ตัวเดียวทุกหลัก |
| `exporter_receives_every_fill` | ทุก fill ออกมาถึงปลายทาง ตรวจด้วยปริมาณซื้อรวมเท่ากับขายรวม ไม่ใช่จำนวน fill |
| `writer_thread_finishes_after_the_run` | thread ปลายทางจบได้หลัง backtest จบ (ใช้ deadline แทน `join()` เปล่า ๆ เพื่อให้ test fail แทนที่จะค้าง) |

test แบบนี้คือ "ตาข่าย" ของการ refactor และการอัปเกรด ถ้าวันหนึ่ง Nautilus เปลี่ยนลำดับการส่ง message หรือวิธี dispose
component test ชุดนี้จะบอกก่อนที่ระบบจริงจะเสียหาย

## Test handler ของ strategy โดยตรง [#handler-tests]

ถ้าต้องการ test พฤติกรรมของ handler ทีละตัวโดยไม่รัน backtest ทั้งชุด ก็ลงทะเบียน strategy กับ component ที่สร้างเอง
(`VirtualClock`, `Cache`, `Portfolio`) ผ่าน `StrategyCore::register` แล้วเรียก `on_bar` ด้วย bar ที่สร้างเองได้
ตัวอย่างอยู่ใน test ของ strategy `EmaCross` ใน repository ของ NautilusTrader
(`crates/trading/src/examples/strategies/ema_cross/tests.rs`) วิธีนี้ละเอียดกว่าแต่ต้องเขียนโค้ดเตรียมมากกว่า
สำหรับ strategy ขนาดนี้ สองชั้นข้างบนก็ครอบคลุมเพียงพอ

## ใน CI [#ci]

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --release
cargo build --release --features live   # ยืนยันว่าส่วน live ยัง compile ได้
```

บรรทัดสุดท้ายสำคัญเพราะ binary ของ live ไม่ถูก compile ตอน `cargo test` ปกติ ถ้าไม่ build แยก
การเปลี่ยน API ใน strategy อาจทำให้ส่วน live พังโดยไม่มีใครรู้จนถึงวันที่จะ deploy

ขั้นต่อไป: [ไปสู่ live](live.md)
