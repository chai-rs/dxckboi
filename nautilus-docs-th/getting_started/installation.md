# การติดตั้ง

NautilusTrader รองรับอย่างเป็นทางการบนแพลตฟอร์ม 64 บิตต่อไปนี้:

| ระบบปฏิบัติการ      | เวอร์ชันที่รองรับ     | สถาปัตยกรรม CPU |
| ---------------- | ------------------ | ---------------- |
| Linux (Ubuntu)   | 22.04 and later    | x86_64           |
| Linux (Ubuntu)   | 22.04 and later    | ARM64            |
| macOS            | 15.0 and later     | ARM64            |
| Windows Server   | 2022 and later     | x86_64           |

:::note
NautilusTrader อาจทำงานได้บนแพลตฟอร์มอื่น แต่มีเฉพาะแพลตฟอร์มข้างต้นที่นักพัฒนาใช้งานเป็นประจำและทดสอบใน CI
:::

CI ที่รันต่อเนื่องมาจาก GitHub Actions runner ที่โปรเจกต์ใช้ build:

- build ของ `Linux (Ubuntu)` ปัจจุบัน pin ไว้ที่ `ubuntu-22.04` เพื่อคงความเข้ากันได้กับ glibc 2.35 แม้ `ubuntu-latest` จะขยับไปข้างหน้าแล้ว
- build ของ `macOS (ARM64)` รันบน `macos-latest` การรองรับจึงขยับตาม image ของ runner นั้น
- build ของ `Windows (x86_64)` ปัจจุบัน pin ไว้ที่ `windows-2022` เพื่อให้ toolchain คงที่

บน Linux ให้ตรวจเวอร์ชัน glibc ด้วย `ldd --version` และตรวจให้แน่ใจว่าเป็น 2.35 หรือใหม่กว่าก่อนดำเนินการต่อ

