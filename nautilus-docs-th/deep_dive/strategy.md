---
title: "เขียน strategy"
description: "โครงสร้างของ strategy, indicator, lifecycle และ hook ผ่าน strategy MeanReversion ทีละส่วน"
---

หน้านี้อธิบาย `src/strategy.rs` ทีละส่วน ถ้ายังไม่เคยเขียน strategy ด้วย Rust ให้อ่าน
[เขียน Strategy (Rust)](../how_to/write_rust_strategy.md) ก่อน หน้านั้นอธิบายกลไกพื้นฐานของ `StrategyCore`
และ `nautilus_strategy!` ส่วนหน้านี้เน้นการนำมาใช้ใน strategy ที่มี state จริง

## Config แยกจาก state [#config]

ค่าที่ผู้ใช้ปรับได้รวมไว้ใน struct เดียว แยกจาก state ที่ strategy สะสมระหว่างรัน:

```rust
#[derive(Clone, Debug)]
pub struct MeanReversionConfig {
    pub strategy_id: StrategyId,
    pub order_id_tag: String,
    pub instrument_id: InstrumentId,
    pub bar_type: BarType,
    pub bb_period: usize,
    pub bb_k: f64,
    pub atr_period: usize,
    /// Stop distance in ATRs.
    pub atr_stop: f64,
    /// Minimum distance to the middle band, in ATRs, to take a trade.
    pub min_reward_atr: f64,
    /// Fraction of equity lost if the stop is hit.
    pub risk_fraction: f64,
    /// Upper bound on entry notional, in quote currency.
    pub max_notional: f64,
    /// Minutes to wait after an order is denied or rejected before entering again.
    pub cooldown_mins: u64,
    /// How often to log a status line.
    pub status_interval_mins: u64,
}
```

เมื่อ config เป็นค่าเดียวที่ส่งเข้า constructor การรัน backtest หลายชุดพารามิเตอร์ หรือย้ายจาก backtest ไป live
ก็คือการสร้าง config ใหม่เท่านั้น ไม่ต้องแก้โค้ดของ strategy

`order_id_tag` ห้ามมีเครื่องหมาย `-` เพราะ runtime อ่าน tag กลับจากส่วนท้ายของ strategy ID ที่คั่นด้วย `-`
(`MEANREV-001` มี tag เป็น `001`)

## State ของ strategy [#state]

```rust
pub struct MeanReversion {
    core: StrategyCore,
    config: MeanReversionConfig,
    bb: BollingerBands,
    atr: AverageTrueRange,
    bars_seen: u64,
    trades_closed: u32,
    rejections: u32,
    /// No new entries before this time (UNIX nanoseconds).
    cooldown_until_ns: u64,
}
```

- `core` คือส่วนที่เชื่อมกับ runtime ชื่อฟิลด์ `core` เป็นค่าที่ `nautilus_strategy!` คาดไว้
- indicator เป็นของ strategy เอง strategy ป้อนข้อมูลให้ indicator ด้วยตัวเองใน `on_bar`
- ตัวนับต่าง ๆ ใช้สำหรับ log สถานะ ส่วน `cooldown_until_ns` เก็บเวลาที่จะกลับมาเข้าเทรดได้หลังถูกปฏิเสธ

สิ่งที่ **ไม่** เก็บไว้ใน struct คือ position และ order ที่เปิดอยู่ ข้อมูลเหล่านี้มีอยู่แล้วใน cache และ portfolio ของ Nautilus
ถ้าเก็บสำเนาไว้เอง สำเนาจะไม่ตรงกับความจริงเมื่อมี fill บางส่วน, order ถูก cancel จาก venue หรือ reconciliation
แก้ state หลัง restart ให้ถามจาก cache และ portfolio ทุกครั้งที่ต้องใช้

## Constructor [#constructor]

```rust
impl MeanReversion {
    pub fn new(config: MeanReversionConfig) -> Self {
        let core = StrategyCore::new(StrategyConfig {
            strategy_id: Some(config.strategy_id),
            order_id_tag: Some(config.order_id_tag.clone()),
            // After a live restart the cache is empty, so orders still open at the
            // venue come back as external orders. Claim them for this strategy.
            external_order_instrument_ids: Some(vec![config.instrument_id]),
            ..Default::default()
        });
        Self {
            core,
            bb: BollingerBands::new(config.bb_period, config.bb_k, None),
            atr: AverageTrueRange::new(config.atr_period, None, None, None),
            config,
            bars_seen: 0,
            trades_closed: 0,
            rejections: 0,
            cooldown_until_ns: 0,
        }
    }

    fn risk(&self) -> RiskSettings {
        RiskSettings {
            risk_fraction: self.config.risk_fraction,
            max_notional: self.config.max_notional,
        }
    }
}
```

ใน constructor ยังเรียก `clock()`, `cache()` หรือ `portfolio()` ไม่ได้ เพราะ strategy ยังไม่ถูกลงทะเบียนกับ engine
งานที่ต้องใช้ runtime ให้ทำใน `on_start`

