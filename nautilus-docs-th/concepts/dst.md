---
title: "DST"
---

<!-- Keep this title as "DST"; longer titles do not render well in the left navigation. -->

Deterministic simulation testing (DST) คือการรัน NautilusTrader ภายใต้ runtime ที่ควบคุมด้วย seed คู่มือนี้กำหนด:

- การรับประกันว่าจะทำซ้ำผลได้ (reproducibility) สำหรับการ execute ที่ควบคุมด้วย seed
- เงื่อนไขที่จำเป็นเพื่อให้การรับประกันนั้นเป็นจริง
- จุดเชื่อม (seam) ใน source และการตรวจสอบที่บังคับใช้ข้อกำหนดนี้
- เส้นทางที่การ execute แบบ deterministic สิ้นสุดลง

แต่ละข้อกล่าวอ้างมีตำแหน่งใน source กำกับไว้ เพื่อให้ผู้ใช้และผู้ตรวจสอบเทียบข้อกำหนดกับโค้ดได้

:::note
Harness ปลายทางที่พึ่งพาความ deterministic ของ NautilusTrader จะใช้เอกสารฉบับนี้ในเวอร์ชันที่ตรงกับ commit ของ
NautilusTrader ที่ pin ไว้ การเปลี่ยนแปลงเอกสารนี้จึงถือเป็นการเปลี่ยนข้อกำหนด (contract) สำหรับผู้ใช้เหล่านั้น และควร
review ในฐานะนั้น
:::

## DST คืออะไร [#what-dst-is]

DST ควบคุมแหล่งที่มาของความไม่ deterministic ในระบบ concurrent ภายในข้อกำหนดที่นิยามไว้ด้านล่าง seed เดียวเป็นตัวกำหนด
การจัดลำดับ task, การ fire ของ timer และค่าสุ่ม การรันสองครั้งที่ใช้ seed, binary, configuration และ platform เดียวกัน
จะให้พฤติกรรมที่สังเกตได้เหมือนกันทุกประการ บันทึก seed ที่ล้มเหลวไว้เพื่อ replay การ execute เดิมซ้ำได้

Async runtime ทั่วไปตัดสินใจเรื่องการจัดลำดับจากสถานะแวดล้อมของ process ได้แก่:

- ลำดับการ wake ของ task
- ความละเอียดของ timer
- การจัดลำดับ thread ของ OS
- Hash seed แบบสุ่ม

Test harness ทั่วไปไม่ได้ควบคุมแหล่งเหล่านี้ race ที่เกิดขึ้นครั้งเดียวใน CI จึงอาจทำซ้ำได้ยาก DST แทนที่สิ่งเหล่านี้ด้วย
ลำดับ pseudorandom ที่มี seed seed ต่างกันจะสำรวจ interleaving ที่ต่างกัน และ seed เดียวกันจะเลือก interleaving เดียวกัน

