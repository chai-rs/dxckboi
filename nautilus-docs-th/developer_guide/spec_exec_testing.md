---
title: "Execution Testing Spec"
---

ใช้ test matrix นี้เพื่อ validate ความสามารถด้าน execution ของ adapter ด้วย strategy `ExecTester` ของ Rust
โค้ด Rust import มันจาก `nautilus_testkit::testers` แต่ละ case ใช้ ID ที่มี prefix (เช่น TC-E01) และจัดกลุ่มตามฟังก์ชัน

**adapter แต่ละตัวต้องผ่านชุด test ย่อยที่ตรงกับความสามารถที่ตนรองรับ**

test ไล่จากง่าย (market order เดียว) ไปซับซ้อน (bracket, การแก้ไขต่อเนื่อง, การจัดการ rejection) adapter ที่ผ่านกลุ่ม 1-5
ถือว่าผ่านเกณฑ์พื้นฐาน (baseline compliant) ควรตรวจการเชื่อมต่อข้อมูลก่อนด้วย
[Data Testing Spec](spec_data_testing.md)

ให้บันทึกพฤติกรรมเฉพาะของ adapter (venue จำลอง market order อย่างไร จัดการตัวเลือก TIF อย่างไร ฯลฯ) ไว้ในคู่มือของ
adapter นั้นเอง ไม่ใช่ที่นี่ คู่มือของแต่ละ adapter ควรมี capability matrix ที่แสดงว่ารองรับ order type, ตัวเลือก
time-in-force, action และ flag ใดบ้าง

## สิ่งที่ต้องมีก่อน [#prerequisites]

ก่อนรัน execution test:

- account แบบ demo/testnet ที่มี API credential ถูกต้อง (แนะนำ แต่ไม่บังคับ)
- account มีเงินพอสำหรับ margin ของ instrument และ quantity ที่ใช้ทดสอบ
- instrument เป้าหมายมีอยู่และโหลดผ่าน instrument provider ได้
- ตั้ง environment variable แล้ว: `{VENUE}_API_KEY`, `{VENUE}_API_SECRET` (หรือตัวแปรแบบ sandbox)
- ถ้า venue มีโหมด demo/testnet ให้ใช้ credential ที่สร้างสำหรับ environment นั้น API key ของ demo และ production
  มักแยกกันและใช้แทนกันไม่ได้ การใช้ credential ผิดจะทำให้เกิด authentication error (เช่น HTTP 401)
- bypass risk engine (`LiveRiskEngineConfig(bypass=True)`) เพื่อไม่ให้รบกวน
- เปิด reconciliation เพื่อตรวจความสอดคล้องของ state

**การตั้งค่า node ด้วย Rust** (อ้างอิง: `crates/adapters/{adapter}/examples/node_exec_tester.rs`):

```rust
use nautilus_testkit::testers::{ExecTester, ExecTesterConfig};
use nautilus_trading::strategy::StrategyConfig;

let tester_config = ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(order_qty)
    .build()?;
let tester = ExecTester::new(tester_config);
node.add_strategy(tester)?;
node.run().await?;
```

## สเกลของ timestamp [#timestamp-scale]

Nautilus เก็บ `ts_event` และ `ts_init` เป็น Unix nanoseconds (`UnixNanos`) ทุก execution message ต้องใช้สเกลนี้:
order event, fill, account state และ reconciliation report (`OrderStatusReport`, `FillReport`,
`PositionStatusReport`, `ExecutionMassStatus`)

- ค่าที่ต่ำกว่า `10^16` ไม่น่าจะเป็น timestamp แบบ Unix nanosecond (`10^16` ns คือประมาณ 116 วันหลัง 1970-01-01)
  และมักหมายความว่า adapter ไม่ได้แปลงสเกลจาก venue
- เวลาจาก venue ที่มีความละเอียดระดับวินาทีซึ่งแปลงถูกต้องจะลงท้ายด้วย `000000000` และยังผ่าน: นั่นคือความละเอียดหยาบ
  ไม่ใช่ความผิดพลาดของสเกล
- `ts_event` ของ live stream ควรใกล้กับเวลานาฬิกาจริงของ session ส่วน reconciliation report อาจเก่ากว่านั้นและยังถูกต้อง
  ถ้าสเกลเป็นนาโนวินาที
- `ts_init` คือ clock ในเครื่องตอนที่ Nautilus สร้าง object อาจมี skew เล็กน้อยที่ `ts_event` > `ts_init` ได้เมื่อ
  clock ของ venue เดินเร็วกว่า

`ExecTester` จะ warn เมื่อ timestamp ของ market data หรือ order event ที่ได้รับไม่ผ่านการตรวจสเกล ส่วน timestamp
ของ report ให้ตรวจจากตัว report เอง ให้ถือว่า warning เรื่องสเกล หรือมีหน่วยวินาที มิลลิวินาที หรือไมโครวินาทีหลงเหลือใน
execution message ใด ๆ เป็นการไม่ผ่าน

## Smoke test พื้นฐาน [#basic-smoke-test]

การตรวจสอบเบื้องต้นแบบเร็วที่รันได้ทุกเมื่อ เช่น หลังแก้ adapter หรือระหว่างรอบการพัฒนา tester จะเปิด position ด้วย
market order ตอนเริ่ม วาง limit order แบบ post-only ทั้งฝั่ง buy และ sell รอ 30 วินาที แล้วหยุด (cancel order ที่เปิดอยู่และ
ปิด position)

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.001"))
    .open_position_on_start_qty(dec!(0.001))
    .enable_limit_buys(true)
    .enable_limit_sells(true)
    .use_post_only(true)
    .build()?
```

**พฤติกรรมที่คาดหวัง**:

1. ตอนเริ่ม: market order ถูก fill และเปิด position
2. วาง limit order สองตัวห่างจาก best bid/ask เป็น `tob_offset_ticks` (default 500 tick)
3. strategy ว่างอยู่ 30 วินาที ตรวจ log ว่ามี error, order ที่ถูก reject หรือการหลุดการเชื่อมต่อหรือไม่
4. ตอนหยุด: limit order ที่เปิดอยู่ถูก cancel และ position ถูกปิดด้วย market order

**เกณฑ์ผ่าน**: ไม่มี error ใน log, position เปิดและปิดได้เรียบร้อย และ venue ยืนยันรับ limit order

---

แต่ละกลุ่มด้านล่างเริ่มด้วยตารางสรุป ตามด้วย test card แบบละเอียด ID ของ test เว้นช่วงตัวเลขไว้เพื่อให้แทรกเพิ่มได้โดยไม่ต้อง
เรียงเลขใหม่

---

## กลุ่ม 1: Market order [#group-1-market-orders]

ทดสอบการ submit และการ fill ของ market order market order ควร execute ทันที

| TC | ชื่อ | คำอธิบาย | ข้ามเมื่อ |
| --- | --- | --- | --- |
| TC-E01 | Market BUY - submit และ fill | เปิด long position ด้วย market buy | ไม่มี market order |
| TC-E02 | Market SELL - submit และ fill | เปิด short position ด้วย market sell | ไม่มี market order |
| TC-E03 | Market order แบบ IOC TIF | market order ที่ระบุ time in force เป็น IOC ชัดเจน | ไม่มี IOC |
| TC-E04 | Market order แบบ FOK TIF | market order ที่ระบุ time in force เป็น FOK ชัดเจน | ไม่มี FOK |
| TC-E05 | Market order แบบ quote qty | market order ที่ใช้ quantity เป็น quote currency | ไม่มี quote quantity |
| TC-E06 | ปิด position ด้วย market | ปิด position ที่เปิดอยู่ด้วย market order ตอนหยุด | ไม่มี market order |

### TC-E01: Market BUY - submit และ fill [#tc-e01-market-buy---submit-and-fill]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, market data ไหลเข้า, ไม่มี position เปิดอยู่ |
| **การกระทำ** | ExecTester เปิด long position ผ่าน `open_position_on_start_qty` |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` |
| **เกณฑ์ผ่าน** | position เปิดด้วย side=LONG, quantity ตรงกับ config, ราคา fill อยู่ในช่วงตลาด, `AccountState` ถูกอัปเดต |
| **ข้ามเมื่อ** | adapter ไม่รองรับ market order |

**ข้อควรพิจารณา**:

- บาง adapter จำลอง market order เป็น limit IOC แบบ aggressive (ดูคู่มือ adapter)
- ลำดับ event จากมุมมองของ strategy ควรเหมือนกันไม่ว่ากลไกของ venue จะเป็นแบบใด
- ราคา fill ควรอยู่ภายใน bid/ask spread ล่าสุด
- partial fill ถือว่าใช้ได้ ให้ตรวจว่า filled quantity สะสมตรงกับ order quantity

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .open_position_on_start_qty(Decimal::new(1, 2))
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .build()?
```

### TC-E02: Market SELL - submit และ fill [#tc-e02-market-sell---submit-and-fill]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, market data ไหลเข้า, ไม่มี position เปิดอยู่ |
| **การกระทำ** | ExecTester เปิด short position ผ่าน `open_position_on_start_qty` ที่เป็นค่าติดลบ |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` |
| **เกณฑ์ผ่าน** | position เปิดด้วย side=SHORT, quantity ตรงกับ config, ราคา fill อยู่ในช่วงตลาด |
| **ข้ามเมื่อ** | adapter ไม่รองรับ market order หรือ short selling |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .open_position_on_start_qty(Decimal::new(-1, 2))
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .build()?
```

### TC-E03: Market order แบบ IOC TIF [#tc-e03-market-order-with-ioc-tif]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, market data ไหลเข้า |
| **การกระทำ** | เปิด position ด้วย `open_position_time_in_force=IOC` |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` |
| **เกณฑ์ผ่าน** | เหมือน TC-E01 โดย order ระบุ IOC TIF อย่างชัดเจน |
| **ข้ามเมื่อ** | ไม่รองรับ IOC |

**ข้อควรพิจารณา**:

- บาง adapter จำลอง market order เป็น limit IOC แบบ aggressive market แบบจำลองที่ไม่ถูก fill จะถูก map เป็น
  `OrderCanceled` ไม่ใช่ `OrderRejected`

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .open_position_on_start_qty(Decimal::new(1, 2))
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .open_position_time_in_force(TimeInForce::Ioc)
    .build()?
```

### TC-E04: Market order แบบ FOK TIF [#tc-e04-market-order-with-fok-tif]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, market data ไหลเข้า |
| **การกระทำ** | เปิด position ด้วย `open_position_time_in_force=FOK` |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` |
| **เกณฑ์ผ่าน** | เหมือน TC-E01 โดย order ระบุ FOK TIF อย่างชัดเจน |
| **ข้ามเมื่อ** | ไม่รองรับ FOK |

**ข้อควรพิจารณา**:

- FOK ต้อง fill ได้ทั้ง quantity ทันที มิฉะนั้น order จะถูก cancel
- ใช้ quantity ทดสอบขนาดเล็ก เพื่อให้ความลึกของ book พอสำหรับการ fill ทั้งหมด

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .open_position_on_start_qty(Decimal::new(1, 2))
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .open_position_time_in_force(TimeInForce::Fok)
    .build()?