### Indicator [#indicators]

- `BollingerBands::new(period, k, ma_type)` จะ panic ถ้า `period` อยู่นอกช่วงที่รองรับ หรือ `k` ไม่ใช่จำนวนบวกที่มีค่าจำกัด
  `None` ใน `ma_type` หมายถึง simple moving average ค่าของ band อ่านจากฟิลด์ `upper`, `middle`, `lower`
- `AverageTrueRange::new(period, ma_type, use_previous, value_floor)` ใช้ค่า default เป็น Wilder smoothing
  ค่า ATR อ่านจากฟิลด์ `value`
- การเรียก `handle_bar` และ `initialized()` ต้อง `use nautilus_indicators::indicator::Indicator` เพราะเป็น method ของ trait

`BollingerBands::handle_bar` ใช้ราคาปิดของ bar ส่วน `AverageTrueRange::handle_bar` ใช้ high, low และ close

## เชื่อมกับ runtime ด้วย macro [#macro]

```rust
nautilus_strategy!(MeanReversion, {
    fn on_order_denied(&mut self, event: OrderDenied) { /* ... */ }
    fn on_order_rejected(&mut self, event: OrderRejected) { /* ... */ }
    fn on_position_closed(&mut self, event: PositionClosed) { /* ... */ }
});

impl std::fmt::Debug for MeanReversion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeanReversion")
            .field("config", &self.config)
            .finish()
    }
}
```

method ที่อยู่ใน block ของ macro คือ hook ของ trait `Strategy` (event ของ order และ position)
ส่วน handler ข้อมูลและ lifecycle (`on_start`, `on_bar`, `on_time_event`, `on_stop`) อยู่ใน `impl DataActor` แยกต่างหาก
signature ของ hook ไม่เหมือนกันทุกตัว ให้ดูจาก trait เสมอ เช่น `on_order_filled` รับ `&OrderFilled`
แต่ `on_order_rejected` รับ `OrderRejected` แบบ by value

## Lifecycle: on_start [#on-start]

```rust
fn on_start(&mut self) -> anyhow::Result<()> {
    self.subscribe_bars(self.config.bar_type, None, None);
    self.clock().set_timer_ns(
        "status",
        DurationNanos::from_mins(self.config.status_interval_mins),
        None,
        None,
        None,
        None,
        None,
    )?;
    Ok(())
}
```

- `subscribe_bars` บอก data engine ให้ส่ง bar ชนิดนี้มาที่ `on_bar`
- `set_timer_ns` ตั้ง timer ซ้ำทุก N นาที เมื่อครบเวลา engine จะเรียก `on_time_event` พร้อมชื่อ timer
  ใน backtest timer เดินตามเวลาของข้อมูล ไม่ใช่เวลาจริง ส่วนใน live เดินตามนาฬิกาจริง โค้ดเดียวกันใช้ได้ทั้งสองแบบ

## Lifecycle: on_bar [#on-bar]

```rust
fn on_bar(&mut self, bar: &Bar) -> anyhow::Result<()> {
    self.bars_seen += 1;
    self.bb.handle_bar(bar);
    self.atr.handle_bar(bar);
    if !self.bb.initialized()
        || !self.atr.initialized()
        || bar.ts_event.as_u64() < self.cooldown_until_ns
        || is_busy(self, self.config.instrument_id, self.config.strategy_id)
    {
        return Ok(());
    }

    let close = bar.close.as_f64();
    let bands = Bands {
        upper: self.bb.upper,
        middle: self.bb.middle,
        lower: self.bb.lower,
        atr: self.atr.value,
    };
    match evaluate(
        close,
        bands,
        self.config.atr_stop,
        self.config.min_reward_atr,
    ) {
        Signal::Long {
            take_profit,
            stop_loss,
        } => enter_bracket(
            self,
            self.config.instrument_id,
            OrderSide::Buy,
            close,
            take_profit,
            stop_loss,
            self.risk(),
        ),
        Signal::Short {
            take_profit,
            stop_loss,
        } => enter_bracket(
            self,
            self.config.instrument_id,
            OrderSide::Sell,
            close,
            take_profit,
            stop_loss,
            self.risk(),
        ),
        Signal::None => Ok(()),
    }
}
```

ลำดับใน handler นี้ตั้งใจไว้:

1. **อัปเดต indicator ก่อนเสมอ** แม้จะไม่เทรดใน bar นี้ ถ้า return ก่อนป้อนข้อมูล indicator จะพลาด bar
   และค่าจะเพี้ยนไปตลอดหลังจากนั้น
2. **ตรวจเงื่อนไขที่ห้ามเทรด** indicator ยังไม่พร้อม, อยู่ในช่วง cooldown หรือมี position/order ค้างอยู่
3. **ตัดสินใจด้วย pure function** แล้วจึงส่ง order

`is_busy` (อยู่ใน `src/execution.rs` เพื่อใช้ร่วมกับ strategy อื่นในหน้า [Integration](integration.md)) ถามสถานะจาก portfolio และ cache:

