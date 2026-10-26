# Benchmarking

คู่มือนี้อธิบายการเขียน รัน และ profile benchmark ของ NautilusTrader ครอบคลุมโครงสร้างของ benchmark,
ตัวอย่าง, คำสั่งที่ใช้ในเครื่อง และขั้นตอนการวัดผลสำหรับผลลัพธ์ที่จะเผยแพร่

สำหรับขอบเขตของ benchmark, หลักฐานที่ต้องมี และนโยบาย CI ดูที่
[`/BENCHMARKING.md`](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/BENCHMARKING.md) ที่ root ของ repository

---

## ภาพรวมของเครื่องมือ {#tooling-overview}

เลือกเครื่องมือตามงานและผลลัพธ์ที่ต้องการ:

| เครื่องมือ                                                  | สิ่งที่วัด                                     | ใช้สำหรับ                                                  |
| --------------------------------------------------------- | ----------------------------------------- | --------------------------------------------------------- |
| [Criterion](https://docs.rs/criterion/latest/criterion/)  | เวลา wall-clock พร้อม confidence interval   | operation ที่ใช้เวลาเกินราว 100 ns และเวลาที่ใช้จริง          |
| [iai](https://docs.rs/iai/latest/iai/)                    | จำนวน CPU instruction ที่ถูก retire ภายใต้ Cachegrind | operation ขนาดเล็กที่ deterministic และการตรวจจับการเปลี่ยนแปลง |
| [CodSpeed](https://codspeed.io/docs/instruments/cpu)      | ต้นทุน CPU และพฤติกรรม cache แบบจำลอง        | การเปรียบเทียบงาน CPU ที่ deterministic ใน pull request อย่างเสถียร |
| [flamegraph](https://github.com/flamegraph-rs/flamegraph) | profile ของ call stack แบบ sampling        | การหาว่างานอยู่ตรงไหนใน slow path ที่เป็นตัวแทน              |

Criterion รายงานเวลาที่ผู้ใช้มองเห็นได้จริง ส่วน iai ให้ค่าที่เสถียรสำหรับ binary, toolchain และ input เดียวกัน
โดยไม่ต้องควบคุม noise ของเครื่อง host ให้เปรียบเทียบผลของ iai เฉพาะภายใต้สมมติฐานการ generate code แบบเดียวกัน
และใช้ Criterion เมื่อผลลัพธ์ที่ต้องการคือเวลาที่ใช้จริง

---

## โครงสร้างไดเรกทอรี {#directory-layout}

แต่ละ crate เก็บ benchmark ไว้ในโฟลเดอร์ `benches/` ของตัวเอง:

```text
crates/<crate_name>/
└── benches/
    ├── foo_criterion.rs
    └── foo_iai.rs
```

ลงทะเบียน benchmark แต่ละตัวอย่างชัดเจนใน `Cargo.toml` ของ crate เพื่อให้ `cargo bench` ค้นพบได้:

```toml
[[bench]]
name = "foo_criterion"
path = "benches/foo_criterion.rs"
harness = false

[[bench]]
name = "foo_iai"
path = "benches/foo_iai.rs"
harness = false
```

หากต้องการใช้ workflow วัดประสิทธิภาพของ CI ที่รันทุกคืน ให้ลงทะเบียน benchmark ไว้ ถ้า `CI_BENCH_CRATES` ยังไม่มี
crate นั้น ให้เพิ่ม crate ใน `Makefile` ของ workspace เพิ่ม target ของ Criterion ที่ deterministic ลงใน
`CODSPEED_BENCH_TARGETS` เมื่อการจำลอง CPU ยังคงวัดสิ่งที่ benchmark ตั้งใจจะวัดได้ อย่าเพิ่ม iai, API
`iter_custom` หรือ `with_filter` ของ Criterion, งานที่ขึ้นกับ OS หรือ benchmark แบบ wall-clock ที่ทำงานพร้อมกัน
ลงในชุดของ CodSpeed

---

## การเขียน benchmark ด้วย Criterion {#writing-criterion-benchmarks}

- **Setup**: วางงานทั้งหมดที่ไม่เปลี่ยนระหว่างแต่ละรอบไว้ในโค้ดรอบนอก หรือใน setup closure ของ
  `iter_batched_ref` ให้อยู่นอก body ที่ส่งให้ `iter`
- **Input**: ห่อ input ด้วย `black_box` เพื่อไม่ให้ optimizer ตัดทิ้งไป
- **Benchmark ที่แก้ไขข้อมูล**: ใช้ `iter_batched_ref` เพื่อไม่ให้การ `Drop` ของ input อยู่ในช่วงที่จับเวลา
  มิฉะนั้นการ drop โครงสร้างขนาดใหญ่ที่เป็นเจ้าของจะครอบงำผลการวัด
- **Throughput**: เพิ่ม `Throughput::Elements(n)` ให้กับกลุ่มที่กำหนดพารามิเตอร์ตามขนาด เพื่อให้ Criterion
  รายงาน throughput ต่อ element
- **เจตนา**: เขียน comment ว่า benchmark วัดอะไร (hot path, worst case หรือกรณี cache-cold)
  เพื่อให้ผู้อ่านเข้าใจว่า regression มีความหมายอย่างไร

```rust
use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

const SIZES: &[usize] = &[10, 100, 1_000];

fn bench_my_op(c: &mut Criterion) {
    let mut group = c.benchmark_group("module/my_op");

    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched_ref(
                || populate(n),
                |state| state.run(black_box(n)),
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_my_op);
criterion_main!(benches);
```

---

## การเขียน benchmark ด้วย iai {#writing-iai-benchmarks}

`iai` ต้องการฟังก์ชันที่ไม่รับพารามิเตอร์ ใช้กับ operation ขนาดเล็กที่เป็น pure เพื่อให้จำนวน instruction
ที่วัดได้เน้นเฉพาะงานที่ตั้งใจวัด

```rust
use std::hint::black_box;

fn bench_add() -> i64 {
    let a = black_box(123);
    let b = black_box(456);
    a + b
}

iai::main!(bench_add);
```

การจองหน่วยความจำ, การสุ่ม และ system call จะเพิ่ม instruction ของตัวเองเข้าไปในผลลัพธ์ ให้วางการ setup
ตัวแปรไว้นอกฟังก์ชันที่วัด และเปรียบเทียบเฉพาะค่าที่ได้จาก toolchain และ target เดียวกัน

---

## การรัน benchmark ในเครื่อง {#running-benches-locally}

| เป้าหมาย                              | คำสั่ง                                                                 |
| ---------------------------------- | --------------------------------------------------------------------- |
| ทุก bench ใน crate เดียว              | `cargo bench -p nautilus-execution`                                   |
| bench module ของ core หนึ่งตัว          | `cargo bench -p nautilus-execution --bench matching`                  |
| bench module ของ engine หนึ่งตัว        | `cargo bench -p nautilus-execution --bench matching_engine`           |
| pattern ชื่อ benchmark ของ core หนึ่งแบบ | `cargo bench -p nautilus-execution --bench matching -- iterate`       |
| pattern ชื่อ benchmark ของ engine หนึ่งแบบ | `cargo bench -p nautilus-execution --bench matching_engine -- submit` |
| smoke run แบบเร็ว (sample น้อย)         | `cargo bench ... -- --quick`                                          |
| ทุก bench ที่ลงทะเบียนไว้สำหรับรอบกลางคืน   | `make cargo-ci-benches`                                               |
| build ชุดของ CodSpeed                 | `make cargo-codspeed-build`                                           |
| ตรวจสอบชุดของ CodSpeed ที่ build แล้ว    | `make cargo-codspeed-run`                                             |

Criterion เขียนรายงาน HTML ไว้ที่ `target/criterion/` เปิดดูได้ที่ `target/criterion/report/index.html`
รายงานมี violin plot ของแต่ละ bench, confidence interval และการเปรียบเทียบกับ baseline ที่บันทึกไว้จากการรันครั้งก่อน

`make install-tools` ติดตั้ง `cargo-codspeed` ตามเวอร์ชันที่ pin ไว้ การรัน CodSpeed ในเครื่องจะตรวจสอบว่า
benchmark target ที่เลือกไว้ build และลงทะเบียนได้ แต่จะไม่อัปโหลดผลการวัด job `codspeed-benchmarks` ใน
`.github/workflows/performance.yml` เป็นผู้วัดและอัปโหลดผลลัพธ์

### Workload มาตรฐานของ backtest {#canonical-backtest-workloads}

กรณี backtest มาตรฐานใช้ 10,000 แถวแรกของไฟล์ `test_data/btc-perp-20211231-20220201_1m.csv` ที่ check-in
ไว้ใน repository โดยมี fixture ร่วมที่ขับเคลื่อนสถานการณ์แบบ replay อย่างเดียว, market order ตามตารางเวลา,
limit order แบบ passive และ bar-EMA ทั้ง test ความถูกต้องและเส้นทางการจับเวลาทั้งสองแบบใช้ fixture loader
และ fingerprint ของผลลัพธ์ชุดเดียวกันทุกประการ

รันการตรวจสอบเชิงความหมายก่อน:

```bash
CARGO_BUILD_JOBS=16 cargo test --locked -p nautilus-backtest \
    --test integration canonical_backtest_workloads::
```

จากนั้นรัน Criterion ในโหมด test เพื่อยืนยันว่าทุกกรณีของ benchmark ที่ได้รับผลกระทบทำงานได้ โดยไม่เก็บ sample:

```bash
CARGO_BUILD_JOBS=16 cargo bench --locked -p nautilus-backtest \
    --bench engine -- canonical --test
```

กรณี `run_preloaded` จะโหลด CSV และ build engine นอกช่วงเวลาที่ `iter_custom` คืนค่า ส่วนกรณี
`load_build_run` รวมการโหลด CSV, การ setup engine, การลงทะเบียนข้อมูล และ `BacktestEngine::run`
ทั้งสองแบบไม่นับการ project ผลลัพธ์และการตรวจสอบ fingerprint ไว้ในเวลาที่รายงาน แต่ยังคงตรวจสอบผลลัพธ์
หลังทุกรอบที่วัด

ดู baseline ที่เผยแพร่, บันทึกการวัด และ profile target ปัจจุบันได้ที่
[`crates/backtest/benches/BENCHMARKS.md`](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/crates/backtest/benches/BENCHMARKS.md)

[คู่มือการย้ายไป v2](https://github.com/nautechsystems/nautilus_trader/blob/04f766b4d434d67fc8672026fefa25135bdc01be/MIGRATION_V2.md#compare-backtest-performance) มีขั้นตอนการเปรียบเทียบประสิทธิภาพ
backtest ข้ามเวอร์ชัน

---

## การวัดผลด้วย Criterion เพื่อเผยแพร่ {#measure-criterion-for-publication}

ใช้ profile `bench-lto` สำหรับผลของ Criterion ที่จะรายงานหรือเผยแพร่ profile นี้สืบทอดจาก `release`,
เก็บ debug symbol ไว้ครบ, เปิด fat LTO และใช้ code generation unit เดียว ส่วน profile `bench` ปกติจะเก็บ
debug symbol ครบแต่ไม่มี LTO และเหมาะกับการทำงานวนรอบในเครื่องมากกว่า

1. ทำให้เครื่องเงียบที่สุด บน Linux ให้ตั้ง CPU governor เป็น `performance` หากคุณดูแลเครื่อง host นั้นเอง
   และสามารถคืนค่าเดิมได้:

   ```bash
   sudo cpupower frequency-set -g performance
   ```

1. บน Linux ให้ปิด ASLR สำหรับ process ของ benchmark และรัน benchmark ที่เลือกด้วย `bench-lto`:

   ```bash
   setarch "$(uname -m)" -R cargo bench --profile bench-lto -p <crate> --bench <name>
   ```

1. รันแบบเต็มหลาย session และระบุว่าแต่ละกรณีใช้ผลที่ดีที่สุดหรือค่ามัธยฐาน

1. บันทึกรุ่น CPU, kernel หรือระบบปฏิบัติการ, Rust toolchain และ build profile ไว้พร้อมกับผลลัพธ์:

   ```text
   Hardware: <CPU model>, <kernel or operating system>
   Toolchain: <rustc version>
   Profile: bench-lto (release + lto = "fat" + codegen-units = 1, debug = full)
   ```

สำหรับการวิเคราะห์เชิงลึก ให้ควบคุม hyper-threading และการปรับความถี่แบบไดนามิกใน firmware ผลลัพธ์ที่เผยแพร่
ต้องบันทึกการควบคุมเหล่านี้ไว้ เมื่อต่างจากสถานะปกติของเครื่อง host

iai ทำงานภายใต้โมเดล CPU เสมือนของ Cachegrind ดังนั้นความเงียบของ host, การปรับความถี่ และ ASLR
จึงไม่มีผลต่อจำนวน instruction ให้รัน iai โดยไม่ต้องใช้การควบคุม noise แบบที่ใช้กับ Criterion

---

## การสร้าง flamegraph {#generating-a-flamegraph}

`cargo-flamegraph` สร้าง profile ของ call stack แบบ sampling สำหรับ bench หนึ่งตัว ใช้เมื่อ benchmark
เกิด regression และยังไม่ชัดว่าการเรียกภายในตัวไหนเป็นต้นเหตุ

1. ติดตั้งครั้งเดียวต่อเครื่อง:

   ```bash
   cargo install flamegraph
   ```

2. รัน bench ที่ต้องการด้วย profile `bench`:

   ```bash
   cargo flamegraph --bench matching -p nautilus-common --profile bench
   ```

3. เปิด `flamegraph.svg` ในเบราว์เซอร์แล้วซูมเข้าไปดู hot path

### Linux {#linux}

ต้องมี `perf` บน Debian/Ubuntu:

```bash
sudo apt install linux-tools-common linux-tools-$(uname -r)
```

หาก `perf_event_paranoid` บล็อกการรัน:

```bash
sudo sh -c 'echo 1 > /proc/sys/kernel/perf_event_paranoid'
```

ค่า `1` มักจะเพียงพอ ตั้งกลับเป็น `2` (ค่าเริ่มต้น) หลังใช้งานเสร็จ หรือกำหนดให้ถาวรผ่าน `/etc/sysctl.conf`

### macOS {#macos}

`DTrace` ต้องใช้สิทธิ์ root ดังนั้นต้องรัน `cargo flamegraph` ด้วย `sudo`

:::warning
การรันด้วย `sudo` จะสร้างไฟล์ใน `target/` ที่มีเจ้าของเป็น root ทำให้คำสั่ง `cargo` ครั้งถัดไปเกิด
permission error คุณอาจต้องลบไฟล์ที่เป็นของ root ด้วยตนเอง หรือรัน `sudo cargo clean`
:::

```bash
sudo cargo flamegraph --bench matching -p nautilus-common --profile bench
```

profile `bench` เก็บ debug symbol ไว้ครบ flamegraph จึงแสดงชื่อฟังก์ชันที่อ่านได้ โดยไม่ทำให้ binary สำหรับ
production บวม (binary เหล่านั้นยังคงใช้ `panic = "abort"` และ build ผ่าน `[profile.release]`)

> **หมายเหตุ** binary ของ benchmark คอมไพล์ด้วย `[profile.bench]` แบบกำหนดเองที่นิยามไว้ใน `Cargo.toml`
> ของ workspace profile นี้สืบทอดจาก `release` และตั้ง `debug = "full"` จึงคงทั้ง optimization เต็มรูปแบบ
> *และ* debug symbol ไว้ ทำให้เครื่องมืออย่าง `cargo flamegraph` หรือ `perf` แสดง stack trace ที่คนอ่านได้

---

## Template {#templates}

ไฟล์ตั้งต้นอยู่ใน [`docs/dev_templates/`](../dev_templates/):

- **Criterion**: [`criterion_template.rs`](../dev_templates/criterion_template.rs)
- **iai**: [`iai_template.rs`](../dev_templates/iai_template.rs)

คัดลอก template ไปไว้ใน `benches/` ของ crate เป้าหมาย ปรับ import และชื่อกลุ่ม ลงทะเบียนใน `Cargo.toml`
แล้วเริ่มวัดผลได้เลย