```

### TC-E05: Market order แบบ quote quantity [#tc-e05-market-order-with-quote-quantity]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, adapter รองรับ quote quantity |
| **การกระทำ** | เปิด position ด้วย `use_quote_quantity=True` โดย quantity เป็นหน่วย quote currency |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` |
| **เกณฑ์ผ่าน** | order ถูก submit ด้วย quantity เป็น quote currency และ fill quantity เป็นหน่วย base currency |
| **ข้ามเมื่อ** | adapter ไม่รองรับ order แบบ quote quantity |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("100"))
    .open_position_on_start_qty(Decimal::from(100))
    .use_quote_quantity(true)
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .build()?
```

### TC-E06: ปิด position ด้วย market order ตอนหยุด [#tc-e06-close-position-via-market-order-on-stop]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี position เปิดอยู่จาก TC-E01 หรือ TC-E02 |
| **การกระทำ** | หยุด strategy แล้ว ExecTester ปิด position ด้วย market order |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` (order ปิด) |
| **เกณฑ์ผ่าน** | position ถูกปิด หรือเหลือเพียงเศษที่ต่ำกว่า precision พอดี และไม่มี order เปิดค้างอยู่ |
| **ข้ามเมื่อ** | adapter ไม่รองรับ market order |

**ข้อควรพิจารณา**:

- test นี้ตามหลัง TC-E01 หรือ TC-E02 โดยธรรมชาติใน session เดียวกัน
- `close_positions_on_stop=True` เป็นค่า default
- order ปิดควรอยู่ฝั่งตรงข้ามกับ position
- ตั้ง `close_positions_qty_precision` เมื่อ venue รับทศนิยมของขนาดได้น้อยกว่า instrument tester จะปิดเฉพาะ quantity
  ที่ venue fill ได้ และ log เศษที่เหลืออยู่

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .open_position_on_start_qty(Decimal::new(1, 2))
    .close_positions_on_stop(true)
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .build()?
```

---

## กลุ่ม 2: Limit order [#group-2-limit-orders]

ทดสอบการ submit, การ accept และพฤติกรรมของ limit order ภายใต้ตัวเลือก time-in-force ต่าง ๆ

| TC | ชื่อ | คำอธิบาย | ข้ามเมื่อ |
| --- | --- | --- | --- |
| TC-E10 | Limit BUY GTC | วาง limit buy แบบ GTC ต่ำกว่า TOB แล้วตรวจว่าถูก accept | ไม่ข้าม |
| TC-E11 | Limit SELL GTC | วาง limit sell แบบ GTC สูงกว่า TOB แล้วตรวจว่าถูก accept | ไม่ข้าม |
| TC-E12 | คู่ Limit BUY และ SELL | วางทั้งสองฝั่งพร้อมกัน ตรวจว่าถูก accept ทั้งคู่ | ไม่ข้าม |
| TC-E13 | Limit IOC aggressive fill | limit IOC ที่ราคา aggressive คาดว่าจะ fill | ไม่มี IOC |
| TC-E14 | Limit IOC passive no fill | ตรวจการ cancel หรือ rejection แบบไม่มี liquidity ตามที่ระบุไว้ | ไม่มี IOC |
| TC-E15 | Limit FOK fill | limit FOK ที่ราคา aggressive คาดว่าจะ fill | ไม่มี FOK |
| TC-E16 | Limit FOK no fill | ตรวจการ cancel หรือ rejection แบบไม่มี liquidity ตามที่ระบุไว้ | ไม่มี FOK |
| TC-E17 | Limit GTD | limit ที่มีเวลาหมดอายุ ตรวจว่าถูก accept | ไม่มี GTD |
| TC-E18 | Limit GTD expiry | ตรวจ terminal event ตอนหมดอายุตามที่ระบุไว้ | ไม่มี GTD |
| TC-E19 | Limit DAY | limit แบบ DAY TIF ตรวจว่าถูก accept | ไม่มี DAY |

### TC-E10: Limit BUY GTC - submit และ accept [#tc-e10-limit-buy-gtc---submit-and-accept]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วาง limit buy ที่ `best_bid - tob_offset_ticks` |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order เปิดอยู่บน venue ด้วย price, quantity ถูกต้อง, side=BUY, TIF=GTC |
| **ข้ามเมื่อ** | ไม่ข้าม |

**ข้อควรพิจารณา**:

- `tob_offset_ticks` (default 500) วาง order ไว้ห่างจากตลาดมากเพื่อเลี่ยงการ fill โดยไม่ตั้งใจ
- ตรวจว่า order ปรากฏใน cache ด้วย `OrderStatus.ACCEPTED`
- order ควรเปิดอยู่จนกว่าจะถูก cancel อย่างชัดเจน

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(false)
    .build()?
```

### TC-E11: Limit SELL GTC - submit และ accept [#tc-e11-limit-sell-gtc---submit-and-accept]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วาง limit sell ที่ `best_ask + tob_offset_ticks` |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order เปิดอยู่บน venue ด้วย price, quantity ถูกต้อง, side=SELL, TIF=GTC |
| **ข้ามเมื่อ** | ไม่ข้าม |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(false)
    .enable_limit_sells(true)
    .build()?
```

### TC-E12: คู่ Limit BUY และ SELL [#tc-e12-limit-buy-and-sell-pair]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วางทั้ง limit buy และ limit sell |
| **ลำดับ event** | สองลำดับที่เป็นอิสระต่อกัน: แต่ละตัว `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order ทั้งสองเปิดอยู่บน venue โดย buy ต่ำกว่า bid และ sell สูงกว่า ask |
| **ข้ามเมื่อ** | ไม่ข้าม |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(true)
    .build()?
```

### TC-E13: Limit IOC aggressive fill [#tc-e13-limit-ioc-aggressive-fill]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | submit limit buy IOC ที่ราคาเท่ากับหรือสูงกว่า best ask (ราคา aggressive) |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` |
| **เกณฑ์ผ่าน** | order ถูก fill ทันที และ position ถูกเปิด |
| **ข้ามเมื่อ** | adapter ไม่รองรับ IOC TIF |

**ข้อควรพิจารณา**:

- test นี้ต้องสร้าง order ด้วยมือหรือใช้ config เฉพาะของ adapter เพราะการวาง limit order แบบ default ของ ExecTester ใช้
  GTC TIF
- venue จะ cancel IOC order ที่ fill ไม่ได้ทันที

### TC-E14: Limit IOC passive - no fill [#tc-e14-limit-ioc-passive---no-fill]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | submit limit buy IOC ที่ต่ำกว่าตลาดมาก (ราคา passive) |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` จากนั้น accept/cancel หรือ rejection ตามที่ระบุไว้ |
| **เกณฑ์ผ่าน** | ไม่มี fill; `Canceled` หรือ `Rejected` ด้วยเหตุผลไม่มี liquidity ตามที่ adapter ระบุไว้ |
| **ข้ามเมื่อ** | adapter ไม่รองรับ IOC TIF |

**ข้อควรพิจารณา**:

- ผลลัพธ์ default คือ `OrderAccepted` -> `OrderCanceled` ถ้า venue reject order อย่างชัดเจนเพราะไม่มี liquidity ให้ตรวจลำดับ
  `OrderSubmitted` -> `OrderRejected` ตามที่ adapter ระบุไว้ เหตุผลการ reject ที่ตรงกันพอดี และ filled quantity เป็นศูนย์
  เหตุผลการ reject อื่นและ `OrderExpired` ไม่ผ่าน case นี้
  ให้คง rejection ที่ venue ส่งมาอย่างชัดเจนไว้ แทนที่จะสังเคราะห์เป็นการ accept หรือ cancel

### TC-E15: Limit FOK fill [#tc-e15-limit-fok-fill]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า, book มีความลึกเพียงพอ |
| **การกระทำ** | submit limit buy FOK ที่ราคา aggressive ด้วย quantity ที่อยู่ภายในความลึกของ top-of-book |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` |
| **เกณฑ์ผ่าน** | order ถูก fill ทั้งหมดใน fill event เดียว |
| **ข้ามเมื่อ** | adapter ไม่รองรับ FOK TIF |

**ข้อควรพิจารณา**:

- FOK ต้อง fill ได้ทั้ง quantity ใช้ quantity ขนาดเล็กเพื่อให้ความลึกของ book เพียงพอ

### TC-E16: Limit FOK no fill [#tc-e16-limit-fok-no-fill]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | submit limit buy FOK ที่ราคา passive (ต่ำกว่าตลาดมาก) |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` จากนั้น accept/cancel หรือ rejection ตามที่ระบุไว้ |
| **เกณฑ์ผ่าน** | ไม่มี fill; `Canceled` หรือ `Rejected` ด้วยเหตุผลไม่มี liquidity ตามที่ adapter ระบุไว้ |
| **ข้ามเมื่อ** | adapter ไม่รองรับ FOK TIF |

**ข้อควรพิจารณา**:

- ใช้การตรวจกรณีไม่มี liquidity เฉพาะของ adapter แบบเดียวกับ [TC-E14](#tc-e14-limit-ioc-passive---no-fill)

### TC-E17: Limit GTD - submit และ accept [#tc-e17-limit-gtd---submit-and-accept]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | วาง limit buy โดยตั้ง `order_expire_time_delta_mins` (เช่น 60 นาที) |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order ถูก accept ด้วย GTD TIF และ timestamp หมดอายุที่ถูกต้อง |
| **ข้ามเมื่อ** | adapter ไม่รองรับ GTD TIF |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(false)
    .order_expire_time_delta_mins(60)
    .build()?
```

### TC-E18: Limit GTD expiry [#tc-e18-limit-gtd-expiry]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี GTD limit order เปิดอยู่จาก TC-E17 โดยใช้เวลาหมดอายุที่สั้นที่สุดที่ venue รองรับ |
| **การกระทำ** | รอจนถึงเวลาหมดอายุของ GTD |
| **ลำดับ event** | `OrderExpired` โดย default หรือ terminal event ที่ adapter ระบุไว้ |
| **เกณฑ์ผ่าน** | order ไปถึง terminal status ตามที่ adapter ระบุไว้เมื่อหมดอายุบน venue |
| **ข้ามเมื่อ** | adapter ไม่รองรับ GTD TIF |

**ข้อควรพิจารณา**:

- ใช้เวลาหมดอายุที่สั้นที่สุดที่ venue ยอมรับ อย่าสมมติว่าหนึ่งหรือสองนาทีใช้ได้เสมอ
- บาง venue รายงานการหมดอายุของ GTD เป็น cancel ให้คงและตรวจการ map ของ adapter ตามที่ระบุไว้ แทนที่จะ normalize
  ทุก venue ให้เป็น `OrderExpired`

### TC-E19: Limit DAY - submit และ accept [#tc-e19-limit-day---submit-and-accept]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, ตลาดอยู่ในเวลาทำการ |
| **การกระทำ** | submit limit buy ด้วย DAY TIF |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order ถูก accept ด้วย DAY TIF และจะถูก cancel อัตโนมัติเมื่อจบวันทำการ |
| **ข้ามเมื่อ** | adapter ไม่รองรับ DAY TIF |

**ข้อควรพิจารณา**:

- DAY order อาจทำงานต่างกันระหว่าง venue crypto ที่เปิด 24/7 กับตลาดดั้งเดิม
- ตรวจพฤติกรรมเมื่อ submit นอกเวลาทำการ (ถ้าเกี่ยวข้อง)

---

## กลุ่ม 3: Stop order และ conditional order [#group-3-stop-and-conditional-orders]

ทดสอบ order ประเภท stop และ conditional order เหล่านี้วางรออยู่บน venue จนกว่าเงื่อนไข trigger จะเป็นจริง adapter ที่
รองรับ conditional order แบบ native ของ venue ควรตรวจด้วยว่า trigger order ที่เปิดอยู่ปรากฏใน reconciliation ตอน restart
ไม่ใช่เฉพาะใน endpoint ของ open order ปกติ

| TC | ชื่อ | คำอธิบาย | ข้ามเมื่อ |
| --- | --- | --- | --- |
| TC-E20 | StopMarket BUY | stop buy สูงกว่า ask ตรวจว่าถูก accept | ไม่มี `STOP_MARKET` |
| TC-E21 | StopMarket SELL | stop sell ต่ำกว่า bid ตรวจว่าถูก accept | ไม่มี `STOP_MARKET` |
| TC-E22 | StopLimit BUY | stop-limit buy ที่มี trigger + limit price | ไม่มี `STOP_LIMIT` |
| TC-E23 | StopLimit SELL | stop-limit sell ที่มี trigger + limit price | ไม่มี `STOP_LIMIT` |
| TC-E24 | MarketIfTouched BUY | MIT buy ต่ำกว่า bid | ไม่มี `MIT` |
| TC-E25 | MarketIfTouched SELL | MIT sell สูงกว่า ask | ไม่มี `MIT` |
| TC-E26 | LimitIfTouched BUY | LIT buy ที่มี trigger + limit price | ไม่มี `LIT` |
| TC-E27 | LimitIfTouched SELL | LIT sell ที่มี trigger + limit price | ไม่มี `LIT` |

### TC-E20: StopMarket BUY [#tc-e20-stopmarket-buy]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วาง stop-market buy สูงกว่า ask ปัจจุบัน |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | stop order ถูก accept บน venue ด้วย trigger price ที่ถูกต้องและ side=BUY |
| **ข้ามเมื่อ** | adapter ไม่รองรับ order แบบ `StopMarket` |

**ข้อควรพิจารณา**:

- trigger price ควรสูงกว่า ask ปัจจุบันเท่ากับ `stop_offset_ticks`
- order ไม่ควร trigger ทันที (trigger price อยู่สูงกว่าตลาด)
- สำหรับ venue ที่มี trigger signature อายุยาว ให้ตรวจการหมดอายุของ signature ของ trigger order เทียบกับช่วงเวลาของ
  trigger order ของ venue ไม่ใช่การหมดอายุของ order ปกติ
- การตรวจ trigger และ fill ต้องอาศัยการเคลื่อนไหวของตลาด ซึ่งอาจไม่เกิดขึ้นระหว่าง test
- หลังถูก accept ให้ restart หรือบังคับ reconciliation ถ้า venue เก็บ trigger order ไว้ใน endpoint แยก ให้ตรวจ open order
  report และตรวจให้แน่ใจว่า order ยังปรากฏอยู่ที่นั่น

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .enable_stop_buys(true)
    .enable_stop_sells(false)
    .stop_order_type(OrderType::StopMarket)
    .build()?
```

### TC-E21: StopMarket SELL [#tc-e21-stopmarket-sell]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วาง stop-market sell ต่ำกว่า bid ปัจจุบัน |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | stop order ถูก accept บน venue ด้วย trigger price ที่ถูกต้องและ side=SELL |
| **ข้ามเมื่อ** | adapter ไม่รองรับ order แบบ `StopMarket` |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .enable_stop_buys(false)
    .enable_stop_sells(true)
    .stop_order_type(OrderType::StopMarket)
    .build()?
```

### TC-E22: StopLimit BUY [#tc-e22-stoplimit-buy]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วาง stop-limit buy โดย trigger price สูงกว่า ask พร้อม limit offset |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | stop-limit order ถูก accept ด้วย trigger price, limit price ที่ถูกต้อง และ side=BUY |
| **ข้ามเมื่อ** | adapter ไม่รองรับ order แบบ `StopLimit` |

**ข้อควรพิจารณา**:

- ต้องตั้ง `stop_limit_offset_ticks` เพื่อกำหนดระยะของ limit price จาก trigger price

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .enable_stop_buys(true)
    .enable_stop_sells(false)
    .stop_order_type(OrderType::StopLimit)
    .stop_limit_offset_ticks(50)
    .build()?
```

### TC-E23: StopLimit SELL [#tc-e23-stoplimit-sell]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วาง stop-limit sell โดย trigger price ต่ำกว่า bid |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | stop-limit order ถูก accept ด้วย trigger price, limit price ที่ถูกต้อง และ side=SELL |
| **ข้ามเมื่อ** | adapter ไม่รองรับ order แบบ `StopLimit` |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .enable_stop_buys(false)
    .enable_stop_sells(true)
    .stop_order_type(OrderType::StopLimit)
    .stop_limit_offset_ticks(50)
    .build()?
```

### TC-E24: MarketIfTouched BUY [#tc-e24-marketiftouched-buy]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | วาง MIT buy โดย trigger ต่ำกว่า bid ปัจจุบัน (ซื้อตอนราคาย่อ) |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | MIT order ถูก accept บน venue ด้วย trigger price ที่ถูกต้อง |
| **ข้ามเมื่อ** | adapter ไม่รองรับ order แบบ `MarketIfTouched` |

### TC-E25: MarketIfTouched SELL [#tc-e25-marketiftouched-sell]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | วาง MIT sell โดย trigger สูงกว่า ask ปัจจุบัน (ขายตอนราคาขึ้น) |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | MIT order ถูก accept บน venue ด้วย trigger price ที่ถูกต้อง |
| **ข้ามเมื่อ** | adapter ไม่รองรับ order แบบ `MarketIfTouched` |

### TC-E26: LimitIfTouched BUY [#tc-e26-limitiftouched-buy]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | วาง LIT buy โดย trigger ต่ำกว่า bid พร้อม limit price offset |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | LIT order ถูก accept ด้วย trigger price และ limit price ที่ถูกต้อง |
| **ข้ามเมื่อ** | adapter ไม่รองรับ order แบบ `LimitIfTouched` |

### TC-E27: LimitIfTouched SELL [#tc-e27-limitiftouched-sell]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | วาง LIT sell โดย trigger สูงกว่า ask พร้อม limit price offset |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | LIT order ถูก accept ด้วย trigger price และ limit price ที่ถูกต้อง |
| **ข้ามเมื่อ** | adapter ไม่รองรับ order แบบ `LimitIfTouched` |

---

## กลุ่ม 4: การแก้ไข order [#group-4-order-modification]

ทดสอบการแก้ไข order (amend) และ workflow แบบ cancel-replace

| TC | ชื่อ | คำอธิบาย | ข้ามเมื่อ |
| --- | --- | --- | --- |
| TC-E30 | แก้ราคา limit BUY | amend limit buy ที่เปิดอยู่ไปราคาใหม่ | ไม่รองรับ modify |
| TC-E31 | แก้ราคา limit SELL | amend limit sell ที่เปิดอยู่ไปราคาใหม่ | ไม่รองรับ modify |
| TC-E32 | Cancel-replace limit BUY | cancel แล้ว submit limit buy ใหม่ที่ราคาใหม่ | ไม่ข้าม |
| TC-E33 | Cancel-replace limit SELL | cancel แล้ว submit limit sell ใหม่ที่ราคาใหม่ | ไม่ข้าม |
| TC-E34 | แก้ trigger price ของ stop | amend trigger price ของ stop order | ไม่มี modify หรือไม่มี stop |
| TC-E35 | Cancel-replace stop order | cancel แล้ว submit stop ใหม่ที่ trigger price ใหม่ | ไม่มี stop order |
| TC-E36 | Modify ถูกปฏิเสธ | modify บน adapter ที่ไม่รองรับ | adapter รองรับ modify |

`trigger_limit_order_maintenance_once=True` ต้องใช้โหมด modify หรือ cancel-replace ของ limit order อย่างใดอย่างหนึ่ง
พอดี และต้องเปิดฝั่ง limit อย่างน้อยหนึ่งฝั่ง ใช้ร่วมกับ bracket entry, `test_modify_rejected` และการ submit limit แบบ
batch สองฝั่งไม่ได้ การ validate config จะปฏิเสธการผสมเหล่านี้

### TC-E30: แก้ราคา limit BUY [#tc-e30-modify-limit-buy-price]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี GTC limit buy เปิดอยู่จาก TC-E10 |
| **การกระทำ** | ExecTester แก้ limit buy ที่เปิดอยู่ไปเป็นราคาใหม่ |
| **ลำดับ event** | `OrderPendingUpdate` -> `OrderUpdated` |
| **เกณฑ์ผ่าน** | มี log ของ event `OrderUpdated` พร้อมราคาใหม่ และ order ออกจาก `PendingUpdate` |
| **ข้ามเมื่อ** | adapter ไม่รองรับการแก้ไข order |

**ข้อควรพิจารณา**:

- `trigger_limit_order_maintenance_once=True` จะ amend ไปเป็นราคาอื่นที่ถูกต้องทันทีตั้งแต่ `OrderAccepted` ตัวแรก
  โดยเลือก passive ขึ้นหนึ่ง tick ก่อน book ที่เงียบไม่ถือว่าสรุปผลไม่ได้
- หลัง workflow แบบครั้งเดียวเริ่มแล้ว มันจะหยุดการดูแล limit ของฝั่งนั้นต่อ ดังนั้น tester จะไม่ amend order กลับไปที่เป้า
  TOB ทันที
- ถ้าไม่มี flag นั้น การ modify จะรอให้ราคาของ order เบี่ยงออกจาก TOB offset เป้าหมาย ถ้าตลาดไม่ขยับและไม่มี
  `OrderUpdated` ถือว่าสรุปผลไม่ได้ ไม่ใช่การไม่ผ่าน
- ตรวจว่า log ของ `OrderUpdated` แสดงราคาที่คาดไว้ ถ้า event ไม่มาถึงเลยหลังการ amend แบบบังคับหรือแบบที่เกิดจากการเบี่ยง
  order จะค้างอยู่ใน `PendingUpdate` และ tester จะหยุดแก้ไขมัน

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(false)
    .modify_orders_to_maintain_tob_offset(true)
    .trigger_limit_order_maintenance_once(true)
    .build()?
```

### TC-E31: แก้ราคา limit SELL [#tc-e31-modify-limit-sell-price]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี GTC limit sell เปิดอยู่จาก TC-E11 |
| **การกระทำ** | ExecTester แก้ limit sell ที่เปิดอยู่ไปเป็นราคาใหม่ |
| **ลำดับ event** | `OrderPendingUpdate` -> `OrderUpdated` |
| **เกณฑ์ผ่าน** | มี log ของ event `OrderUpdated` พร้อมราคาใหม่ และ order ออกจาก `PendingUpdate` |
| **ข้ามเมื่อ** | adapter ไม่รองรับการแก้ไข order |

**ข้อควรพิจารณา**:

- ใช้แนวทางเรื่อง one-shot trigger และ book ที่เงียบแบบเดียวกับ TC-E30

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(false)
    .enable_limit_sells(true)
    .modify_orders_to_maintain_tob_offset(true)
    .trigger_limit_order_maintenance_once(true)
    .build()?
```

### TC-E32: Cancel-replace limit BUY [#tc-e32-cancel-replace-limit-buy]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี GTC limit buy เปิดอยู่ |
| **การกระทำ** | ExecTester cancel แล้ว submit limit buy ใหม่ที่ราคาใหม่ |
| **ลำดับ event** | `OrderPendingCancel` -> `OrderCanceled` -> `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order เดิมถูก cancel และ order ใหม่ถูก accept ที่ราคาที่อัปเดตแล้ว |
| **ข้ามเมื่อ** | ไม่ข้าม (cancel-replace ใช้ได้เสมอ) |

**ข้อควรพิจารณา**:

- นี่คือทางเลือกที่ใช้ได้ทั่วไปเมื่อ adapter ไม่รองรับ modify แบบ native
- จะมี order สองตัวที่แยกกันใน cache: ตัวเดิมที่ถูก cancel และตัวใหม่ที่มาแทน
- `trigger_limit_order_maintenance_once=True` จะขอ cancel หนึ่งครั้งทันทีตั้งแต่ `OrderAccepted` ตัวแรก แล้ว submit
  ตัวแทนหลังจากได้ `OrderCanceled` แล้วเท่านั้น
- ถ้าไม่มี one-shot trigger cancel-replace จะรอให้ TOB เบี่ยง ถ้าตลาดไม่ขยับและไม่มีตัวแทน ถือว่าสรุปผลไม่ได้ ไม่ใช่การไม่ผ่าน

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(false)
    .cancel_replace_orders_to_maintain_tob_offset(true)
    .trigger_limit_order_maintenance_once(true)
    .build()?
```

### TC-E33: Cancel-replace limit SELL [#tc-e33-cancel-replace-limit-sell]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี GTC limit sell เปิดอยู่ |
| **การกระทำ** | ExecTester cancel แล้ว submit limit sell ใหม่ที่ราคาใหม่ |
| **ลำดับ event** | `OrderPendingCancel` -> `OrderCanceled` -> `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order เดิมถูก cancel และ order ใหม่ถูก accept ที่ราคาที่อัปเดตแล้ว |
| **ข้ามเมื่อ** | ไม่ข้าม |

**ข้อควรพิจารณา**:

- ใช้แนวทางเรื่อง one-shot trigger และ book ที่เงียบแบบเดียวกับ TC-E32

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(false)
    .enable_limit_sells(true)
    .cancel_replace_orders_to_maintain_tob_offset(true)
    .trigger_limit_order_maintenance_once(true)
    .build()?
```

### TC-E34: แก้ trigger price ของ stop [#tc-e34-modify-stop-trigger-price]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี stop order เปิดอยู่จาก TC-E20 หรือ TC-E22 |
| **การกระทำ** | ExecTester แก้ trigger price ของ stop ตามการเคลื่อนไหวของตลาด (`modify_stop_orders_to_maintain_offset=True`) |
| **ลำดับ event** | `OrderPendingUpdate` -> `OrderUpdated` |
| **เกณฑ์ผ่าน** | มี log ของ event `OrderUpdated` พร้อม trigger price ใหม่ และ order ออกจาก `PendingUpdate` |
| **ข้ามเมื่อ** | adapter ไม่รองรับ modify stop แบบ native หรือไม่รองรับ stop order |

**ข้อควรพิจารณา**:

- บาง venue อนุญาตให้ modify limit order แต่ปฏิเสธการ replace trigger order สำหรับ adapter เหล่านั้น ให้ข้าม TC-E34 และ
  รัน TC-E35 แทน

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_stop_buys(true)
    .modify_stop_orders_to_maintain_offset(true)
    .build()?
```

### TC-E35: Cancel-replace stop order [#tc-e35-cancel-replace-stop-order]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี stop order เปิดอยู่ |
| **การกระทำ** | ExecTester cancel แล้ว submit stop ใหม่ที่ trigger price ใหม่ |
| **ลำดับ event** | `OrderPendingCancel` -> `OrderCanceled` -> `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | stop เดิมถูก cancel และ stop ใหม่ถูก accept ที่ trigger price ที่อัปเดตแล้ว |
| **ข้ามเมื่อ** | ไม่รองรับ stop order |

**ข้อควรพิจารณา**:

- นี่คือ path ที่จำเป็นสำหรับ venue ที่ไม่รองรับการ replace trigger order แบบ native
- หลัง stop ใหม่ถูก accept ให้ restart หรือบังคับ reconciliation แล้วตรวจว่ามี trigger order ปัจจุบันเหลืออยู่หนึ่งตัวพอดี

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_stop_buys(true)
    .cancel_replace_stop_orders_to_maintain_offset(true)
    .build()?
```

### TC-E36: Modify ถูกปฏิเสธ [#tc-e36-modify-rejected]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี limit order เปิดอยู่ และ adapter *ไม่* รองรับ modify |
| **การกระทำ** | พยายาม modify order (ผ่านโค้ด ไม่ใช่ผ่านการดูแลอัตโนมัติของ ExecTester) |
| **ลำดับ event** | `OrderModifyRejected` |
| **เกณฑ์ผ่าน** | การพยายาม modify ได้ event `OrderModifyRejected` พร้อมเหตุผล และ order เดิมไม่เปลี่ยนแปลง |
| **ข้ามเมื่อ** | adapter รองรับการแก้ไข order |

**ข้อควรพิจารณา**:

- test นี้ทดสอบ path การปฏิเสธของ adapter ไม่ใช่ logic cancel-replace ของ ExecTester
- เหตุผลการปฏิเสธควรระบุว่าไม่รองรับการแก้ไข

---

## กลุ่ม 5: การ cancel order [#group-5-order-cancellation]

ทดสอบ workflow การ cancel order

| TC | ชื่อ | คำอธิบาย | ข้ามเมื่อ |
| --- | --- | --- | --- |
| TC-E40 | Cancel limit order ตัวเดียว | cancel limit order ที่เปิดอยู่ | ไม่ข้าม |
| TC-E41 | Cancel ทั้งหมดตอนหยุด | การหยุด strategy จะ cancel order ที่เปิดอยู่ทั้งหมด (default) | ไม่ข้าม |
| TC-E42 | Cancel ทีละตัวตอนหยุด | cancel order ทีละตัวตอนหยุด | ไม่ข้าม |
| TC-E43 | Batch cancel ตอนหยุด | cancel order ผ่าน batch API ตอนหยุด | ไม่มี batch cancel |
| TC-E44 | Cancel order ที่ถูก cancel แล้ว | ตรวจ rejection ตามที่ระบุไว้ หรือผลแบบ idempotent | ไม่ข้าม |

### TC-E40: Cancel limit order ตัวเดียว [#tc-e40-cancel-single-limit-order]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี GTC limit order เปิดอยู่จาก TC-E10 หรือ TC-E11 |
| **การกระทำ** | หยุด strategy แล้ว ExecTester cancel limit order ที่เปิดอยู่ |
| **ลำดับ event** | `OrderPendingCancel` -> `OrderCanceled` |
| **เกณฑ์ผ่าน** | สถานะ order เปลี่ยนเป็น CANCELED และไม่มี order เปิดเหลืออยู่ |
| **ข้ามเมื่อ** | ไม่ข้าม |

**ข้อควรพิจารณา**:

- `cancel_orders_on_stop=True` (default) จะ trigger การ cancel เมื่อ strategy หยุด
- ตรวจว่า event `OrderCanceled` มี `venue_order_id` ที่ถูกต้อง

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(false)
    .cancel_orders_on_stop(true)
    .build()?
```

### TC-E41: Cancel ทั้งหมดตอนหยุด [#tc-e41-cancel-all-on-stop]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี order เปิดอยู่หลายตัว (limit buy + limit sell จาก TC-E12) |
| **การกระทำ** | หยุด strategy โดย `cancel_orders_on_stop=True` (default) |
| **ลำดับ event** | สำหรับแต่ละ order: `OrderPendingCancel` -> `OrderCanceled` |
| **เกณฑ์ผ่าน** | order ที่เปิดอยู่ทั้งหมดถูก cancel และไม่มี order เปิดเหลืออยู่ |
| **ข้ามเมื่อ** | ไม่ข้าม |

**ข้อควรพิจารณา**:

- การหยุดแบบ default ใช้ `cancel_all` บาง venue implement สิ่งนี้แบบครอบคลุมทั้ง account ให้ยืนยันขอบเขตของ
  `CancelAllOrders` ของ adapter ก่อนรันข้าง ๆ order อื่นที่เปิดอยู่ ใช้ `use_individual_cancels_on_stop` หรือ
  `use_batch_cancel_on_stop` เพื่อแยกขอบเขต

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(true)
    .cancel_orders_on_stop(true)
    .build()?
```

### TC-E42: Cancel ทีละตัวตอนหยุด [#tc-e42-individual-cancels-on-stop]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี order เปิดอยู่หลายตัว |
| **การกระทำ** | หยุดโดย `use_individual_cancels_on_stop=True` |
| **ลำดับ event** | `OrderPendingCancel` -> `OrderCanceled` แยกกันสำหรับแต่ละ order |
| **เกณฑ์ผ่าน** | แต่ละ order ถูก cancel ทีละตัว และทุก order ไปถึงสถานะ CANCELED |
| **ข้ามเมื่อ** | ไม่ข้าม |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(true)
    .use_individual_cancels_on_stop(true)
    .build()?
```

### TC-E43: Batch cancel ตอนหยุด [#tc-e43-batch-cancel-on-stop]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี order เปิดอยู่หลายตัว และ adapter รองรับ batch cancel |
| **การกระทำ** | หยุดโดย `use_batch_cancel_on_stop=True` |
| **ลำดับ event** | `OrderPendingCancel` -> `OrderCanceled` แบบ batch สำหรับทุก order |
| **เกณฑ์ผ่าน** | ทุก order ถูก cancel ผ่าน batch request เดียว และทุกตัวไปถึงสถานะ CANCELED |
| **ข้ามเมื่อ** | adapter ไม่รองรับ batch cancel |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(true)
    .use_batch_cancel_on_stop(true)
    .build()?
```

### TC-E44: Cancel order ที่ถูก cancel แล้ว [#tc-e44-cancel-already-canceled-order]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี order ที่ถูก cancel ไปแล้ว (จาก TC-E40) |
| **การกระทำ** | พยายาม cancel order เดิมซ้ำอีกครั้ง |
| **ลำดับ event** | `OrderCancelRejected` โดย default หรือไม่มี event ใหม่สำหรับผลแบบ idempotent ที่ระบุไว้ |
| **เกณฑ์ผ่าน** | ผลตรงกับ contract ของ adapter และไม่มี terminal event ซ้ำ |
| **ข้ามเมื่อ** | ไม่ข้าม |

**ข้อควรพิจารณา**:

- contract แบบ default ทดสอบการจัดการ error ของ adapter สำหรับ cancel request ที่ไม่ถูกต้อง เหตุผลการปฏิเสธควรระบุว่า
  order ไม่อยู่ในสถานะที่ cancel ได้
- `Strategy.cancel_order` และ order manager จะทิ้ง cancel ของ order ที่ปิดแล้วในเครื่อง ดังนั้น `ExecTester` จึงไปไม่ถึง
  adapter หลัง TC-E40 การเรียก `cancel_order` ครั้งที่สองที่ไม่เคยออกจาก process ถือว่าสรุปผลไม่ได้
- venue อาจถือว่าการ cancel order ที่ terminal แล้วเป็นแบบ idempotent ให้บันทึกพฤติกรรมนั้นไว้ และใช้ test ที่เน้น adapter
  ซึ่ง submit `CancelOrder` พร้อม venue order ID หลังจาก order terminal แล้ว

---

## กลุ่ม 6: Bracket order [#group-6-bracket-orders]

ทดสอบการ submit bracket order (entry + take-profit + stop-loss)

| TC | ชื่อ | คำอธิบาย | ข้ามเมื่อ |
| --- | --- | --- | --- |
| TC-E50 | Bracket BUY | entry limit buy + TP limit sell + SL stop sell | ไม่รองรับ bracket |
| TC-E51 | Bracket SELL | entry limit sell + TP limit buy + SL stop buy | ไม่รองรับ bracket |
| TC-E52 | Entry fill แล้ว bracket ทำงาน | ตรวจว่า TP/SL active หลัง entry ถูก fill | ไม่รองรับ bracket |
| TC-E53 | Bracket ที่ entry เป็น post-only | entry order ใช้ flag post-only | ไม่มี bracket หรือ PO |

### TC-E50: Bracket BUY [#tc-e50-bracket-buy]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester submit bracket order: entry เป็น limit buy + take-profit sell + stop-loss sell |
| **ลำดับ event** | Entry: `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` ส่วน TP และ SL: `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order สามตัวถูกสร้างและ accept: entry ต่ำกว่า bid, TP สูงกว่า ask, SL ต่ำกว่า entry |
| **ข้ามเมื่อ** | adapter ไม่รองรับ bracket order |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_brackets(true)
    .bracket_entry_order_type(OrderType::Limit)
    .bracket_offset_ticks(500)
    .enable_limit_buys(true)
    .enable_limit_sells(false)
    .build()?
```

### TC-E51: Bracket SELL [#tc-e51-bracket-sell]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester submit bracket: entry เป็น limit sell + TP buy + SL buy |
| **ลำดับ event** | รูปแบบเดียวกับ TC-E50 แต่เป็นฝั่ง sell |
| **เกณฑ์ผ่าน** | order สามตัวถูกสร้างและ accept บนฝั่ง sell |
| **ข้ามเมื่อ** | adapter ไม่รองรับ bracket order |

### TC-E52: Entry fill แล้ว TP/SL ทำงาน [#tc-e52-bracket-entry-fill-activates-tpsl]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | bracket order จาก TC-E50 ที่ entry order ถูก fill |
| **การกระทำ** | entry order ถูก fill แล้วตรวจว่า contingent order TP และ SL active |
| **ลำดับ event** | Entry: `OrderFilled` จากนั้น TP และ SL เปลี่ยนจาก contingent เป็น active |
| **เกณฑ์ผ่าน** | หลัง entry ถูก fill order TP และ SL ทำงานอยู่บน venue |
| **ข้ามเมื่อ** | adapter ไม่รองรับ bracket order |

**ข้อควรพิจารณา**:

- ต้องให้ entry order ถูก fill จริง ซึ่งอาจต้องใช้ราคาแบบ aggressive
- กลไกการทำให้ TP/SL active แตกต่างกันตาม venue (บางที่ active ทันที บางที่เป็นกลุ่ม OCA)

### TC-E53: Bracket ที่ entry เป็น post-only [#tc-e53-bracket-with-post-only-entry]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter รองรับ bracket และ post-only |
| **การกระทำ** | submit bracket ด้วย `use_post_only=True` (ใช้กับ entry และ TP) |
| **ลำดับ event** | เหมือน TC-E50 โดย entry มี flag post-only |
| **เกณฑ์ผ่าน** | entry และ TP ถูก accept แบบ post-only (maker) ส่วน SL ไม่ใช่ post-only |
| **ข้ามเมื่อ** | ไม่รองรับ bracket หรือไม่รองรับ post-only |

---

## กลุ่ม 7: Order flag [#group-7-order-flags]

ทดสอบ flag ระดับ order และพารามิเตอร์พิเศษ

| TC | ชื่อ | คำอธิบาย | ข้ามเมื่อ |
| --- | --- | --- | --- |
| TC-E60 | PostOnly ถูก accept | limit แบบ post-only วางห่างจาก TOB | ไม่มี post-only |
| TC-E61 | ReduceOnly ตอนปิด | ปิด position ด้วย flag reduce-only | ไม่มี reduce-only |
| TC-E62 | Display quantity | iceberg order ที่ quantity ที่มองเห็น < ทั้งหมด | ไม่มี display quantity |
| TC-E63 | Custom order params | พารามิเตอร์เฉพาะของ adapter ผ่าน `order_params` | ไม่เกี่ยวข้อง |

### TC-E60: PostOnly ถูก accept [#tc-e60-postonly-accepted]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วาง limit buy ด้วย `use_post_only=True` ที่ราคา passive |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order ถูก accept เป็น maker order และ venue รับรู้ flag post-only |
| **ข้ามเมื่อ** | adapter ไม่รองรับ flag post-only |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(false)
    .use_post_only(true)
    .build()?
```

### TC-E61: ReduceOnly ตอนปิด [#tc-e61-reduceonly-on-close]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี position เปิดอยู่ (จาก TC-E01) |
| **การกระทำ** | หยุด strategy โดย `reduce_only_on_stop=True` order ปิดจะใช้ flag reduce-only |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` (พร้อม reduce-only) |
| **เกณฑ์ผ่าน** | order ปิดมี flag reduce-only และ position ถูกปิดทั้งหมด |
| **ข้ามเมื่อ** | adapter ไม่รองรับ flag reduce-only |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .open_position_on_start_qty(Decimal::new(1, 2))
    .reduce_only_on_stop(true)
    .close_positions_on_stop(true)
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .build()?
```

### TC-E62: Display quantity (iceberg) [#tc-e62-display-quantity-iceberg]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว และ adapter รองรับ display quantity |
| **การกระทำ** | วาง limit order โดย `order_display_qty` < `order_qty` |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order ถูก accept พร้อมตั้ง display quantity และมองเห็นเฉพาะ display qty บน book |
| **ข้ามเมื่อ** | adapter ไม่รองรับ display quantity / iceberg order |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("1.0"))
    .enable_limit_buys(true)
    .enable_limit_sells(false)
    .order_display_qty(Quantity::from("0.1"))
    .build()?
```

### TC-E63: Custom order params [#tc-e63-custom-order-params]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว และ adapter รับพารามิเตอร์เพิ่มเติมได้ |
| **การกระทำ** | วาง order พร้อม `order_params` ที่มีพารามิเตอร์เฉพาะของ adapter |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order ถูก accept และพารามิเตอร์เฉพาะของ adapter ถูกส่งผ่านไปถึง venue |
| **ข้ามเมื่อ** | ไม่เกี่ยวข้อง (ขึ้นกับ adapter) |

**ข้อควรพิจารณา**:

- `order_params` เป็นข้อมูลทึบ (opaque) สำหรับ ExecTester และถูกส่งผ่านไปยัง adapter โดยตรง
- ดูพารามิเตอร์ที่รองรับได้จากคู่มือของ adapter

---

## กลุ่ม 8: การจัดการ rejection [#group-8-rejection-handling]

ทดสอบว่า adapter จัดการและรายงาน order rejection ได้ถูกต้อง

| TC | ชื่อ | คำอธิบาย | ข้ามเมื่อ |
| --- | --- | --- | --- |
| TC-E70 | PostOnly rejection | post-only order ที่จะ cross spread | ไม่มี post-only |
| TC-E71 | ReduceOnly rejection | reduce-only order ที่ไม่มี position ให้ลด | ไม่มี reduce-only |
| TC-E72 | Order type ที่ไม่รองรับ | submit order type ที่ adapter ไม่รองรับ | ไม่ข้าม |
| TC-E73 | TIF ที่ไม่รองรับ | submit order ด้วย time in force ที่ไม่รองรับ | ไม่ข้าม |
| TC-E74 | Submit ล้มเหลวแบบกำกวม | ความล้มเหลวของ transport, timeout หรือการส่งตอน submit | ไม่มี mock path |
| TC-E75 | Cancel ล้มเหลวแบบกำกวม | ความล้มเหลวของ transport, timeout หรือการส่งตอน cancel | ไม่มี cancel |
| TC-E76 | Modify ล้มเหลวแบบกำกวม | ความล้มเหลวของ transport, timeout หรือการส่งตอน modify | ไม่มี modify |
| TC-E77 | Batch ล้มเหลวแบบกำกวม | ทั้ง batch ล้มเหลวโดยไม่มีผลราย order | ไม่มี batch |
| TC-E78 | Batch reject ราย order | response ของ batch มี rejection ราย order อย่างชัดเจน | ไม่มี batch |

TC-E74 ถึง TC-E78 ระบุรวมกันด้านล่าง เพราะปกติต้องใช้ mock ที่ขอบเขต HTTP หรือ WebSocket แทน venue จริง

### ความล้มเหลวที่ผลลัพธ์กำกวม [#ambiguous-outcome-failures]

case เหล่านี้พิสูจน์ว่าความล้มเหลวของ request จาก adapter จะไม่กลายเป็น terminal rejection event เมื่อไม่ทราบผลลัพธ์
ที่ venue เกณฑ์ผ่านยังกำหนดข้อยกเว้นสำหรับความล้มเหลวตอนเตรียม request ในเครื่อง: ต้องทราบแน่ชัดว่า command ยังไม่ได้
ถูกส่ง และความล้มเหลวต้องระบุได้ว่าเป็นของ cancel หรือ modify command ตัวเดียว ถ้าเงื่อนไขทั้งสองเป็นจริง adapter จึง
ปล่อย rejection event ที่ตรงกันได้

**เกณฑ์ผ่าน**:

- ความล้มเหลวของ submit จาก transport error, timeout, การส่ง WebSocket ล้มเหลว, retry หมด หรือ parse response
  ล้มเหลว จะไม่ปล่อย `OrderRejected`
- ความล้มเหลวของ cancel จาก transport error, timeout, การส่ง WebSocket ล้มเหลว, retry หมด หรือ server ล้มเหลวทั้ง
  request จะไม่ปล่อย `OrderCancelRejected`
- ความล้มเหลวของ modify จาก transport error, timeout, การส่ง WebSocket ล้มเหลว, retry หมด หรือ server ล้มเหลวทั้ง
  request จะไม่ปล่อย `OrderModifyRejected`
- ความล้มเหลวตอนเตรียม cancel ในเครื่องที่พิสูจน์ได้ว่าส่ง command ไม่ได้ อาจปล่อย `OrderCancelRejected` ได้ เมื่อ adapter
  ระบุได้ว่าความล้มเหลวเป็นของ cancel command ตัวเดียว
- ความล้มเหลวตอนเตรียม modify ในเครื่องที่พิสูจน์ได้ว่าส่ง command ไม่ได้ อาจปล่อย `OrderModifyRejected` ได้ เมื่อ
  adapter ระบุได้ว่าความล้มเหลวเป็นของ modify command ตัวเดียว
- ความล้มเหลวของ request ทั้ง batch จะไม่ปล่อย rejection ทีละ order เมื่อ venue ไม่ได้คืนผลราย order
- rejection ราย order ที่ venue ระบุชัดเจนยังคงปล่อย rejection event ที่ตรงกันพร้อมเหตุผลจาก venue

order จะยังคงอยู่ในสถานะ in-flight ที่เหมาะสม จนกว่า update จาก venue, ผล query หรือรอบ reconciliation จะ resolve มัน

### TC-E70: PostOnly rejection [#tc-e70-postonly-rejection]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วาง post-only order ผิดฝั่งของ book (`test_reject_post_only=True`) ทำให้ cross spread |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderRejected` |
| **เกณฑ์ผ่าน** | venue reject order, `OrderRejected.due_post_only=true` และเหตุผลระบุว่าละเมิด post-only |
| **ข้ามเมื่อ** | adapter ไม่รองรับ flag post-only |

**ข้อควรพิจารณา**:

- โหมด `test_reject_post_only` ของ ExecTester ตั้งราคา order ให้ cross โดยเจตนา
- บาง venue อาจ partial fill แทนการ reject พฤติกรรมขึ้นกับ venue
- adapter ที่ปล่อย `OrderRejected` สำหรับการ reject เพราะ post-only cross ควรตั้ง `due_post_only=true` เพื่อให้ strategy
  แยกกรณีนี้ออกจาก rejection อื่นของ venue ได้

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .enable_limit_buys(true)
    .enable_limit_sells(false)
    .use_post_only(true)
    .test_reject_post_only(true)
    .build()?
```

### TC-E71: ReduceOnly rejection [#tc-e71-reduceonly-rejection]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว และไม่มี position เปิดอยู่สำหรับ instrument นั้น |
| **การกระทำ** | ExecTester เปิด market position ด้วย `reduce_only=True` ผ่าน `test_reject_reduce_only=True` และ `open_position_on_start_qty` ขณะที่ไม่มี position ให้ลด |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderRejected` |
| **เกณฑ์ผ่าน** | order ถูก reject โดยมี event `OrderRejected` พร้อมเหตุผลที่ระบุว่าละเมิด reduce-only |
| **ข้ามเมื่อ** | adapter ไม่รองรับ flag reduce-only |

**ข้อควรพิจารณา**:

- flag `test_reject_reduce_only` ใช้กับ market order เปิดที่ submit ผ่าน `open_position_on_start_qty` เท่านั้น
- ตรวจว่าไม่มี position เดิมของ instrument อยู่ก่อนรัน test นี้

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .open_position_on_start_qty(Decimal::new(1, 2))
    .test_reject_reduce_only(true)
    .enable_limit_buys(false)
    .enable_limit_sells(false)
    .build()?
```

### TC-E72: Order type ที่ไม่รองรับ [#tc-e72-unsupported-order-type]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว และ order type ไม่อยู่ในชุดที่ adapter รองรับ |
| **การกระทำ** | submit order type ที่ adapter ไม่รองรับ |
| **ลำดับ event** | `OrderDenied` (adapter ปฏิเสธก่อน submit) |
| **เกณฑ์ผ่าน** | order ถูก deny ก่อนถึง venue โดยมี event `OrderDenied` พร้อมเหตุผล |
| **ข้ามเมื่อ** | ไม่ข้าม (ทุก adapter มี order type ที่ไม่รองรับให้ทดสอบ) |

**ข้อควรพิจารณา**:

- `OrderDenied` เกิดที่ระดับ adapter ก่อนที่ order จะถึง venue
- ต่างจาก path ปกติของ `OrderRejected` ซึ่งเกิดตามหลังการ reject จาก venue reconciliation ก็สังเคราะห์ `OrderRejected`
  ได้เช่นกัน ดู [Terminal reconciliation
  provenance](../concepts/execution/policies.md#terminal-reconciliation-provenance)
- ทดสอบโดยตั้งค่า stop order type ที่ adapter ไม่รองรับ

### TC-E73: TIF ที่ไม่รองรับ [#tc-e73-unsupported-tif]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว และ TIF ไม่อยู่ในชุดที่ adapter รองรับ |
| **การกระทำ** | submit order ด้วย TIF ที่ adapter ไม่รองรับ |
| **ลำดับ event** | `OrderDenied` (adapter ปฏิเสธก่อน submit) |
| **เกณฑ์ผ่าน** | order ถูก deny ก่อนถึง venue โดยมี event `OrderDenied` พร้อมเหตุผล |
| **ข้ามเมื่อ** | ไม่ข้าม (ทุก adapter มีตัวเลือก TIF ที่ไม่รองรับให้ทดสอบ) |

**ข้อควรพิจารณา**:

- คล้าย TC-E72 แต่เป็นตัวเลือก time-in-force
- ทดสอบด้วยค่า TIF จาก enum ของ Nautilus ที่ adapter ไม่ได้ map

---

## กลุ่ม 9: Lifecycle และ reconciliation [#group-9-lifecycle-and-reconciliation]

ทดสอบพฤติกรรม lifecycle ของ strategy และการกู้คืน execution state ตอนเริ่ม ตอนหยุด และเมื่อ report ล้มเหลว

TC-E88 และ TC-E89 รันแบบ offline เป็น unit test หรือ integration test ของ adapter ที่ให้ผลแน่นอน (deterministic) แทนการ
รันผ่าน `ExecTester` กับ venue

| TC | ชื่อ | คำอธิบาย | ข้ามเมื่อ |
| --- | --- | --- | --- |
| TC-E80 | เปิด position ตอนเริ่ม | เปิด position ทันทีเมื่อ strategy เริ่ม | ไม่มี market order |
| TC-E81 | Cancel order ตอนหยุด | cancel order ที่เปิดอยู่ทั้งหมดเมื่อ strategy หยุด | ไม่ข้าม |
| TC-E82 | ปิด position ตอนหยุด | ปิด position ที่เปิดอยู่เมื่อ strategy หยุด | ไม่มี market order |
| TC-E83 | Unsubscribe ตอนหยุด | unsubscribe จาก data feed เมื่อ strategy หยุด | ไม่รองรับ unsub |
| TC-E84 | Reconcile open order | reconcile open order ที่มีอยู่จาก session ก่อนหน้า | ไม่ข้าม |
| TC-E85 | Reconcile filled order | reconcile order ที่ fill แล้วจาก session ก่อนหน้า | ไม่ข้าม |
| TC-E86 | Reconcile open long | reconcile long position ที่เปิดอยู่ | ไม่ข้าม |
| TC-E87 | Reconcile open short | reconcile short position ที่เปิดอยู่ | ไม่ข้าม |
| TC-E88 | Commission ล้มเหลวตอน reconciliation | แทนค่า commission ของ fill ที่จำเป็นไม่ได้ | ไม่มี logic ของ fill commission |
| TC-E89 | Commission ล้มเหลวทาง WebSocket | แทนค่า commission ของ private fill ไม่ได้ | ไม่มี private fill stream หรือ logic ของ commission |

### TC-E80: เปิด position ตอนเริ่ม [#tc-e80-open-position-on-start]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด instrument แล้ว, ไม่มี position เดิม |
| **การกระทำ** | strategy เริ่มทำงานโดยตั้ง `open_position_on_start_qty` |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` |
| **เกณฑ์ผ่าน** | position เปิดตอนเริ่ม market order ถูก submit และ fill ก่อนที่การดูแล limit order จะเริ่ม |
| **ข้ามเมื่อ** | adapter ไม่รองรับ market order |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .open_position_on_start_qty(Decimal::new(1, 2))
    .build()?
```

### TC-E81: Cancel order ตอนหยุด [#tc-e81-cancel-orders-on-stop]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี limit order เปิดอยู่จาก session ของ strategy |
| **การกระทำ** | หยุด strategy โดย `cancel_orders_on_stop=True` (default) |
| **ลำดับ event** | สำหรับแต่ละ order ที่เปิดอยู่: `OrderPendingCancel` -> `OrderCanceled` |
| **เกณฑ์ผ่าน** | open order ทั้งหมดที่ strategy เป็นเจ้าของถูก cancel ตอนหยุด |
| **ข้ามเมื่อ** | ไม่ข้าม |

### TC-E82: ปิด position ตอนหยุด [#tc-e82-close-positions-on-stop]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี position เปิดอยู่จาก session ของ strategy |
| **การกระทำ** | หยุด strategy โดย `close_positions_on_stop=True` (default) |
| **ลำดับ event** | order ปิด: `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` |
| **เกณฑ์ผ่าน** | position ถูกปิด หรือเหลือเพียงเศษที่ต่ำกว่า precision พอดี และไม่มี order เปิดค้างอยู่ |
| **ข้ามเมื่อ** | adapter ไม่รองรับ market order |

### TC-E83: Unsubscribe ตอนหยุด [#tc-e83-unsubscribe-on-stop]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี data subscription ที่ active อยู่ (quote, trade, book) |
| **การกระทำ** | หยุด strategy โดย `can_unsubscribe=True` (default) |
| **ลำดับ event** | data subscription ถูกยกเลิก |
| **เกณฑ์ผ่าน** | ไม่ได้รับ data event อีกหลังหยุด และตัดการเชื่อมต่ออย่างเรียบร้อย |
| **ข้ามเมื่อ** | adapter ไม่รองรับ unsubscribe |

**Rust config**:

```rust
ExecTesterConfig::builder()
    .base(StrategyConfig {
        strategy_id: Some(strategy_id),
        ..Default::default()
    })
    .instrument_id(instrument_id)
    .client_id(client_id)
    .order_qty(Quantity::from("0.01"))
    .can_unsubscribe(true)
    .build()?
```

### TC-E84: Reconcile open order [#tc-e84-reconcile-open-orders]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี limit order เปิดอยู่บน venue อย่างน้อยหนึ่งตัวจาก session ก่อนหน้า |
| **การกระทำ** | เริ่ม node โดย `reconciliation=True` |
| **ลำดับ event** | สร้าง `OrderStatusReport` สำหรับแต่ละ open order |
| **เกณฑ์ผ่าน** | open order แต่ละตัวถูกโหลดเข้า cache ด้วย `venue_order_id`, status=ACCEPTED, price, quantity, side และ order type ที่ถูกต้อง |
| **ข้ามเมื่อ** | ไม่ข้าม |

**ข้อควรพิจารณา**:

- ปล่อย limit order เปิดค้างไว้จาก test session ก่อนหน้า (ไม่ cancel ตอนหยุด)
- ตั้ง `external_order_instrument_ids` เพื่อให้การลงทะเบียน strategy สร้าง claim ที่ active ซึ่งใช้ assign order ที่
  reconcile แล้ว
- ตรวจว่าจำนวน order ที่ reconcile ตรงกับจำนวนที่ venue รายงาน
- mass-status อาจรวม terminal order ในอดีตด้วย ตัวที่ไม่มีใคร claim จะปรากฏเป็น EXTERNAL ให้เทียบจำนวน open order
  กับ endpoint open order ของ venue ไม่ใช่ความยาวทั้งหมดของ mass-status
- ค่า `ts_event` ของ report ต้องเป็น Unix nanoseconds แม้ venue จะมีความละเอียดแค่ระดับวินาที

### TC-E85: Reconcile filled order [#tc-e85-reconcile-filled-orders]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี order ที่ fill แล้วบน venue อย่างน้อยหนึ่งตัวจาก session ก่อนหน้า |
| **การกระทำ** | เริ่ม node โดย `reconciliation=True` |
| **ลำดับ event** | สร้าง `FillReport` สำหรับแต่ละ fill ในอดีต |
| **เกณฑ์ผ่าน** | order ที่ fill แล้วแต่ละตัวถูกโหลดเข้า cache ด้วย `venue_order_id`, status=FILLED, fill price, fill quantity และ commission ที่ถูกต้อง |
| **ข้ามเมื่อ** | ไม่ข้าม |

**ข้อควรพิจารณา**:

- ต้องมี order ที่ fill ใน session ก่อนหน้า
- ตรวจว่า fill price, quantity และ commission ตรงกับค่าที่ venue รายงาน
- บาง adapter อาจรายงานเฉพาะ fill ภายในช่วง lookback
- การ replay fill ในอดีตอาจเปลี่ยน commission ที่เป็นศูนย์ใน live ไปเป็นค่าธรรมเนียมจาก REST การเปลี่ยนนี้เป็นเรื่องที่คาดไว้
  เมื่อ stream ไม่ส่งค่าธรรมเนียมมาและ reconciliation เป็นผู้เติมให้
- ค่า `ts_event` ของ report ต้องเป็น Unix nanoseconds แม้ venue จะมีความละเอียดแค่ระดับวินาที

### TC-E86: Reconcile open long position [#tc-e86-reconcile-open-long-position]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี long position เปิดอยู่บน venue จาก session ก่อนหน้า |
| **การกระทำ** | เริ่ม node โดย `reconciliation=True` |
| **ลำดับ event** | สร้าง `PositionStatusReport` สำหรับ long position |
| **เกณฑ์ผ่าน** | position ถูกโหลดเข้า cache ด้วย instrument, side=LONG, quantity และ entry price ที่ตรงกับ venue |
| **ข้ามเมื่อ** | ไม่ข้าม |

**ข้อควรพิจารณา**:

- เปิด long position ใน session ก่อนหน้า แล้วหยุด strategy โดยไม่ปิด position (`close_positions_on_stop=False`)
- ตรวจว่า quantity และราคาเข้าเฉลี่ยของ position ที่ reconcile ตรงกับ venue
- หลัง reconciliation strategy ควรจัดการหรือปิด position นี้ได้

### TC-E87: Reconcile open short position [#tc-e87-reconcile-open-short-position]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี short position เปิดอยู่บน venue จาก session ก่อนหน้า |
| **การกระทำ** | เริ่ม node โดย `reconciliation=True` |
| **ลำดับ event** | สร้าง `PositionStatusReport` สำหรับ short position |
| **เกณฑ์ผ่าน** | position ถูกโหลดเข้า cache ด้วย instrument, side=SHORT, quantity และ entry price ที่ตรงกับ venue |
| **ข้ามเมื่อ** | ไม่ข้าม |

**ข้อควรพิจารณา**:

- เปิด short position ใน session ก่อนหน้า แล้วหยุด strategy โดยไม่ปิด position (`close_positions_on_stop=False`)
- ตรวจว่า quantity และราคาเข้าเฉลี่ยของ position ที่ reconcile ตรงกับ venue
- หลัง reconciliation strategy ควรจัดการหรือปิด position นี้ได้

### TC-E88: Commission ล้มเหลวตอน reconciliation [#tc-e88-reconciliation-commission-failure]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | fixture แบบ deterministic สร้าง commission ที่อยู่นอกช่วงสำหรับ fill ที่เป็นของเราและได้รับการยืนยันแล้ว |
| **การกระทำ** | ทดสอบ path ของ commission ทั้งแบบ direct fill, mass-status, targeted report และ inferred fill ด้วย fixture นั้น |
| **ลำดับ event** | report request คืน error, engine reconciliation log hook error และไม่ปล่อย inferred fill สำรอง |
| **เกณฑ์ผ่าน** | สังเกตเห็น error ได้ การเริ่มระบบล้มเหลวหรืองานแบบ inferred ถูกเลื่อนไป และ response ที่ถูกต้องในภายหลัง reconcile ได้ครั้งเดียวพอดี |
| **ข้ามเมื่อ** | adapter ไม่ได้คำนวณ commission สำหรับ fill report |

**ข้อควรพิจารณา**:

- assert ว่าไม่มี commission ที่เป็นศูนย์หรือค่าทั่วไปมาแทนการคำนวณของ venue ที่ล้มเหลว
- ทดสอบ inferred-fill hook ด้วย นอกเหนือจากการสร้าง `FillReport` โดยตรง
- ข้ามส่วน inferred fill เมื่อ adapter ไม่ได้ override commission hook ที่ใช้ร่วมกัน
- assert ว่า input ด้าน quantity, price และ liquidity ของ hook ตรงกับ inferred fill ที่ปล่อยออกมา รวมถึงราคาที่คำนวณย้อน
  (back-solved) ของเศษส่วนเพิ่มหลัง fill ก่อนหน้า
- fill ที่ระบุชัดเจนและถูกต้องอาจ apply ได้ แต่ quantity ที่เหลือและ terminal transition ที่ขึ้นกับมันยังคง pending
- การแก้ไขแบบสังเคราะห์เฉพาะ position ที่ไม่มีหลักฐานการ trade ไม่ถือเป็นความล้มเหลวในการคำนวณ commission

### TC-E89: Commission ล้มเหลวทาง WebSocket [#tc-e89-websocket-commission-failure]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | fixture ของ private fill สร้าง commission ที่อยู่นอกช่วง และมี venue trade ID ที่คงที่ |
| **การกระทำ** | ส่ง trade ที่ไม่ถูกต้อง แทน commission ด้วยค่าที่ถูกต้อง แล้วส่ง trade ID เดิมซ้ำ |
| **ลำดับ event** | การส่งครั้งแรกไม่ปล่อย fill และไม่เปลี่ยน state ด้าน fill, terminal หรือ deduplication ส่วนการ replay ปล่อย fill หนึ่งตัว |
| **เกณฑ์ผ่าน** | ไม่เกิด panic หรือ fallback, replay ที่ถูกต้อง apply ครั้งเดียวพอดี และ REST reconciliation กู้ replay ที่พลาดไปได้ |
| **ข้ามเมื่อ** | adapter ไม่มี private fill stream หรือไม่ได้คำนวณ commission สำหรับ private fill |

**ข้อควรพิจารณา**:

- สำหรับ venue trade ที่ fill order ของเราหลายตัว ให้ทำให้ report หนึ่งตัวล้มเหลว แล้ว assert ว่าการส่งครั้งแรกไม่ปล่อย report
  ใดเลย เพื่อป้องกันไม่ให้ replay สร้าง report ซ้ำกับที่สร้างไว้ก่อนเกิดความล้มเหลว
- assert ว่า adapter ใช้ deduplication key หลังจาก report ทั้งหมดสร้างและ route สำเร็จแล้วเท่านั้น
- assert ว่า trade ที่ล้มเหลวไม่ทำให้ order ของมันถูกยืนยันหรือเข้าสู่ terminal

---

## กลุ่ม 10: การเทรด option [#group-10-options-trading]

ทดสอบพฤติกรรม execution เฉพาะของ option instrument ประเภท option มักมีข้อจำกัดต่างจาก linear derivative: venue
อาจจำกัด order type, รองรับโหมดการตั้งราคาแบบอื่น หรือไม่อนุญาต conditional order ข้อจำกัดที่แน่ชัดแตกต่างกันตาม venue
ให้ดูคู่มือของ adapter

test เหล่านี้ต้องใช้ instrument แบบ `CryptoOption` ให้ใช้ option แบบ OTM ที่มี liquidity พอสมควรสำหรับการ fill

| TC | ชื่อ | คำอธิบาย | ข้ามเมื่อ |
| --- | --- | --- | --- |
| TC-E90 | Limit BUY option | วาง limit buy บน option instrument | ไม่รองรับ option |
| TC-E91 | Limit SELL option | วาง limit sell บน option instrument | ไม่รองรับ option |
| TC-E92 | Limit แบบตั้งราคาทางเลือก | วาง limit order ที่ตั้งราคาแบบเฉพาะของ adapter ผ่าน `order_params` | ไม่มีการตั้งราคาทางเลือก |
| TC-E94 | Order type ที่ไม่รองรับถูก deny | submit order type ที่ adapter ปฏิเสธสำหรับ option | ไม่รองรับ option |
| TC-E96 | Conditional order ถูก reject | submit stop/conditional order บน option และคาดว่าจะถูก reject | ไม่รองรับ option |
| TC-E99 | FOK limit option | วาง FOK limit order บน option instrument | ไม่มี FOK สำหรับ option |
| TC-E100 | Cancel option order | cancel limit order ที่เปิดอยู่บน option instrument | ไม่รองรับ option |
| TC-E101 | Reconcile option position | reconcile option position ที่เปิดอยู่จาก session ก่อนหน้า | ไม่รองรับ option |

### TC-E90: Limit BUY option [#tc-e90-limit-buy-option]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด option instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วาง limit buy บน option ที่ราคา passive |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | venue accept order ด้วย instrument, side, price และ quantity ที่ถูกต้อง |
| **ข้ามเมื่อ** | adapter ไม่รองรับการเทรด option |

### TC-E91: Limit SELL option [#tc-e91-limit-sell-option]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด option instrument แล้ว, quote ไหลเข้า |
| **การกระทำ** | ExecTester วาง limit sell บน option ที่ราคา passive |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | venue accept order ด้วย instrument, side, price และ quantity ที่ถูกต้อง |
| **ข้ามเมื่อ** | adapter ไม่รองรับการเทรด option |

### TC-E92: Limit แบบตั้งราคาทางเลือก [#tc-e92-limit-with-alternative-pricing]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว และโหลด option instrument แล้ว |
| **การกระทำ** | วาง limit order ที่ตั้งราคาแบบเฉพาะของ adapter ผ่าน `order_params` |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` |
| **เกณฑ์ผ่าน** | order ถูก accept และ venue รับรู้โหมดการตั้งราคาทางเลือก |
| **ข้ามเมื่อ** | adapter ไม่รองรับโหมดการตั้งราคาทางเลือกสำหรับ option |

**ข้อควรพิจารณา**:

- ฟิลด์ `price` ของ order อาจเป็นแค่ค่า placeholder เมื่อใช้การตั้งราคาทางเลือก ดู key ของพารามิเตอร์ที่รองรับได้จากคู่มือ
  ของ adapter
- ตัวอย่าง: OKX รองรับ `px_usd` (ราคาเป็น USD) และ `px_vol` (implied volatility)
- ตรวจใน response ของ venue ว่าโหมดการตั้งราคาสะท้อนออกมาถูกต้อง

### TC-E94: Order type ที่ไม่รองรับถูก deny สำหรับ option [#tc-e94-unsupported-order-type-denied-for-options]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว และโหลด option instrument แล้ว |
| **การกระทำ** | submit order type ที่ venue ไม่รองรับสำหรับ option (เช่น market order) |
| **ลำดับ event** | ขึ้นกับ adapter: `OrderDenied` (ก่อน submit) หรือ `OrderSubmitted` -> `OrderRejected` (หลัง submit) |
| **เกณฑ์ผ่าน** | order ไม่ถูก fill และเหตุผลของการ deny หรือ reject อ้างถึง order type ที่ไม่รองรับ |
| **ข้ามเมื่อ** | adapter ไม่รองรับ option |

**ข้อควรพิจารณา**:

- จุดที่ปฏิเสธแตกต่างกันตาม adapter บาง adapter deny ในเครื่องก่อน submit บางตัว submit แล้วส่งต่อ rejection จาก venue
- ExecTester trigger market order ได้ผ่าน `open_position_on_start_qty` บน option instrument บาง type ที่ไม่รองรับ
  (เช่น `MarketToLimit`) ต้อง submit ด้วยมือหรือผ่านโค้ด
- ทดสอบทุก type ที่ไม่รองรับตามที่ adapter ระบุไว้

### TC-E96: Conditional order ถูก reject สำหรับ option [#tc-e96-conditional-order-rejected-for-options]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว และโหลด option instrument แล้ว |
| **การกระทำ** | submit conditional order บน option instrument |
| **ลำดับ event** | ขึ้นกับ adapter: `OrderDenied` (ก่อน submit) หรือ `OrderSubmitted` -> `OrderRejected` (หลัง submit) |
| **เกณฑ์ผ่าน** | order ไม่ถูก fill และเหตุผลอ้างถึง conditional order type ที่ไม่รองรับ |
| **ข้ามเมื่อ** | adapter ไม่รองรับ option หรือ adapter รองรับ conditional สำหรับ option |

**ข้อควรพิจารณา**:

- ทดสอบ conditional type แต่ละตัวที่ adapter ระบุว่าไม่รองรับสำหรับ option (เช่น `STOP_MARKET`, `STOP_LIMIT`,
  `MARKET_IF_TOUCHED`, `LIMIT_IF_TOUCHED`, `TRAILING_STOP_MARKET`)
- ExecTester trigger conditional order ได้ผ่าน `enable_stop_buys`/`enable_stop_sells` ร่วมกับ `stop_order_type` บน
  option instrument

### TC-E99: FOK limit option [#tc-e99-fok-limit-option]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | adapter เชื่อมต่อแล้ว, โหลด option instrument แล้ว, book มีความลึกเพียงพอ |
| **การกระทำ** | วาง limit order ด้วย `TimeInForce::Fok` บน option instrument |
| **ลำดับ event** | `OrderInitialized` -> `OrderSubmitted` -> `OrderAccepted` -> `OrderFilled` หรือ `OrderCanceled` |
| **เกณฑ์ผ่าน** | order ถูก fill ทั้งหมดหรือถูก cancel ไม่มี partial fill |
| **ข้ามเมื่อ** | adapter ไม่รองรับ FOK สำหรับ option |

**ข้อควรพิจารณา**:

- บาง venue ใช้ order type เฉพาะสำหรับ FOK order ของ option (เช่น OKX ใช้ `op_fok`) adapter จัดการการ map นี้ให้โดยไม่ต้อง
  ทำอะไรเพิ่ม
- ใช้ quantity เล็กและราคา aggressive เพื่อให้ได้ fill ในกรณีบวก

### TC-E100: Cancel option order [#tc-e100-cancel-option-order]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี limit order เปิดอยู่จาก TC-E90 หรือ TC-E91 |
| **การกระทำ** | cancel limit order ที่เปิดอยู่ |
| **ลำดับ event** | `OrderPendingCancel` -> `OrderCanceled` |
| **เกณฑ์ผ่าน** | order ถูก cancel และไม่ปรากฏใน open order บน venue อีก |
| **ข้ามเมื่อ** | adapter ไม่รองรับ option |

### TC-E101: Reconcile option position [#tc-e101-reconcile-option-position]

| ฟิลด์ | ค่า |
| --- | --- |
| **เงื่อนไขก่อนทดสอบ** | มี option position เปิดอยู่จาก session ก่อนหน้า |
| **การกระทำ** | เริ่ม node โดย `reconciliation=True` |
| **ลำดับ event** | สร้าง `PositionStatusReport` สำหรับ option position |
| **เกณฑ์ผ่าน** | position ถูกโหลดเข้า cache ด้วย instrument, side, quantity และ entry price ที่ถูกต้อง |
| **ข้ามเมื่อ** | adapter ไม่รองรับ option |

**ข้อควรพิจารณา**:

- เปิด option position ใน session ก่อนหน้า แล้วหยุดโดยไม่ปิด (`close_positions_on_stop=False`)
- ตรวจว่า position ที่ reconcile ตรงกับ state ที่ venue รายงาน

---

## Reference ของ config ExecTester [#exectester-configuration-reference]

reference แบบย่อของแต่ละพารามิเตอร์ของ `ExecTesterConfig` ค่า default คือค่าที่ resolve แล้วหลังการสร้าง builder ของ Rust
ใช้ default ที่เทียบเท่ากัน

| พารามิเตอร์ | Default | กลุ่มที่เกี่ยวข้อง |
| --- | --- | --- |
| `strategy_id` | `None` | ทั้งหมด |
| `order_id_tag` | `None` | ทั้งหมด |
| `use_hyphens_in_client_order_ids` | `True` | ทั้งหมด |
| `use_uuid_client_order_ids` | `False` | ทั้งหมด |
| `external_order_instrument_ids` | `None` | 9 |
| `instrument_id` | `BTCUSDT-PERP.BINANCE` | ทั้งหมด |
| `client_id` | `None` | ทั้งหมด |
| `order_qty` | `0.001` | ทั้งหมด |
| `order_display_qty` | `None` | 2, 7 |
| `order_expire_time_delta_mins` | `None` | 2 |
| `order_params` | `None` | 7, 10 |
| `subscribe_book` | `False` |  |
| `subscribe_quotes` | `True` |  |
| `subscribe_trades` | `True` |  |
| `book_type` | `L2_MBP` |  |
| `book_depth` | `None` |  |
| `book_interval_ms` | `1000` |  |
| `book_levels_to_print` | `10` |  |
| `open_position_on_start_qty` | `None` | 1, 9 |
| `open_position_on_first_quote` | `False` | 1 |
| `open_position_time_in_force` | `GTC` | 1 |
| `enable_limit_buys` | `True` | 2, 4, 5, 6 |
| `enable_limit_sells` | `True` | 2, 4, 5, 6 |
| `enable_stop_buys` | `False` | 3, 4 |
| `enable_stop_sells` | `False` | 3, 4 |
| `tob_offset_ticks` | `500` | 2, 4 |
| `limit_time_in_force` | `None` | 2, 6 |
| `stop_order_type` | `STOP_MARKET` | 3 |
| `stop_offset_ticks` | `100` | 3 |
| `stop_limit_offset_ticks` | `None` | 3 |
| `stop_trigger_type` | `DEFAULT` | 3 |
| `stop_time_in_force` | `None` | 3 |
| `trailing_offset` | `None` | 3 |
| `trailing_offset_type` | `BASIS_POINTS` | 3 |
| `enable_brackets` | `False` | 6 |
| `batch_submit_limit_pair` | `False` | 2, 5 |
| `bracket_entry_order_type` | `LIMIT` | 6 |
| `bracket_offset_ticks` | `500` | 6 |
| `modify_orders_to_maintain_tob_offset` | `False` | 4 |
| `modify_stop_orders_to_maintain_offset` | `False` | 4 |
| `cancel_replace_orders_to_maintain_tob_offset` | `False` | 4 |
| `cancel_replace_stop_orders_to_maintain_offset` | `False` | 4 |
| `trigger_limit_order_maintenance_once` | `False` | 4 |
| `use_post_only` | `False` | 2, 6, 7, 8 |
| `limit_aggressive` | `False` | 2 |
| `use_quote_quantity` | `False` | 1, 7 |
| `emulation_trigger` | `None` | 2, 3 |
| `use_individual_cancels_on_stop` | `False` | 5 |
| `cancel_orders_on_stop` | `True` | 5, 9 |
| `close_positions_on_stop` | `True` | 9 |
| `close_positions_qty_precision` | `None` | 9 |
| `close_positions_time_in_force` | `None` | 9 |
| `reduce_only_on_stop` | `True` | 7, 9 |
| `use_batch_cancel_on_stop` | `False` | 5 |
| `dry_run` | `False` |  |
| `log_data` | `True` |  |
| `test_reject_post_only` | `False` | 8 |
| `test_reject_reduce_only` | `False` | 8 |
| `test_modify_rejected` | `False` | 4 |
| `can_unsubscribe` | `True` | 9 |
| `clamp_to_instrument_price_range` | `False` | 1-8, 10 |
| `log_events` | `True` | ทั้งหมด |
| `log_commands` | `True` | ทั้งหมด |
