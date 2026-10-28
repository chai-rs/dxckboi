---
title: "Order และความเสี่ยง"
description: "bracket order, position sizing ตามความเสี่ยง, ข้อจำกัดของ instrument, denied กับ rejected และการตั้งค่า risk engine"
---

หน้านี้อธิบาย `enter_bracket()` ใน `src/execution.rs` และเกราะป้องกันรอบ ๆ มัน ซึ่งเป็นส่วนที่พลาดแล้วเสียเงินจริง
ฟังก์ชันนี้รับ strategy ใดก็ได้ ทั้ง `MeanReversion` และ `SignalExecutor` ในหน้า [Integration](integration.md) เรียกใช้ตัวเดียวกัน

## ภาพรวมของ enter_bracket() [#enter]

```rust
/// Sizes an entry from account equity and submits it as a bracket order.
pub fn enter_bracket<S>(
    strategy: &mut S,
    instrument_id: InstrumentId,
    side: OrderSide,
    entry: f64,
    take_profit: f64,
    stop_loss: f64,
    risk: RiskSettings,
) -> anyhow::Result<()>
where
    S: Strategy + StrategyNative + DataActorNative,
{
    let Some(instrument) = strategy.cache().instrument(&instrument_id) else {
        anyhow::bail!("instrument {instrument_id} not in cache");
    };
    let quote = instrument.quote_currency();
    let equity = strategy
        .cache()
        .account_for_venue(&instrument_id.venue)
        .and_then(|account| account.balance_total(Some(quote)))
        .map_or(0.0, |money| money.as_f64());

    let limits = SizeLimits {
        increment: instrument.size_increment().as_f64(),
        min_units: instrument.min_quantity().map_or(0.0, |q| q.as_f64()),
        max_units: instrument
            .max_quantity()
            .map_or(f64::INFINITY, |q| q.as_f64()),
        max_notional: risk.max_notional,
    };
    let units = units_for_risk(equity, risk.risk_fraction, entry, stop_loss, limits);
    if units <= 0.0 {
        log::warn!("Skipping {side:?}: position size rounds to zero (equity {equity})");
        return Ok(());
    }

    let orders = strategy
        .order()
        .bracket()
        .instrument_id(instrument_id)
        .order_side(side)
        .quantity(instrument.make_qty(units, Some(true)))
        .tp_price(instrument.make_price(take_profit))
        .tp_post_only(false)
        .sl_trigger_price(instrument.make_price(stop_loss))
        .call();
    log::info!("Entering {side:?} {units} @ ~{entry:.5} tp={take_profit:.5} sl={stop_loss:.5}");
    strategy.submit_order_list(orders, None, None, None)
}
```

ทุกค่าที่ขึ้นกับ venue อ่านจาก cache ตอนส่ง order ไม่ได้เขียนตายตัว: instrument (precision, increment, ขนาดขั้นต่ำ/สูงสุด)
และยอดเงินในบัญชี เมื่อย้ายไปรันกับ venue อื่น โค้ดนี้จึงไม่ต้องแก้

`balance_total` มาจาก trait `nautilus_model::accounts::Account` จึงต้อง `use` trait นี้ก่อนเรียก
ส่วน `quote_currency`, `make_qty` และ `make_price` มาจาก trait `nautilus_model::instruments::Instrument`

## Bracket order [#bracket]

bracket คือ order สามตัวที่ผูกกัน:

| Order | ชนิด (default) | บทบาท |
| --- | --- | --- |
| entry | MARKET | เปิด position |
| take profit | LIMIT | ปิดด้วยกำไรที่ `tp_price` |
| stop loss | STOP_MARKET | ปิดด้วยขาดทุนเมื่อราคาแตะ `sl_trigger_price` |

take profit และ stop loss จะทำงานหลังจาก entry fill แล้วเท่านั้น และเมื่อตัวหนึ่ง fill อีกตัวจะถูกยกเลิก (One-Updates-Other)
`bracket()` เป็น builder: ใส่เฉพาะค่าที่ต้องการแล้วปิดด้วย `.call()` ผลลัพธ์คือ `Vec<OrderAny>` ซึ่งส่งด้วย `submit_order_list`

:::warning[tp_post_only เป็น true โดย default]
`bracket()` ตั้ง take profit เป็น post-only โดย default คือยอมเป็น maker เท่านั้น ถ้าราคาของ take profit
จะ match ทันทีที่วาง (เช่น ราคากระโดดข้ามเส้นกลางไปแล้ว) venue จะปฏิเสธ order นั้น strategy ในบทนี้จึงตั้ง
`.tp_post_only(false)` ให้ชัดเจน ถ้าต้องการเป็น maker เพื่อลดค่าธรรมเนียมก็ใช้ค่า default ได้ แต่ต้องรับมือกับการถูกปฏิเสธ
:::