[FoundationDB](https://apple.github.io/foundationdb/testing.html) ใช้ pattern นี้ทดสอบ distributed database ที่ใช้งาน
จริงใน production ส่วนใน ecosystem ของ Rust [madsim](https://crates.io/crates/madsim) จะดักจับ primitive ของ `tokio`
เพื่อให้มี scheduler แบบ deterministic

DST มุ่งเป้าไปที่ข้อบกพร่องด้าน concurrency เช่น:

- ลำดับการ wakeup ของ channel
- Race ระหว่างการ drain ตอน shutdown
- ลำดับขั้นตอนตอน startup
- ลำดับของ reconciliation
- ความถูกต้องของเส้นทางการกู้คืน (recovery)

Test ชั้นอื่นไม่สามารถครอบคลุม interleaving เหล่านี้ได้ทั้งหมด scheduler แบบ deterministic สามารถสำรวจได้ข้าม seed ต่าง ๆ
และ replay schedule ที่ล้มเหลวได้

## เป้าหมาย [#goals]

| เป้าหมาย                        | ข้อกำหนด                                                                                                     |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------- |
| การ execute ที่ทำซ้ำได้ด้วย seed     | Runtime ที่อยู่ในขอบเขตให้พฤติกรรมที่สังเกตได้เหมือนเดิมเมื่อใช้ seed และ input เดียวกัน                                   |
| ขอบเขตที่ชัดเจน                   | ทุกการ fallback ไปใช้เวลาจริง, ค่าสุ่มที่ไม่มี seed หรือความไม่ deterministic อื่น ๆ ต้องมีเอกสารกำกับ                     |
| การบังคับใช้ใน source              | Static check ปฏิเสธ pattern ที่ห้ามใช้บนเส้นทาง DST ก่อนที่จะต้องพึ่งความใส่ใจของ reviewer                             |
| ดักจับเท่าที่จำเป็นขั้นต่ำ             | เวลา, การจัดลำดับ task และค่าสุ่มจะผ่านแหล่งแบบ deterministic เฉพาะในจุดที่ข้อกำหนดต้องการเท่านั้น                     |

## แนวทาง [#approach]

Implementation มีสองชั้น ชั้นแรกแทนที่ primitive บางตัวของ `tokio` ด้วย `madsim` ชั้นที่สองควบคุมแหล่งความไม่
deterministic ที่อยู่นอก primitive เหล่านั้น

### ชั้นที่ 1: สลับ runtime [#layer-1-runtime-swap]

ภายใต้ Cargo feature `simulation` ของ `nautilus-common` มี facade submodule สี่ตัวที่จะเลือกพฤติกรรมแบบ simulation
เมื่อตั้ง `RUSTFLAGS="--cfg madsim"`:

| Submodule | ครอบคลุม                                          | Build ปกติ     | Build แบบ simulation                                              | สถานะการนำไปใช้                                     |
| --------- | ----------------------------------------------- | ------------- | ---------------------------------------------------------------- | ------------------------------------------------- |
| `time`    | Timer, interval และ `Instant` แบบ monotonic      | `tokio`       | `madsim`                                                         | เสร็จสมบูรณ์                                        |
| `task`    | การ spawn และ join async task                    | `tokio`       | `madsim`                                                         | เสร็จสมบูรณ์                                        |
| `runtime` | Runtime builder และ handle                      | `tokio`       | `madsim`                                                         | เสร็จสมบูรณ์                                        |
| `signal`  | Process signal เช่น `ctrl_c` และ `SIGTERM`       | `tokio` หรือ OS | `madsim` สำหรับ `ctrl_c` และ future ที่ไม่มีวันเสร็จสำหรับ `terminate` | บางส่วน ดู [การจัดการ signal](#signal-handling)       |

Re-export เหล่านี้อยู่ใน `nautilus_common::live::dst` call site บนเส้นทาง DST สำหรับ `time`, `task` และ `runtime` จะ
import จาก module นี้ build ปกติจะ resolve import เป็น `tokio` ส่วน `simulation` ร่วมกับ `cfg(madsim)` จะ resolve เป็น `madsim`

Submodule `sync`, `io`, `fs` และ `net` รวมถึง macro `select!` ยังคงใช้ `tokio` จริง network crate มี
[transport boundary](#simulated-http-and-websocket-transport) แยกต่างหากสำหรับการเชื่อมต่อ HTTP แบบ plaintext และ
Tungstenite WebSocket โดยไม่ได้แทนที่ Tokio ภายใน dependency

### ชั้นที่ 2: แทนที่แหล่งความไม่ deterministic [#layer-2-nondeterminism-substitution]

ความไม่ deterministic ที่อยู่นอก runtime ที่ทำ alias ไว้ต้องมี seam ที่ชัดเจน

#### เวลาแบบ wall-clock [#wall-clock-time]

การอ่าน wall-clock จะผ่าน `nautilus_core::time::duration_since_unix_epoch` ภายใต้ simulation seam นี้จะเรียก
`madsim::time::TimeHandle::try_current()` เพื่อคง semantics แบบ Unix epoch สำหรับ timestamp ของ order และ fill

Body ของ `#[rstest]` ธรรมดาจะรันนอก madsim runtime ในบริบทนั้น seam จะ fallback ไปใช้ `SystemTime::now()` ซึ่งใช้ syscall
จริงตัวเดียวกับ build ปกติ ส่วนเส้นทาง production ภายใต้ simulation จะรันใน madsim runtime และได้รับเวลาเสมือน

#### เวลาแบบ monotonic [#monotonic-time]

การอ่านเวลาแบบ monotonic จะผ่าน `nautilus_common::live::dst::time::Instant` build ปกติจะ resolve type นี้เป็น
`tokio::time::Instant` เพื่อให้ใช้ร่วมกับ `#[tokio::test(start_paused = true)]` ได้ ส่วน build แบบ simulation จะ resolve
เป็น `madsim::time::Instant`

#### เวลาแบบ monotonic ภายใน network [#network-local-monotonic-time]

โค้ดด้าน network อ่านเวลาแบบ monotonic ผ่าน `nautilus_network::dst::time` เนื่องจาก `nautilus-network` อยู่ต่ำกว่า
`nautilus-common` ใน dependency graph จึงมี module re-export ภายในของตัวเองที่ทำงานแบบเดียวกัน

#### ลำดับการ iterate [#iteration-order]

ลำดับการ iterate ที่สังเกตได้ใช้ `IndexMap`, `IndexSet` หรือการ sort อย่างชัดเจน แทนการพึ่ง `AHashMap` หรือ `AHashSet`
เพราะ `AHash` สุ่ม hasher ต่อ process ลำดับที่คงที่เป็นสิ่งจำเป็นเมื่อการ iterate ควบคุมการ publish event หรือลำดับการสุ่ม
จาก `FillModel` ที่มี seed

#### ลำดับการ poll ของ select [#select-polling-order]

`tokio::select!` ทุกจุดใน production บนเส้นทาง DST ขึ้นต้นด้วย `biased;` หากไม่มี RNG ที่ไม่ได้ถูกดักจับจะเป็นตัวเลือก
ลำดับการ poll ของ branch

## ข้อกำหนดด้าน determinism [#determinism-contract]

ภายใต้เงื่อนไขด้านล่าง run ที่ระบุด้วย `(seed, binary hash, configuration hash)` บน platform เดียวกันจะให้ผลที่เหมือนกัน
ระดับ bit ใน:

- ลำดับการจัด schedule ของ async task
- การ fire ของ timer (monotonic เสมือนและ wall-clock เสมือน)
- ผลลัพธ์ RNG จาก `madsim::rand`
- ลำดับการส่งบน channel ของ `tokio::sync`

### เงื่อนไขที่จำเป็น [#required-conditions]

ข้อกำหนดนี้เป็นจริง **เฉพาะเมื่อทุกแถวด้านล่างได้รับการปฏิบัติ**:

| แหล่งความไม่ deterministic | เงื่อนไขที่จำเป็น                                                            | ผลเมื่อข้ามไป                                                                                                                                             |
| ------------------------ | ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| การเลือก build            | เปิด feature `simulation` และตั้ง `RUSTFLAGS="--cfg madsim"`             | ตั้งค่าเพียงอย่างใดอย่างหนึ่งจะ fallback ไปใช้ `tokio` จริงโดยไม่มี error cfg นี้ยังเปิดการดักจับ libc ของ madsim สำหรับ `clock_gettime` และ `getrandom` |
| `tokio::select!`         | ใส่ `biased;` ไว้แรกสุดในทุก block ของ production บนเส้นทาง DST              | RNG ที่ไม่ได้ถูกดักจับจะเลือกลำดับการ poll                                                                                                                  |
| เวลาแบบ monotonic         | ใช้ `nautilus_common::live::dst::time` หรือ `nautilus_network::dst::time` | การเรียก `std::time::Instant::now` โดยตรงจะอ่านนาฬิกาของ host                                                                                              |
| เวลาแบบ wall-clock        | ใช้ `nautilus_core::time::duration_since_unix_epoch`                     | การอ่านนาฬิกาโดยตรงจะข้ามเวลาเสมือน                                                                                                                       |
| ค่าสุ่ม                     | ใช้ `madsim::rand`                                                       | `rand::thread_rng`, `rand::rng()`, `fastrand`, `getrandom` และ `OsRng` ไม่ถูกดักจับ                                                                      |
| ลำดับการ iterate           | ใช้ `IndexMap`, `IndexSet` หรือ sort ณ จุดที่ใช้งาน                          | การ iterate hash แบบสุ่มจะเปลี่ยนลำดับที่สังเกตได้                                                                                                          |
| Local task               | Gate `tokio::task::LocalSet` ออกภายใต้ simulation                          | `madsim` ไม่มี `LocalSet` ให้ใช้ `spawn_local` โดยไม่ต้องมีมัน                                                                                               |
| Blocking task            | Gate ออกหรือลบ `tokio::task::spawn_blocking`                               | Blocking call จะหลุดออกจาก scheduler แบบ deterministic                                                                                                   |

### ข้อกำหนด DST สำหรับ adapter [#adapter-dst-contract]

Adapter จะถือว่าเป็นไปตาม DST ก็ต่อเมื่อทุกข้อกำหนดด้านล่างเป็นจริงสำหรับทุก **เส้นทางที่อ้างว่ารองรับ (claimed path)**
[คู่มือนักพัฒนา adapter](../developer_guide/adapters.md#deterministic-simulation) ระบุ implementation อ้างอิงและ
ข้อกำหนดสำหรับ adapter ที่ได้รับการดูแล ส่วนนี้ระบุตัวข้อกำหนดเอง

| ข้อกำหนด              | กฎ                                                                                                                                                                                                                                                                     |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ขอบเขตของ build       | เปิดเผยและส่งต่อ feature `simulation` เส้นทางที่อ้างว่ารองรับต้อง build และรันได้ภายใต้ `cfg(madsim)` โดยไม่มีพฤติกรรม production ที่มีไว้สำหรับ test เท่านั้น                                                                                                                |
| Static gate          | ไฟล์ production ทุกไฟล์ของเส้นทางที่อ้างว่ารองรับต้องอยู่ภายใต้ `check-dst-conventions` หรือมีเหตุผลการยกเว้นที่บันทึกไว้                                                                                                                                                       |
| Input แบบ deterministic | Wall time, เวลาแบบ monotonic, timer, ค่าสุ่ม และ identifier ที่ generate ขึ้น ต้องมาจาก facade `nautilus_common::live::dst`, seam wall-clock ของ `nautilus_core::time` หรือ input ที่ประกาศไว้และ inject เข้ามา input ด้าน auth ต้องมาจากแหล่งแบบ deterministic         |
| การควบคุม async        | Task, timer และการควบคุม runtime ผ่าน DST facade, มี `biased;` ในทุก `select!` ของ production และการ connect, reconnect, retry, unsubscribe และ shutdown ต้อง deterministic                                                                                               |
| Identity บน wire      | ใช้ collection ที่มีลำดับหรือ sort อย่างชัดเจนในทุกขอบเขตที่ส่งข้อมูลออกทาง wire และ domain การ retry และ reconnect ต้องคง protocol identifier ไว้ และสร้าง request ที่เหมือนกันทุก byte จาก input ที่ประกาศไว้เดียวกัน ผลลัพธ์ที่ไม่แน่นอนใช้ taxonomy ของ command-failure ร่วมกัน |
| Transport ที่ควบคุมได้   | ทุก endpoint ของเส้นทางที่อ้างว่ารองรับต้องชี้ไปยัง peer แบบ local ที่ควบคุมได้ผ่าน configuration ของ production ที่รองรับ ไม่มี host ของ venue, credential หรือสถานะของ venue จริงแบบ hardcoded อนุญาต credential สังเคราะห์แบบคงที่บนเส้นทาง private                    |
| พิสูจน์ด้วย process ใหม่  | Seed, configuration และ peer script เดียวกันต้องให้ wire transcript และ domain output ที่เหมือนกันข้าม process ใหม่ และต้องปฏิเสธ mutation ที่ปรับเทียบไว้อย่างน้อยหนึ่งตัว ฝั่ง upstream pin subscribe byte และ wire field ไว้ ส่วนการเปรียบเทียบแบบเต็มอยู่ฝั่ง downstream |

ความสามารถที่ adapter ไม่ได้อ้างว่ารองรับจะอยู่นอกข้อกำหนดจนกว่าจะมี slice ที่พิสูจน์ได้ slice ต่าง ๆ จะขยายชุดที่พิสูจน์แล้ว
และ gate จะป้องกันการถดถอย

## การบังคับใช้แบบ static [#static-enforcement]

การบังคับใช้แบบ static มีสองชั้น:

| ชั้น                      | บังคับใช้                                                                                                                  |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| Clippy policy           | `clippy.toml` และ `[workspace.lints.clippy]` ปฏิเสธการเรียก `getrandom::{fill,u32,u64}` โดยตรงและ `tokio::task::LocalSet` |
| `check-dst-conventions` | pre-commit hook ที่ใช้การตรวจสอบเชิงโครงสร้างแบบรู้ path และรู้ cfg ซึ่ง Clippy แสดงออกได้ไม่สะอาด                              |

Hook อยู่ที่ `.pre-commit-hooks/check_dst_conventions.sh` และรันในชุด pre-commit มาตรฐานและใน CI ขอบเขตต่างกันไปตามกฎ:

- **กฎข้อ 1 ถึง 4 และ 6**: workspace crate 17 ตัวที่อยู่ในขอบเขต และไฟล์ adapter ที่ตรวจสอบแล้วในรายการ `ADAPTER_PATHS`
  ของ hook
- **กฎข้อ 5**: ไฟล์ที่ตรวจสอบแล้วสองไฟล์ของกฎนี้
- **กฎข้อ 7**: crate เก้าตัวบนเส้นทาง build ของ madsim, ไฟล์ adapter เหล่านั้น และ `crates/network/src/websocket/client.rs`

| กฎ | ปฏิเสธ                                                                                                                                          | ขอบเขตหรือข้อยกเว้น                                                                                            |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| 1    | การอ่าน `std::time::Instant::now()`, `SystemTime::now()`, `jiff::Timestamp::now()` และ `jiff::Zoned::now()` แบบดิบ รวมถึงรูปแบบที่ import มาแบบเปล่า | อนุญาต seam wall-clock, การจับเวลาสำหรับ log หรือ progress ที่ตรวจสอบแล้ว และบรรทัดที่มี marker                       |
| 2    | การใช้ `rand::thread_rng`, `rand::rng()`, `fastrand::`, `getrandom::`, `OsRng` และ `Uuid::new_v4()` แบบดิบ                                      | อนุญาตบรรทัดที่ gate ด้วย cfg และบรรทัดที่มี marker                                                                |
| 3    | Block `tokio::select!` ใน production ที่ไม่มี `biased;` ในสามบรรทัดแรก                                                                           | ไม่รวม test และบรรทัดที่มี marker                                                                                |
| 4    | `std::thread::spawn`, `std::thread::Builder::new` และ `tokio::task::spawn_blocking`                                                            | อนุญาตจุดที่ใช้เฉพาะ test, ที่ไม่ใช่ madsim และที่มี marker                                                          |
| 5    | `AHashMap` หรือ `AHashSet` ใน reconciliation manager และ matching engine                                                                       | ครอบคลุมสองไฟล์ที่ตรวจสอบแล้ว ไฟล์ที่เหลือยังอยู่นอกกฎ static นี้จนกว่าจะตรวจสอบ                                         |
| 6    | การเรียก `tokio::net::TcpStream::connect` และ `tokio::net::TcpListener::bind` โดยตรง                                                            | ผู้เรียกต้องใช้ `nautilus_network::net` ซึ่งจะสลับไปใช้ `turmoil::net` ภายใต้ feature `turmoil`                       |
| 7    | Path `tokio::{time,task,runtime,signal}` แบบดิบในโค้ด production บนเส้นทาง build ของ madsim                                                     | อนุญาต facade, Tokio runtime จริงระดับทั้ง process, โครงสร้างพื้นฐานของ test, จุดที่ gate ด้วย cfg และบรรทัดที่มี marker |

เส้นทาง HTTP และ WebSocket ของ Madsim ที่รองรับใช้ `nautilus_network::dst::net` ซึ่งจะเลือก byte stream แบบจำลอง
และ re-export `nautilus_network::net` ใน build ปกติ

Hook รองรับข้อยกเว้นสองรูปแบบ:

- Marker `// dst-ok` แบบ inline บนบรรทัดที่ระบุ ซึ่งมักมาพร้อมเหตุผลสั้น ๆ (เช่น การจับเวลา wall-clock ที่ใช้แค่ log และไม่มีผล
  ต่อสถานะ)
- Allowlist ระดับไฟล์ขนาดเล็กใน script ของ hook เอง สำหรับจุดที่การตรวจสอบ codebase จัดว่าให้คงไว้ตามเดิม (การจับเวลา log ใน
  module cache, การใส่ timestamp ให้ log record ใน logging bridge และ writer, การรายงาน progress ใน module DeFi)

Hook จะไม่รวม:

- ไฟล์ใต้ directory `tests/`, `python/` และ `ffi/`
- ไฟล์ชื่อ `tests.rs`, `*_test.rs` หรือ `*_tests.rs`
- บรรทัดภายใน module `#[cfg(test)]` แบบ inline

Path เหล่านี้ไม่ใช่ส่วนหนึ่งของข้อกำหนด DST สำหรับ production

### Crate ที่อยู่ในขอบเขต [#in-scope-crates]

Transitive closure ของ `nautilus-live` มี crate ที่อยู่ในขอบเขต 16 ตัว:

- `analysis`
- `common`
- `core`
- `cryptography`
- `data`
- `execution`
- `indicators`
- `live`
- `model`
- `network`
- `persistence`
- `portfolio`
- `risk`
- `serialization`
- `system`
- `trading`

Hook ยังครอบคลุม `backtest` ด้วย รวมเป็น 17 crate

Adapter crate และ infrastructure crate (Redis, Postgres) อยู่นอกขอบเขต เว้นแต่จะมี audited slice บันทึกไว้ในคู่มือ integration
ของ adapter นั้น audited slice จะระบุไฟล์บนเส้นทาง DST ที่ static gate ครอบคลุมและเหตุผลการยกเว้นของไฟล์ที่เหลือ รวมถึงระบุ
runtime slice ที่ test `dst` ของ adapter พิสูจน์ได้

:::warning
การครอบคลุมแบบ static เพียงอย่างเดียวไม่ได้ยืนยันว่ามีสิทธิ์ใช้งานที่ runtime: ไฟล์ที่ครอบคลุมอาจให้บริการเส้นทางที่อยู่นอก
runtime slice ที่พิสูจน์แล้วด้วย
:::

## Transport HTTP และ WebSocket แบบจำลอง [#simulated-http-and-websocket-transport]

เมื่อใช้ `simulation` และ `cfg(madsim)` `nautilus-network` จะ execute HTTP/1.1 request แบบ plaintext และการเชื่อมต่อ
Tungstenite WebSocket ผ่าน byte stream ของ Madsim

- **HTTP**: เส้นทาง production และแบบจำลองใช้ค่าเริ่มต้นของ request, การ encode URL และ query และการตรวจสอบ response
  ร่วมกัน การตรวจสอบ response ที่ใช้ร่วมกันรวมถึงขีดจำกัดของ body ที่ buffer ไว้และ deadline ที่ครอบคลุมการอ่าน body
  branch ของ simulation ใน `crates/network/src/http/simulation.rs` ใช้ Hyper สำหรับการแลกเปลี่ยน HTTP/1.1 และเป็นเจ้าของ
  connection task ของ Madsim เอง
- **WebSocket**: traffic ใช้ codec ของ Tungstenite โดย `crates/network/src/dst.rs` จะส่ง task และ transport stream ที่ตัวเอง
  เป็นเจ้าของไปยัง Madsim

Build ปกติใช้การเชื่อมต่อ HTTP ของ Hyper แบบ pooled และ Tokio task

กำหนด endpoint `http://` และ `ws://` ที่ควบคุมได้ และเลือก `TransportBackend::Tungstenite` simulation จะปฏิเสธ HTTPS/WSS,
proxy ที่ระบุอย่างชัดเจน และ Sockudo ก่อนเปิดการเชื่อมต่อ เพราะ timer ภายในของ Sockudo ต้องใช้ Tokio runtime ค่าตั้ง HTTP
proxy จากสภาพแวดล้อมไม่มีผลต่อการแลกเปลี่ยนแบบจำลอง

พฤติกรรมการ redirect ขึ้นกับ `HttpRedirectPolicy`:

- `Follow` (ค่าเริ่มต้น): response แบบ redirect ที่มี `Location` จะคืน error แทนการตาม redirect
- `Reject`: คืน response แบบ redirect ให้ผู้เรียก

Model นี้ครอบคลุม byte ของ HTTP request, payload ของ WebSocket message และ domain event แต่ไม่รับประกันว่า handshake key
หรือ frame mask ของ WebSocket, TLS record, การจัดลำดับ TCP ของ OS, partial write, socket option หรือ TCP half-close
จะทำซ้ำได้

ใน `crates/network/src/dst/stream.rs` การ shutdown stream จะ flush byte และการ drop stream จะปิดการเชื่อมต่อ การใช้
`simulation`, `cfg(madsim)` และ `turmoil` ร่วมกันจะเป็น compile error ให้รัน simulator ทั้งสองแยก build กัน

Simulation จะเรียง request header ตามชื่อเพื่อให้ byte ของ HTTP ทำซ้ำได้ ลำดับมาตรฐานนี้ไม่ได้รับประกันว่าจะตรงทุก byte กับ
ลำดับ header ของ request จริง HTTP request แต่ละตัวจะเปิดการเชื่อมต่อใหม่ ไม่ได้จำลอง connection pooling และการนำ
keep-alive กลับมาใช้

Network test ครอบคลุมพฤติกรรมของ transport แต่ไม่ได้ยืนยันความสอดคล้องแบบ end-to-end ของ adapter, ผลิตภัณฑ์ หรือ channel

## Network seed soak [#network-seed-soaks]

[Turmoil](https://crates.io/crates/turmoil) จำลอง network ภายใต้ scheduler ที่มี seed transport test ของ
`nautilus-network` ใช้มันเพื่อเข้าถึงลำดับของ link และ reconnect ที่อยู่นอกการสลับ runtime ของ madsim

### Nightly test แบบ seed คงที่ [#fixed-seed-nightly-tests]

ชุด nightly รันสถานการณ์ที่ทำซ้ำได้สำหรับ:

- การเชื่อมต่อครั้งแรก
- การเชื่อมต่อใหม่ (reconnection)
- Network partition
- การปิดระหว่าง reconnection
- การปิดระหว่าง backoff
- การที่ server หลุดซ้ำ ๆ พร้อม assertion ลำดับ message แบบตรงตัว

### Reconnect seed soak [#reconnect-seed-soak]

Reconnect test ที่ถูก ignore ไว้จะไล่ seed ของ Turmoil ไปเรื่อย ๆ จนกว่าจะถูกหยุดหรือถึงขีดจำกัดที่ตั้งไว้ ในแต่ละ seed soak
จะรัน backend Tungstenite WebSocket ก่อน แล้วจึงรัน Sockudo เมื่อเปิด feature `transport-sockudo` ทำให้ backend ทั้งสอง
ใช้เส้นทางการค้นหา schedule เดียวกัน

| ตัวแปร                                    | ค่าเริ่มต้น   | ผล                                                    |
| ----------------------------------------- | --------- | --------------------------------------------------- |
| `NAUTILUS_TURMOIL_SOAK_START`             | `0`       | เลือก seed แรก ทำให้กลับมาไล่ต่อได้                      |
| `NAUTILUS_TURMOIL_SOAK_COUNT`             | ไม่จำกัด    | หยุดหลังจากครบจำนวน seed ที่ระบุ                         |
| `NAUTILUS_TURMOIL_SOAK_PROGRESS_INTERVAL` | `100`     | Log ความคืบหน้าทุก ๆ จำนวน seed นี้ต่อ backend           |

### รัน soak [#run-the-soak]

รัน soak แบบต่อเนื่องด้วย:

```bash
scripts/soak-network-turmoil.sh
```

รัน soak แบบมีขอบเขตด้วย:

```bash
env NAUTILUS_TURMOIL_SOAK_COUNT=100 scripts/soak-network-turmoil.sh
```

แต่ละ seed จะเปิดการสุ่มลำดับ node และ latency ของ link ตั้งแต่ 1 ms ถึง 25 ms สถานการณ์นี้จะทำให้ server หลุดซ้ำ ๆ, วน client
ผ่านสถานะ reconnect ต่าง ๆ และ assert ลำดับ message ระดับ application แบบตรงตัว

Soak ไม่ได้เปิด `fail_rate` ของ Turmoil สำหรับ TCP การสูญเสีย packet โดยไม่มี model การ retransmit จะทำให้ข้อกำหนดการส่ง
ของ client ดูเกินจริงใน test ที่ตรวจการรักษาลำดับ

### การครอบคลุม platform [#platform-coverage]

Turmoil test ใช้ network แบบจำลองและไม่ได้ gate ไว้เฉพาะ Linux การไล่ seed จึงรันบน macOS ได้ด้วย unit test ของ socket และ
WebSocket บน localhost จริงหลายตัวใช้ `target_os = "linux"` เพื่อความเสถียรของ CI การรันบน macOS จึงไม่ได้ครอบคลุม TCP
ของ host ส่วนนั้น ให้ถือว่าชุด network test ทั้งหมดครอบคลุมแล้วก็ต่อเมื่อรันบน Linux CI หรือ Linux workstation แล้วเท่านั้น

## หมายเหตุด้าน implementation [#implementation-notes]

ส่วนเหล่านี้ map seam แบบ deterministic แต่ละตัวไปยัง source, วัตถุประสงค์ และข้อยกเว้น

### Seam ด้านลำดับการ iterate [#iteration-order-seams]

จุดใน production ใช้ collection ที่มีลำดับหรือการ sort อย่างชัดเจนเมื่อการ iterate สังเกตได้บนเส้นทาง DST

#### Matching engine [#matching-engine]

`crates/execution/src/matching_engine/mod.rs` ใช้ collection ที่มีลำดับสิบเอ็ดตัว:

- `execution_bar_types`
- `execution_bar_deltas`
- `account_ids`
- `cached_filled_qty`
- `post_match_order_ids`
- `bid_consumption`
- `ask_consumption`
- `queue_pending`
- `queue_ahead_orders`
- `queue_ahead_total`
- `queue_excess`

การลบออกจาก `queue_pending`, `queue_ahead_orders` และ `queue_ahead_total` ใช้ `.shift_remove()` เพื่อให้การลบรายการ
ไม่เปลี่ยนลำดับของรายการที่เหลือ

#### Reconciliation manager [#reconciliation-manager]

กฎข้อ 5 ครอบคลุม `crates/live/src/execution/manager.rs` map `orders` และ `fills` ใน `ReconciliationResult` ใช้ `IndexMap`
ใน `crates/execution/src/reconciliation/types.rs`

#### Balance ของ account [#account-balances]

Account trait คืน `IndexMap` จาก:

- `balances`
- `balances_total`
- `balances_free`
- `balances_locked`
- `starting_balances`

การจัดเก็บ balance และ margin บน `BaseAccount` และ `MarginAccount` ก็ใช้ `IndexMap` เช่นกัน ส่วน field `commissions` และ
`leverages` ยังเป็น `AHashMap`

#### Commission ของ position [#position-commissions]

`Position::commissions` ใน `crates/model/src/position.rs` ใช้ `IndexMap` position snapshot ใช้ map นี้ผ่าน `.values()` ใน
`crates/model/src/events/position/snapshot.rs` ทำให้ลำดับของมันสังเกตได้

#### การ aggregate ระดับ portfolio [#portfolio-aggregation]

`crates/portfolio/src/portfolio.rs` เก็บ `unrealized_pnls`, `realized_pnls` และ `net_positions` ใน `IndexMap` ส่วน
`accumulate_mark_values` สร้าง `IndexMap<Currency, Decimal>`

#### Data engine [#data-engine]

`crates/data/src/engine/` ใช้การจัดเก็บที่มีลำดับสำหรับ `book_snapshot_counts`, `bar_aggregators` และ `BookSnapshotInfos`
การลบระหว่าง iterate ใช้ `.shift_remove()`

#### Execution engine [#execution-engine]

`ExecutionEngine.clients` ใช้ `IndexMap` ตัวสะสม `client_ids` และ `venues` ใน `get_clients_for_orders()` ใช้ `IndexSet`

#### Backtest engine และ exchange [#backtest-engine-and-exchange]

`BacktestEngine.venues` และ `SimulatedExchange.matching_engines` คงลำดับการ iterate ของ venue และ instrument ไว้สำหรับ:

- Settlement
- Expiration
- Liquidation
- การสุ่มของ `FillModel` ที่มี seed

ตำแหน่งใน source คือ `crates/backtest/src/engine.rs` และ `crates/backtest/src/exchange.rs`

#### Trading algorithm [#trading-algorithm]

`strategy_event_handlers` ใน `crates/trading/src/algorithm/core.rs` ใช้ `IndexMap` เพื่อขับเคลื่อนการ fan-out
`msgbus::unsubscribe_*` แบบมีลำดับ

#### Analyzer [#analyzer]

`account_balances` และ `account_balances_starting` ใน `crates/analysis/src/analyzer.rs` ใช้ `IndexMap`

#### Cache API [#cache-api]

`get_orders_for_ids` และ `get_positions_for_ids` ใน `crates/common/src/cache/mod.rs` จะ sort vector ที่คืนกลับตาม
`client_order_id` และ `position_id` ส่วนการจัดเก็บเบื้องล่างยังใช้ `AHashSet` เพราะมี semantics แบบ set

#### Instrument store [#instrument-store]

`InstrumentStore.instruments` ใน `crates/common/src/providers.rs` ใช้ `IndexMap` กับ hasher `ahash` ลำดับนี้สังเกตได้ใน
adapter ที่ publish `DataEvent::Instrument` หนึ่งตัวต่อหนึ่งรายการจาก `get_all()` หรือ `list_all()`

#### Order emulator [#order-emulator]

`on_reset` ใน `crates/execution/src/order_emulator/emulator.rs` จะ sort set สามตัวที่ drain ออกมาก่อนการ fan-out
`msgbus::unsubscribe_*` แบบมีลำดับ:

- `subscribed_quotes`
- `subscribed_trades`
- `subscribed_strategies`

เส้นทางของ quote และ trade ยังทำให้ลำดับการสุ่มของ `UUID4::new` ที่มี seed เดินหน้าด้วย การจัดเก็บยังคงใช้ `AHashSet`

#### WebSocket subscription [#websocket-subscriptions]

`topics_from_map` ใน `crates/network/src/websocket/subscription.rs` จะ sort vector ที่คืนกลับเพื่อคงลำดับการ replay ตอน
reconnect ที่อยู่เบื้องหลัง `all_topics()` ขณะที่การจัดเก็บยังใช้ `DashMap` ที่มีค่าเป็น `AHashSet`

#### ข้อจำกัดของ collection ที่ไม่มีลำดับ [#unordered-collection-limits]

จุดที่ใช้ `AHashMap` / `AHashSet` ใน closure ของ `nautilus-live` ใช้เพื่อ lookup อย่างเดียว, อยู่หลัง wrapper แบบ shared
ownership ที่ทำงานพร้อมกันได้ (`Arc<DashMap>`, `AtomicMap`) หรือส่งต่อไปยังการ aggregate แบบสลับที่ได้ (commutative)
`backtest` มี hash collection เพิ่มเติมที่อยู่นอกขอบเขตการบังคับใช้สองไฟล์ของกฎข้อ 5 รวมถึงการตรวจสอบก่อนรันและ map
ของผลลัพธ์ ให้ถือว่าลำดับการ iterate ของสิ่งเหล่านี้อยู่นอกการรับประกันแบบ static จนกว่าแต่ละเส้นทางจะได้รับการตรวจสอบ

### Seam ด้านเวลา [#time-seams]

จุดที่ยังเหลือของ `Instant::now` และ `SystemTime::now` เป็น test, ตำแหน่งที่อยู่ใน file allowlist หรือข้อยกเว้นที่มี marker:

| ตำแหน่ง                               | การใช้งาน                                                  | การจัดการ                                                     |
| ------------------------------------ | -------------------------------------------------------- | ---------------------------------------------------------- |
| `crates/common/src/testing.rs`       | Timer ของ `wait_until` และ `wait_until_async`            | `// dst-ok` แบบ inline: ตัวควบคุมของ test ตั้งใจใช้เวลาจริง      |
| `crates/execution/src/engine/mod.rs` | จับเวลาเพื่อ log ตอน initialize ใน `load_cache`             | `// dst-ok` แบบ inline: การจับเวลาไม่มีผลต่อสถานะ DST         |
| `crates/common/src/cache/mod.rs`     | จับเวลาใน `check_integrity` และ `audit_own_order_books`   | File allowlist                                             |
| `crates/model/src/defi/reporting.rs` | Log ความคืบหน้า                                           | File allowlist                                             |
| `crates/core/src/time.rs`            | นิยามของ seam wall-clock                                 | ข้อยกเว้นของ seam ที่ระบุไว้ชัดเจน                                |

`jiff::Timestamp::now` และ `jiff::Zoned::now` ถูก hook ห้ามใช้ใน crate ที่อยู่ในขอบเขต จุดเรียก timestamp ที่เหลือคือ logging
bridge และ writer ซึ่งถูกตัดออกจากขอบเขตตาม [Logging ทำงานบน OS thread จริง](#logging-runs-on-real-os-threads)
`crates/core/src/datetime.rs::is_within_last_24_hours` เรียกผ่าน `nautilus_core::time::nanos_since_unix_epoch()` และ
เปรียบเทียบเป็น nanos แบบ `u64` โดยตรง

### Seam ด้านค่าสุ่ม [#randomness-seams]

ค่าสุ่มใน production บนเส้นทาง DST ใช้ seam ดังต่อไปนี้

#### การสร้าง UUID [#uuid-generation]

`crates/core/src/uuid.rs::UUID4::new()` ใช้ `madsim::rand::thread_rng()` ภายใน madsim runtime เมื่ออยู่ภายใต้
simulation ส่วน build ปกติและ body ของ `#[rstest]` ธรรมดาที่อยู่นอก madsim runtime ใช้ `rand::rng()`

เส้นทาง simulation ของ production รันภายใน runtime และใช้ byte ที่มี seed factory ของ order และ event ใน `nautilus-common`
และ `nautilus-risk` จะเรียกถึง seam นี้

#### ค่าสุ่มของ fill model [#fill-model-randomness]

`crates/execution/src/models/fill.rs::default_std_rng()` แยกตาม runtime แบบเดียวกัน `ProbabilisticFillState::new()` จะ
เรียกมันเมื่อไม่มี seed ให้ เมื่อผู้เรียกระบุ seed `StdRng::seed_from_u64` จะ deterministic ได้โดยไม่ต้องใช้ seam

#### ID แบบสุ่มของ matching engine [#random-matching-engine-ids]

เส้นทาง `use_random_ids` ใน `crates/execution/src/matching_engine/ids_generator.rs` จะเรียก `nautilus_core::UUID4::new()`
สำหรับ position ID และ venue order ID ส่วนรูปแบบ ID เริ่มต้น `{venue}-{raw_id}-{count}` เป็น deterministic โดยไม่ต้องใช้
byte สุ่ม

#### Jitter ของการ reconnect [#reconnect-jitter]

`crates/network/src/backoff.rs` สุ่มจาก `madsim::rand::thread_rng()` ภายใน simulation runtime ส่วน build ปกติและการเรียก
นอก Madsim runtime ใช้ `rand::rng()` การเริ่ม runtime ใหม่ด้วย seed เดิมจะเริ่มลำดับ jitter แบบจำลองใหม่ด้วย

### การแบ่ง submodule ของ Tokio [#tokio-submodule-split]

Facade ส่วนกลางส่ง `time`, `task`, `runtime` และ `signal` ผ่าน `madsim` ส่วนการใช้ submodule `sync`, `io`, `fs` และ `net`
ของ Tokio โดยตรง รวมถึง `select!` ยังคงใช้ `tokio` จริงภายใต้ simulation ขอบเขต `dst::net` ภายใน network จะเลือก byte
stream ของ Madsim แยกต่างหากสำหรับเส้นทาง HTTP และ WebSocket ที่รองรับ

การแทนที่ network ของ Tokio ทั้งระบบจะกระทบ dependency ต่าง ๆ ด้วย เช่น:

- `tokio-tungstenite`
- `tokio-rustls`
- `hyper-util`
- `hyper-rustls`

ขอบเขตภายใน network หลีกเลี่ยงการแทนที่ทั้งระบบนั้นโดยส่ง stream ให้ Hyper และ Tungstenite ส่วน TLS อยู่นอกขอบเขตของ
simulation นี้

การใช้งานโดยตรงที่อยู่ในขอบเขตได้แก่:

| ตำแหน่ง                                | ส่วนของ Tokio จริงที่ใช้                       | ขอบเขต                                                                    |
| ------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------ |
| `crates/network/src/net.rs`           | `tokio::net::{TcpListener, TcpStream}`     | Re-export transport ปกติไว้หลัง seam `crate::net`                          |
| `crates/network/src/socket/client.rs` | `tokio::io::{AsyncReadExt, AsyncWriteExt}` | ทำ I/O บน transport ที่เลือก                                               |
| `crates/network/src/tls.rs`           | `tokio::io::{AsyncRead, AsyncWrite}`       | นิยามขอบเขต I/O ของ TLS                                                   |
| `crates/network/src/socket/types.rs`  | `tokio::io::{ReadHalf, WriteHalf}`         | แยก `MaybeTlsStream<TcpStream>` โดย `TcpStream` มาจาก `crate::net`         |

เส้นทางเหล่านี้ใช้ socket จริงแม้อยู่ภายใต้ simulation implementation ของ channel ใน `tokio::sync` ก็ยังเป็นของจริง แต่ madsim
เป็นผู้จัด schedule ของ task ฝั่ง sender และ receiver ลำดับการส่งบน channel จึงยังเป็นส่วนหนึ่งของข้อกำหนดการจัด schedule
แบบ deterministic

### กฎการหลุดออกของ thread แบบดิบ [#raw-thread-escape-rules]

กฎข้อ 4 ของ hook ห้ามการ spawn thread แบบดิบ ยกเว้นสามกรณี:

- Test module แบบ `#[cfg(test)]`
- จุดใน production ที่เป็น `#[cfg(not(madsim))]` หรือ `#[cfg(not(all(feature = "simulation", madsim)))]` (เช่น thread
  ของ logging writer)
- Marker `// dst-ok` แบบ inline

`tokio::task::LocalSet` และ `tokio::task::spawn_blocking` ไม่รองรับภายใต้ `madsim` การตรวจสอบ codebase ไม่พบการใช้ทั้งสอง
ใน production ภายใน crate ที่อยู่ในขอบเขต จุดใหม่ต้องมี cfg gate หรือ marker `// dst-ok`

### Logging test ภายใต้ simulation [#logging-tests-under-simulation]

Thread ของ logging writer ถูก gate ออกด้วย cfg ภายใต้ simulation และภายใต้ `cfg(madsim)` log event จะถูกทิ้ง test ที่
initialize file-logging writer จะค้างหรือ assert กับไฟล์ log ที่ว่างเปล่า submodule ที่ได้รับผลกระทบจึงถูก gate ออกที่ขอบเขต
ของ module:

- `crates/common/src/logging/logger.rs::tests::serial_tests`.
- `crates/common/src/logging/macros.rs::tests`.

`logger.rs::tests::sim_tests::test_init_under_madsim_skips_writer_thread_and_forces_bypass` รันภายใต้ simulation และ
pin พฤติกรรมที่ถูก gate ไว้

## ขอบเขตของข้อกำหนด [#scope-boundaries]

ข้อกำหนดถูกจำกัดด้วยขอบเขตด้านล่าง แต่ละหัวข้อย่อยระบุพฤติกรรมที่อาจแตกต่างกันระหว่างการรัน หรืออยู่นอกเส้นทาง DST
ที่ตรวจสอบแล้ว

### Python และ FFI ไม่อยู่ในขอบเขตของ DST [#python-and-ffi-are-not-in-dst-scope]

DST รันภายใต้ test harness แบบ native ของ Rust ข้อกำหนดนี้ไม่รวม binding ใต้ `crates/*/src/python/`, FFI module ใต้
`crates/core/src/ffi/` และ `crates/model/src/ffi/` รวมถึงโค้ดที่เข้าถึงได้ผ่าน binding เหล่านี้เท่านั้น เส้นทาง Rust ใดก็ตาม
ที่ harness DST แบบ native เข้าถึงได้ต้องเป็นไปตามข้อกำหนด แม้ type เดียวกันจะถูก export ผ่าน binding ด้วยก็ตาม

Hook `check-dst-conventions` ใช้นโยบายนี้โดยข้าม path `/python/` และ `/ffi/` ใน crate ที่อยู่ในขอบเขต การเรียก clock, RNG และ
threading ที่อยู่หลัง path เหล่านั้นไม่อยู่ภายใต้ข้อกำหนด

DST ครอบคลุม state machine ของวงจรชีวิต order, reconciliation, matching, risk และ execution ใน Rust engine เป็นหลัก
strategy ของผู้ใช้จะ replay ได้ก็ต่อเมื่อเขียนด้วย Rust หรือขับเคลื่อนผ่าน test harness แบบ Rust-native เท่านั้น

### จำกัดตาม platform [#platform-scoped]

การ override libc ของ `madsim` สำหรับ `clock_gettime` และ `getrandom` ขึ้นกับ platform ข้อกำหนดนี้ไม่ได้อ้างว่าทำซ้ำได้
ระดับ bit ข้าม platform seed ที่ทำซ้ำความล้มเหลวได้บน Linux x86_64 อาจทำซ้ำไม่ได้บน macOS aarch64

### Dependency ที่ไม่ได้ทำ alias จะหลุดออกไปเงียบ ๆ [#non-aliased-dependencies-escape-silently]

Dependency จะหลุดออกจาก simulator โดยไม่มี error เมื่อเข้าถึง OS ผ่าน:

- การเรียก `libc` โดยตรง
- การข้ามผ่าน `std::net`
- ค่าสุ่มที่ไม่ได้ส่งผ่าน seam เช่น `fastrand` หรือ `OsRng`

Crate ที่อยู่ในขอบเขตได้รับการตรวจสอบแล้ว adapter crate และ infrastructure crate ต้องได้รับการตรวจสอบแยกต่างหากก่อน
เข้าสู่เส้นทาง DST

### ข้อจำกัดด้านขอบเขตของ transport [#transport-scope-limits]

Dependency ต่อไปนี้ใช้ `tokio` จริงภายใน:

- `tokio-tungstenite`
- `tokio-rustls`
- `hyper-util`
- `hyper-rustls`
- `redis`
- `sqlx`

[Transport HTTP และ WebSocket แบบจำลอง](#simulated-http-and-websocket-transport) ส่ง stream ของ Madsim ให้กับเส้นทาง
plaintext ที่รองรับ ส่วน raw socket client, TLS, Redis และ SQL อยู่นอกขอบเขตนั้น Turmoil ให้การจำลอง network แยกต่างหาก
ตามที่อธิบายไว้ใน [Network seed soak](#network-seed-soaks)

Test module ต่อไปนี้ใช้ socket บน localhost จริงและถูก gate ออกด้วย cfg ภายใต้ `all(feature = "simulation", madsim)`:

- `crates/network/src/http/client.rs::tests`
- `crates/network/src/http/tests.rs`
- `crates/network/src/socket/client.rs::tests`
- `crates/network/src/socket/client.rs::rust_tests`
- `crates/network/src/websocket/client.rs::tests`
- `crates/network/src/websocket/client.rs::rust_tests`
- `crates/network/tests/integration/websocket_proxy.rs`

เส้นทาง production ของ test เหล่านี้เรียกถึง time primitive ของ madsim ผ่าน `dst::time::*` ซึ่งจะ panic เมื่อถูกเรียกจาก
runtime ของ `#[tokio::test]`

Retry module ใน `crates/network/src/retry.rs` รันได้ทั้งสอง runtime โดย attribute ของ test จะสลับระหว่าง
`#[tokio::test(start_paused = true)]` และ `#[madsim::test]` การอ่านเวลาและการ sleep ใช้ `crate::dst::time` การเลื่อนเวลา
เสมือนอย่างชัดเจนใช้ฟังก์ชัน `advance_clock` ที่ gate ด้วย cfg body ของ test ชุดเดียวกันจึงครอบคลุมทั้ง build ปกติและ
build แบบ simulation

### การจัดการ signal [#signal-handling]

`nautilus_common::live::dst::signal` เปิดเผย re-export `ctrl_c` และ `terminate` ที่ส่งผ่าน seam แล้ว run loop ใน
`crates/live/src/node/mod.rs` ใช้สิ่งเหล่านี้ ภายใต้ `cfg(madsim)` test สามารถ inject การ shutdown ของ node ผ่าน
`madsim::runtime::Handle::send_ctrl_c` ได้ ส่วน entry point ของ binary ของ adapter ยังเรียก `tokio::signal::ctrl_c` โดยตรง
และอยู่นอกขอบเขต

### Logging ทำงานบน OS thread จริง [#logging-runs-on-real-os-threads]

ระบบย่อย logging spawn writer thread ผ่าน `std::thread::Builder` และใช้ `std::sync::mpsc` ภายใต้ simulation thread นี้
จะไม่ถูก spawn และ log event จะถูกทิ้ง ผลลัพธ์ของ log อยู่นอกข้อกำหนดด้าน determinism: writer ทำหน้าที่เขียนอย่างเดียว
ไม่เคยอ่านหรือเปลี่ยนแปลงสถานะของ simulation

### Adapter [#adapters]

พฤติกรรมแบบ end-to-end ของ adapter อยู่นอกขอบเขตของ simulation test ฝั่ง upstream เส้นทางที่ยังไม่ได้ตรวจสอบอาจมี
สิ่งต่อไปนี้ ขึ้นอยู่กับ adapter:

- การเรียก `jiff::Timestamp::now` หรือ `jiff::Zoned::now` โดยตรง
- การเรียก `SystemTime::now` โดยตรง
- การเรียก RNG ที่ไม่ได้ส่งผ่าน seam
- Transport client แบบดิบ

Adapter ต้องได้รับการตรวจสอบจุดเหล่านี้ก่อน ข้อกำหนด DST จึงจะครอบคลุมพฤติกรรมของมัน

## ความสัมพันธ์กับการทดสอบชั้นอื่น [#relationship-to-other-testing-layers]

DST เสริมการทดสอบที่มีอยู่ ไม่ได้มาแทนที่ส่วนใดเลย

| ชั้น                      | ครอบคลุม                                               | ความสัมพันธ์กับ DST                                 |
| ----------------------- | ---------------------------------------------------- | ----------------------------------------------- |
| Unit test               | Logic ล้วน, การคำนวณ, parser, transformer             | ไม่เปลี่ยนแปลง                                     |
| Integration test        | ปฏิสัมพันธ์ระหว่าง component, ขอบเขตของ I/O             | ไม่เปลี่ยนแปลง DST รันควบคู่กัน ไม่ได้มาแทน             |
| Property-based test     | Invariant บนโดเมนของ input (parser, roundtrip)        | ไม่เปลี่ยนแปลง                                     |
| Acceptance test         | สถานการณ์ backtest และ live แบบ end-to-end            | ไม่เปลี่ยนแปลง                                     |
| Deterministic sim (DST) | จังหวะเวลาของ async, การจัด schedule, ความถูกต้องของการกู้คืน | เพิ่มการสำรวจที่ replay ได้ด้วย seed                 |

DST ครอบคลุม concurrency แบบ async และความถูกต้องของ state machine ตัวอย่างความล้มเหลวได้แก่:

- Shutdown message ถูกทิ้งภายใต้ลำดับการ wakeup ของ task แบบหนึ่ง
- Reconciliation event หายไปเมื่อลำดับการ iterate เปลี่ยน

การทดสอบชั้นอื่นยังคงรับผิดชอบขอบเขตเดิมของตน

## สถานะ [#status]

### การสลับ runtime [#runtime-swap]

ชั้นที่ 1 implement แล้ว `nautilus_common::live::dst` เปิดเผย re-export ที่ส่งผ่าน seam สำหรับ `time`, `task`, `runtime`
และ `signal` call site ใน production สำหรับ `time`, `task` และ `runtime` ใช้ seam นี้ ส่วนการนำ signal มาใช้ยังเป็นเพียงบางส่วน
ดู [การจัดการ signal](#signal-handling)

### การแทนที่แหล่งความไม่ deterministic [#nondeterminism-substitution]

ชั้นที่ 2 implement แล้วใน crate ที่อยู่ในขอบเขตทั้ง 17 ตัว seam ครอบคลุมเวลาแบบ wall-clock, เวลาแบบ monotonic, ค่าสุ่ม
และลำดับการ iterate ที่สังเกตได้ [หมายเหตุด้าน implementation](#implementation-notes) แสดงรายการเส้นทางที่ตรวจสอบแล้ว
และข้อยกเว้นที่เหลืออยู่

### สถานะของการบังคับใช้แบบ static [#static-enforcement-status]

`check-dst-conventions` รันใน pre-commit และ CI ครอบคลุมเงื่อนไขเชิงโครงสร้างที่เป็นแกนสำคัญ และอนุญาตข้อยกเว้นรายบรรทัด
ที่ review แล้วผ่าน `// dst-ok`

### ข้อจำกัดของการยืนยันที่ runtime [#runtime-verification-limit]

การยืนยันครอบคลุม test ของ seam และ static check repository นี้ไม่ได้เปรียบเทียบการรัน adapter แบบเต็มข้าม process ใหม่

### Simulation smoke gate [#simulation-smoke-gate]

Nightly workflow และ pre-flight แบบ local ใช้ DST target ชุดเดียวกัน:

| Entry point                           | ลำดับที่เกี่ยวข้อง                                            | วัตถุประสงค์                                                  |
| ------------------------------------- | --------------------------------------------------------- | -------------------------------------------------------- |
| `.github/workflows/nightly-tests.yml` | `check-code-sim` > `cargo-test-sim`                       | รัน DST smoke gate แบบ nightly และแบบสั่งรันด้วยมือ              |
| `make pre-flight`                     | `check-code-sim` > `cargo-test-sim` > `cargo-test-extras` | ล้มเหลวตั้งแต่เนิ่น ๆ เมื่อ DST lint ไม่ผ่าน ก่อนถึงชุด test ของ Rust |

`check-code-sim` รัน Clippy เวอร์ชัน stable ที่ pin ไว้ด้วย `--features simulation` และ `cfg(madsim)` กับ
`nautilus-common`, `nautilus-core`, `nautilus-event-store`, `nautilus-network`, `nautilus-execution` และ `nautilus-live`
มีอีกขาหนึ่งที่ใช้ `--no-default-features` แยกต่างหาก เพื่อ compile และ lint adapter ที่ตรวจสอบแล้วแต่ละตัวโดยไม่มี default
feature ในขาของ core แบบ standard precision

`cargo-test-sim` ใช้การเรียก nextest สามครั้งที่มีชุด feature สอดคล้องกัน:

| Precision | Package                                                                                                               | Feature                     | การเลือก                                                                                             |
| --------- | --------------------------------------------------------------------------------------------------------------------- | --------------------------- | --------------------------------------------------------------------------------------------------- |
| Standard  | `nautilus-common`, `nautilus-core`, `nautilus-event-store`, `nautilus-network`, `nautilus-execution`, `nautilus-live` | `simulation`                | Test ที่เข้ากันได้ทั้งหมดของ common, event-store, network และ execution รวมถึง test เฉพาะจุดของ live และ core |
| Standard  | Adapter ที่ตรวจสอบแล้วแต่ละตัว                                                                                            | `simulation`                | เฉพาะ integration test `dst` โดยไม่มี default feature ของ adapter                                    |
| High      | `nautilus-common`, `nautilus-execution`                                                                               | `simulation,high-precision` | Test ทั้งหมดในทั้งสอง package                                                                          |

Nextest compile library และ test target ที่เลือกไว้ gate นี้จึงไม่ต้องรัน Cargo build แยกต่างหาก การเรียกแต่ละครั้ง resolve
ชุด feature เพียงครั้งเดียวข้าม package ในชุดนั้น เมื่อรวมกันแล้วจะทดสอบเส้นทาง `QuantityRaw` (`u64` / `u128`) และ
`PriceRaw` (`i64` / `i128`) ที่ส่งผ่าน seam ในความกว้าง fixed-point ทั้งสองแบบ

#### Common test [#common-tests]

การรันแบบ standard precision จะ execute test ทั้งหมดของ `nautilus-common` ที่เข้ากันได้กับ simulation feature graph ของมัน
ส่งต่อ `nautilus-core/simulation` ทำให้เลือก cfg branch `wall_clock_now` ตลอดทั้งชุด

Body ของ `#[rstest]` ธรรมดารันนอก madsim runtime และใช้ fallback `SystemTime::now()` ของ seam ซึ่งเป็นเส้นทางเดียวกับที่
libc shim ของ madsim ใช้เมื่ออยู่นอก runtime

Test module ของ `LiveClock` ถูก gate ออกด้วย cfg เพราะ case แบบ `#[rstest]` ธรรมดาของมันเริ่ม task `LiveTimer` โดยไม่มี
madsim runtime และส่วนใหญ่รอให้ wall-clock เดินไปข้างหน้า

`live::dst::tests::test_dst_wall_clock_advances_with_virtual_time` รันภายใต้ `#[madsim::test]` และ assert ว่า
`nanos_since_unix_epoch` เดินหน้าไปพร้อมกับ `madsim::time::sleep` ซึ่ง pin พฤติกรรมของ wall-clock เสมือนภายใน runtime

#### Event store test [#event-store-tests]

การรันแบบ standard precision จะ execute test ทั้งหมดของ `nautilus-event-store` ที่เข้ากันได้กับ simulation โดยทดสอบ event
writer และ marker writer แบบ synchronous ภายใต้ `cfg(madsim)` รวมถึงการจัดลำดับ sequence แบบ deterministic test ที่พึ่ง
OS thread แบบ blocking ยังคงมีการทดสอบแบบ native และถูก gate ออก เพราะ thread เหล่านั้นทำงานนอกการควบคุมการจัด schedule
ของ madsim

Smoke lane ของ event-store จะ compile และทดสอบ implementation ด้าน simulation ที่มีอยู่ โดยไม่ได้ขยาย static convention
hook ไปยังโค้ด production ของ event-store

#### Reconciliation ตอน startup ของ live [#live-startup-reconciliation]

Regression test เฉพาะจุดของ `nautilus-live` รันภายใต้ madsim โดยยืนยันว่า mass-status request ที่ค้างอยู่จะถึง timeout ที่ตั้งไว้
และรายงาน error ที่คาดไว้ และยืนยันการ cleanup ของ node ด้วย flow ของ timeout และ cleanup ทั้งหมดไม่เคยเข้าไปใช้ timer
ของ Tokio จริงเลย

#### Network test [#network-tests]

การรันจะ execute test ทั้งหมดของ `nautilus-network` ยกเว้น module ที่ผูกกับ transport ซึ่งถูก gate ออกด้วย cfg ที่ source
การครอบคลุมรวมถึง test ของ seam เวลาเสมือนสำหรับ sleep, timeout และ rate limiter รวมถึงชุด retry ที่ทดสอบจังหวะของ backoff
`crates/network/tests/simulation.rs` เพิ่มการตรวจสอบ WebSocket reconnect, การปฏิเสธ endpoint ที่ไม่รองรับและ Sockudo และ
การ reset jitter unit test ใน `crates/network/src/http/simulation.rs` ครอบคลุม byte ของ request, ขีดจำกัดของ response,
deadline ของ body, การยกเลิก, redirect policy และการปฏิเสธ HTTPS/proxy

`#[madsim::test]` ใช้ seed ที่เปลี่ยนไปเป็นค่าเริ่มต้นและรายงาน seed เมื่อล้มเหลว ตั้ง `MADSIM_TEST_SEED` เพื่อ replay schedule
job `dst smoke (cfg madsim)` ใน `.github/workflows/nightly-tests.yml` เลือก seed ต่อเนื่องห้าตัวจาก
`GITHUB_RUN_NUMBER * MADSIM_TEST_NUM` test การ reset runtime ของ jitter จะ pin seed ของตัวเองและเสริมการตรวจสอบ network
แบบข้าม seed

#### Execution test [#execution-tests]

การรันจะ execute test ทั้งหมดของ `nautilus-execution` case แบบ `#[rstest]` ธรรมดาเหล่านี้ทดสอบ branch ที่ gate ด้วย cfg ใน
matching engine, fill model และ execution engine โดยไม่เข้าไปใน madsim runtime `default_std_rng()` จึงใช้ fallback ไปยัง
RNG ของ host ใน test เหล่านี้

#### Core seam test [#core-seam-tests]

การเลือก test เฉพาะจุดของ `nautilus-core` pin `wall_clock_now` เทียบกับเวลาเสมือน

#### Adapter test [#adapter-tests]

Adapter ที่ตรวจสอบแล้วแต่ละตัวรัน integration test `dst` ของตนภายใต้ `simulation` โดยไม่มี default feature ของ crate
case แบบ `#[madsim::test]` เหล่านั้น pin slice ที่บันทึกไว้ในคู่มือ integration ของ adapter ได้แก่ subscribe byte, พฤติกรรม
reconnect, frame ของ authentication และ wire field ราย operation

#### การครอบคลุมโดยรวมของ gate [#overall-gate-coverage]

Case แบบ `#[madsim::test]` ใน `nautilus-common`, `nautilus-core`, `nautilus-network`, `nautilus-live` และ adapter ที่ตรวจสอบแล้ว
ให้การครอบคลุมด้วย scheduler แบบ deterministic gate ทั้งหมดจับการเบี่ยงเบนใน seam ที่ gate ด้วย cfg ได้ แต่ไม่ได้ยืนยัน
ความ deterministic แบบ end-to-end ของ adapter

## อ่านเพิ่มเติม [#further-reading]

- `.pre-commit-hooks/check_dst_conventions.sh` นิยามกฎการบังคับใช้ทั้งเจ็ดข้อแบบเต็ม และอธิบายข้อตกลงเรื่อง marker `// dst-ok`
- [FoundationDB testing philosophy](https://apple.github.io/foundationdb/testing.html)
- [บทความ blog เรื่อง simulation testing ของ TigerBeetle](https://tigerbeetle.com/blog/)
- [madsim repository](https://github.com/madsim-rs/madsim) runtime แบบ deterministic
- [Turmoil repository](https://github.com/tokio-rs/turmoil) ตัวจำลอง network แบบ deterministic
