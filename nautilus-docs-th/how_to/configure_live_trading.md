---
title: "ตั้งค่า Live Trading Node"
---

ตั้งค่า `LiveNode` สำหรับเชื่อมต่อกับตลาดจริง สำหรับ lifecycle ของ node ดู
[Live trading](../concepts/live.md) สำหรับผลลัพธ์ของ command ดู
[Execution policies](../concepts/execution/policies.md#command-outcomes) สำหรับการกู้คืนสถานะ ดู
[Execution reconciliation](../concepts/execution/reconciliation.md)

:::warning[หนึ่ง LiveNode ต่อหนึ่ง process]
ไม่รองรับการรัน `LiveNode` หลาย instance พร้อมกันใน process เดียวกัน เพราะ
สถานะของ runtime ไม่ได้แยกออกจากกัน ให้เพิ่มหลาย strategy ไว้ใน node เดียว หรือรัน node เพิ่มเติมใน process แยก

ดูรายละเอียดที่ [Processes and threads](../concepts/architecture.md#processes-and-threads)
:::

:::warning[อย่า block event loop]
โค้ดของผู้ใช้บน thread ของ event loop (strategy callback, actor handler และ time event callback)
ต้อง return อย่างรวดเร็ว operation ที่ block เช่น model
inference, การคำนวณหนัก หรือ synchronous I/O จะทำให้พลาด fill ได้ข้อมูลที่ล้าสมัย และ
การ submit order ล่าช้า ให้ย้ายงานที่ใช้เวลานานไปทำใน executor หรือ thread/process แยก
:::

:::info[ความแตกต่างของแพลตฟอร์ม]
การจัดการ signal บน Windows แตกต่างจากระบบแบบ Unix หากรันบน Windows โปรดอ่าน
หมายเหตุเรื่อง [การจัดการ signal บน Windows](#windows-signal-handling) เพื่อดูแนวทางเกี่ยวกับพฤติกรรม
graceful shutdown และการรองรับ Ctrl+C (SIGINT)
:::

## LiveNodeConfig [#livenodeconfig]

`LiveNodeConfig` เป็นเจ้าของการตั้งค่า component หลักของ node ให้ลงทะเบียน data และ execution client ผ่าน
builder ของ `LiveNode` ไม่ใช่ผ่าน dictionary ของ client บน config นี้ สำหรับพื้นฐานเรื่องค่า default ของ config
และความหมายของ `Option<T>` ดู
คู่มือแนวคิด [Configuration](../concepts/configuration.md)

### พารามิเตอร์หลักของการตั้งค่า [#core-configuration-parameters]

| Setting                       | Default      | คำอธิบาย                                                                      |
| ----------------------------- | ------------ | -------------------------------------------------------------------------------- |
| `trader_id`                   | "TRADER-001" | identifier ของ trader ที่ไม่ซ้ำ (รูปแบบ name-tag) โดย tag ต้องไม่ซ้ำกันข้าม node |
| `instance_id`                 | `None`       | identifier ของ instance ที่ไม่ซ้ำ (optional)                                       |
| `timeout_connection_secs`     | 60.0         | timeout ของการเชื่อมต่อ หน่วยวินาที                                                |
| `timeout_reconciliation_secs` | 30.0         | timeout ของ reconciliation หน่วยวินาที                                            |
| `timeout_portfolio_secs`      | 10.0         | timeout ของการ initialize portfolio                                              |
| `timeout_disconnection_secs`  | 10.0         | timeout ของการตัดการเชื่อมต่อ                                                     |
| `delay_post_stop_secs`        | 10.0         | ระยะเวลารอ event ที่ค้างอยู่หลังหยุด                                                |
| `timeout_shutdown_secs`       | 5.0          | timeout ของการ shutdown task ที่ pending หน่วยวินาที                               |

:::warning[Tag ของ Trader ID ต้องไม่ซ้ำ]
ส่วน tag หลังขีดกลางตัวสุดท้ายคือส่วนที่ไปปรากฏใน client order ID, order list ID และ
position ID ที่ถูกสร้างขึ้น ส่วนชื่อด้านหน้าไม่ได้ไปปรากฏ ดังนั้น node สองตัวที่เทรดบัญชี venue เดียวกันจึง
ต้องใช้ tag ต่างกัน เพราะ `MY-TRADER-001` และ `OTHER-TRADER-001` ใช้ tag `001` ร่วมกันและอาจ
สร้าง ID ที่เหมือนกันได้ การตั้ง `use_uuid_client_order_ids` บน strategy จะลด
ความเสี่ยงนี้ได้เฉพาะ client order ID เท่านั้น เพราะ order list ID และ position ID ยังคงใช้ tag อยู่ดี
จึงยังต้องใช้ tag ที่ไม่ซ้ำกัน
:::

### การตั้งค่า cache database [#cache-database-configuration]

ระบบ live แบบ Rust-native เก็บพฤติกรรมของ cache ไว้ใน `CacheConfig` และการตั้งค่าการเชื่อมต่อ Redis ไว้ใน
`RedisCacheConfig`

```rust
use nautilus_common::{
    cache::{CacheConfig, database::CacheDatabaseFactory},
    enums::SerializationEncoding,
};
use nautilus_infrastructure::redis::cache::RedisCacheConfig;

let config = CacheConfig {
    encoding: SerializationEncoding::MsgPack,
    timestamps_as_iso8601: true,
    buffer_interval_ms: Some(100),
    flush_on_start: false,
    ..Default::default()
};

let database = RedisCacheConfig {
    host: Some("localhost".to_string()),
    port: Some(6379),
    username: Some("nautilus".to_string()),
    password: Some("pass".to_string()),
    connection_timeout: 2,
    response_timeout: 2,
    ..Default::default()
};

let cache_database = database
    .create(trader_id, instance_id, config.clone())
    .await?;
```

ให้แนบ adapter หลังจาก build node แบบ Rust-native แล้วและก่อนเริ่ม node node จะกู้คืน
database ก่อน reconciliation เมื่อเปิด `exec_engine.load_cache` ไว้ ซึ่งเป็นค่า default

```rust
let node_config = LiveNodeConfig {
    trader_id,
    ..Default::default()
};
let mut node = LiveNode::build("LiveNode".to_string(), Some(node_config))?;
node.set_cache_database(cache_database)?;
node.run().await?;
```

ตั้ง `CacheConfig.flush_on_start = true` เพื่อล้าง backing ที่แนบไว้แทนการกู้คืน
backing แบบ Postgres จะล้างเฉพาะแถวของ trader ของ node นั้น

backing แบบ Postgres จะโหลด เขียน และ flush เฉพาะ trader ของ node ดังนั้น node ที่มี
trader ID ต่างกันจึงใช้ database เดียวกันได้ ดู [Cache](../concepts/cache.md) ว่า flush แบบจำกัดขอบเขตเก็บอะไรไว้
และวิธีกำหนด account event ที่ถูก persist ไว้ก่อนมีการแบ่งขอบเขตตาม trader

เมื่อตั้ง `snapshot_orders=True` execution engine จะ persist order snapshot ระหว่างการประมวลผล submission
และหลังการเปลี่ยนสถานะทุกครั้ง order snapshot ต้องใช้ cache backing แบบ Redis หรือ Postgres

เมื่อตั้ง `snapshot_positions=True` execution engine จะ publish snapshot เมื่อ position เปิด
เปลี่ยนแปลง หรือปิด ตั้ง `snapshot_positions_interval_secs` แยกต่างหากเพื่อเพิ่ม snapshot แบบเป็นระยะ
ของทุก position ที่เปิดอยู่ cache backing แบบ Redis และ Postgres จะ persist ทั้งสองเส้นทาง หากไม่มี backing
snapshot จะยังใช้ได้บน message bus ภายใน process แต่จะไม่ถูก persist

การ persist state ของ actor และ strategy (การโหลดและบันทึก state) ต้องใช้
Redis backing ส่วน Postgres adapter รองรับเฉพาะ state ของ cache: เมื่อมี actor หรือ strategy ที่ลงทะเบียนไว้
การเปิดโหลด state จะล้มเหลวตอน trader เริ่ม ส่วนการเปิดบันทึก state จะล้มเหลวตอน
node หยุดหรือถูก dispose ตอน startup kernel จะส่ง state ที่ persist ไว้ซึ่งไม่ว่างไปยัง `on_load` และเมื่อ
หยุดหรือ dispose node จะ persist สิ่งที่ `on_save` return

:::warning
การ persist state ไม่ใช่ checkpoint แบบต่อเนื่อง kernel บันทึก state อย่างมากหนึ่งครั้งต่อ run ดังนั้น
`SIGKILL` หรือ crash จะทำให้สูญเสียการเปลี่ยนแปลงทั้งหมดตั้งแต่การบันทึกครั้งล่าสุด ให้ dispose node เพื่อให้ `dispose()` ปิด
backing และ flush การเขียนที่ buffer ไว้ การ return ออกจาก `run()` ทันทีอาจทำให้การบันทึกครั้งสุดท้ายหายไป
:::

### การตั้งค่า MessageBus [#messagebus-configuration]

พฤติกรรมของ message bus อยู่ใน `MessageBusConfig` ส่วนการตั้งค่าการเชื่อมต่อ Redis อยู่ใน
`RedisMessageBusConfig` ซึ่ง implement `MessageBusBackingFactory` และสร้าง backing
จากการตั้งค่าเหล่านั้น

```rust
use nautilus_common::{
    enums::SerializationEncoding,
    msgbus::{MessageBusBackingFactory, MessageBusConfig},
};
use nautilus_infrastructure::redis::msgbus::RedisMessageBusConfig;

let config = MessageBusConfig {
    encoding: SerializationEncoding::Json,
    timestamps_as_iso8601: true,
    use_instance_id: false,
    types_filter: Some(vec!["QuoteTick".to_string(), "TradeTick".to_string()]),
    stream_per_topic: false,
    autotrim_mins: Some(30),
    heartbeat_interval_secs: Some(1),
    ..Default::default()
};

let redis_config = RedisMessageBusConfig {
    connection_timeout: 2,
    response_timeout: 2,
    ..Default::default()
};

let backing = redis_config.create(trader_id, instance_id, config.clone())?;
```

`MessageBusConfig` เพียงอย่างเดียวไม่ได้ติดตั้ง backing ต้องจับคู่กับ factory ดังที่แสดงข้างต้น
factory จะติดตั้ง external egress เสมอ และการเรียก `run()` จะ consume external
stream ที่ตั้งค่าไว้ด้วย entry ที่อยู่ใน stream ก่อน node เริ่มจะไม่ถูก replay ดู [การตั้งค่า
message bus backing](../concepts/message_bus.md#backing-config) สำหรับรายละเอียดของ lifecycle และ ingress
producer ภายนอกที่เขียนลง Redis โดยตรงต้องใส่ฟิลด์ `type` ที่จำเป็น ดู
[external egress และ ingress](../concepts/message_bus.md#external-egress-and-ingress) สำหรับฟิลด์
ของ wire format

## การตั้งค่าหลาย venue [#multi-venue-configuration]

node หนึ่งตัวเชื่อมต่อกับ client หลายตัวได้ เช่น ลงทะเบียน data client ของ Binance spot และ USD-M futures
ก่อน build node

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## การตั้งค่า ExecutionEngine [#executionengine-configuration]

`LiveExecutionEngineConfig` ควบคุมการประมวลผล order, execution event และ
venue reconciliation ดูรายละเอียดทั้งหมดได้ที่
[Rust API docs](https://nautilustrader.io/docs/core-latest/)

### Reconciliation [#reconciliation]

กู้คืน order และ position event ที่พลาดไป เพื่อให้สถานะของระบบสอดคล้องกับ venue

| Setting                         | Default | คำอธิบาย                                                                   |
| ------------------------------- | ------- | ----------------------------------------------------------------------------- |
| `reconciliation`                | True    | เปิด reconciliation ตอน startup เพื่อให้สถานะภายในตรงกับ venue                  |
| `reconciliation_lookback_mins`  | None    | ย้อนหลังกี่นาทีในการ request event ในอดีตเพื่อ reconcile สถานะที่ไม่ได้อยู่ใน cache  |
| `reconciliation_instrument_ids` | None    | รายการ instrument ID ที่จะ reconcile                                           |
| `filtered_client_order_ids`     | None    | client order ID ที่จะข้ามระหว่าง reconciliation (สำหรับรายการซ้ำฝั่ง venue)        |

ดูรายละเอียดที่ [Execution reconciliation](../concepts/execution/reconciliation.md)

### การกรอง order [#order-filtering]

ควบคุมว่าระบบจะประมวลผล order event และ report ใดบ้าง เพื่อป้องกันความขัดแย้ง
ระหว่าง trading node

| Setting                            | Default | คำอธิบาย                                                                 |
| ---------------------------------- | ------- | --------------------------------------------------------------------------- |
| `filter_unclaimed_external_orders` | False   | ทิ้ง external order ที่ไม่มีผู้ claim เพื่อไม่ให้กระทบ strategy                 |
| `filter_position_reports`          | False   | ทิ้ง position status report มีประโยชน์เมื่อหลาย node เทรดบัญชีเดียวกัน           |

:::note[พฤติกรรมการติด tag ให้ order]
reconciliation ติด tag ให้ order ตามที่มา:

- **tag `VENUE`**: external order ที่พบที่ venue (ส่งจากนอกระบบนี้)
- **tag `RECONCILIATION`**: order สังเคราะห์ที่สร้างขึ้นเพื่อปรับความคลาดเคลื่อนของ position

เมื่อเปิด `filter_unclaimed_external_orders` จะกรองเฉพาะ order ที่มี tag `VENUE` เท่านั้น
order ที่มี tag `RECONCILIATION` จะไม่ถูกกรองเลย การปรับ position จึงสำเร็จเสมอ
:::

### Continuous reconciliation [#continuous-reconciliation]

continuous reconciliation ทำให้สถานะ execution ระหว่าง runtime สอดคล้องกันหลัง startup โดยตรวจสอบ
order ที่อยู่ระหว่างทาง (in-flight), poll open order, ตรวจสถานะ position และ audit own order book
ตั้งค่า loop ด้วย setting เหล่านี้ สำหรับกฎการเปลี่ยนสถานะระหว่าง runtime, การประสานการ retry
และข้อควรระวัง ดู [Runtime checks](../concepts/execution/reconciliation.md#runtime-checks)

| Setting                              | Default        | คำอธิบาย                                                                                                                                                                                    |
| ------------------------------------ | -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `inflight_check_interval_ms`         | 2,000&nbsp;ms  | ความถี่ในการตรวจสถานะของ in-flight order ตั้งเป็น 0 เพื่อปิด                                                                                                                                |
| `inflight_check_threshold_ms`        | 5,000&nbsp;ms  | เวลาก่อนที่ in-flight order จะกระตุ้นการตรวจสถานะที่ venue ลดลงได้หากเป็น colocated                                                                                                              |
| `inflight_check_retries`             | 5&nbsp;retries | จำนวนครั้งที่ retry เพื่อยืนยัน in-flight order กับ venue                                                                                                                                    |
| `open_check_interval_secs`           | None           | ความถี่ (วินาที) ในการตรวจ open order ที่ venue None หรือ 0.0 คือปิด แนะนำ: 5-10s                                                                                               |
| `open_check_open_only`               | True           | เมื่อเป็น true จะ query เฉพาะ open order เมื่อเป็น false จะดึงประวัติทั้งหมด (ใช้ทรัพยากรมาก)                                                                                                        |
| `open_check_lookback_mins`           | 60&nbsp;min    | ช่วงย้อนหลัง (นาที) สำหรับการ poll สถานะ order เฉพาะ order ที่ถูกแก้ไขภายในช่วงนี้                                                                                                   |
| `open_check_threshold_ms`            | 5,000&nbsp;ms  | เวลาขั้นต่ำนับจาก event ล่าสุดใน cache ก่อนจะดำเนินการกับความคลาดเคลื่อนที่ venue                                                                                                                     |
| `open_check_missing_retries`         | 5&nbsp;retries | จำนวน retry สูงสุดก่อนทำ not-found resolution แบบเจาะจงสำหรับ order ที่เข้าเกณฑ์                                                                                                                          |
| `max_single_order_queries_per_cycle` | 10             | เพดานจำนวน query แบบ order เดียวต่อรอบ ป้องกันการใช้ rate limit จนหมด                                                                                                                         |
| `single_order_query_delay_ms`        | 100&nbsp;ms    | ระยะหน่วง (ms) ระหว่าง query แบบ order เดียว เพื่อหลีกเลี่ยง rate limit                                                                                                                                  |
| `reconciliation_startup_delay_secs`  | 10.0&nbsp;s    | ระยะหน่วง (วินาที) *หลัง* startup reconciliation ก่อนที่การตรวจแบบต่อเนื่องจะเริ่ม                                                                                                                 |
| `own_books_audit_interval_secs`      | None           | ระยะห่าง (วินาที) ระหว่างการ audit ที่ลบ order ใน own book ซึ่ง cache ไม่ได้ถือว่า active แล้ว None คือปิด                                                                              |
| `position_check_interval_secs`       | None           | ระยะห่าง (วินาที) ระหว่างการตรวจความสอดคล้องของ position เมื่อพบความคลาดเคลื่อนจะ query หา fill ที่ขาดหาย None คือปิด แนะนำ: 30-60s                                                         |
| `position_check_lookback_mins`       | 60&nbsp;min    | ช่วงย้อนหลัง (นาที) สำหรับ query fill report เมื่อ position คลาดเคลื่อน                                                                                                                   |
| `position_check_threshold_ms`        | 5,000&nbsp;ms  | เวลาขั้นต่ำนับจากกิจกรรมภายในล่าสุดก่อนจะดำเนินการกับความคลาดเคลื่อนของ position และระยะเวลาที่ fill ซึ่งนำไปใช้ไม่สำเร็จจะเลื่อน synthetic reconciliation ออกไป                                       |
| `position_check_retries`             | 3&nbsp;retries | จำนวนครั้งสูงสุดต่อ instrument/account ก่อนที่ engine จะหยุด retry ความคลาดเคลื่อนนั้น เมื่อเกินแล้วจะ log error และจะไม่ reconcile ความคลาดเคลื่อนนั้นอย่างแข็งขันอีกจนกว่ามันจะหายไป |

:::warning

- **`open_check_lookback_mins`**: อย่าลดต่ำกว่า 60 นาที ช่วงเวลาที่สั้น
  จะกระตุ้นการ resolve ว่า "order หายไป" แบบผิด ๆ เพราะ order อยู่นอกช่วงของ query
- **`open_check_threshold_ms`**: เพิ่มค่านี้หาก timestamp ของ venue ช้ากว่า clock ภายในเครื่อง เพื่อไม่ให้
  order ที่เพิ่งอัปเดตถูก mark ว่าหายไปก่อนเวลาอันควร
- **`reconciliation_startup_delay_secs`**: อย่าลดต่ำกว่า 10 วินาทีใน production
  ระยะหน่วงนี้ให้ระบบเสถียรหลัง startup reconciliation ก่อนที่การตรวจแบบต่อเนื่อง
  จะเริ่ม

:::

### ตัวเลือกเพิ่มเติม [#additional-options]

| Setting                            | Default | คำอธิบาย                                                                                                        |
| ---------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------ |
| `allow_overfills`                  | False   | อนุญาต fill ที่เกินจำนวนของ order (log warning) มีประโยชน์เมื่อ reconciliation แข่งกับ fill                       |
| `generate_missing_orders`          | True    | สร้าง reconciliation order จาก fill และ position report ที่ไม่ match                                           |
| `snapshot_orders`                  | False   | persist order snapshot ระหว่างการประมวลผล submission และหลังการเปลี่ยนสถานะทุกครั้ง เมื่อตั้งค่า cache backing ไว้ |
| `snapshot_positions`               | False   | publish position snapshot เมื่อเปิด เปลี่ยนแปลง และปิด และ persist เมื่อตั้งค่า cache backing ไว้          |
| `snapshot_positions_interval_secs` | None    | ระยะห่าง (วินาที) ระหว่าง position snapshot                                                                     |
| `debug`                            | False   | เปิด debug logging สำหรับ execution                                                                                |

### การจัดการหน่วยความจำ [#memory-management]

purge order ที่ปิดแล้ว, position ที่ปิดแล้ว และ account event ออกจาก
cache ในหน่วยความจำเป็นระยะ เพื่อให้หน่วยความจำมีขอบเขตระหว่าง session ที่รันนานหรือ HFT

| Setting                                | Default | คำอธิบาย                                                                        |
| -------------------------------------- | ------- | ---------------------------------------------------------------------------------- |
| `purge_closed_orders_interval_mins`    | None    | ความถี่ (นาที) ในการ purge order ที่ปิดแล้วออกจากหน่วยความจำ แนะนำ: 10-15 min    |
| `purge_closed_orders_buffer_mins`      | None    | order ต้องปิดไปนานเท่าใด (นาที) ก่อนจะ purge แนะนำ: 60 min    |
| `purge_closed_positions_interval_mins` | None    | ความถี่ (นาที) ในการ purge position ที่ปิดแล้วออกจากหน่วยความจำ แนะนำ: 10-15 min |
| `purge_closed_positions_buffer_mins`   | None    | position ต้องปิดไปนานเท่าใด (นาที) ก่อนจะ purge แนะนำ: 60 min  |
| `purge_account_events_interval_mins`   | None    | ความถี่ (นาที) ในการ purge account event ออกจากหน่วยความจำ แนะนำ: 10-15 min   |
| `purge_account_events_lookback_mins`   | None    | account event ต้องมีอายุเท่าใด (นาที) ก่อนจะ purge แนะนำ: 60 min    |

การตั้ง interval จะเปิด purge loop หากไม่ตั้งจะปิดทั้งการ schedule และการลบ
แต่ละ loop จะเรียกใช้ cache API ที่อธิบายไว้ใน
[Cache](../concepts/cache.md)

## การตั้งค่า strategy [#strategy-configuration]

สำหรับรายการพารามิเตอร์ทั้งหมดของ `StrategyConfig` ดู
[Rust API docs](https://nautilustrader.io/docs/core-latest/)

### การระบุตัวตน [#identification]

| Setting        | Default | คำอธิบาย                                                                |
| -------------- | ------- | -------------------------------------------------------------------------- |
| `strategy_id`  | None    | identifier ของ strategy ที่ไม่ซ้ำ                                                |
| `order_id_tag` | None    | tag ที่ไม่ซ้ำซึ่งต่อท้าย order ID ของ strategy นี้ ห้ามมีขีดกลาง |

### การจัดการ order [#order-management]

| Setting                         | Default | คำอธิบาย                                                                                        |
| ------------------------------- | ------- | -------------------------------------------------------------------------------------------------- |
| `oms_type`                      | None    | [OMS type](../concepts/execution/index.md#oms-configuration) สำหรับ position ID และการประมวลผล order |
| `use_uuid_client_order_ids`     | False   | ใช้ค่า UUID4 สำหรับ client order ID                                                             |
| `external_order_instrument_ids` | None    | intent ที่ serialize ได้สำหรับ claim external order และกิจกรรม reconciliation ตาม instrument            |
| `manage_contingent_orders`      | False   | จัดการความสัมพันธ์ OTO, OCO และ OUO ที่เปิดอยู่และไม่ใช่ active-local                                     |
| `manage_gtd_expiry`             | False   | จัดการการหมดอายุแบบ GTD ของ order                                                                 |

ดู [การ claim external order](../concepts/strategies.md#claiming-external-orders) สำหรับ lifecycle ของ
การ claim ที่ active และการอัปเดตระหว่าง runtime

`OrderEmulator` จะเก็บ contingent order แบบ active-local ไว้ ดู
[Advanced orders](../concepts/orders/advanced.md#strategy-managed-contingencies) สำหรับขอบเขตด้านความเป็นเจ้าของและ
การรองรับของ venue

## การจัดการ signal บน Windows [#windows-signal-handling]

`LiveNode` จัดการ Ctrl+C (SIGINT) และบน Unix จะจัดการ SIGTERM ด้วยใน run loop ของ Rust
เพื่อให้ runner และ task ต่าง ๆ shutdown ได้อย่างเรียบร้อย
