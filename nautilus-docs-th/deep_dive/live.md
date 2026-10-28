---
title: "ไปสู่ live: Bybit testnet"
description: "นำ strategy เดียวกันไปรันกับ exchange จริงบน testnet: dependency, credential, reconciliation และการหยุด node อย่างปลอดภัย"
---

strategy ในบทนี้ไม่มีโค้ดส่วนไหนที่รู้ว่ากำลังรันใน backtest หรือ live ทุกอย่างที่ต่างกันอยู่ในส่วนประกอบระบบ
(`backtest.rs` กับ `bin/live.rs`) หน้านี้นำ `MeanReversion` ตัวเดิมไปรันบน **Bybit testnet** กับ USDT linear perpetual

:::warning[สถานะของโค้ดในหน้านี้]
`src/bin/live.rs` compile ผ่านกับ NautilusTrader commit `04f766b4` แต่ไม่ได้รันกับ exchange ระหว่างเขียนบทนี้
เพราะต้องใช้ API key ของ testnet ก่อนใช้ให้รันบน testnet เองจนมั่นใจ และอ่าน log ทุกบรรทัดในรอบแรก
:::

## อะไรเปลี่ยนเมื่อไป live [#what-changes]

| เรื่อง | Backtest | Live |
| --- | --- | --- |
| instrument | สร้างเองแล้ว `add_instrument` | adapter โหลดจาก exchange ตอนเชื่อมต่อ |
| ข้อมูล | ป้อนล่วงหน้าทั้งหมด | stream ผ่าน WebSocket ตามเวลาจริง |
| fill | fill model จำลอง | exchange จริง มีคิว, partial fill และ latency |
| นาฬิกา | เดินตามเวลาของข้อมูล | นาฬิกาจริง timer ใน `on_start` ก็ทำงานตามเวลาจริง |
| state ตอนเริ่ม | ว่างเปล่าเสมอ | อาจมี order และ position ค้างจากรอบก่อน จึงต้อง reconcile |
| error | หยุดทั้งการรัน | network หลุด, API rate limit, exchange ปิดปรับปรุง |

## Dependency [#dependencies]

```toml
nautilus-bybit = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be", default-features = false, optional = true }
nautilus-live = { git = "https://github.com/nautechsystems/nautilus_trader", rev = "04f766b4d434d67fc8672026fefa25135bdc01be", default-features = false, features = ["node"], optional = true }
tokio = { version = "1", features = ["macros", "rt-multi-thread"], optional = true }
```

- `default-features = false` บน `nautilus-bybit` ปิด `high-precision` ที่ adapter เปิดไว้โดย default
  ถ้าเปิดไว้ flag จะลามไปทั้ง dependency graph และเปลี่ยนชนิดข้อมูลภายในของ `Price`/`Quantity` ในส่วน backtest ด้วย
  ถ้าตั้งใจจะใช้ `high-precision` ให้เปิดที่ `nautilus-model` โดยตรงเพื่อให้เห็นชัดใน `Cargo.toml`
- `nautilus-live` ต้องเปิด feature `node` เพื่อใช้ `LiveNode`
- live node ทำงานบน tokio จึงต้องมี tokio ที่เปิด `macros` และ `rt-multi-thread`

## Credential [#credentials]

adapter อ่าน API key จาก environment variable ตาม environment ที่เลือก:

| `BybitEnvironment` | API key | API secret |
| --- | --- | --- |
| `Testnet` | `BYBIT_TESTNET_API_KEY` | `BYBIT_TESTNET_API_SECRET` |
| `Demo` | `BYBIT_DEMO_API_KEY` | `BYBIT_DEMO_API_SECRET` |
| `Mainnet` | `BYBIT_API_KEY` | `BYBIT_API_SECRET` |

การแยกชื่อตัวแปรตาม environment ช่วยกันพลาดได้ดี key ของ mainnet จะไม่ถูกใช้ตอนตั้งค่าเป็น testnet
ถ้ากรอก `api_key`/`api_secret` ใน config เป็น `None` adapter จะอ่านจาก environment ให้เอง ห้ามเขียน key ลงในโค้ด

```bash
export BYBIT_TESTNET_API_KEY=...
export BYBIT_TESTNET_API_SECRET=...
cargo run --release --features live --bin live
```

