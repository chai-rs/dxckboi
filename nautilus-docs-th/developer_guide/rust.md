# Rust

หน้านี้กำหนด convention ของ NautilusTrader สำหรับ source code ภาษา Rust, Cargo manifest และ test

NautilusTrader ใช้ Rust สำหรับ core ที่มีความสำคัญสูงสุด (mission-critical) เพราะภาษานี้ผสานระบบ type ที่แข็งแรง,
ownership model และประสิทธิภาพที่คาดการณ์ได้ไว้ด้วยกัน Rust แบบ safe ป้องกัน data race และ memory error
จำนวนมากได้ตั้งแต่ตอน compile ส่วนโค้ด `unsafe` ต้องระบุ invariant ที่ compiler ไม่สามารถตรวจสอบได้
ให้ชัดเจน

`rustfmt` และ lint ของ workspace เป็นผู้กำหนด style ทั่วไปของ Rust กฎด้านล่างนี้เป็นส่วนเสริม

## แหล่งอ้างอิงหลัก {#sources-of-truth}

| เรื่อง                       | แหล่งอ้างอิง                                        |
| -------------------------- | -------------------------------------------------- |
| การจัดรูปแบบและ import      | `rustfmt.toml`                                     |
| Lint ของ workspace          | `Cargo.toml` และ `clippy.toml`                     |
| โครงสร้างของ Cargo          | `.pre-commit-hooks/check_cargo_conventions.sh`     |
| โครงสร้างของ Rust           | `.pre-commit-hooks/check_formatting_rs.sh`         |
| Convention ของ type ใน Nautilus | `.pre-commit-hooks/check_nautilus_conventions.sh` |
| ขอบเขตของ async            | `.pre-commit-hooks/check_tokio_usage.sh`           |
| ขอบเขตของ DST              | `.pre-commit-hooks/check_dst_conventions.sh`       |
| Anyhow                     | `.pre-commit-hooks/check_anyhow_usage.sh`          |
| ชื่อของ error               | `.pre-commit-hooks/check_error_conventions.sh`     |
| Logging                    | `.pre-commit-hooks/check_logging_conventions.sh`   |
| สัญญาใน Rustdoc            | `.pre-commit-hooks/check_docs_conventions.sh`      |
| รูปแบบของ test              | `.pre-commit-hooks/check_testing_conventions.sh`   |
| การเลือก test ของ workspace  | `scripts/ci/check-workspace-test-coverage.sh`      |

เมื่อเครื่องมือไม่ได้ชี้ขาดทางเลือกใด ให้ทำตามโค้ดที่อยู่ใกล้เคียง หากต้องการเปลี่ยนไฟล์ที่ generate ขึ้น ให้เปลี่ยน
input ของ generator แล้ว generate ใหม่ แทนการแก้ไขไฟล์ที่ generate ขึ้นโดยตรง

## การตั้งค่า lint {#lint-configuration}

`Cargo.toml` กำหนดระดับของ lint ที่ใช้ร่วมกันไว้ใต้ `[workspace.lints]` แต่ละ package สืบทอดค่าเหล่านี้ด้วย
`[lints] workspace = true` ส่วน `clippy.toml` ตั้งค่าพฤติกรรมของ lint รวมถึง threshold, ข้อยกเว้นสำหรับ test
และ API ที่ห้ามใช้ แต่ไม่ได้กำหนดระดับของ lint `.cargo/config.toml` และคำสั่ง Clippy ของ repository
จะ deny warning เพื่อให้การตรวจสอบที่เปิดไว้ทำให้คำสั่งล้มเหลว

crate จะเปิดใช้การตรวจสอบแบบ pedantic ผ่าน `#![warn(clippy::pedantic)]` ใน root ของแต่ละ target ที่ migrate แล้ว
การเปิดใช้ใน library ไม่ได้เปิดใช้กับ binary, example หรือ integration test ของมันด้วย ให้ปิด pedantic ไว้
ใน target ที่ยังไม่ได้ migrate และใช้ allowance ใน source ที่แคบและมีเหตุผลกำกับสำหรับข้อยกเว้นเฉพาะจุด
รวมถึงข้อยกเว้นของ macro ที่มีเอกสารกำกับ

