# เริ่มต้นใช้งาน Lighter

Lighter ใช้งานได้ผ่าน engine ของ Rust และสามารถใช้จากโปรเจกต์ Rust ล้วนได้

เส้นทางที่สั้นที่สุดคือเริ่มจาก public data เมื่อการ subscribe ข้อมูลทำงานได้แล้ว จึงเพิ่ม credential สำหรับการ
execution แล้วเพิ่ม strategy ที่ส่ง order ได้

## เลือกเส้นทางการตั้งค่า {#choose-a-setup-path}

| เส้นทาง        | ใช้เมื่อ                                                  | ขั้นแรก                                      |
| :---------- | :-------------------------------------------------- | :----------------------------------------- |
| Pure Rust   | ต้องการแอปที่ compile แล้วและไม่ต้องมี Python runtime        | คัดลอก Rust quickstart                      |
| ตัวอย่าง RWA | ต้องการใช้ข้อมูล signal จาก Databento ร่วมกับการเทรดบน Lighter | อ่าน tutorial composite market making       |

เริ่มจากไฟล์เหล่านี้:

- Rust quickstart: `examples/quickstarts/lighter-rust-data-client/`
- tutorial RWA: [tutorial composite market making][lighter-rwa-composite-mm]

เส้นทาง Rust ใช้ชิ้นส่วนเหล่านี้:

- `LighterDataClientConfig` เลือก deployment ของ Lighter หรือ Robinhood, mainnet หรือ testnet, custom venue
  (ไม่บังคับ) และการตั้งค่า transport (ไม่บังคับ)
- `LighterExecutionClientConfig` เพิ่ม account ID และ resolve credential โดย issuer ของ account ต้องตรงกับ venue
  ที่ resolve ได้
- `LighterDataClientFactory` และ `LighterExecutionClientFactory` ลงทะเบียน client กับ `LiveNode`
- `DataTester` และ `ExecTester` เป็น actor สำหรับ smoke test ก่อนที่จะเขียน strategy ของตัวเอง

## เริ่มต้นด้วย Pure Rust {#pure-rust-starter}

คัดลอก quickstart ไปยัง workspace ของคุณ:

```bash
cp -R examples/quickstarts/lighter-rust-data-client ~/lighter-rust-data-client
cd ~/lighter-rust-data-client
cargo run
```

คำสั่งนี้สร้าง `LiveNode`, ลงทะเบียน data client ของ Lighter, เพิ่ม `DataTester` และเชื่อมต่อกับ public stream
ของ testnet หยุดได้ด้วย Ctrl+C

การตั้งค่าหลักใช้ builder ซึ่งเติมค่าเริ่มต้นของตัวเลือกที่ไม่บังคับให้:

```rust
let data_config = LighterDataClientConfig::builder()
    .environment(LighterEnvironment::Testnet)
    .build();

let mut node = LiveNode::builder(trader_id, Environment::Live)?
    .with_name("LIGHTER-DATA-STARTER-001".to_string())
    .add_data_client(
        None,
        Box::new(LighterDataClientFactory::new()),
        Box::new(data_config),
    )?
    .build()?;
```

เมื่อเส้นทางข้อมูลทำงานได้แล้ว ให้เพิ่ม execution client ใน builder ก่อนเรียก `.build()`:

```rust
let exec_config = LighterExecutionClientConfig::builder()
    .account_id(account_id)
    .environment(LighterEnvironment::Testnet)
    .build();

let mut node = LiveNode::builder(trader_id, Environment::Live)?
    .with_name("LIGHTER-EXEC-STARTER-001".to_string())
    .add_data_client(
        None,
        Box::new(LighterDataClientFactory::new()),
        Box::new(data_config),
    )?
    .add_exec_client(
        None,
        Box::new(LighterExecutionClientFactory::new()),
        Box::new(exec_config),
    )?
    .build()?;
```

สำหรับการ execution ให้ทำตาม[การตั้งค่า account และ API key](../integrations/lighter.md#account-and-api-key-setup)
แล้วตั้ง environment variable ที่ตรงกันก่อนเชื่อมต่อ:

```bash
export LIGHTER_TESTNET_ACCOUNT_INDEX="123456"
export LIGHTER_TESTNET_API_KEY_INDEX="4"
export LIGHTER_TESTNET_API_SECRET="your-lighter-api-secret"
```

deployment และ environment เป็นตัวเลือก namespace ของ credential:

| Deployment | Environment | Prefix ของ credential           |
| ---------- | ----------- | ----------------------------- |
| Lighter    | Mainnet     | `LIGHTER_*`                   |
| Lighter    | Testnet     | `LIGHTER_TESTNET_*`           |
| Robinhood  | Mainnet     | `LIGHTER_ROBINHOOD_*`         |
| Robinhood  | Testnet     | `LIGHTER_ROBINHOOD_TESTNET_*` |

แต่ละ namespace ประกอบด้วย `ACCOUNT_INDEX`, `API_KEY_INDEX` และ `API_SECRET`

## ก้าวต่อไปสู่ strategy {#move-to-a-strategy}

เส้นทางเริ่มต้นเหล่านี้พิสูจน์การเชื่อมต่อ client, การ subscribe และการค้นหา credential ขั้นถัดไปคือการแทนที่ tester
ด้วย strategy:

- ใช้ [Write a Strategy (Rust)](write_rust_strategy.md) สำหรับ strategy แบบ Rust ล้วน
- ใช้ [Composite market making บน Lighter RWA][lighter-rwa-composite-mm] เมื่อต้องการตั้งค่า signal จาก Databento
  แบบเต็มรูปแบบ

:::warning
ตัวอย่าง execution ฝั่ง Rust มาพร้อม `DRY_RUN = false` และอาจส่ง order จริงได้ทันทีที่รัน ตั้ง `DRY_RUN` เป็น `true`
เพื่อเชื่อมต่อโดยไม่ส่ง order เริ่มบน testnet หรือใช้ขนาดที่เล็กที่สุดที่ยอมรับ และตรวจสอบ instrument, deployment,
environment, account index, API key index และ private key ให้แน่ใจก่อนรัน
:::

สำหรับการเก็บกวาดฉุกเฉิน `cargo run --bin lighter-flatten -p nautilus-lighter` จะยกเลิก open order และปิด position
ของ account ใน deployment ที่เลือก ให้หยุดตัวเขียนอื่น ๆ ของ account และทบทวน
[Emergency account cleanup](../integrations/lighter.md#emergency-account-cleanup) ก่อนใช้งาน เครื่องมือนี้ส่ง
การยกเลิกแบบทั้ง account หนึ่งครั้ง และรอบการปิด position หนึ่งรอบ โดยไม่ reconcile ต่อจนกว่า account จะ flat

[lighter-rwa-composite-mm]: ../tutorials/lighter_rwa_composite_mm.md