entry แบบ LIMIT ทำได้ด้วย `.entry_order_type(OrderType::Limit).entry_price(price)` ถ้าต้องการคุมราคาเข้า
แต่แลกกับความเสี่ยงที่จะไม่ได้เข้าเทรดเลย

## Position sizing ตามความเสี่ยง [#position-sizing]

หลักคือกำหนดก่อนว่ายอมเสียเท่าไรถ้าโดน stop แล้วคำนวณขนาดย้อนกลับ:

```text
units = (equity × risk_fraction) / |entry − stop|
```

ตัวอย่าง equity 100,000 USD, `risk_fraction` 1% และระยะ stop 0.0050 จะได้ 1,000 / 0.0050 = 200,000 units
ถ้าโดน stop จะเสียราว 1,000 USD ไม่ว่าระยะ stop จะกว้างหรือแคบ stop แคบได้ position ใหญ่ stop กว้างได้ position เล็ก

```rust
/// Units to trade so that hitting the stop loses `risk_fraction` of `equity`,
/// within `limits`. Returns 0.0 when no valid size exists.
pub fn units_for_risk(
    equity: f64,
    risk_fraction: f64,
    entry: f64,
    stop: f64,
    limits: SizeLimits,
) -> f64 {
    let stop_distance = (entry - stop).abs();
    if equity <= 0.0 || stop_distance <= 0.0 || entry <= 0.0 || limits.increment <= 0.0 {
        return 0.0;
    }
    let by_risk = equity * risk_fraction / stop_distance;
    let units = by_risk
        .min(limits.max_notional / entry)
        .min(limits.max_units);
    // f64 subtraction leaves tiny errors (0.7000 - 0.6950 = 0.0050000000000000044),
    // so nudge by an epsilon before flooring or exact sizes lose one increment.
    let units = (units / limits.increment + 1e-9).floor() * limits.increment;
    if units < limits.min_units { 0.0 } else { units }
}
```

ข้อควรระวัง:

- **ขาดทุนจริงมักมากกว่าที่คำนวณ** เพราะ stop loss เป็น STOP_MARKET ซึ่ง fill ที่ราคาตลาดหลังราคาแตะ trigger
  ใน backtest ของบทนี้ `risk_fraction` 0.5% ของ 100,000 คือ 500 USD แต่ค่าเฉลี่ยของเทรดที่ขาดทุนอยู่ราว 750 USD
  ส่วนต่างมาจาก slippage, ค่าธรรมเนียม และการที่ equity โตขึ้นระหว่างทาง