[Attribute ใน source](https://doc.rust-lang.org/rustc/lints/levels.html#priority-of-lint-level-sources)
จะ override flag ของ lint ใน Cargo
[priority](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section) ของ Cargo ใช้จัดลำดับ flag เหล่านั้น
และไม่สามารถคง allowance ที่ใช้ร่วมกันไว้เมื่อ source เปิดใช้ทั้ง group ได้ นอกจากนี้ Cargo ยังปฏิเสธตาราง lint
ระดับ package ที่ใช้ร่วมกับการสืบทอดจาก workspace ด้วย repository จึง generate attribute ของ allowance ที่ใช้ร่วมกันไว้หลัง
การเปิดใช้ pedantic ใน source แต่ละจุด วิธีนี้ทำให้คำสั่ง Cargo โดยตรงและการทยอยนำมาใช้ยังทำงานได้ โดยไม่ต้อง copy
ตาราง lint หรือใช้ Clippy runner ที่เขียนเอง attribute ที่ generate ขึ้นจะทำซ้ำตัวเลือกที่ใช้ร่วมกันไว้ใน source
โดยมี `Cargo.toml` เป็นนิยามที่เชื่อถือได้

หลังจากเปลี่ยน allowance ที่ใช้ร่วมกันหรือเพิ่มการเปิดใช้ ให้รัน:

```bash
python3 -B scripts/update-clippy-allowances.py
make check-clippy-allowances
```

generator ยังครอบคลุมการเปิดใช้ระดับ module ด้วย: group attribute ใน scope ชั้นในจะ override allowance
ที่สืบทอดมา ให้คง allowance ที่ generate ขึ้นไว้หลังการเปิดใช้ และ generate ใหม่แทนการแก้ไขเอง
คำสั่ง pre-commit และ Clippy ของ repository จะตรวจหาความคลาดเคลื่อน (drift) รัน
`python3 -B scripts/ci/test_clippy_allowances.py` สำหรับ regression probe ของ compiler ที่แยกออกมา

`make check-code` ตรวจสอบ target ของ library และ test ส่วน `scripts/clippy-changed.sh` ตรวจสอบ binary ด้วย
`make check-all-targets` รวม example และ bench ด้วย gate เหล่านี้ยังคง `-D warnings` ไว้
`make clippy-pedantic-crate-<package>` เปิด pedantic ให้ target ของ package หนึ่ง และต่อท้าย flag ของ
allowance ที่ใช้ร่วมกันจาก generator ตัวเดียวกัน
`make clippy-strict-audit` ใช้ `--force-warn` เพื่อรายงาน lint ที่เป็นตัวเลือก แม้ในจุดที่จงใจอนุญาตไว้
ผลที่พบจะไม่ทำให้ audit นั้นล้มเหลว

## โครงสร้างของ module {#module-layout}

จัดเรียง module ที่เขียนเองแต่ละตัวให้พฤติกรรมหลักปรากฏก่อนรายละเอียดของ implementation ที่เป็นส่วนสนับสนุน

### Module root และการประกาศ {#module-roots-and-declarations}

คงรูปแบบของ module root เดิมไว้เมื่อเพิ่ม module ลูก ในแต่ละ block การประกาศที่ต่อเนื่องกันในไฟล์ `mod.rs`
ให้เรียงการประกาศ module ที่อยู่นอกไฟล์ตามหมวดต่อไปนี้:

1. module ที่มี `#[macro_use]`
1. module แบบ public
1. module แบบจำกัดการมองเห็น เช่น `pub(crate)`
1. module ที่ไม่ใช่ test แต่ถูกกั้นด้วย `#[cfg(...)]`
1. module แบบ private
1. module สำหรับ test เท่านั้น

เรียงการประกาศตามตัวอักษรภายในแต่ละหมวด และเว้นบรรทัดว่างหนึ่งบรรทัดระหว่างหมวด
formatting hook บังคับลำดับนี้ในไฟล์ `mod.rs`

ใช้ visibility ที่แคบที่สุดที่ยังรองรับผู้เรียกได้ workspace จะ deny item แบบ `pub` ที่เข้าถึงไม่ได้

### Import {#imports}

ให้ import อยู่ที่ด้านบนของไฟล์หรือ module ใช้ import เฉพาะที่ (local import) ก็ต่อเมื่อ scope ที่แคบ
ช่วยให้อ่านเข้าใจง่ายขึ้นอย่างมีนัยสำคัญ

### ค่าคงที่และ global state {#constants-and-global-state}

จัดกลุ่มค่าคงที่และ global state ระดับ module ไว้ใกล้ด้านบนของ module ให้ item แบบ `const` อยู่ก่อน
การประกาศ `static` และ `thread_local!` ส่วนค่าคงที่หรือ static ที่ใช้เฉพาะจุดให้วางไว้ข้างผู้ใช้งานแทน

### Type หลักและ implementation {#primary-types-and-implementations}

ให้ type หลักและ inherent implementation ของมันอยู่ใกล้ด้านบนของ module

### Enum {#enums}

วาง enum ตามบทบาทแทนการรวมไว้ใน block เดียวทั้ง module:

- ให้ enum หลักหรือ enum แบบ public อยู่กับ type หลักของ module
- ให้ enum ของสถานะขนาดเล็กอยู่ข้าง global ที่มันแทนสถานะ
- วาง enum แบบ private ที่เป็นส่วนสนับสนุนไว้ใต้ผู้เรียกตัวแรก

### นิยามที่เป็นส่วนสนับสนุน {#supporting-definitions}

วางฟังก์ชันและ type แบบ private ไว้ใต้ผู้เรียก ใน adapter ให้วาง route type แบบ private,
enum ที่ใช้ตัดสินใจ และฟังก์ชัน parse ไว้ใต้ implementation หลักของ client

### Comment แบบกล่อง (banner) {#box-style-banner-comments}

อย่าเพิ่ม comment แบบกล่องหรือเส้นคั่นเพื่อแบ่งเนื้อหาของ module ให้ใช้ module และ
implementation block ในการแสดงโครงสร้าง header ลิขสิทธิ์และ license มาตรฐานเป็นข้อยกเว้น

## Cargo manifest {#cargo-manifests}

### Dependency และ section {#dependencies-and-sections}

- ใช้การสืบทอดจาก workspace สำหรับ dependency ที่ใช้ร่วมกัน เช่น
  `serde = { workspace = true }` ปักเวอร์ชันใน crate ก็ต่อเมื่อ dependency นั้นไม่ได้
  จัดการโดย workspace
- คั่นกลุ่ม dependency ด้วยบรรทัดว่างและเรียงแต่ละกลุ่มตามตัวอักษร โดยปกติ manifest จะจัดกลุ่ม
  crate `nautilus-*` ภายใน, crate ภายนอกที่จำเป็น และ crate ภายนอกแบบ optional แต่ให้คงกลุ่มเฉพาะ
  ที่มีความหมายของ manifest นั้นไว้
- ให้ crate แบบ optional อยู่ในกลุ่มของตัวเองที่คั่นด้วยบรรทัดว่าง ยกเว้นเมื่อทุก crate ในกลุ่มเป็น
  crate `nautilus-*`
- คงลำดับ section มาตรฐาน:

  1. Package
  1. Lints
  1. Library
  1. Features
  1. metadata ของ `cargo-machete`
  1. metadata ของ docs.rs
  1. Dependencies
  1. Development dependencies
  1. Build dependencies
  1. Benches
  1. Binaries
  1. Examples
  1. Tests

- เพิ่ม `[lints] workspace = true` ให้ทุก crate ใน workspace ที่มี target แบบ library หรือ binary
- เก็บ dependency ของ adapter ไว้ใน section `# Adapter dependencies` ของ `Cargo.toml` ระดับ workspace
  crate หลักต้องไม่ depend on รายการจาก section นั้น
- รักษาความเข้ากันได้ของตระกูล dependency ที่เกี่ยวข้องกัน Cargo convention hook ตรวจสอบข้อจำกัด
  ที่ทราบ รวมถึง `arrow` กับ `parquet` และ
  `dydx-proto` กับ `prost` และ `tonic`
- ระบุเฉพาะ dependency ที่ประกาศไว้ใต้ `[package.metadata.cargo-machete] ignored`
- ลบรายการใน `[workspace.dependencies]` ระดับ root เมื่อไม่มี crate ใดใช้ Cargo tool ที่เก็บไว้สำหรับ CI
  เท่านั้นได้รับการยกเว้นจากการตรวจสอบนี้
- ลบฟิลด์ใน `[workspace.package]` ระดับ root เมื่อไม่มี crate ใดสืบทอด
- ระบุแต่ละรายการใน `[workspace] members` เป็น path ตรงตัว convention hook จะ resolve
  manifest ของสมาชิกโดยตรงและไม่ขยาย glob ของสมาชิกใน Cargo
- ให้ crate ของ adapter ได้ `libfuzzer-sys` ผ่าน `nautilus-live` อย่าเพิ่มโดยตรงลงใน manifest
  ของ adapter

### ฟิลด์ของ package {#package-fields}

section `[package]` ของ crate ใช้ prefix มาตรฐานนี้ Cargo จะอนุมาน `README.md` ที่อยู่ข้าง
manifest เอง ดังนั้นให้ละ `readme` ไว้ ส่วนฟิลด์อื่นที่แสดงเป็นฟิลด์ที่จำเป็น

```toml
[package]
name = "nautilus-example"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
description = "Example crate for NautilusTrader"
categories.workspace = true
keywords.workspace = true
documentation.workspace = true
repository.workspace = true
homepage.workspace = true
```

วางฟิลด์ optional `publish`, `build` และ `include` ไว้หลัง `homepage.workspace`

### Feature {#features}

- คงสัญญาของ default feature เดิมของ crate ไว้ crate หลักส่วนใหญ่มี default ว่าง
  ในขณะที่ crate ของ adapter จำนวนมากเปิด `high-precision` เป็นค่าเริ่มต้น
- ส่งต่อ `high-precision` ไปยัง crate ของ Nautilus ที่ depend on กันซึ่งเก็บหรือสร้างค่า fixed-point
  ของโดเมน
- จัดทำเอกสารของ feature แบบ public ที่ไม่ใช่ค่าเริ่มต้นแต่ละตัวเพียงครั้งเดียว เรียงตามตัวอักษร ใต้ `## Feature flags` ใน
  README ของ crate และ `# Feature Flags` ในเอกสารระดับ crate ของ library

### Target {#targets}

- ใช้ชื่อไฟล์แบบ snake_case สำหรับ source ใน `bin/` และชื่อ executable แบบ kebab-case เช่น
  `path = "bin/ws_data.rs"` และ `name = "hyperliquid-ws-data"`
- ตั้ง `doc = false` บน target แบบ binary และ example และตั้ง `test = false` บน target แบบ binary ด้วย
- เรียง crate type ของ library เป็น `rlib`, `staticlib` แล้วจึง `cdylib` เมื่อ crate สร้างมากกว่า
  หนึ่ง type

## ข้อกำหนดของ header ไฟล์ {#file-header-requirements}

copy header ลิขสิทธิ์และ license มาตรฐานจากไฟล์ Rust ที่เขียนเองซึ่งอยู่ใกล้เคียง ไฟล์ที่ generate ขึ้น
ให้คง header ของ generator ไว้แทน copyright hook จะตรวจสอบปี

เปลี่ยน input ของ generator แล้วรัน generator ใหม่ แทนการแก้ไข Rust, C header
หรือ doc comment ของ wrapper ที่ generate ขึ้น

## การจัดรูปแบบและ attribute {#formatting-and-attributes}

`rustfmt` จัดกลุ่ม import ของ standard library, crate ภายนอก และของภายใน แล้วเรียงแต่ละกลุ่มตามตัวอักษร
ให้รัน formatter แทนการเรียง import ด้วยมือ

เว้นบรรทัดว่างหนึ่งบรรทัด:

- ระหว่างฟังก์ชัน รวมถึง test
- เหนือ doc comment `///` หรือ `//!` แต่ละอัน
- เหนือ expression `if`, `match`, `for`, `while` และ `loop` ที่อยู่เดี่ยวๆ
- เหนือการเรียก spawn task รวมถึง `spawn_local` และ `spawn_blocking`
- ก่อน `let` ที่มี braced expression หลายบรรทัด เว้นแต่จะเป็นบรรทัดแรกของ block
- ก่อน struct literal หลายบรรทัดหรือการเรียก `Type::new(...)` แบบระบุ path โดยตรง เว้นแต่จะเป็นบรรทัดแรกของ block
- หลัง statement ที่มี braced expression หลายบรรทัด ก่อน statement ถัดไปใน block เดียวกัน

กฎของ control flow และ spawn ไม่ใช้เมื่อ expression เป็นบรรทัดแรกของ block, เป็นการต่อเนื่องจาก
การดำเนินการก่อนหน้า หรือมี comment หรือ attribute ติดอยู่ การเรียกที่ขึ้นบรรทัดใหม่และ method chain เพียงอย่างเดียว
ไม่จำเป็นต้องมีตัวคั่น

ระยะห่างของการสร้าง (construction) ใช้กับการ assign, การ return อย่างชัดเจน และการเรียกที่ตามด้วย `?` หรือ `.await` ด้วย
การสร้างต้องกินพื้นที่หลายบรรทัดหลังจัด layout ด้วย rustfmt ส่วน initializer บรรทัดเดียวให้อยู่ติดกัน

คง comment และ attribute ให้ติดกับ statement ของมันเมื่อแทรกตัวคั่น ข้ามเนื้อหาของ macro และ
พื้นที่ `rustfmt::skip` ใช้กฎของ statement หลายบรรทัดรอบโค้ดที่เปลี่ยนแปลง ปล่อยโค้ดที่ไม่เกี่ยวข้องไว้ตามเดิม
เพิ่มเฉพาะบรรทัดว่างที่ `rustfmt` คงไว้

formatting hook:

- ตรวจสอบกฎของ control flow, spawn และการเรียง module ที่มีอยู่ตรงขอบเขตที่เปลี่ยนแปลง
- เปรียบเทียบการเปลี่ยนแปลงทั้งที่ staged และ unstaged กับ `HEAD` หรือกับ merge base เมื่อตั้ง `CHANGED_BASE_SHA` ไว้
- ตรวจสอบไฟล์ Rust ที่ track ไว้ทั้งหมดใต้ `crates`, `examples` และ `docs` เมื่อ resolve base ที่ตั้งค่าไว้ไม่ได้
- อ่านไฟล์ที่เปลี่ยนแปลงทั้งไฟล์เพื่อใช้เป็นบริบท และรวมบรรทัดที่ใช้ในการตรวจสอบข้อยกเว้นเมื่อเลือก diagnostic
- ไม่แก้ไขไฟล์

`make pre-commit` ตรวจสอบการเปลี่ยนแปลงของ Rust ทั้งที่ commit, staged และ unstaged เทียบกับ merge base กับ
ref `origin/develop` ในเครื่อง แล้วจึงรัน prek หากผ่านการตรวจสอบรูปแบบ ตั้ง `CHANGED_BASE_SHA` เพื่อเลือก
base อื่น ค่าเริ่มต้นนี้ใช้กับการตรวจสอบรูปแบบนี้เท่านั้น hook ตอน commit และการรัน
`prek run --all-files` โดยตรงจะเปรียบเทียบกับ `HEAD` เว้นแต่จะตั้ง `CHANGED_BASE_SHA` ไว้

ข้อต่อไปนี้ยังคงเป็น convention สำหรับการ review:

- ระยะห่างของ statement หลายบรรทัด
- ระยะห่างของการสร้าง
- ระยะห่างของ `spawn_local` และ `spawn_blocking`

ทำให้โค้ดที่เปลี่ยนแปลงอ่านง่ายสำหรับคน หากตัวคั่นเหล่านั้นยังทำให้ block ที่หนาแน่นอ่านยาก
ให้เพิ่มบรรทัดว่างอีก

ใช้ inline format argument สำหรับตัวแปรที่มีอยู่:

```rust
anyhow::bail!("Failed to subtract {n} months from {datetime}");
```

หากการทิ้งค่าที่คืนมานั้นแทบจะแน่นอนว่าเป็นความผิดพลาด ให้เพิ่ม `#[must_use]` ให้ฟังก์ชันเหล่านี้:

- constructor
- accessor
- การแปลงแบบ pure
- method `with_*` ที่ consume ตัวเอง

return type อย่าง `Result` มี annotation `must_use` ของตัวเองอยู่แล้ว

เมื่อ suppress `missing_panics_doc` หรือ `missing_errors_doc` ให้ใส่ reason ที่อธิบายว่าทำไม
lint นั้นไม่เกี่ยวข้อง:

```rust
#[allow(
    clippy::missing_errors_doc,
    reason = "result type is retained for API compatibility but no error is returned",
)]
```

ใช้ `try_update` แทน `fetch_update` ที่ deprecated แล้วบน atomic ของ standard library pre-commit hook
จะปฏิเสธ `fetch_update`

## การระบุ path ของ type {#type-qualification}

| Item                          | Convention                                                                       |
| ----------------------------- | -------------------------------------------------------------------------------- |
| `anyhow`                      | import เฉพาะ `anyhow::Context` ส่วน macro และ `Result` ให้ระบุ path เต็ม             |
| Domain type ของ Nautilus       | import type แล้วใช้ชื่อสั้น                                                         |
| time, sync และ task ของ Tokio   | ระบุ path เต็ม ใช้ `std::time::Duration`                                          |
| `Debug` และ `Display`          | import trait                                                                     |
| `Formatter` และ `fmt::Result`  | ใช้ `std::fmt::Formatter` และ `std::fmt::Result` ที่ implementation                 |
| Macro ของ Nautilus             | import `nautilus_actor!` หรือ `nautilus_strategy!` แล้วเรียกโดยไม่ต้องระบุ path        |

ใช้ `debug_struct(stringify!(TypeName))` ใน implementation ของ `Debug` ที่เขียนเอง

ใช้ `// nautilus-import-ok` เฉพาะจุดที่ macro, path ที่ generate ขึ้น หรือ import แบบมีเงื่อนไขต้องการ
type ของ Nautilus แบบระบุ path เต็ม ให้ใส่ marker ไว้บนบรรทัดที่ได้รับผลกระทบหรือเหนือ block ที่แคบนั้นโดยตรง

## การจัดการ error และสัญญา {#error-handling-and-contracts}

### Design by contract {#design-by-contract}

ใช้กลไกที่แคบที่สุดที่แสดงสัญญานั้นได้:

| สถานการณ์                                            | กลไก                                          |
| -------------------------------------------------- | -------------------------------------------- |
| กฎของ state หรือ ownership ที่ตรวจได้ตอน compile        | type, lifetime, newtype และ visibility        |
| precondition ของ input สาธารณะ                       | `check_*` จาก `nautilus_core::correctness`   |
| การสร้างค่าที่ผ่านการตรวจสอบ                             | `new_checked()` พร้อม wrapper `new()`         |
| ความล้มเหลวของการ parse, I/O หรือเครือข่ายที่กู้คืนได้         | `Result<T, E>`                               |
| invariant ภายในที่ละได้ใน release build                 | `debug_assert!` ที่มี test เจาะจงครอบคลุม       |
| invariant ด้าน soundness หรือที่ต้องเปิดตลอดเวลา         | `assert!` หรือเส้นทาง error ที่ตรวจสอบแล้ว        |

วาง assertion ไว้ ณ จุดแรกที่โค้ดเริ่มพึ่งพา invariant นั้น ห้ามใช้ `debug_assert!` สำหรับ
การตรวจสอบ input สาธารณะหรือเงื่อนไขด้าน soundness เพราะ release build จะตัดมันออก

ขึ้นต้นข้อความของ debug assertion ด้วย `Invariant:` และระบุกฎในเชิงบวก prefix ที่ใช้ร่วมกัน
`Condition failed: ...` ใช้ระบุการละเมิด input ของผู้เรียก ส่วน `Invariant: ...` ใช้ระบุ bug ของ
สัญญาภายใน

### ขอบเขตของ error {#error-boundaries}

เลือก error type ที่ขอบเขตของ API:

| ขอบเขต                                 | Return type                          |
| ------------------------------------- | ------------------------------------ |
| API ของ library หรือโดเมนที่นำกลับมาใช้ได้   | `Result<T, E>` แบบมี type             |
| การประสานงานของ application หรือ adapter | `anyhow::Result<T>`                  |
| การตรวจสอบ input สาธารณะ               | `CorrectnessResult<T>` เมื่อเหมาะสม    |

- กำหนด error แบบมี type ด้วย `thiserror` เมื่อผู้เรียกต้องตรวจสอบหรือกู้คืนจากความล้มเหลว
- ตั้งชื่อ error pattern และ closure เป็น `e` ไม่ใช่ `err` หรือ `error`
- ใช้ `anyhow::bail!` สำหรับการ return ก่อนกำหนดจากฟังก์ชันที่คืน `anyhow::Result` ใช้
  `anyhow::anyhow!` เมื่อต้องการค่า error เช่น ภายใน `ok_or_else`
- ขึ้นต้นข้อความของ `.context()` ด้วยตัวพิมพ์เล็ก เพื่อให้ error ที่ต่อกันเป็นสายอ่านได้เป็นธรรมชาติ ให้คง
  ตัวพิมพ์ของคำนามเฉพาะหรือตัวย่อที่อยู่ต้นข้อความไว้
- อย่าใช้ `", got"` ใน error หรือ assertion ให้ใช้ `", was"`, `", received"` หรือ `", found"`
  ตามบริบท

```rust
parse_timestamp(value).context("failed to parse timestamp")?;
connect().context("Bybit websocket did not become active")?;
```

### นโยบายเรื่อง panic {#panic-policy}

panic ใช้ได้สำหรับการละเมิด invariant ภายในที่ผู้เรียกไม่สามารถกู้คืนได้อย่างสมเหตุสมผล
ให้ API เป็นแบบไม่ล้มเหลว (infallible) เมื่อการคืน `Result` จะเพียงบังคับให้ผู้เรียกต้องจัดการข้อบกพร่องของ
โปรแกรม ใช้ assertion ที่เปิดตลอดเวลาสำหรับ invariant ที่ปกป้อง soundness หรือป้องกันไม่ให้โค้ด unsafe
ทำงานต่อด้วยสถานะที่ไม่ถูกต้อง

คืน `Result` ที่ขอบเขตที่รับ input ที่ไม่น่าเชื่อถือ และคืน `Result` ที่ขอบเขตที่มีความล้มเหลว
จาก configuration, I/O, ข้อมูลภายนอก, การยกเลิก task หรือ lifecycle ของ resource ด้วย สิ่งเหล่านี้เป็นความล้มเหลว
เชิงปฏิบัติการ แม้ว่าจุดที่เรียกบางจุดจะคาดว่ามันจะไม่เกิดขึ้นเลยก็ตาม

สำหรับค่าที่ผ่านการตรวจสอบ ให้ทำตาม[รูปแบบ constructor `new()` และ `new_checked()`](#constructor-patterns)
เปิดเส้นทางที่ล้มเหลวได้สำหรับค่าที่ไม่น่าเชื่อถือ และใช้ wrapper เพื่อความสะดวกเฉพาะเมื่อสัญญาของผู้เรียก
อนุญาตให้ panic ได้ ระบุ panic ที่อาจเกิดขึ้นได้ใน Rust API สาธารณะไว้ในเอกสาร และให้ข้อความ panic แต่ละอันระบุชื่อ
invariant ที่ถูกละเมิด

ให้ถือว่าผลที่พบของ lint ต่อไปนี้จาก `make clippy-strict-audit` เป็นจุดที่ควรพิจารณาในการ review:

- `clippy::panic`
- `clippy::unwrap_used`
- `clippy::expect_used`

audit ยังรายงาน `clippy::panic_in_result_fn` ซึ่ง Clippy gate ของ workspace ที่จำเป็นบังคับใช้อยู่แล้ว
ให้ลบ panic ออกเมื่อความล้มเหลวกู้คืนได้ และคง panic ของ invariant ที่มีเหตุผลไว้ โดยใส่ reason ของ lint
แบบจำกัด scope เมื่อจำเป็น audit ใช้ warning แบบบังคับ ดังนั้นยอดรวมจะรวมจุดที่จงใจไว้ซึ่งมี allowance ของ lint
เฉพาะจุดและ diagnostic จากการขยาย macro ด้วย

### ตัวอย่างสัญญาของความล้มเหลว {#failure-contract-examples}

API ที่มีสัญญาเรื่อง panic ระบุไว้ในเอกสารจะใช้ panic สำหรับ:

- ข้อผิดพลาดของโปรแกรมเมอร์ (bug ทาง logic, การใช้ API ผิดวิธี)
- ข้อมูลที่ละเมิด invariant พื้นฐาน (timestamp ติดลบ, ราคาเป็น NaN)
- การคำนวณที่จะให้ผลลัพธ์ผิดโดยไม่มีการแจ้งเตือน

API จะคืน `Result` หรือ `Option` เมื่อผู้เรียก รวมถึง crate ปลายทาง สามารถจัดการความล้มเหลวหรือ
การไม่มีค่าได้ ซึ่งรวมถึง:

- ความล้มเหลวที่คาดได้ระหว่าง runtime (error ของเครือข่าย, file I/O)
- การตรวจสอบตาม business logic (ข้อจำกัดของ order, risk limit)
- การตรวจสอบ input ของผู้ใช้

API เป็นตัวกำหนดว่าการดำเนินการที่ไม่ถูกต้องจะล้มเหลวอย่างไร:

```rust
let later = timestamp + duration_ns; // Panics on overflow.

let price = Price::new_checked(f64::NAN, precision); // Returns Err.

let later = timestamp.checked_add(duration_ns); // Returns None on overflow.
```

นโยบายนี้ถูก implement ทั่วทั้ง core type (`UnixNanos`, `Price`, `Quantity` เป็นต้น)
และช่วยให้ NautilusTrader รักษาความถูกต้องของข้อมูลได้อย่างแข็งแรงสำหรับการเทรดใน production

release profile ของ repository ตั้ง `panic = "abort"` ไว้ ดังนั้น panic จะยุติ process เพื่อให้
supervisor หรือระบบ orchestration จัดการต่อ binary ของ Rust ปลายทางควบคุม release
profile ของตัวเอง

## Logging {#logging}

- ระบุ path เต็มของ log macro เช่น `log::debug!` และ `log::info!`
- ขึ้นต้นข้อความด้วยคำที่ขึ้นต้นด้วยตัวพิมพ์ใหญ่ และไม่ใส่จุดปิดท้าย
- จุดปิดท้ายอาจเป็นส่วนหนึ่งของค่าที่ log แทนที่จะเป็นเครื่องหมายวรรคตอน สำหรับกรณีนี้ให้ใช้
  `// log-period-ok` บนการเรียกหรือภายในสามบรรทัดเหนือการเรียก
- ให้ lifecycle ของการเชื่อมต่อและ client, การ reconnect, reconciliation และสรุป mass-status อยู่ที่ระดับ
  `INFO`
- ให้รายละเอียดของ subscription, การยืนยันราย order, จำนวน instrument, การยืนยันตัวตน และ
  รายละเอียดภายในของ WebSocket อยู่ที่ระดับ `DEBUG`
- log เป็น warning หรือ error เมื่อมีการจัดการเงื่อนไขที่ไม่คาดคิด, ผู้ใช้ต้องดำเนินการ หรือข้อมูลสูญหาย
  ภายในจุดนั้น อย่า log error ที่ยังคงถูกส่งต่อผ่าน `?` หรือ `anyhow::bail!`
- เว้นบรรทัดว่างเหนือการเรียก log เว้นแต่จะเป็นบรรทัดแรกของฟังก์ชัน
- อย่าเขียนไปยัง stdout หรือ stderr หรือยุติ process จากโค้ด library ใน production ส่วน binary,
  example, bench, test, adapter, CLI และ testkit สามารถควบคุม process ของตัวเองได้เมื่อเป็น
  ส่วนหนึ่งของบทบาท

## โค้ด async {#async-code}

crate หลักแบบ synchronous จะไม่ใช้ Tokio เป็น dependency ปกติ crate `common` ให้ Tokio
เป็น optional ส่วนรายชื่อ crate ที่แน่นอนกำหนดโดย Tokio convention hook

โค้ด production ของ adapter ใช้ runtime ที่ใช้ร่วมกัน เพราะการเรียกอาจมาจาก thread ที่ไม่มี
Tokio context ประจำ thread:

```rust
use nautilus_common::live::get_runtime;

get_runtime().spawn(async move {
    run_client().await;
});
```

- import `get_runtime` ผ่าน `nautilus_common::live` ไม่ใช่ `live::runtime`
- ใช้ `get_runtime().block_on()` เมื่อโค้ด adapter แบบ synchronous ต้องเรียกฟังก์ชัน async
- ใน test ให้ใช้ runtime ที่ `#[tokio::test]` จัดให้ ซึ่ง `tokio::spawn()` ใช้ได้ในกรณีนั้น
- บางจุดอาจไม่สามารถใช้รูปแบบ path เต็มหรือ runtime ที่ใช้ร่วมกันได้ เฉพาะจุดเหล่านั้นเท่านั้น ให้ใส่
  `// tokio-import-ok` บนบรรทัดของ import หรือ spawn
- ติดตั้ง runtime ที่กำหนดเองก่อน `LiveNode::build()` หรือก่อนใช้ adapter ใดๆ ให้สร้าง runtime แบบ
  multi-threaded ที่เปิด driver ทั้งหมด
- `set_runtime()` จะข้ามตัว initialize ค่าเริ่มต้น

โค้ดที่อยู่บนเส้นทางของ deterministic simulation ต้องทำตาม
[สัญญาด้าน determinism ของ DST](../concepts/dst.md#determinism-contract) ให้ส่ง clock, ค่าสุ่ม,
การ spawn task และการเข้าถึงเครือข่ายผ่าน seam ของโปรเจกต์ และใช้ `biased;` ใน block `tokio::select!`
บนเส้นทางนั้น

## Ownership และการเข้าถึงใน runtime {#runtime-ownership-and-access}

### การเข้าถึง order ใน cache {#cache-order-access}

`Cache` ซ่อนที่เก็บ order แบบ `SharedCell` ไว้หลัง accessor ที่ผูกกับ lifetime ให้ใช้ accessor
ที่ผูกกับ lifetime เหล่านี้:

- **อ่านแบบจำกัด scope**: `order_ref()` หรือ `try_order_ref()`
- **ยืมเพื่อเขียนแบบผูกขาด**: `order_mut()`
- **snapshot ข้ามขอบเขต**: `order_owned()` หรือ `try_order_owned()`

รูปแบบ `try_*` จะคืน `OrderLookupError` เมื่อไม่มี order นั้น

`order_mut()` ต้องใช้ `&mut Cache` ดังนั้นโค้ด `CacheView` ที่ adapter ใช้จึงแก้ไข order ไม่ได้ ก่อนการ dispatch
event หรือการยืมที่อาจกลับเข้ามาที่ order เดิมอีก (re-enter) ให้ drop `OrderRef` หรือ `OrderRefMut` ก่อน แล้วจึงค้นหา
order อีกครั้งเพื่อดูสถานะหลัง event

### Invariant ของ runtime {#runtime-invariants}

actor registry, component registry และ message bus ใช้ที่เก็บแบบ `thread_local!` object
ที่ลงทะเบียนบน thread หนึ่งจะมองไม่เห็นจาก thread อื่น `LiveNodeHandle` คือช่องทางควบคุม
ข้าม thread ที่ตั้งใจไว้

ใช้ `ActorRef` ภายใน scope แบบ synchronous เดียวเท่านั้น อย่าเก็บไว้ใน struct, ถือไว้ข้าม
`.await` หรือส่งไปยัง thread อื่น ใน callback ที่มีอายุยาว ให้เก็บ actor ID ไว้แล้วค้นหา
actor ทุกครั้งที่ callback ทำงาน

component registry จะปฏิเสธการเข้าถึงแบบ mutable ที่ซ้อนกัน (aliased) ด้วย borrow guard แบบจำกัด scope อย่าทำให้
การดำเนินการ lifecycle ของ component เป็นแบบ re-entrant

## การสร้างและการแปลง {#construction-and-conversion}

### รูปแบบ constructor {#constructor-patterns}

value type ที่ผ่านการตรวจสอบจะจับคู่ `new_checked()` ที่ล้มเหลวได้กับ `new()` เพื่อความสะดวก:

```rust
pub fn new_checked<T: AsRef<str>>(value: T) -> CorrectnessResult<Self> {
    let value = value.as_ref();
    check_valid_string_ascii(value, stringify!(value))?;
    Ok(Self(Ustr::from(value)))
}

pub fn new<T: AsRef<str>>(value: T) -> Self {
    Self::new_checked(value).expect_display(FAILED)
}
```

ใช้ค่าคงที่ `FAILED` ที่ใช้ร่วมกันกับ `CorrectnessResultExt::expect_display` เพื่อให้ข้อความ panic ใช้
prefix มาตรฐาน `Condition failed: ...` ระบุ error ในเอกสารของ `new_checked()` และ panic ในเอกสารของ
`new()`

หาก constructor ที่ยาวมีฟิลด์ optional เป็นส่วนใหญ่ ให้ใช้ builder ของ `bon` สำหรับ type เหล่านั้น อย่าเปิด
method `new()` และ `new_checked()` แบบ positional เป็น public ให้ใส่ `#[bon::bon]` บน inherent implementation และให้
finish method ของ builder มอบหมายไปยังฟังก์ชันตรวจสอบแบบ private หรือตรวจสอบโดยตรง ให้มีเส้นทางการตรวจสอบ
และการกำหนดค่าเริ่มต้นเพียงเส้นทางเดียว

```rust
#[builder(start_fn = builder, finish_fn = build)]
pub fn build_checked(/* same inputs as new_checked */) -> CorrectnessResult<Self> {
    Self::new_checked(/* forward inputs */)
}
```

ฟิลด์ที่จำเป็นยังคงจำเป็นใน typestate ของ `bon` ฟิลด์ optional ยังละได้ และ `build()`
คืน `CorrectnessResult` แบบเดียวกับเส้นทางการตรวจสอบภายใน

### รูปแบบการแปลง {#conversion-patterns}

- implement `From` เฉพาะการแปลงที่ไม่ล้มเหลวและครบถ้วนเท่านั้น
- implement `TryFrom` เมื่อ source อาจมีค่าที่ไม่ถูกต้องหรือแทนค่าไม่ได้
- implement `FromStr` สำหรับการ parse string
- ให้ถือว่า implementation `From<T: AsRef<str>>` แบบ generic ที่ panic เป็นพื้นผิวเพื่อความเข้ากันได้ อย่า
  copy รูปแบบนั้นไปใช้ใน API ใหม่
- แยก enum ของ wire ของ venue ออกจาก enum ของโดเมน Nautilus ใช้ชื่อ variant แบบ Rust ที่เป็นธรรมชาติ
  ระบุการสะกดบน wire ด้วย attribute ของ `serde` หรือ `strum` และแปลงอย่างชัดเจนที่
  ขอบเขต
- deserialize payload ของ adapter เป็น wire model ก่อนสร้าง domain object ให้การ parse
  และการตรวจสอบอยู่ในฟังก์ชันการแปลง แทนการฝังรายละเอียด wire ของ venue ไว้ใน domain type

### Type ตัวเลขของโดเมน {#domain-numeric-types}

คงค่าทางการเงินที่ไม่ต่อเนื่องไว้เป็น decimal ตั้งแต่ตอนนำเข้า:

| ค่า                                          | Type และการสร้าง                                           |
| ------------------------------------------- | -------------------------------------------------------- |
| ราคาหรือ quantity                             | `Price::from_decimal_dp` หรือ `Quantity::from_decimal_dp` |
| เงิน, ค่าธรรมเนียม, margin หรือยอดเงินคงเหลือ       | `Decimal` แล้วจึง `Money::from_decimal` หรือ `Money::zero` |
| อัตราส่วนของ signal ต่อเนื่องหรือ timing curve      | `f64` เมื่อ precision แบบ decimal ไม่มีความหมายทางโดเมน     |

อย่าส่งค่าจาก wire ผ่าน constructor แบบ `f64` ใน test ให้เปรียบเทียบ `.as_decimal()` กับ
`dec!(value)`

## การจัดเก็บ identifier {#identifier-storage}

### String interning ทำงานอย่างไร {#how-string-interning-works}

string interning จะเก็บสำเนาที่ใช้ร่วมกันหนึ่งชุดของแต่ละ string ที่ไม่ซ้ำกันไว้ใน cache ส่วนกลาง ค่าที่ซ้ำกัน
จะอ้างถึง byte ใน cache ชุดเดียวกันแทนการจัดสรรสำเนาใหม่ handle ขนาดเล็กทำให้ copy และเปรียบเทียบค่า
ได้ถูก ในขณะที่ hash ที่ cache ไว้ช่วยให้ไม่ต้องอ่าน string ทั้งหมดซ้ำระหว่างการ hash

NautilusTrader ใช้ `Ustr` สำหรับส่วนประกอบของ identifier ที่ถูก intern แต่ละ `Ustr` เป็น handle แบบ `Copy`
ขนาดเท่า pointer ที่มี hash คำนวณไว้ล่วงหน้าและเข้าถึง string ได้โดยตรงอย่างเสถียร type แบบประกอบอย่าง
`InstrumentId` คงความหมายของการ copy ที่ถูกแบบเดียวกันไว้ด้วยการเก็บ handle เหล่านี้

### ขอบเขตของการคืนหน่วยความจำ {#reclamation-boundary}

string cache เก็บทุกค่าที่ไม่ซ้ำไว้ตลอดอายุของ process การเก็บไว้แบบนี้ทำให้ handle ที่ copy ไปและ
string slice ที่คืนออกไปยังใช้ได้ โดยไม่ต้องมี reference counting, access guard หรือ lifetime parameter
แบบชัดเจนบน identifier type การยุติ process คือขอบเขตปกติของการคืนหน่วยความจำ

การรับประกันเหล่านี้ทำให้ไม่สามารถคืนหน่วยความจำของแต่ละรายการได้อย่างปลอดภัย Rust สามารถ copy ค่า `Copy`
ได้โดยไม่ต้องรันโค้ดใดๆ ดังนั้น atomic reference count จึงไม่สามารถเห็นทุกสำเนาได้ การออกแบบที่เพิ่ม
การคืนหน่วยความจำจะเปลี่ยนสัญญาของ identifier:

- reference counting ต้องใช้ `Clone` และ `Drop` ซึ่งทำให้ identifier และ type ที่มี identifier อยู่ข้างใน
  ไม่เป็น `Copy` อีกต่อไป
- ที่เก็บแบบยืมหรือแบบป้องกันด้วย epoch ต้องใช้ lifetime หรือ access guard ทุกจุดที่เข้าถึง string
- handle แบบ generational อนุญาตให้คืนหน่วยความจำได้ แต่ทำให้การค้นหาล้มเหลวได้และทำให้ handle ที่ค้างอยู่ใช้ไม่ได้
- การ reset cache ทั้งหมดจะปลอดภัยก็ต่อเมื่ออยู่ ณ จุดที่พิสูจน์แล้วว่าไม่มีการทำงานใดๆ (quiescent point) handle, reference
  และ foreign pointer ทั้งหมดต้องถูกทำลายก่อน และไม่มี task หรือ thread ใดถือไว้ได้ ณ จุดนั้น

### ขอบเขตของการจัดเก็บ {#storage-boundaries}

interning เหมาะที่สุดกับ identifier ที่มาจากชุดค่าที่มีขอบเขตภายใน process และค่าที่ซ้ำกัน
มากพอที่จะได้ประโยชน์จากการตัดค่าซ้ำ identifier ที่จำนวนค่าไม่ซ้ำอาจเพิ่มขึ้นตามทุก
order, trade หรือข้อความ จะทำให้ cache โตขึ้นตลอดอายุของ process

ที่เก็บแบบ inline ที่มีความจุคงที่ยังคง `Copy` ไว้ได้เมื่อ protocol ภายนอกกำหนดค่าสูงสุดที่เหมาะสม
ตัวอย่างเช่น `TradeId` ใช้ `StackStr` ขนาด 36 ตัวอักษร ส่วนที่เก็บแบบมีเจ้าของหรือแบบ reference-counted ให้
ความจุแบบ dynamic เมื่อการคืนหน่วยความจำสำคัญกว่า `Copy`

domain model ยังมีข้อยกเว้นเพื่อความเข้ากันได้ `ClientOrderId`, `VenueOrderId`,
`PositionId` และ `OrderListId` ยังใช้ `Ustr` เป็นที่เก็บ จึงเก็บทุกค่าที่ไม่ซ้ำไว้
การจัดเก็บ identifier เป็นส่วนหนึ่งของ C ABI แบบ by-value ที่รองรับ ดังนั้นการออกแบบใหม่ในวงกว้างกว่านี้จึงขึ้นอยู่กับ
binding แบบอาศัยการแปลงเข้ามาแทนการใช้ layout ร่วมกันโดยตรง

ขอบเขตของการจัดเก็บรวมถึงการประมาณการล่วงหน้าของแต่ละค่าที่ไม่ซ้ำและ string ระหว่างทางทั้งหมดที่ถูก intern
ระหว่างการ parse การใช้ `Ustr` ทั้งหมดใน process ใช้ cache ร่วมกัน ดังนั้นต้นทุนหน่วยความจำจึงมาจากชุดค่ารวม
ไม่ใช่งบประมาณแยกของแต่ละ identifier type

### ตัวอย่างระดับสเกลของ Polymarket {#polymarket-scale-example}

symbol ของ instrument ใน Polymarket ประกอบด้วย condition ID ขนาด 66 byte กับ token ID ขนาด 77 หรือ 78 byte รวมเป็น
symbol ที่ถูก intern ขนาด 144 หรือ 145 byte ด้วย layout ของ `ustr` 1.1.0 แบบ 64 บิต ค่า `InstrumentId` ที่ไม่ซ้ำ
600,000 ค่าต้องใช้ประมาณ 150 MiB ซึ่งครอบคลุมค่า identifier ที่เก็บไว้, ตาราง lookup ของ cache และพื้นที่
string ที่จองไว้

เส้นทางการ parse ของ Polymarket ยัง intern token ID ดิบแต่ละตัวและ condition ID แต่ละตัวด้วย สำหรับ
instrument 600,000 ตัวจากประมาณ 300,000 ตลาด รายการเหล่านี้ทำให้ค่าประมาณเพิ่มเป็นราว 300 MiB ก่อนรวม
instrument object, คำอธิบาย, map และ metadata อื่น ค่าประมาณนี้สมมติว่า instrument ID และ token ID
ไม่ซ้ำกัน และรวมความจุที่ allocator แบบ geometric ของ cache จองไว้ จึงไม่ใช่การวัด resident set
ที่แม่นยำ

NautilusTrader ยอมรับต้นทุนที่มีขอบเขตนี้เพื่อรักษา `Copy`, การเข้าถึงโดยตรงที่เสถียร และการตัดค่าซ้ำแบบ global
ทั่วทั้งชุด instrument ส่วน stream ของ external ID ที่ไม่ซ้ำแบบไม่มีขอบเขตยังคงอยู่นอก storage model นี้

## Collection {#collections}

เลือก hash collection ตามความหมายของการวนซ้ำ (iteration) และขอบเขตความน่าเชื่อถือ:

| ความต้องการ                                     | Collection                 |
| ---------------------------------------------- | -------------------------- |
| การวนซ้ำตามลำดับการแทรกที่สังเกตเห็นได้               | `IndexMap` หรือ `IndexSet` |
| lookup ที่ใช้บ่อยโดยไม่มีลำดับการวนซ้ำที่สังเกตเห็นได้      | `AHashMap` หรือ `AHashSet` |
| key ที่บุคคลที่สามที่ไม่น่าเชื่อถือเป็นผู้เลือก                | `HashMap` หรือ `HashSet`   |
| API ภายนอกต้องการ collection มาตรฐาน               | `HashMap` หรือ `HashSet`   |

instance ของ `AHashMap` และ `AHashSet` แบบค่าเริ่มต้นจะสุ่ม hasher state ดังนั้นลำดับการวนซ้ำจึง
ไม่คงที่ หากลำดับการวนซ้ำส่งผลต่อผลลัพธ์ใดต่อไปนี้ ให้ทำให้ลำดับนั้นเป็นแบบ deterministic:

- event ที่ส่งออก
- คำสั่งและ traffic ที่ส่งไปยัง venue
- output ที่จัดเก็บถาวร
- การใช้ตัวเลขสุ่ม

การวนซ้ำที่ไม่มีผู้เรียกคนใดสังเกตลำดับไม่จำเป็นต้องทำให้คงที่

ใช้ `IndexMap` หรือ `IndexSet` เมื่อลำดับการแทรกเป็นทั้งแบบ deterministic และเป็นลำดับที่ต้องการ มิฉะนั้น
ให้คง hash collection ไว้แล้วเรียงลำดับ ณ จุดที่ใช้งาน ควรเลือกการเรียงลำดับเมื่อการไล่ตามลำดับเกิดขึ้นไม่บ่อย หรือ
การลบแบบรักษาลำดับเกิดขึ้นบ่อย `shift_remove` รักษาลำดับสัมพัทธ์ในเวลาเฉลี่ย O(n) ส่วน `swap_remove`
ทำงานในเวลาเฉลี่ย O(1) แต่อาจย้ายรายการสุดท้าย

`AHash` ไม่ปลอดภัยในเชิง cryptography ให้ใช้ `HashMap` หรือ `HashSet` ในจุดที่ key ที่ไม่น่าเชื่อถือทำให้
ความทนทานต่อ hash flooding เป็นส่วนหนึ่งของขอบเขตความปลอดภัย

## เอกสาร {#documentation}

### ความครอบคลุมและน้ำเสียง {#coverage-and-tone}

- เพิ่ม doc comment ให้ item แบบ public ส่วนพฤติกรรมแบบ private ให้อธิบายด้วย comment ปกติเฉพาะเมื่อ
  บริบทที่ไม่ชัดเจนอาจทำให้เข้าใจผิดได้
- เพิ่มเอกสารระดับ module ให้ module แบบ public และ module ที่มีสัญญาที่ไม่ชัดเจน อย่าเพิ่ม
  ข้อความสำเร็จรูป (boilerplate) ให้ module ปลายทางแบบ private
- ใช้รูปประโยคบอกเล่า (indicative mood): "Returns the account ID" ไม่ใช่ "Return the account ID"
- ปิดท้ายเอกสารของฟิลด์ public และ variant ของ enum ด้วยจุด
- ให้ความหนาแน่นของเอกสารสอดคล้องกับ item ข้างเคียง อย่าเพิ่ม comment ที่ไม่มีเนื้อหาเพียงเพื่อให้ block
  ที่ไม่มีเอกสารดูเป็นแบบเดียวกัน
- ใส่บริบทสำคัญของฟิลด์ private ไว้ในเอกสารระดับ type แทนการเขียนเอกสาร
  ให้แต่ละฟิลด์

เขียนเอกสารให้ฟังก์ชัน public เมื่อสัญญาของมันไม่ชัดเจนจาก type และชื่อทั้งหมด ให้ครอบคลุม
เงื่อนไขที่ล้มเหลวได้, เงื่อนไขที่ panic, หน้าที่ด้านความปลอดภัย และความหมายของ input ที่ไม่ชัดเจน

### Section ใน Rustdoc {#rustdoc-sections}

ใช้ Title Case สำหรับหัวข้อ section ใน Rustdoc:

- `# Examples`
- `# Errors`
- `# Panics`
- `# Safety`
- `# Notes`
- `# Thread Safety`
- `# Feature Flags`

ขึ้นต้นคำอธิบายใน `# Errors` ด้วย `Returns an error if` หรือ ``Returns `None` if`` สำหรับ return แบบ `Option`
ขึ้นต้นคำอธิบายใน `# Panics` ด้วย `Panics if` อธิบายเงื่อนไขของความล้มเหลวโดยไม่ต้อง
ซ้ำ error type ที่เป็นรูปธรรมจาก signature ให้คงความแตกต่างที่ส่งผลต่อพฤติกรรมของผู้เรียกไว้
เช่น ความล้มเหลวที่ retry ได้เทียบกับความล้มเหลวถาวร

ใช้ประโยคเดียวที่ปิดท้ายด้วยจุดสำหรับเงื่อนไขเดียว:

```rust
/// # Errors
///
/// Returns an error if the currency conversion fails.
```

สำหรับหลายเงื่อนไข ให้ใช้คำขึ้นต้นเดียวกันตามด้วย colon ทันที:
`Returns an error if:`, ``Returns `None` if:`` หรือ `Panics if:` ใส่แต่ละเงื่อนไขเป็น bullet ที่
ขึ้นต้นด้วยตัวพิมพ์ใหญ่และปิดท้ายด้วยจุด bullet ที่ขึ้นบรรทัดใหม่ต้องมีจุดที่ท้าย
บรรทัดสุดท้ายของมัน

```rust
/// # Errors
///
/// Returns an error if:
/// - The market price cannot be found.
/// - The conversion rate calculation fails.
```

```rust
/// # Panics
///
/// Panics if:
/// - The quantity is zero.
/// - The quantity exceeds the supported range.
```

documentation hook ตรวจสอบคำขึ้นต้นและจุดท้าย bullet เหล่านี้ใน section `# Errors` และ `# Panics` โดย
ข้าม code block แบบ fenced และหยุดที่หัวข้อ section ถัดไปหรือท้าย doc block marker สำหรับ suppress
ด้านล่างยกเว้นเฉพาะการตรวจสอบสัญญาเท่านั้น ไม่ได้ยกเว้นการตรวจสอบถ้อยคำหรือเครื่องหมายวรรคตอน

ใช้ `# Errors` เฉพาะกับฟังก์ชันที่คืน `Result` หรือ `Option` ใช้ `# Panics` เฉพาะเมื่อ
ฟังก์ชันสามารถ panic ได้ หากฟังก์ชัน panic ไม่ได้ ให้ลบ section นั้นออก อย่าเขียนระบุว่ามันไม่ panic

documentation hook จะรู้จักจุด panic โดยตรงในฟังก์ชันที่คืน `Result` ให้ใส่
`// panics-doc-ok` ไว้เหนือ doc block ทันทีเมื่อฟังก์ชันที่ถูกเรียกเป็นผู้ทำให้เกิด panic ที่ระบุไว้ในเอกสาร
ใช้ `// errors-doc-ok` ในตำแหน่งเดียวกันเฉพาะสำหรับสัญญา error พิเศษที่
การตรวจสอบ signature ไม่สามารถรู้จักได้

### ตัวอย่างในเอกสาร {#doc-examples}

`make cargo-test-doc` จะ compile และรันตัวอย่างในเอกสาร ให้ระบุ annotation ให้ทุก fence:

| Fence          | พฤติกรรม                   | ใช้สำหรับ                                                  |
| -------------- | ------------------------- | -------------------------------------------------------- |
| `rust`         | compile และรัน             | ตัวอย่างที่สมบูรณ์ในตัวโดยไม่มี dependency ภายนอก                |
| `rust,no_run`  | compile แต่ไม่รัน           | ตัวอย่างที่ต้องใช้ catalog, เครือข่าย หรือ venue                 |
| `ignore`       | ไม่ compile และไม่รัน        | pseudocode ให้ระบุเหตุผลที่ compile ไม่ได้                       |
| `compile_fail` | ต้อง compile ไม่ผ่าน          | การสาธิตการใช้งานที่ถูกปฏิเสธ                                  |
| `text`         | ไม่ใช่โค้ด                   | directory tree, output หรือแผนภาพ                          |
| `bash`, `json` | ไม่ใช่ Rust                 | คำสั่งหรือ payload                                          |

ควรใช้ `no_run` แทน `ignore` เมื่อสิ่งที่ขาดไปมีเพียง dependency ตอน runtime ใส่ `#` นำหน้าบรรทัดที่ใช้ตั้งค่า
เมื่อบรรทัดเหล่านั้นต้อง compile ได้แต่จะบดบังตัวอย่างที่แสดงผล

ใส่ตัวอย่างไว้บน item แบบ public Rustdoc รวบรวม fence จากเอกสารของ item แบบ private ด้วย แต่ตัวอย่างเหล่านั้น
ไม่สามารถ import item ที่ตัวเองอธิบายได้

## Convention ของการทดสอบ {#testing-conventions}

- ใช้ `mod tests` สำหรับ test แบบ inline ทั่วไป
- ใช้ `#[rstest]` แทน `#[test]` รวมถึง test ที่ไม่มีพารามิเตอร์
- ใช้ `#[tokio::test]` สำหรับ test แบบ async ที่ไม่มีพารามิเตอร์
- คง `#[cfg(test)]` ไว้บน test module และไฟล์สำหรับ test เท่านั้น อย่าเพิ่มพฤติกรรมหรือ interface สำหรับ test ลงใน
  โค้ด production
- เก็บ JSON fixture ไว้ใต้ไดเรกทอรี `test_data/` ของ crate และโหลดด้วย `include_str!`
- ใช้ input ที่แตกต่างกันและไม่ใช่ค่าเริ่มต้น พร้อมค่าที่คาดหวังแบบแน่นอน assert ทุกฟิลด์ที่คงที่
- จัดกลุ่ม assertion ไว้หลังการตั้งค่าและการกระทำ เว้นแต่ test จะตรวจสอบการเปลี่ยนแปลงสถานะทีละขั้น
- อย่าใช้ comment คั่นแบบ Arrange, Act, Assert

สมาชิกแต่ละตัวของ workspace ปรากฏเพียงครั้งเดียวใน `CORE_CRATES`, `ADAPTER_CRATES` หรือ `NO_TEST_CRATES` ใน
`Makefile` เพิ่ม crate เข้า `NO_TEST_CRATES` ก็ต่อเมื่อเป็นไปตามทั้งสองเงื่อนไข:

- ไม่ได้กำหนด test target ของ Rust ใดเลย
- ไม่มีฟังก์ชัน `#[test]`, `#[rstest]` หรือ `#[test_case]`

ใช้ parametrize เมื่อพฤติกรรมเดียวกันใช้กับหลาย input:

```rust
#[rstest]
#[case("AUDUSD", false)]
#[case("CL.FUT", true)]
fn test_symbol_is_composite(#[case] input: &str, #[case] expected: bool) {
    assert_eq!(Symbol::new(input).is_composite(), expected);
}
```

### Test spec {#test-specs}

event ที่มี argument ของ constructor จำนวนมากจะใช้ spec แบบ fluent ของ `bon` ที่อยู่ข้าง event ใต้
`events/order/spec/` ให้กั้น module ด้วย `#[cfg(any(test, feature = "test-support"))]` เพื่อให้ test
ปลายทางเลือกใช้ได้โดยไม่ต้องเพิ่ม spec ลงใน production build

- derive `bon::Builder` ด้วย `finish_fn = into_spec`
- ให้ฟิลด์ที่จำเป็นมีค่าเริ่มต้นที่ถูกต้องและ deterministic ส่วนฟิลด์ optional ให้คงเป็น `Option<T>`
- ใช้ `test_uuid()` จาก `crate::stubs` เป็นค่าเริ่มต้นของ event ID
- implement `build()` โดยส่งต่อผ่าน constructor ของ production
- คืน event โดยตรงเพราะค่าเริ่มต้นของ spec ถูกต้องตั้งแต่การสร้าง
- ตรึงค่าเริ่มต้นทุกค่าไว้ใน test เดียวใน module ของ spec

override เฉพาะฟิลด์ที่เกี่ยวข้องกับผู้เรียก:

```rust
let fill = OrderFilledSpec::builder()
    .last_qty(Quantity::from(50_000))
    .trade_id(TradeId::from("TRADE-1"))
    .build();
```

เมื่อใช้ `cargo test` ธรรมดา ให้เรียก `reset_test_uuid_rng()` ก่อน test ที่เปรียบเทียบลำดับของ UUID
`cargo nextest` เริ่มแต่ละ test ใน process ใหม่ ลำดับจึง reset ให้อัตโนมัติ

### Property-based test {#property-based-tests}

ใช้ `proptest` เมื่อ invariant ครอบคลุมกลุ่ม input ที่ตัวอย่างไม่สามารถครอบคลุมได้ ให้ strategy อยู่
ใกล้ property suite และผสาน range กับ edge case ที่ระบุชัดเจน

ขึ้นต้นชื่อ property ด้วย `prop_` และคง `#[rstest]` ไว้บน test ภายใน `proptest!` suite ขนาดใหญ่
อาจใช้ module `property_tests` หรือไฟล์เฉพาะ แต่ repository ก็ยังเก็บ property ที่เจาะจง
ไว้ข้าง unit test ของมันด้วย

## Unsafe Rust {#unsafe-rust}

โค้ด unsafe ต้องทำให้หน้าที่ในการพิสูจน์ (proof obligation) ของมันตรวจสอบ (review) ได้:

- ให้ฟังก์ชัน unsafe แต่ละตัวมี section `# Safety` ที่ระบุหน้าที่ของผู้เรียกอย่างครบถ้วน
- ใส่ comment `SAFETY:` ไว้เหนือการดำเนินการ unsafe แต่ละอันโดยตรง และอธิบายว่าทำไม precondition
  จึงเป็นจริง ณ จุดนั้น
- ห่อการดำเนินการ unsafe แต่ละอันด้วย block `unsafe { ... }` ของตัวเอง crate ใน workspace จะ deny
  `unsafe_op_in_unsafe_fn`
- ใช้การตรวจสอบที่เปิดตลอดเวลาสำหรับ null, alignment, provenance และเงื่อนไขด้าน soundness อื่นๆ
- เพิ่ม test ที่เจาะจงสำหรับพฤติกรรมที่สังเกตได้รอบโค้ด unsafe test สนับสนุนการพิสูจน์แต่ไม่ได้
  ยืนยัน soundness
- implementation `Send` หรือ `Sync` แบบ unsafe ต้องมีการพิสูจน์ครอบคลุมพฤติกรรมทั้งหมดที่เข้าถึงได้:

  - state
  - alias
  - callback
  - generic parameter
  - safe method
  - การ clone
  - การทำลาย (destruction)

สำหรับ raw vector ที่ข้าม FFI ให้ทำตาม[สัญญาด้านหน่วยความจำของ FFI](ffi.md) โค้ดฝั่ง foreign เป็นเจ้าของ
allocation หลังการโอน และต้องเรียกฟังก์ชัน `vec_drop_*` ที่ตรงกันเพียงครั้งเดียวเท่านั้น

## Artifact อื่นที่ generate ขึ้น {#other-generated-artifacts}

### FFI binding และ precision {#ffi-bindings-and-precision}

มีเพียง `nautilus-core` และ `nautilus-model` ที่มี feature `ffi` ให้ตรวจสอบทั้งสอง crate โดยตรงเมื่อ
เปลี่ยน C ABI ของมัน:

```bash
cargo check -q -p nautilus-core --features ffi
cargo check -q -p nautilus-model --features ffi,python,high-precision
```

ไฟล์ `cbindgen.toml` ใน crate กำหนด layout ของ header สำหรับผู้ใช้แบบ native อย่าเพิ่ม feature
`ffi` หรือ module `src/ffi` ให้ crate อื่นใน workspace

รัน `make check-cbindgen-abi` คำสั่งนี้ generate header ทั้งสองไฟล์และตรวจสอบชื่อ public และค่า enum
เพื่อความเข้ากันได้ รวมถึง compile ผู้ใช้ภาษา C11 กับ header เหล่านั้น test ที่ตั้งเวลาไว้ทุกคืน (nightly) จะรันการตรวจสอบนี้