```rust
/// True while `strategy_id` holds a position in, or has working orders for, the instrument.
pub fn is_busy<S>(strategy: &S, instrument_id: InstrumentId, strategy_id: StrategyId) -> bool
where
    S: Strategy + StrategyNative + DataActorNative,
{
    !strategy.portfolio().is_net_flat(&instrument_id)
        || strategy.cache().has_orders_open(
            None,
            Some(&instrument_id),
            Some(&strategy_id),
            None,
            None,
        )
}
```

ต้องตรวจทั้งสองอย่าง หลังส่ง bracket order แล้ว entry ยังไม่ fill ทันที ช่วงนั้น portfolio ยัง flat อยู่
ถ้าตรวจแค่ position strategy จะส่ง bracket ชุดใหม่ซ้อนทุก bar จนกว่า entry จะ fill

`self.cache()` คืน `CacheApi` ซึ่งอ่านค่าเป็น snapshot ไม่ได้คืน reference ที่ยืมค้างไว้
จึงเรียกต่อกันหลายครั้งใน method เดียวได้โดยไม่ติด borrow

## Pure function สำหรับการตัดสินใจ [#signal]

```rust
pub fn evaluate(close: f64, bands: Bands, atr_stop: f64, min_reward_atr: f64) -> Signal {
    if bands.atr <= 0.0 {
        return Signal::None;
    }
    let reward = (bands.middle - close).abs();
    if reward < min_reward_atr * bands.atr {
        return Signal::None;
    }
    if close < bands.lower {
        Signal::Long {
            take_profit: bands.middle,
            stop_loss: close - atr_stop * bands.atr,
        }
    } else if close > bands.upper {
        Signal::Short {
            take_profit: bands.middle,
            stop_loss: close + atr_stop * bands.atr,
        }
    } else {
        Signal::None
    }
}
```

`min_reward_atr` กรองสัญญาณที่ระยะถึงเส้นกลางสั้นเกินไป ช่วงที่ band แคบมาก ราคาหลุด band ได้ง่ายแต่ระยะทำกำไรสั้น
จนค่าธรรมเนียมและ slippage กินกำไรหมด

## Timer และ on_time_event [#timers]

```rust
fn on_time_event(&mut self, event: &TimeEvent) -> anyhow::Result<()> {
    if event.name.as_str() == "status" {
        log::info!(
            "Status: bars={} trades_closed={} rejections={} flat={}",
            self.bars_seen,
            self.trades_closed,
            self.rejections,
            self.portfolio().is_net_flat(&self.config.instrument_id),
        );
    }
    Ok(())
}
```

ให้ implement `on_time_event` ของ **`DataActor`** เท่านั้น trait `Strategy` ก็มี method ชื่อเดียวกัน
แต่เป็นส่วนที่ framework ใช้ส่งต่อ event ภายใน ห้าม override

strategy ใน production ควรมี log สถานะเป็นระยะแบบนี้เสมอ ถ้าบรรทัดสถานะหายไป แปลว่า node ค้างหรือไม่ได้รับข้อมูล
ซึ่งเป็นสัญญาณที่ระบบ monitoring จับได้ง่าย

## Lifecycle: on_stop [#on-stop]

```rust
fn on_stop(&mut self) -> anyhow::Result<()> {
    let id = self.config.instrument_id;
    self.cancel_all_orders(id, None, None, true, None)?;
    self.close_all_positions(id, None, None, None, None, Some(true), None, None)?;
    self.unsubscribe_bars(self.config.bar_type, None, None);
    Ok(())
}
```

- `cancel_all_orders(..., strategy_only = true, ...)` ยกเลิกเฉพาะ order ของ strategy นี้ ไม่ไปยุ่งกับ order ของ strategy อื่น
  หรือ order ที่เปิดด้วยมือ
- `close_all_positions` ปิด position ด้วย market order โดยตั้ง `reduce_only` เป็น `Some(true)`
  เพื่อไม่ให้ order ปิด position กลายเป็นการเปิด position ฝั่งตรงข้ามโดยไม่ตั้งใจ

การปิด position ตอนหยุดเป็นนโยบาย ไม่ใช่กฎตายตัว บางระบบต้องการให้ position อยู่ต่อข้าม restart
(ปล่อยให้ stop loss ที่ venue ดูแล) ให้เลือกตามลักษณะของ strategy และเขียนไว้ให้ชัดเจน

## Logging จากโค้ด strategy [#logging]

โค้ด strategy ใช้ macro ของ crate `log` ได้ตรง ๆ (`log::info!`, `log::warn!`) Nautilus ตั้งค่า logger ให้เองตอนสร้าง kernel
แต่ละบรรทัดจะมี trader ID และ module นำหน้า:

```text
[INFO] TRADER-001.meanrev::strategy: Entering Buy 619227 @ ~0.67877 tp=0.68029 sl=0.67796
```

ขั้นต่อไป: [Order และความเสี่ยง](orders_and_risk.md) อธิบาย `enter_bracket()` ซึ่งเป็นส่วนที่คำนวณขนาดและส่ง bracket order