**การใช้งานจาก Rust**: crate ของ Nautilus เผยแพร่อยู่บน crates.io และเพิ่มลงใน `Cargo.toml` ได้โดยตรง
ดูรายการ crate, feature flag และเวอร์ชัน Rust ขั้นต่ำ (MSRV) ได้ที่หัวข้อ
[การตั้งค่าโปรเจกต์](../concepts/rust.md#project-setup) ส่วนหัวข้อด้านล่างอธิบายการเตรียม toolchain และการ build จาก source

## ตรวจสอบที่มาของ build {#verifying-build-provenance}

artifact ทุกตัวที่โปรเจกต์เผยแพร่มี cryptographic attestation ที่สร้างโดย CI/CD pipeline:

- Docker image (`ghcr.io/nautechsystems/nautilus_trader`): ลายเซ็น [cosign](https://github.com/sigstore/cosign) แบบ keyless พร้อม SPDX SBOM attestation

attestation ออกผ่าน [Sigstore](https://www.sigstore.dev/) และผูกกับ commit SHA ที่เจาะจง การตรวจสอบจึงยืนยันได้ว่า
artifact ถูกสร้างโดย GitHub Actions workflow ทางการของ NautilusTrader และไม่ถูกแก้ไขหลังจากนั้น

ดูคำสั่งตรวจสอบทีละขั้นตอนได้ที่ [Verifying releases](https://github.com/nautechsystems/nautilus_trader/blob/develop/SECURITY.md#verifying-releases) ใน `SECURITY.md`

:::info
การตรวจสอบ Docker image ต้องใช้ [cosign](https://github.com/sigstore/cosign)
:::

## ติดตั้งจาก source {#from-source}

### 1. ติดตั้ง rustup {#1-install-rustup}

ติดตั้ง [rustup](https://rustup.rs/) (ตัวติดตั้ง Rust toolchain):

```bash tab="Linux/macOS"
curl https://sh.rustup.rs -sSf | sh
```

```powershell tab="Windows"
# Download and install rustup-init.exe from https://win.rustup.rs/x86_64
# Also install "Desktop development with C++" via Build Tools for Visual Studio 2022
```

ตรวจสอบ: `rustc --version`

### 2. เปิดใช้ cargo {#2-enable-cargo}

เปิดใช้ `cargo` ใน shell ปัจจุบัน:

```bash tab="Linux/macOS"
source $HOME/.cargo/env
```

```powershell tab="Windows"
# Start a new PowerShell session
```

### 3. ติดตั้ง clang {#3-install-clang}

ติดตั้ง [clang](https://clang.llvm.org/) (frontend ภาษา C สำหรับ LLVM) บน Linux คำสั่งนี้จะติดตั้ง
[lld](https://lld.llvm.org/) ด้วย ซึ่งถูกตั้งค่าให้เป็น linker ของ Rust เพื่อให้ build เร็วขึ้น:

```bash tab="Linux"
sudo apt-get install clang lld
```

```powershell tab="Windows"
# 1. Add Clang via Visual Studio Installer:
#    Modify > C++ Clang tools for Windows (latest) > Modify
# 2. Add to PATH:
[System.Environment]::SetEnvironmentVariable('path', "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\Llvm\x64\bin\;" + $env:Path,"User")
```

ตรวจสอบ: `clang --version`

### 4. Clone source {#5-clone-and-sync-dependencies}

clone source ด้วย `git`:

```bash
git clone --branch develop --depth 1 https://github.com/nautechsystems/nautilus_trader
cd nautilus_trader
```

สำหรับเครื่องที่ใช้พัฒนาและ image ของ CI runner ให้ดู
[แหล่งอ้างอิงเวอร์ชันเพียงแหล่งเดียว](../developer_guide/environment_setup.md#single-source-of-truth-for-versions)
ก่อนติดตั้งเครื่องมือที่ pin เวอร์ชันไว้

:::note
flag `--depth 1` ดึงเฉพาะ commit ล่าสุด ทำให้ clone ได้เร็วและเบากว่า
:::

### 5. ติดตั้ง Cap'n Proto สำหรับ HyperSync {#6-install-capn-proto-for-hypersync}

compiler ของ [Cap'n Proto](https://capnproto.org) ยังมีไว้สำหรับการพัฒนา schema ของ HyperSync
dependency ของ HyperSync ที่ pin ไว้มาพร้อม Rust binding ที่ generate ไว้แล้ว build ปกติจึงไม่ต้อง generate schema ใหม่
บน Linux หรือ macOS ให้ใช้ script ใน repository เพื่อติดตั้ง compiler ตามเวอร์ชันที่ pin ไว้ใน
`.nautilus-engineering/tools.toml`:

```bash
./scripts/install-capnp.sh
```

ตรวจสอบ: `capnp --version`

:::note
Cap'n Proto เป็น development dependency
:::

ขั้นตอนถัดไป: ดูแนวทางการเขียนและทดสอบโค้ด Rust ของโปรเจกต์ได้ที่ [Rust](../developer_guide/rust.md)
ในคู่มือนักพัฒนา หรือเริ่มเขียนโปรเจกต์ของตัวเองตาม [รัน backtest (Rust)](../how_to/run_rust_backtest.md)
และ [Write a Strategy (Rust)](../how_to/write_rust_strategy.md)

## เวอร์ชันและการ release {#versioning-and-releases}

NautilusTrader ยังอยู่ระหว่างการพัฒนาอย่างต่อเนื่อง บาง feature อาจยังไม่สมบูรณ์ และแม้ API จะเริ่มเสถียรขึ้น
แต่ก็อาจมี breaking change ระหว่าง release ได้ โปรเจกต์พยายามบันทึกการเปลี่ยนแปลงเหล่านี้ไว้ใน release notes
แบบ **best-effort**

โปรเจกต์ตั้งเป้า **release ทุกสองสัปดาห์** แม้ feature ทดลองหรือ feature ขนาดใหญ่อาจทำให้ล่าช้าได้

ใช้ NautilusTrader เฉพาะเมื่อคุณพร้อมจะปรับตัวตามการเปลี่ยนแปลงเหล่านี้

## Redis {#redis}

การใช้ [Redis](https://redis.io) กับ NautilusTrader เป็น**ทางเลือก** และจำเป็นเฉพาะเมื่อตั้งค่าให้เป็น backend
ของ cache database หรือ [message bus](../concepts/message_bus.md)

:::info
Redis เวอร์ชันต่ำสุดที่รองรับคือ 6.2 (จำเป็นสำหรับความสามารถ [streams](https://redis.io/docs/latest/develop/data-types/streams/))
:::

สำหรับการตั้งค่าอย่างรวดเร็ว แนะนำให้ใช้ [Redis Docker container](https://hub.docker.com/_/redis/) ดูตัวอย่างการตั้งค่าได้ใน
directory `.docker` หรือรันคำสั่งต่อไปนี้เพื่อเริ่ม container:

```bash
docker run -d --name redis -p 6379:6379 redis:latest
```

คำสั่งนี้จะ:

- ดึง Redis เวอร์ชันล่าสุดจาก Docker Hub ถ้ายังไม่ได้ดาวน์โหลด
- รัน container ในโหมด detached (`-d`)
- ตั้งชื่อ container ว่า `redis` เพื่อให้อ้างถึงได้ง่าย
- เปิด Redis ที่ port มาตรฐาน 6379 ให้ NautilusTrader บนเครื่องเข้าถึงได้

การจัดการ Redis container:

- เริ่มด้วย `docker start redis`
- หยุดด้วย `docker stop redis`

:::tip
แนะนำให้ใช้ [Redis Insight](https://redis.io/insight/) เป็น GUI เพื่อดูและ debug ข้อมูลใน Redis อย่างมีประสิทธิภาพ
:::

## Precision mode {#precision-mode}

NautilusTrader รองรับ precision mode สองแบบสำหรับ value type หลัก (`Price`, `Quantity`, `Money`)
ซึ่งต่างกันที่ความกว้างบิตภายในและจำนวนทศนิยมสูงสุด

- **High-precision**: integer 128 บิต ทศนิยมได้สูงสุด 16 ตำแหน่ง และช่วงค่ากว้างกว่า
- **Standard-precision**: integer 64 บิต ทศนิยมได้สูงสุด 9 ตำแหน่ง และช่วงค่าแคบกว่า

:::note
สำหรับ crate แบบ Rust ล้วน high-precision ทำงานได้บนทุกแพลตฟอร์ม (รวมถึง Windows) เพราะ Rust จัดการ
`i128`/`u128` ด้วยการจำลองระดับ software ค่าเริ่มต้นคือ standard-precision เว้นแต่จะเปิด feature flag
`high-precision` อย่างชัดเจน
:::

:::note
benchmark เปรียบเทียบประสิทธิภาพของทั้งสองโหมดยังอยู่ระหว่างจัดทำ
:::

### การตั้งค่า build {#build-configuration}

precision mode ถูกเลือกตอน compile ผ่าน Rust feature flag `high-precision`

### Rust feature flag {#rust-feature-flag}

หากต้องการเปิดโหมด high-precision (128 บิต) ใน Rust ให้เพิ่ม feature `high-precision` ลงใน `Cargo.toml`:

```toml
[dependencies]
nautilus-model = { version = "*", features = ["high-precision"] }
```

:::info
ดูข้อกำหนดของ [Value Types](../concepts/overview.md#value-types) สำหรับรายละเอียดเพิ่มเติม
:::
