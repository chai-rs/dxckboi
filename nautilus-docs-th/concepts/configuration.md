# Configuration

NautilusTrader ใช้ config object แบบมี type สำหรับ data client, execution client, engine และ strategy
config ระดับสูงจะประกอบขึ้นจาก config ของ component เหล่านี้ ตัวอย่างเช่น `LiveNodeConfig` เป็นเจ้าของการตั้งค่า
component หลักของ node ส่วน adapter client ให้ลงทะเบียนผ่าน `LiveNode.builder(...)` adapter จะแยก config
ของ data client และ execution client ออกจากกันเมื่อความสามารถหรือ credential ของทั้งสองแตกต่างกัน

## หลักการออกแบบ {#design-principles}

### ฟิลด์แบบ concrete เก็บค่าที่ resolve แล้ว {#concrete-fields-carry-resolved-values}

ฟิลด์ config ใน Rust โดยปกติจะเก็บค่าแบบ concrete เมื่อ component ต้องการค่าที่ resolve แล้ว
ตัวอย่างเช่น timeout ของ adapter, จำนวนครั้งที่ retry, ระยะเวลา backoff และ heartbeat interval มักใช้
type พื้นฐานอย่าง `u64` หรือ `u32` ขั้นตอนการสร้าง (construction) จะ resolve ฟิลด์เหล่านี้ก่อน component เริ่มทำงาน
ทำให้โค้ดที่อยู่ปลายทางใช้ค่าได้ทันทีโดยไม่ต้องเขียน logic การกำหนดค่าเริ่มต้นซ้ำ

### ความหมายของ Option ขึ้นกับแต่ละฟิลด์ {#option-semantics-are-field-specific}

ใน config ของ Rust ที่เก็บไว้แล้ว `Option<T>` จะมีค่าเป็น `Some(value)` หรือ `None` อย่างใดอย่างหนึ่ง ค่าที่เก็บไว้
ไม่ได้บันทึกว่าผู้เรียกละเว้น input นั้นหรือไม่ component อาจตีความ `None` ว่าเป็นการปิดฟีเจอร์,
ปล่อยให้ช่วงย้อนหลัง (lookback window) ไม่มีขอบเขต, ถอยกลับไปใช้ค่าจาก runtime environment หรือใช้
ค่าเริ่มต้นภายใน **เอกสารของแต่ละฟิลด์เป็นตัวกำหนดความหมาย**

ความแตกต่างนี้ทำให้ความหมายของ config ปรากฏอยู่ใน type เลย `u64` ธรรมดาจะมีค่าเสมอ ในขณะที่
โค้ดที่ใช้ `Option<u64>` ต้องจัดการกรณีที่ไม่มีค่าเอง

### ค่าเริ่มต้นขึ้นกับแต่ละ type {#defaults-are-type-specific}

type ของ config ใน Rust กำหนดค่าเริ่มต้นผ่าน annotation `#[builder(default = value)]`, implementation
ของ `Default` ที่เขียนเอง หรือทั้งสองอย่าง

`#[serde(default)]` ระดับ container บน config struct จะเติมฟิลด์ที่ขาดหายไปตอน deserialize จาก
implementation `Default` ของ config นั้น ส่วน `#[serde(default)]` ระดับฟิลด์จะใช้ค่าเริ่มต้นของ type ของฟิลด์แทน
เว้นแต่ attribute จะระบุฟังก์ชันอื่น

`Type::default()` และ `Type::builder().build()` เป็นเส้นทางการสร้างที่แยกจากกัน implementation `Default`
ที่เขียนเองอาจมอบหมายการสร้างบางส่วนให้ builder ก็ได้ แต่ขึ้นกับแต่ละ type อย่าทึกทักเอาว่าสองเส้นทางนี้
ใช้แทนกันได้ เว้นแต่ implementation หรือเอกสารจะรับประกันไว้

### การจัดการฟิลด์ที่ไม่รู้จักขึ้นกับเส้นทางการสร้าง {#unknown-field-handling-depends-on-the-construction-path}

แต่ละเส้นทางการสร้างบังคับใช้กฎเรื่องฟิลด์ที่ไม่รู้จักแยกกัน:

- **Rust**: ตัวอย่างเช่น `BybitDataClientConfig` ใช้ `#[serde(deny_unknown_fields)]` และปฏิเสธ key ส่วนเกิน
  ตอน deserialize ส่วน type ใน Rust ที่ไม่มี attribute นี้อาจยอมรับ key เหล่านั้น

อย่าอนุมานความเข้มงวดของเส้นทางการสร้างหนึ่งจากอีกเส้นทางหนึ่ง

## Config ใน Rust {#rust-configs}