สร้าง API key ของ testnet ได้ที่เว็บไซต์ testnet ของ Bybit ซึ่งแยกบัญชีจาก mainnet และใช้เงินทดสอบ

## ประกอบ node [#node]

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let environment = BybitEnvironment::Testnet;
    let instrument_id = InstrumentId::from("BTCUSDT-LINEAR.BYBIT");

    let data_config = BybitDataClientConfig {
        environment,
        api_key: None,    // read from BYBIT_TESTNET_API_KEY
        api_secret: None, // read from BYBIT_TESTNET_API_SECRET
        product_types: vec![BybitProductType::Linear],
        ..Default::default()
    };
    let exec_config = BybitExecutionClientConfig {
        environment,
        api_key: None,
        api_secret: None,
        product_types: vec![BybitProductType::Linear],
        account_id: Some(AccountId::from("BYBIT-001")),
        ..Default::default()
    };
    // Re-check open orders and positions against the venue while running.
    let exec_engine_config = LiveExecutionEngineConfig {
        open_check_interval_secs: Some(10.0),
        position_check_interval_secs: Some(30.0),
        ..Default::default()
    };

    let mut node = LiveNode::builder(TraderId::from("TRADER-001"), Environment::Live)?
        .with_name("MEANREV-NODE".to_string())
        .with_exec_engine_config(exec_engine_config)
        .add_data_client(
            None,
            Box::new(BybitDataClientFactory::new()),
            Box::new(data_config),
        )?
        .add_exec_client(
            None,
            Box::new(BybitExecutionClientFactory::new()),
            Box::new(exec_config),
        )?
        .with_reconciliation(true)
        .with_delay_post_stop_secs(5)
        .build()?;

    node.add_strategy(MeanReversion::new(MeanReversionConfig {
        strategy_id: StrategyId::from("MEANREV-001"),
        order_id_tag: "001".to_string(),
        instrument_id,
        bar_type: BarType::from("BTCUSDT-LINEAR.BYBIT-1-MINUTE-LAST-EXTERNAL"),
        bb_period: 20,
        bb_k: 2.0,
        atr_period: 14,
        atr_stop: 1.5,
        min_reward_atr: 1.0,
        risk_fraction: 0.001,
        max_notional: 200.0, // USDT; keep testnet orders small
        cooldown_bars: 30,
        status_interval_mins: 60,
    }))?;

    node.run().await?;
    Ok(())
}
```

จุดที่ควรสังเกต:

- **instrument ID ของ Bybit มีประเภทสินค้าต่อท้าย** `BTCUSDT-LINEAR.BYBIT` คือ perpetual แบบ USDT-margined
  ส่วน bar type ใช้ `LAST` ซึ่งตรงกับ kline ของ exchange และเป็นราคาที่ risk engine ใช้ได้ ([ข้อมูล](data.md#price-type))
- **ค่าความเสี่ยงต่างจาก backtest** `risk_fraction` 0.1% และ `max_notional` 200 USDT ทำให้ order บน testnet มีขนาดเล็ก
  config แยกจาก strategy จึงเปลี่ยนค่าเหล่านี้ได้โดยไม่แตะ logic
- **`account_id`** กำหนดชื่อบัญชีใน Nautilus (`BYBIT-001`) ซึ่งจะปรากฏใน event และ report

## Reconciliation [#reconciliation]

เมื่อ node เริ่มทำงาน มันจะถาม exchange ว่ามี order และ position อะไรเปิดอยู่ แล้วปรับ state ภายในให้ตรงก่อนเริ่มเทรด
(`with_reconciliation(true)`) ในระหว่างรัน config ของ execution engine กำหนดให้ตรวจซ้ำเป็นระยะ:

| ฟิลด์ใน `LiveExecutionEngineConfig` | ค่าที่ใช้ | ความหมาย |
| --- | --- | --- |
| `open_check_interval_secs` | `Some(10.0)` | ตรวจสถานะ order ที่เปิดอยู่กับ exchange ทุก 10 วินาที |
| `position_check_interval_secs` | `Some(30.0)` | ตรวจ position กับ exchange ทุก 30 วินาที |
| `reconciliation_lookback_mins` | default | ย้อนดูประวัติ order กี่นาทีตอนเริ่ม |
| `reconciliation_startup_delay_secs` | default (10 วินาที) | รอก่อนเริ่ม reconcile เพื่อให้การเชื่อมต่อพร้อม |

### Order ที่ค้างจากรอบก่อน [#external-orders]

cache ของ node เก็บไว้ในหน่วยความจำ หลัง restart cache จึงว่างเปล่า order ที่ยังเปิดอยู่บน exchange จะถูกสร้างขึ้นใหม่ระหว่าง
reconciliation เป็น **external order** และถูกมอบให้ strategy ที่ "claim" instrument นั้นไว้ ถ้าไม่มี strategy ใด claim
order จะไปอยู่กับ strategy ชื่อ `EXTERNAL` ซึ่งเป็นค่า fallback

strategy ในบทนี้ claim instrument ของตัวเองไว้ใน constructor:

```rust
external_order_instrument_ids: Some(vec![config.instrument_id]),
```

ถ้าไม่ claim order ที่ค้างจากรอบก่อนจะไม่ใช่ของ `MEANREV-001` และ `is_busy()` ซึ่งกรองด้วย strategy ID จะมองไม่เห็นมัน
strategy อาจส่ง bracket ชุดใหม่ซ้อนกับ take profit/stop loss ชุดเก่าที่ยังทำงานอยู่บน exchange
ดูกติกาการ claim ทั้งหมดที่ [การสร้าง external order](../concepts/execution/reconciliation.md#external-order-creation)

การตรวจเป็นระยะคือสิ่งที่ทำให้ node รู้ตัวเมื่อ event หลุดหายระหว่าง WebSocket ขาดการเชื่อมต่อ ซึ่งเกิดขึ้นจริงใน live
ถ้าไม่มีการตรวจนี้ `is_busy()` ของ strategy อาจคิดว่ายังมี order เปิดอยู่ตลอดไป และ strategy จะไม่เทรดอีกเลย
รายละเอียดทั้งหมดอยู่ที่ [Reconciliation](../concepts/execution/reconciliation.md)

## หยุด node อย่างปลอดภัย [#shutdown]

`node.run()` รับสัญญาณ SIGINT (Ctrl+C) และ SIGTERM แล้วปิดตามลำดับนี้:

1. หยุด strategy ซึ่งเรียก `on_stop` ของเรา: ยกเลิก order ของ strategy นี้และปิด position ด้วย reduce-only market order
2. ประมวลผล event ที่ตามมาต่ออีกช่วงหนึ่ง (`with_delay_post_stop_secs(5)` คือ 5 วินาที) เพื่อรับ cancel และ fill ที่ยังมาไม่ถึง
3. ปิด kernel และตัดการเชื่อมต่อกับ exchange

ถ้ายังมี order ที่ส่งไปแล้วแต่ไม่ได้รับคำตอบเมื่อหมดเวลาในข้อ 2 `run()` จะคืน error ให้รู้ ช่วงเวลานี้ต้องนานพอ
สำหรับ latency ของ exchange ที่ใช้ ถ้า process ถูก kill แบบ SIGKILL ขั้นตอนเหล่านี้จะไม่เกิดขึ้นเลย
order และ position ที่ค้างจะรอให้ reconciliation จัดการในรอบถัดไป

## ลำดับขั้นจาก backtest ถึงเงินจริง [#rollout]

1. backtest ผ่าน test ทุกตัว รวมถึง [control market](backtest.md#control-market)
2. รันบน testnet อย่างน้อยหลายวัน ดู log ทุกบรรทัดที่เป็น `WARN` หรือ `ERROR`
3. ทดลอง restart ระหว่างที่มี position เปิดอยู่ แล้วดูว่า reconciliation คืนสถานะได้ถูกต้อง
4. ทดลองตัดอินเทอร์เน็ตชั่วคราว แล้วดูว่า node เชื่อมต่อใหม่และตรวจพบ order ที่เปลี่ยนสถานะระหว่างหลุด
5. ไป mainnet ด้วยขนาดเล็กที่สุดที่ exchange ยอมรับ แล้วค่อยเพิ่มขนาดทีละขั้น

ก่อนขั้นที่ 5 ให้ผ่าน [Production checklist](production.md) ให้ครบ
ดูการตั้งค่า live node แบบละเอียดได้ที่ [Configure live trading](../how_to/configure_live_trading.md)
และ [รัน live trading ด้วย Rust](../how_to/run_rust_live_trading.md) ส่วนความสามารถของ adapter Bybit ดูที่
[Bybit integration](../integrations/bybit.md)
