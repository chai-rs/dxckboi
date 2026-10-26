# Environment Setup

ใช้ editor ที่รองรับภาษา Rust เวอร์ชันปัจจุบัน เช่น Visual Studio Code

[prek](https://github.com/j178/prek) ใช้รัน pre-commit check, auto-formatter และ linting tool ต่าง ๆ โดยอัตโนมัติตอน commit

การ build จาก source และ Rust crate ต้องใช้ [Rust](https://www.rust-lang.org)
([คู่มือการติดตั้ง](https://www.rust-lang.org/tools/install))

เครื่องมือ [Cap'n Proto](https://capnproto.org) ยังคงไว้สำหรับการพัฒนา HyperSync เวอร์ชันที่ pin ไว้ระบุอยู่ใน
`.nautilus-engineering/tools.toml` package default ของ Ubuntu มักเก่าเกินไป จึงอาจต้องติดตั้งจาก source (ดูด้านล่าง)

:::info
NautilusTrader *ต้อง* compile และรันได้บน **Linux, macOS และ Windows** โปรดคำนึงถึง portability เสมอ: ใช้
`std::path::Path` ในโค้ด และทำตาม [shell portability policy](shell.md#define-the-portability-target) สำหรับ script
:::

## การตั้งค่า {#setup}

ขั้นตอนต่อไปนี้สำหรับระบบแบบ UNIX และทำเพียงครั้งเดียว

### ตั้งค่าแบบเร็ว {#quick-setup}

ใช้เส้นทางนี้เป็นการตั้งค่าแบบกระชับสำหรับเครื่องพัฒนา Linux หรือ macOS ใหม่ หัวข้อละเอียดด้านล่างอธิบายแต่ละขั้นตอน
และทางเลือกอื่น

ติดตั้งเครื่องมือของ platform ก่อน:

```bash tab="Ubuntu"
sudo apt-get update
sudo apt-get install -y build-essential clang lld curl git make pkg-config
```

```bash tab="macOS"
xcode-select --install
```

จากนั้น clone repository และติดตั้งเครื่องมือของโปรเจกต์ที่ pin เวอร์ชันไว้:

```bash
git clone --branch develop https://github.com/nautechsystems/nautilus_trader
cd nautilus_trader

curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"

cargo install cargo-binstall --locked
make install-tools
./scripts/install-capnp.sh

prek install
```

ผู้ใช้ Windows ควรทำตามขั้นตอนการติดตั้งจาก source ใน
[คู่มือการติดตั้ง](../getting_started/installation.md#from-source) แล้วใช้คำสั่งที่เกี่ยวข้องจากคู่มือนี้

### 1. ติดตั้ง dependency {#1-install-dependencies}

ทำตาม[คู่มือการติดตั้ง](../getting_started/installation.md)

### 2. ติดตั้งเครื่องมือพัฒนา {#2-install-development-tools}

NautilusTrader pin เวอร์ชันของเครื่องมือพัฒนาทุกตัว เพื่อให้ contributor ทุกคนและ CI ใช้เวอร์ชันเดียวกัน
Makefile target เดียวติดตั้งได้ครบชุด:

```bash
make install-tools
```

สิ่งที่ติดตั้ง:

- **Shared Cargo CLI** ที่ pin ไว้ใน `.nautilus-engineering/tools.toml`: `cargo-audit`, `cargo-deny`,
  `cargo-edit`, `cargo-llvm-cov`, `cargo-nextest` และ `cargo-vet`
- **NautilusTrader Cargo CLI** ที่ pin ไว้ใน `Cargo.toml` ใต้ `[workspace.metadata.tools]`:
  `cargo-codspeed`, `cargo-fuzz`, `cargo-hawk`, `cargo-machete`, `cbindgen`, `flamegraph` และ `lychee`
- **NautilusTrader Cargo CLI** ที่ pin ไว้ใน `tools.toml`: `tokei` (ตัวนับบรรทัด)
- **Prebuilt binary** ที่ pin ไว้ใน `.nautilus-engineering/tools.toml`: `prek` (ตัวรัน pre-commit) และ
  `osv-scanner` (ตัวสแกนช่องโหว่)
- **uv** ที่เวอร์ชัน pin ร่วม

Cap'n Proto ก็ pin ไว้ใน `.nautilus-engineering/tools.toml` เช่นกัน แต่ติดตั้งแยก ดูหัวข้อ
[Cap'n Proto](#capn-proto) ด้านล่าง

fuzz target ยังต้องใช้ Rust nightly toolchain ตอนรัน เพราะ `cargo-fuzz` ใช้ `libfuzzer-sys` และ compiler flag ที่ยัง
unstable:

```bash
rustup toolchain install nightly
```

การตรวจความเข้ากันได้กับ docs.rs ใช้ nightly แบบระบุวันที่ที่ pin ไว้ใน `tools.toml`:

```bash
rustup toolchain install "$(bash scripts/tool-version.sh nightly)" --profile minimal
```

#### สิ่งที่ต้องติดตั้งครั้งเดียว: cargo-binstall {#one-off-prerequisite-cargo-binstall}

`make install-tools` ใช้ [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall) ดึง `prek` มาเป็น
prebuilt binary แทนการ compile จาก source ติดตั้ง `cargo-binstall` ครั้งเดียวต่อเครื่อง:

```bash
cargo install cargo-binstall --locked
```

ขั้นตอนนี้ทำครั้งเดียว การรัน `make install-tools` ครั้งต่อไปจะใช้ `cargo-binstall` ที่ติดตั้งไว้แล้ว

#### แหล่งความจริงเดียวของเวอร์ชัน {#single-source-of-truth-for-versions}

manifest ของ repository คือแหล่งอ้างอิงหลักของเวอร์ชัน dependency และเครื่องมือ ถ้ามีวิธีอ่านเวอร์ชันปัจจุบันจาก
manifest ได้ อย่าคัดลอกเลขเวอร์ชันไปใส่ใน docs, runner image หรือ script

| ไฟล์หรือ section ต้นทาง                       | นิยามอะไร                                                  |
| ----------------------------------------- | -------------------------------------------------------- |
| `rust-toolchain.toml`                     | Rust toolchain                                           |
| `Cargo.toml` และ `Cargo.lock`              | dependency ของ Rust workspace และการ resolve ที่แน่นอน         |
| `Cargo.toml` `[workspace.metadata.tools]` | Cargo tool เฉพาะของ NautilusTrader                         |
| `.nautilus-engineering/tools.toml`        | เครื่องมือ engineering ที่ใช้ร่วมกัน                              |
| `tools.toml`                              | เครื่องมือเฉพาะของ NautilusTrader ที่ไม่มี native manifest        |

catalog ที่ใช้ร่วมกันประกอบด้วย uv, `prek`, `pip-audit`, `osv-scanner`, Cap'n Proto และ Cargo tool ทั่วไป ส่วน
catalog ภายในเก็บ pin ของ docs.rs nightly, Miri toolchain และ `pypi-attestations`

Makefile อ่านค่าเหล่านี้ผ่าน `scripts/cargo-tool-version.sh`, `scripts/tool-version.sh` และ
`scripts/uv-version.sh` ดังนั้นการเปลี่ยนเวอร์ชันในไฟล์ต้นทางจึงเป็นการเปลี่ยนเวอร์ชันเพียงจุดเดียวที่ต้องทำ ตรวจ
เวอร์ชัน cargo tool ที่ pin ไว้เทียบกับ crates.io ได้ด้วย:

```bash
make outdated
```

### 3. ตั้งค่า Git hook {#3-set-up-git-hooks}

ตั้งค่า hook ของไฟล์และ commit message ซึ่งจะรันอัตโนมัติตอน commit:

```bash
prek install
```

ให้รัน `prek install` ใหม่หลัง pull การเปลี่ยนแปลงที่กระทบชนิดของ hook ที่ตั้งค่าไว้

hook ของ Dockerfile ติดตั้ง Hadolint แบบ native ใน environment แยกของมันเอง ไม่ต้องใช้ Docker

ก่อนเปิด pull request ให้รันชุด formatting และ lint ในเครื่อง เพื่อให้ CI ผ่านตั้งแต่ครั้งแรก:

```bash
make format
make pre-commit
```

ตรวจให้แน่ใจว่า Rust compiler รายงาน **error เป็นศูนย์** -- build ที่พังทำให้ทุกคนช้าลง

## การจัดการ dependency {#dependency-management}

### Rust dependency cooldown {#rust-dependency-cooldown}

การตรวจ cooldown ของ Cargo จะปฏิเสธเวอร์ชันจาก registry ที่ resolve แล้วซึ่ง publish ภายในช่วงเวลาที่ตั้งโดย
`[workspace.metadata.cooldown]` ใน `Cargo.toml` โดยตรวจทุกไฟล์ `Cargo.lock` ที่ track ไว้ รวมถึงเวอร์ชันที่ commit
แล้วหรือ pull มาจาก branch อื่น ก่อนที่ build script หรือ procedural macro ของ dependency จะได้รัน

การตรวจรันที่จุดต่อไปนี้:

- **Build**: ทุก Make target ที่ compile Rust ได้ depend on `check-cargo-cooldown` ครอบคลุม target สำหรับ build,
  stub, check, Clippy, test, coverage, documentation, benchmark, Docker และการติดตั้ง CLI ในเครื่อง การ compile
  จะรอการตรวจ แม้ใช้ Make แบบ parallel
- **Pre-commit**: hook `cargo-cooldown` จะรันเมื่อ commit มีการ stage ไฟล์ `Cargo.toml` หรือ `Cargo.lock`,
  `.supply-chain/crate-dates.json` หรือ `scripts/check-cargo-cooldown.sh`
- **CI**: job pre-commit รัน `prek run --all-files` และ job ที่ build ผ่าน Make target จะรันการตรวจก่อน
- **ตามต้องการ**: `make check-cargo-cooldown` รันการตรวจแบบเต็มชุดเดียวกัน
- **การอัปเดต dependency**: `make cargo-update` จะคืนเวอร์ชันก่อนหน้าของการ upgrade ใหม่แต่ละตัวที่ไม่อนุญาต

Make target ที่ compile ยังส่ง `--locked` ด้วย ดังนั้น Cargo จะล้มเหลวแทนที่จะ resolve เวอร์ชันที่การตรวจยังไม่เคยเห็น

เวอร์ชันที่อยู่ในช่วง cooldown ต้องมีทั้ง entry ที่ตรงกันพอดีใน `[workspace.metadata.cooldown.allow]` และ cargo-vet
audit ที่ตรงกัน registry ที่ไม่รองรับจะทำให้การตรวจล้มเหลว วันที่ publish มาจาก database ที่ commit ไว้ที่
`.supply-chain/crate-dates.json` วันที่ที่บันทึกไว้ถือว่าเชื่อถือได้แบบ offline เวอร์ชันที่ไม่มีใน database จะถูก
ค้นหาบน crates.io และจะล้มเหลวแบบ fail closed เมื่อติดต่อ registry ไม่ได้ Git diff ที่สะอาดไม่ได้พิสูจน์ว่า dependency
เก่าพอ

branch หนึ่งอาจเพิ่ม entry ใน database ที่ย้อนวันที่ไว้ข้างเวอร์ชันที่มันนำเข้ามา การตั้งค่า
`trusted-base = "origin/develop"` ปิดช่องโหว่นี้: การตรวจจะยืนยันซ้ำกับ crates.io สำหรับแต่ละ entry ใน database ที่
`origin/develop` ไม่มี แล้ว cache ผลที่ผ่าน branch จะส่ง request เหล่านั้นซ้ำเฉพาะเมื่อตัวมันเองหรือ `origin/develop`
เปลี่ยน ไม่ใช่ทุกครั้งที่ build

- **Clone ที่ไม่มี `origin/develop`**: การตรวจจะล้มเหลวจนกว่าคุณจะ fetch มา
- **CI checkout แบบ shallow**: การตรวจ resolve trusted base ไม่ได้ จึงเชื่อวันที่ที่บันทึกไว้แทน
- **Job pre-commit แบบ full history**: การตรวจยืนยันกับ trusted base

`make cargo-update` บันทึกวันที่ของแต่ละการเปลี่ยนแปลงที่ยอมรับ หลังแก้ lockfile ด้วยมือ ให้รัน
`bash scripts/check-cargo-cooldown.sh --update-db` เพื่อ reconcile database ซึ่งจะลบ entry ที่ไม่มี lock ที่ track ไว้
resolve ถึงด้วย คำสั่งอัปเดต dependency จะยืนยันวันที่ที่เพิ่มใหม่กับ crates.io แยกต่างหาก ส่วน pre-commit และการตรวจ
แบบเต็มตามปกติใช้ database ที่ commit ไว้แบบ offline

hook pre-commit และ `make check-cargo-cooldown` cache ผลการตรวจแบบเต็มที่สำเร็จไว้เป็น `.cargo-cooldown.json` ใน
`CARGO_TARGET_DIR` เมื่อไม่ได้ตั้ง Cargo target directory hook จะใช้ `target/` ที่ root ของ repository และ Make
ใช้ `TARGET_DIR` ของมัน CI ใช้ Cargo target directory ที่ตั้งค่าไว้ เพื่อให้ runner แบบถาวรเก็บ cache ไว้ข้าม job

การเปลี่ยนแปลงของ lockfile ที่ตรวจ, policy, audit, database, check script หรือ database และ lockfile ของ trusted base
จะทำให้ cache ใช้ไม่ได้ การตรวจที่ล้มเหลวจะไม่ถูก cache ให้ถือว่าไฟล์นี้เป็น state การตรวจสอบในเครื่อง อย่า restore
จากแหล่งที่ไม่น่าเชื่อถือ

**ข้อจำกัดของ gate**: cooldown ลดความเสี่ยงจาก release ที่เป็นอันตรายซึ่งเพิ่ง publish บน registry แต่ไม่ได้พิสูจน์ว่า
release ที่เก่ากว่าปลอดภัย ไม่ได้ sandbox build script และไม่ได้ตรวจ dependency แบบ Git และ local path คำสั่ง bootstrap
ของเครื่องมือพัฒนาอย่าง `make install-tools` ติดตั้ง package ภายนอกด้วยการ resolve dependency แยกต่างหาก และอยู่นอก
gate ของ lockfile ใน repository นี้

การเรียก Cargo โดยตรงจะข้าม Make ไป editor integration อย่าง rust-analyzer อาจรัน build script ทันทีที่ lockfile
เปลี่ยน ก่อนเปิด branch ที่ไม่น่าเชื่อถือใน editor ให้รัน `make check-cargo-cooldown` ส่ง `--locked` เมื่อ build โค้ด
ของ repository โดยตรง และอย่าเปลี่ยน manifest หรือ lockfile ระหว่างการตรวจกับการ compile

## การ build {#builds}

Nautilus CLI แบบ standalone เป็น build artifact แยกต่างหาก ดู
[คู่มือนักพัฒนา Nautilus CLI](#nautilus-cli-developer-guide) เมื่อต้องการแก้ไขหรือใช้ CLI

### Refresh หลัง pull การเปลี่ยนแปลง {#refresh-after-pulling-changes}

ใช้คำสั่งที่อัปเดต artifact ที่ได้รับผลกระทบ

| input ที่เปลี่ยน                                       | คำสั่ง                        | artifact ที่อัปเดต                                     |
| --------------------------------------------------- | ---------------------------- | ------------------------------------------------- |
| โค้ด CLI, โค้ด SQL initialization หรือ `schema/sql`   | `make install-cli`           | binary `nautilus` แบบ standalone ใน bin path ของ Cargo |
| pin ของ Cargo, uv, `prek` หรือ OSV Scanner           | `make install-tools`         | เครื่องมือพัฒนาที่ pin ไว้                               |
| เวอร์ชัน Cap'n Proto ใน catalog ที่ใช้ร่วมกัน               | `./scripts/install-capnp.sh` | Cap'n Proto compiler                              |

## Cap'n Proto {#capn-proto}

เครื่องมือ [Cap'n Proto](https://capnproto.org) ยังมีให้ใช้สำหรับการพัฒนา HyperSync dependency ของ HyperSync
ที่ pin ไว้มาพร้อม Rust binding ที่ generate แล้ว ดังนั้น build ปกติจะไม่ generate schema ใหม่ NautilusTrader ไม่มี
Cap'n Proto codec ให้ เวอร์ชันที่ต้องใช้นิยามไว้ใน `.nautilus-engineering/tools.toml`

ติดตั้งเวอร์ชันที่ถูกต้องสำหรับ platform ของคุณ:

```bash tab="Script (Linux/macOS)"
./scripts/install-capnp.sh
```

```bash tab="macOS (Homebrew)"
brew install capnp
```

```bash tab="Linux (source)"
CAPNP_VERSION=$(bash scripts/tool-version.sh capnp)
cd ~
wget https://capnproto.org/capnproto-c++-${CAPNP_VERSION}.tar.gz
tar xzf capnproto-c++-${CAPNP_VERSION}.tar.gz
cd capnproto-c++-${CAPNP_VERSION}
./configure
make -j$(nproc)
sudo make install
sudo ldconfig
```

```bash tab="Windows (Chocolatey)"
choco install capnproto
```

ตรวจว่าเวอร์ชันที่ติดตั้งตรงกับ catalog ที่ใช้ร่วมกัน:

```bash
capnp --version
```

install script จะรับประกันว่าเวอร์ชันที่ pin ไว้ถูกติดตั้ง ถ้า Homebrew หรือ Chocolatey ให้เวอร์ชันที่เก่ากว่า ให้ติดตั้ง
จาก source หรือดู [คู่มือการติดตั้ง Cap'n Proto](https://capnproto.org/install.html)

## Build ให้เร็วขึ้น {#faster-builds}

codegen backend Cranelift ช่วยลดเวลา build ในเครื่องสำหรับการพัฒนา, test และการตรวจของ IDE ได้ ต้องใช้ Rust
nightly toolchain และแก้ `Cargo.toml` ในเครื่อง:

```bash
rustup toolchain install nightly --component rust-analyzer
```

บันทึก patch ด้านล่าง แล้ว apply ด้วย `git apply <patch>` และถอดออกด้วย `git apply -R <patch>` ก่อน push

:::warning
ห้าม commit การเปลี่ยนแปลงเหล่านี้ patch cranelift มีไว้สำหรับการพัฒนาในเครื่องเท่านั้น และจะทำให้ CI พังถ้า push ขึ้นไป
:::

```diff
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -1,3 +1,5 @@
+cargo-features = ["codegen-backend"]
+
 [workspace]
 resolver = "2"
 members = [
@@ -424,6 +426,7 @@
 lto = false
 panic = "unwind"
 incremental = true
+codegen-backend = "cranelift"

 # Compile third-party deps at opt-level=1 in dev/test profiles. Workspace
 # members keep opt-level=0 (fast iteration); deps recompile rarely so the
@@ -444,6 +447,7 @@
 strip = false
 lto = false
 incremental = true
+codegen-backend = "cranelift"

 [profile.test.package."*"]
 opt-level = 1
@@ -452,6 +456,7 @@
 inherits = "test"
 debug = false # Improves compile times
 strip = "debuginfo" # Improves compile times
+codegen-backend = "cranelift"

 [profile.ci-pr]
 inherits = "test"
```

รันคำสั่ง build ในเครื่องด้วย `RUSTUP_TOOLCHAIN=nightly` เช่น:

```bash
RUSTUP_TOOLCHAIN=nightly make build-debug
```

ตั้ง toolchain เดียวกันใน [การตั้งค่า rust-analyzer](#rust-analyzer-settings) ด้วยเมื่อใช้ patch ในเครื่องนี้

## Services {#services}

ไฟล์ Compose pin image ของ PostgreSQL 18, Redis และ pgAdmin ก่อนเริ่ม PostgreSQL 18 กับ volume ที่มีอยู่จาก major
version ก่อนหน้า ให้ backup database และ migrate ด้วย
[dump/restore หรือ `pg_upgrade`](https://www.postgresql.org/docs/18/upgrading.html) การเปลี่ยน image ไม่ได้ upgrade
ข้อมูลที่เก็บไว้

สำหรับการ migrate จาก Debian ไป Alpine ให้ backup และ dump/restore database แม้ major version ของ PostgreSQL
จะไม่เปลี่ยน เพราะ image ทั้งสองใช้กฎ collation ของ libc ต่างกัน การใช้ data directory เดิมซ้ำอาจทำให้ index ใช้ไม่ได้

initialize PostgreSQL, Redis และ pgAdmin จาก root ของ repository:

```bash
make init-services
```

คำสั่งนี้จะเริ่ม container และ initialize database schema ของ NautilusTrader ถ้าต้องการเริ่ม container โดยไม่
initialize schema ใหม่ ให้รัน `make start-services` ถ้าต้องการเริ่ม service เดียว ให้ใช้ไฟล์ Compose โดยตรง:

```bash
docker compose -f .docker/docker-compose.yml up -d postgres
```

service สำหรับการพัฒนาได้แก่:

- `postgres`: PostgreSQL ที่โดย default มี `POSTGRES_USER=nautilus`, `POSTGRES_PASSWORD=pass` และ
  `POSTGRES_DB=nautilus`
- `redis`: Redis server
- `pgadmin`: pgAdmin 4 สำหรับจัดการและดูแล database

:::info
โปรดใช้สิ่งนี้เป็น environment สำหรับการพัฒนาเท่านั้น สำหรับ production ให้ใช้การตั้งค่าที่เหมาะสมและปลอดภัยกว่านี้
:::

ใช้ `make stop-services` เพื่อหยุด container โดยไม่ลบข้อมูล ใช้ `make purge-services` เฉพาะเมื่อตั้งใจจะลบ volume
ของการพัฒนา

test ที่ใช้ PostgreSQL แต่ละตัวอาจเปิดหลาย connection บน workstation ที่มี core จำนวนมาก concurrency ของ nextest
ในเครื่องอาจเกินขีดจำกัด connection ของ container สำหรับการพัฒนา ให้ใช้ profile ของ CI เพื่อให้ concurrency ต่ำเท่ากับ CI:

```bash
NEXTEST_PROFILE=ci make cargo-test-extras
```

ถ้าต้องการคง parallelism ในเครื่องไว้มากกว่านั้น ให้ตั้งจำนวน worker แบบมีขอบเขตอย่างชัดเจน เช่น:

```bash
NEXTEST_TEST_THREADS=8 make cargo-test-extras
```

## คู่มือนักพัฒนา Nautilus CLI {#nautilus-cli-developer-guide}

Nautilus CLI คือ Rust binary แบบ standalone สำหรับดูแล PostgreSQL และงานอื่น ๆ ของ repository

### Build และเลือก CLI {#build-and-select-the-cli}

ติดตั้ง CLI จาก checkout ปัจจุบันด้วย:

```bash
make install-cli
```

target นี้รัน `cargo install --locked --force` และวาง `nautilus` ไว้ในไดเรกทอรี binary ของ Cargo ซึ่งปกติคือ
`~/.cargo/bin` ให้ติดตั้งใหม่หลัง pull การเปลี่ยนแปลงใน `crates/cli`, โค้ด SQL initialization หรือ `schema/sql`
มิฉะนั้น CLI ที่ติดตั้งไว้อาจเก่ากว่า checkout ขณะที่อ่านไฟล์ schema ที่ใหม่กว่าจาก checkout นั้น

ก่อนรันคำสั่งที่ขึ้นกับ repository ให้ตรวจว่า shell resolve ไปที่ binary ตัวไหนและเป็นเวอร์ชันใด:

```bash
command -v nautilus
nautilus --version
```

ถ้าต้องการ build และรัน CLI จาก checkout โดยตรงโดยไม่แทนที่ binary ที่ติดตั้งไว้ ให้ใช้:

```bash
cargo run --locked --package nautilus-cli --bin nautilus -- --help
```

:::warning
บนระบบ Linux ที่ใช้ GNOME `/usr/bin/nautilus` ปกติคือ file manager ของ GNOME ให้เลือก CLI ของ NautilusTrader
ด้วยวิธีใดวิธีหนึ่งต่อไปนี้:

- วาง `~/.cargo/bin` ไว้ก่อน `/usr/bin` ใน `PATH`
- รัน `~/.cargo/bin/nautilus` โดยระบุ path ตรง ๆ
- เพิ่ม `alias nautilus="$HOME/.cargo/bin/nautilus"` ลงในการตั้งค่า shell

:::

การติดตั้งจาก source บน Windows ต้องใช้ GNU Make ผ่าน MSYS2 หรือ WSL workflow แบบ nightly ยัง publish archive
ของ CLI สำหรับ Windows x86-64 ด้วย

รัน `nautilus --help` เพื่อดูกลุ่มคำสั่งที่มี

### คำสั่ง database {#database-commands}

คำสั่ง database รับการตั้งค่า connection เป็น argument บน command line หรือผ่านไฟล์ `.env` ใน working directory
ปัจจุบันหรือไดเรกทอรีแม่ CLI ยังรับ environment variable ที่ตรงกันด้วย

| Flag         | Environment variable | หน้าที่                                                           |
| ------------ | -------------------- | ------------------------------------------------------------- |
| `--host`     | `POSTGRES_HOST`      | host ของ database                                             |
| `--port`     | `POSTGRES_PORT`      | port ของ database                                             |
| `--username` | `POSTGRES_USERNAME`  | administrator ที่ใช้เชื่อมต่อ ปกติคือ role `postgres`               |
| `--password` | `POSTGRES_PASSWORD`  | password ของ administrator และ password ของ application role   |
| `--database` | `POSTGRES_DATABASE`  | ชื่อ database และ application role ที่สร้างระหว่าง `init`          |
| `--schema`   | `SCHEMA_DIR`         | ไดเรกทอรีที่มีไฟล์ SQL schema                                    |

ตัวอย่าง:

```dotenv
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_USERNAME=postgres
POSTGRES_PASSWORD=pass
POSTGRES_DATABASE=nautilus
```

`nautilus database init` สร้างหรืออัปเดต role และ schema จากไฟล์ SQL ให้ระบุไดเรกทอรี schema อย่างชัดเจน เพื่อให้
clone ที่เปลี่ยนชื่อและ worktree ไม่ต้องพึ่งการตรวจหา path ของ checkout:

```bash
nautilus database init --schema "$PWD/schema/sql"
```

ให้ใช้ CLI ที่ build จาก checkout เดียวกับไฟล์ schema เหล่านี้ การ initialize ออกแบบให้รันซ้ำได้ รวมถึงหลังจากการรัน
ครั้งก่อนหยุดกลางคัน

`nautilus database assign-account` assign account event ที่ persist ไว้ก่อนมี trader-scoped caching ให้กับ trader
Postgres cache จะปฏิเสธการเชื่อมต่อขณะที่ยังมี event แบบนั้นอยู่ ดังนั้นให้รันคำสั่งนี้กับทุก account ที่แสดงใน error นั้น:

```bash
nautilus database assign-account --account-id SIM-001 --trader-id TRADER-001
```

:::danger
`nautilus database drop` จะลบ schema, privilege, role และข้อมูลที่เก็บไว้ของเป้าหมาย ใช้เฉพาะกับ database ที่ทิ้งได้
หรือหลังยืนยันแล้วว่าลบข้อมูลได้
:::

รัน `nautilus database --help` เพื่อดู syntax ของคำสั่งทั้งหมด

## การตั้งค่า Rust analyzer {#rust-analyzer-settings}

Rust analyzer เป็น language server ที่นิยมสำหรับ Rust และทำงานร่วมกับ IDE ได้หลายตัว ตัวอย่างด้านล่างครอบคลุม
VS Code และ AstroNvim สำหรับการตั้งค่าอื่น ดู
[rust-analyzer configuration](https://rust-analyzer.github.io/book/configuration.html)

```json tab="VSCode"
{
    "rust-analyzer.restartServerOnConfigChange": true,
    "rust-analyzer.linkedProjects": [
        "Cargo.toml"
    ],
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.check.workspace": false,
    "rust-analyzer.check.extraEnv": {
        "VIRTUAL_ENV": "<path-to-nautilus-trader>/python/.venv",
        "CC": "clang",
        "CXX": "clang++"
    },
    "rust-analyzer.cargo.extraEnv": {
        "VIRTUAL_ENV": "<path-to-nautilus-trader>/python/.venv",
        "CC": "clang",
        "CXX": "clang++"
    },
    "rust-analyzer.runnables.extraEnv": {
        "VIRTUAL_ENV": "<path-to-nautilus-trader>/python/.venv",
        "CC": "clang",
        "CXX": "clang++"
    },
    "rust-analyzer.check.features": "all",
    "rust-analyzer.testExplorer": true
}
```

```lua tab="Neovim (AstroLSP)"
config = {
  rust_analyzer = {
    settings = {
      ["rust-analyzer"] = {
        restartServerOnConfigChange = true,
        linkedProjects = { "Cargo.toml" },
        cargo = {
          features = "all",
          extraEnv = {
            VIRTUAL_ENV = "<path-to-nautilus-trader>/python/.venv",
            CC = "clang",
            CXX = "clang++",
          },
        },
        check = {
          workspace = false,
          command = "check",
          features = "all",
          extraEnv = {
            VIRTUAL_ENV = "<path-to-nautilus-trader>/python/.venv",
            CC = "clang",
            CXX = "clang++",
          },
        },
        runnables = {
          extraEnv = {
            VIRTUAL_ENV = "<path-to-nautilus-trader>/python/.venv",
            CC = "clang",
            CXX = "clang++",
          },
        },
        testExplorer = true,
      },
    },
  },
}
```