config struct ใน Rust จำนวนมาก derive [`bon::Builder`](https://bon-rs.com) ซึ่งสร้าง builder แบบ type-safe
ที่ตรวจสอบฟิลด์ที่จำเป็นได้ตั้งแต่ตอน compile builder สามารถละเว้นฟิลด์ที่ประกาศค่าเริ่มต้นของ builder ไว้ได้

ให้ใช้รูปแบบการสร้างตามที่เอกสารของ config type นั้นระบุ สำหรับ `DataEngineConfig` ทั้งแบบ builder และแบบ
struct update ด้านล่างจะเปิด delta buffering และคงค่าเริ่มต้นที่ประกาศไว้สำหรับฟิลด์อื่น:

```rust
use nautilus_data::engine::config::DataEngineConfig;

let with_builder = DataEngineConfig::builder()
    .buffer_deltas(true)
    .build();

let with_struct_update = DataEngineConfig {
    buffer_deltas: true,
    ..Default::default()
};
```

ใช้ `DataEngineConfig::default()` เมื่อไม่ต้องการ override ฟิลด์ใดเลย

## ฟิลด์ config ของ adapter {#adapter-config-fields}

ชื่อฟิลด์มักซ้ำกันในหลาย adapter config แต่ type และค่าเริ่มต้นขึ้นกับ adapter และ client
ตัวอย่างเช่น `BybitDataClientConfig` กำหนดฟิลด์ดังด้านล่าง คอลัมน์ `Default` แสดงค่าจาก
`BybitDataClientConfig::default()`:

| ฟิลด์ Rust                       | Type ใน Rust   | Default    | วัตถุประสงค์                         |
| ------------------------------- | ------------- | ---------- | --------------------------------- |
| `http_timeout_secs`             | `u64`         | `60`       | timeout ของ REST request          |
| `max_retries`                   | `u32`         | `3`        | จำนวนครั้งสูงสุดที่ retry              |
| `retry_delay_initial_ms`        | `u64`         | `1_000`    | ระยะเวลา backoff เริ่มต้น            |
| `retry_delay_max_ms`            | `u64`         | `10_000`   | ระยะเวลา backoff สูงสุด             |
| `heartbeat_interval_secs`       | `u64`         | `20`       | interval ของ WebSocket keepalive  |
| `recv_window_ms`                | `u64`         | `5_000`    | ช่วงเวลาหมดอายุของ signed request   |
| `instrument_poll_interval_secs` | `Option<u64>` | `Some(60)` | การ poll instrument และสถานะ       |

ในทางกลับกัน `BybitDataClientConfig::builder().build()` จะปล่อยให้
`instrument_poll_interval_secs` เป็น `None` ซึ่งจะปิดการ poll instrument และสถานะเป็นระยะ
นี่คือกรณีหนึ่งที่ **เส้นทาง default และ builder ของ type ให้ผลต่างกัน** ผู้เรียกฝั่ง Rust สามารถตั้ง
`instrument_poll_interval_secs` เป็น `None` เพื่อปิดการ poll instrument และสถานะเป็นระยะได้

ฟิลด์เฉพาะของแต่ละ adapter เช่น rate limit, polling interval และ margin mode มีอธิบายไว้ใน
[คู่มือ integration](../integrations/index.md)

## Config ของ engine {#engine-configs}

config ของ engine ใช้แนวทางฟิลด์แบบมี type เช่นเดียวกัน ใน `LiveExecutionEngineConfig` ฟิลด์อย่าง
`reconciliation`, `inflight_check_interval_ms` และ `open_check_threshold_ms` มีค่าเริ่มต้นแบบ concrete:

| ฟิลด์                         | Default | วัตถุประสงค์                                                 |
| ---------------------------- | ------- | ---------------------------------------------------------- |
| `reconciliation`             | `True`  | รัน reconciliation ระหว่าง startup                          |
| `inflight_check_interval_ms` | `2_000` | ตรวจสอบว่า order ที่อยู่ระหว่างดำเนินการ (in-flight) เกิน threshold หรือไม่ |
| `open_check_threshold_ms`    | `5_000` | รอก่อนดำเนินการเมื่อพบความไม่ตรงกันของ open order              |

ฟิลด์แบบ optional อย่าง `open_check_interval_secs` และ `position_check_interval_secs` ใช้เปิดหรือ
ปิดการตรวจสอบเป็นระยะของแต่ละอย่าง โดย `position_check_interval_secs` ปิดอยู่เป็นค่าเริ่มต้น

ตัวอย่างเช่น config ที่ตั้ง `open_check_interval_secs` เป็น 30 และ `open_check_lookback_mins` เป็น 60
จะทำให้หลัง startup เมื่อมี execution client พร้อมใช้งาน ระบบจะตั้งเวลาขอ report ของ open order ทุก 30
วินาที แต่ละ request ครอบคลุมเฉพาะช่วง 60 นาทีก่อนหน้า และจะไม่ตั้งเวลาขอ report ของ position เป็นระยะ
interval ที่กำหนดสำหรับการตรวจสอบเป็นระยะต้องเป็นค่าบวกที่จำกัด (finite) และมีค่าอย่างน้อยหนึ่งนาโนวินาที

ฟิลด์ `reconciliation` ที่แยกต่างหากใช้เปิด reconciliation ตอน startup และมีค่าเริ่มต้นเป็น `True` ฟิลด์ interval
ควบคุมการตรวจสอบเป็นระยะอย่างอิสระ เมื่อเปิด reconciliation ตอน startup ไว้
`reconciliation_startup_delay_secs` จะหน่วงการตรวจสอบเป็นระยะครั้งแรกหลัง startup ด้วย ค่าหน่วงนี้ต้อง
เป็นค่าจำกัดและอยู่ในช่วง [0, 86400] วินาที

ดูตัวเลือกทั้งหมดของ live engine ได้ที่
[การตั้งค่า ExecutionEngine](../how_to/configure_live_trading.md#executionengine-configuration)