- **ระยะ stop เป็นศูนย์ต้องไม่ส่ง order** ถ้าไม่ตรวจ การหารด้วยศูนย์จะได้ขนาดเป็นอนันต์
- **ปัดลงเสมอ** ทั้งใน `units_for_risk` และใน `make_qty(units, Some(true))` ขนาดที่ปัดขึ้นทำให้เสี่ยงเกินที่ตั้งใจ
- การปัดเศษกับ `f64` มีกับดัก ดู [Testing: floating point](testing.md#floating-point)

## ข้อจำกัดของ instrument [#instrument-limits]

นี่คือปัญหาที่สองที่เจอจริง โค้ดเวอร์ชันแรกคำนวณขนาดจากความเสี่ยงอย่างเดียว ไม่ได้ดู `max_quantity` ของ instrument
(`AUD/USD.SIM` จำกัดไว้ที่ 1,000,000 units) พอ ATR แคบ ขนาดที่คำนวณได้ก็เกินขีดจำกัด:

```text
[WARN] nautilus_risk::engine: SubmitOrder for O-20240101-143900-001-001-127 DENIED:
       QUANTITY_EXCEEDS_MAXIMUM: effective=1012743, max=1000000
[WARN] nautilus_risk::engine: SubmitOrder for O-20240101-143900-001-001-128 DENIED:
       ORDER_LIST_DENIED: OL-20240101-143900-001-001-43
```

เมื่อ entry ถูก deny order ที่เหลือใน bracket ถูก deny ตามด้วย (`ORDER_LIST_DENIED`) ทั้งชุด ซึ่งถูกต้องแล้ว
ปัญหาที่ร้ายกว่าคือ bar ถัดมาเงื่อนไขยังเหมือนเดิม strategy ก็ส่งชุดเดิมซ้ำอีก และซ้ำไปทุกแท่ง
ในการรันครั้งนั้นมี order ถูก deny เพราะเหตุนี้หลายพันครั้ง

แก้สองชั้น:

1. **คำนวณให้อยู่ในขอบเขตตั้งแต่ต้น** ส่ง `min_quantity` และ `max_quantity` ของ instrument เข้าไปใน `SizeLimits`
   ถ้าขนาดต่ำกว่าขั้นต่ำก็ไม่เทรดเลย ถ้าเกินขั้นสูงก็ตัดลงมา
2. **ถอยเมื่อถูกปฏิเสธ** เมื่อ order ถูก deny หรือ reject ให้หยุดเข้าเทรดใหม่ไประยะหนึ่ง:

```rust
// Denied = stopped by our own risk engine before reaching the venue.
// Back off instead of resubmitting the same order on every bar.
fn on_order_denied(&mut self, event: OrderDenied) {
    self.cooldown_until_ns = cooldown_end(self, self.config.cooldown_mins);
    log::warn!("Order {} denied: {}", event.client_order_id, event.reason);
}
```

`cooldown_end` อยู่ใน `src/execution.rs` คำนวณจากเวลาของ `self.clock()` จึงให้ผลเดียวกันทั้งใน backtest และ live:

```rust
/// Clock time plus `mins`, as nanoseconds: the end of a cooldown that starts now.
pub fn cooldown_end<S: DataActor + DataActorNative>(strategy: &S, mins: u64) -> u64 {
    strategy.clock().timestamp_ns().as_u64() + mins * 60 * 1_000_000_000
}
```

ชั้นที่สองสำคัญกว่าใน live เพราะเหตุผลที่ order ถูกปฏิเสธมีหลายอย่างที่คาดไม่ถึง (เงินไม่พอ, venue ปิดปรับปรุง, rate limit ของ API)
strategy ที่ส่งซ้ำทุกแท่งโดยไม่ถอยอาจโดน exchange แบน API key ได้

## Denied กับ Rejected ต่างกันอย่างไร [#denied-vs-rejected]

| Event | ใครปฏิเสธ | order ไปถึง venue ไหม | ตัวอย่างเหตุผล |
| --- | --- | --- | --- |
| `OrderDenied` | risk engine ของ Nautilus เอง | ไม่ | ขนาดเกินขีดจำกัด, ไม่มีราคาตลาด, เกิน rate limit, เกิน notional |
| `OrderRejected` | venue (exchange หรือ venue จำลอง) | ถึงแล้ว แต่ถูกปฏิเสธ | margin ไม่พอ, post-only จะ match ทันที, ราคาผิด tick |

strategy ในบทนี้ทำ cooldown ทั้งสองกรณี แต่นับ `rejections` แยกไว้ เพราะ reject จาก venue ใน live มักเป็นสัญญาณว่า
ค่าที่คำนวณไว้ไม่ตรงกับกฎของ exchange ซึ่งควรมีคนตรวจสอบ

## ตั้งค่า risk engine [#risk-engine]

risk engine ตรวจ order ทุกตัวก่อนออกจากระบบ ตั้งค่าใน backtest ด้วย `RiskEngineConfig`:

```rust
let risk = RiskEngineConfig::builder()
    // At most 10 order submissions per second, and no single order above 2M USD notional.
    .max_order_submit(RateLimit::new(10, DurationNanos::from_secs(1)))
    .max_notional_per_order(AHashMap::from([(instrument_id, Decimal::from(2_000_000))]))
    .build()?;
let config = BacktestEngineConfig::builder()
    .risk_engine(risk)
    // ...
    .build();
```

| ฟิลด์ | ความหมาย |
| --- | --- |
| `max_order_submit` | จำนวน order ที่ส่งได้ต่อช่วงเวลา (default 100 ต่อวินาที) |
| `max_order_modify` | จำนวนการแก้ order ต่อช่วงเวลา |
| `max_notional_per_order` | notional สูงสุดต่อ order แยกตาม instrument |
| `bypass` | ปิด risk engine ทั้งหมด (ห้ามใช้ใน live) |

ใน live ใช้ `LiveRiskEngineConfig` ซึ่งรับค่าเป็น string แทน เช่น `max_order_submit_rate = "100/00:00:01"`
(100 order ต่อ 1 วินาที) แล้วส่งให้ node ผ่าน `LiveNode::builder(...).with_risk_engine_config(...)`

การมีขีดจำกัดสองชั้น คือ `max_notional` ใน config ของ strategy และ `max_notional_per_order` ใน risk engine
ไม่ซ้ำซ้อน ชั้นแรกเป็นนโยบายของ strategy ชั้นที่สองเป็นตาข่ายนิรภัยของทั้ง node ที่ยังทำงานแม้ strategy จะมีบั๊ก
ให้ตั้งชั้นที่สองหลวมกว่าชั้นแรกเล็กน้อย ถ้า risk engine deny order ใดก็แปลว่า strategy ทำงานผิดจากที่ออกแบบไว้

ดูการตรวจทั้งหมดของ risk engine ได้ที่ [Execution](../concepts/execution/index.md) และ [Orders](../concepts/orders/index.md)
