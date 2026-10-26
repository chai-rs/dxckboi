# Test Datasets

มาตรฐานเป้าหมายสำหรับการคัดสรร (curate), จัดเก็บ และใช้งานชุดข้อมูลภายนอกที่ใช้เป็น test fixture
ชุดข้อมูลใหม่ควรเป็นไปตามมาตรฐานนี้ ส่วนชุดข้อมูลเดิมที่มีอยู่ก่อนนโยบายนี้บันทึกไว้ใน
[ชุดข้อมูลแบบ legacy](#legacy-datasets)

## ประเภทของชุดข้อมูล {#dataset-categories}

**ข้อมูลขนาดเล็ก** (< 1 MB) ถูก check in เข้า `test_data/<source>/` โดยตรงพร้อมไฟล์ `metadata.json`
ไฟล์เหล่านี้ใช้ได้เสมอโดยไม่ต้องเข้าถึงเครือข่าย

**ข้อมูลขนาดใหญ่** (> 1 MB) ถูกเก็บเป็น Parquet ใน bucket test-data ของ R2 โดยบันทึก checksum แบบ SHA-256
ไว้ใน `test_data/large/checksums.json` ก่อนรัน test ที่ใช้ข้อมูลขนาดใหญ่ ให้เตรียม fixture จาก root ของ repository:

```bash
cargo run --locked -p nautilus-testkit --bin prepare-test-data
```

คำสั่งนี้ดาวน์โหลดไฟล์ที่ขาด และตรวจสอบ fixture ทุกตัวใน checksum manifest ที่ติดตามไว้ มันจะแทนที่ไฟล์ใน cache
ที่ checksum ไม่ตรง ไม่แก้ไข manifest และจะปฏิเสธพร้อมลบไฟล์ที่ดาวน์โหลดมาแต่ checksum ไม่ตรง CI จะรันขั้นตอนนี้
หลังจากกู้ cache ของ test-data แล้ว

ฟังก์ชัน `ensure_test_data_exists()` ตรวจเพียงว่ามีไฟล์ในเครื่องหรือไม่ test ที่ต้องการ fixture ที่ขาดอยู่จะล้มเหลว
พร้อมข้อความที่ระบุคำสั่งเตรียมข้อมูล โดยไม่ดาวน์โหลดข้อมูลเอง ทั้งขั้นตอนเตรียมข้อมูลและ test ต่างเคารพ
`TEST_DATA_ROOT_PATH`

**ข้อมูลที่ผู้ใช้ดึงเอง (user-fetched)** ใช้ในกรณีที่ license ของผู้ให้บริการ, รูปแบบสิทธิ์การใช้งาน หรือการควบคุม
การเข้าถึงไม่อนุญาตให้ NautilusTrader เผยแพร่ซ้ำต่อสาธารณะ ข้อจำกัดนี้ครอบคลุมทั้ง repo สาธารณะและ bucket R2
สาธารณะ ในรูปแบบนี้ repo จะเก็บเพียง manifest, คำแนะนำการดึงข้อมูล และโค้ดสำหรับแปลงข้อมูล ผู้ใช้แต่ละคน
ดาวน์โหลดข้อมูลต้นทางด้วย account ของผู้ให้บริการของตนเองและแปลงข้อมูลในเครื่อง

ใช้รูปแบบ user-fetched เมื่อเข้าข่ายข้อใดข้อหนึ่งต่อไปนี้:

- ผู้ให้บริการกำหนดให้ผู้ใช้แต่ละคนต้องมี account, API key หรือ license ข้อมูลย้อนหลังของตัวเอง
- license อนุญาตให้ใช้ภายใน แต่ไม่ได้อนุญาตอย่างชัดเจนให้เผยแพร่ fixture ที่ derive มาซ้ำ
- ชุดข้อมูลเหมาะกับตัวอย่างหรือ integration test แบบ opt-in แต่ไม่เหมาะกับ CI ค่าเริ่มต้น

## Metadata ที่จำเป็น {#required-metadata}

ชุดข้อมูลที่คัดสรรแล้วทุกชุดที่เก็บหรือเผยแพร่ artifact จริงต้องมี `metadata.json` ที่มีอย่างน้อย:

| Field          | คำอธิบาย                                            |
| -------------- | ------------------------------------------------- |
| `file`         | ชื่อไฟล์ของชุดข้อมูล                                    |
| `sha256`       | hash แบบ SHA-256 ของไฟล์                            |
| `size_bytes`   | ขนาดไฟล์หน่วยไบต์                                      |
| `original_url` | URL ดาวน์โหลดของข้อมูลต้นทาง                             |
| `licence`      | เงื่อนไข license และข้อจำกัดการเผยแพร่ซ้ำ (ถ้ามี)              |
| `added_at`     | timestamp แบบ ISO 8601 ที่คัดสรรชุดข้อมูล                  |

ฟิลด์เหล่านี้ตรงกับ output ของ `scripts/curate-dataset.sh` ฟิลด์เพิ่มเติมที่แนะนำเพื่อให้ข้อมูลที่มา (provenance)
ครบถ้วนยิ่งขึ้น:

| Field           | คำอธิบาย                                                    |
| --------------- | --------------------------------------------------------- |
| `instrument`    | symbol ของ instrument ที่ครอบคลุม                             |
| `date`          | วันที่ซื้อขายที่ครอบคลุม                                         |
| `format`        | รูปแบบการจัดเก็บ (เช่น "Nautilus OrderBookDelta Parquet")       |
| `original_file` | ชื่อไฟล์เดิมของผู้ให้บริการก่อนแปลง                                 |
| `parser`        | parser ที่ใช้แปลง (เช่น "itchy 0.3.4")                         |

ชุดข้อมูลแบบ user-fetched ใช้ฟิลด์ metadata เดียวกันในส่วนที่ใช้ได้ และควรมีฟิลด์ต่อไปนี้ด้วย:

| Field               | คำอธิบาย                                                               |
| ------------------- | -------------------------------------------------------------------- |
| `distribution`      | ต้องเป็น `"user-fetch"`                                                 |
| `fetch_method`      | วิธีที่ผู้ใช้ได้ข้อมูลต้นทางมา (API, web portal, CLI ฯลฯ)                        |
| `fetch_reference`   | URL หรือเอกสารอ้างอิงของขั้นตอนดาวน์โหลดสำหรับผู้ใช้                              |
| `auth`              | credential หรือสิทธิ์ที่ต้องมี (ถ้ามี)                                         |
| `transform_version` | เวอร์ชันของ pipeline แปลงข้อมูลในเครื่องที่สร้างไฟล์สุดท้าย                         |
| `redistribution`    | หมายเหตุสั้น ๆ อธิบายข้อจำกัดการเผยแพร่ซ้ำของชุดข้อมูล                            |
| `public_mirror`     | ต้องเป็น `false` สำหรับชุดข้อมูลของผู้ให้บริการที่มีข้อจำกัด                         |

สำหรับชุดข้อมูลแบบ user-fetched:

- หากไม่มี artifact เดียวที่ commit หรือ mirror ไว้ อาจละ `file`, `sha256` และ `size_bytes` ออกจาก
  `metadata.json` ได้ ในกรณีนั้น `target_files` ใน `manifest.json` เป็นแหล่งอ้างอิงหลักสำหรับไฟล์ output ในเครื่อง
- ผู้ให้บริการอาจสร้างไฟล์เฉพาะตาม account หรือคำขอของผู้ใช้แต่ละราย ในกรณีนั้น `original_url` อาจชี้ไปยังจุดเริ่มต้น
  การดาวน์โหลดของผู้ให้บริการแทน URL ของไฟล์จริง

ฟิลด์ metadata อื่น ๆ ยังแนะนำให้ใส่ในส่วนที่ใช้ได้ โดยเฉพาะ `licence` และ `added_at` ควรบันทึกไว้เสมอแม้เป็น
ชุดข้อมูลแบบ user-fetched

## รูปแบบการจัดเก็บ {#storage-format}

ชุดข้อมูลใหม่ควรเก็บเป็น **Nautilus Parquet** (ไม่ใช่รูปแบบดิบของผู้ให้บริการ) ซึ่งช่วยให้:

- data type สอดคล้องกันในทุกชุดข้อมูลของ test
- ไม่ต้อง parse รูปแบบของผู้ให้บริการตอนรัน test
- สถานะงาน derivative ด้าน license ชัดเจน

ใช้การบีบอัด ZSTD (level 3) กับ row group ขนาด 1M

ชุดข้อมูลแบบ user-fetched ควรได้ผลลัพธ์เป็น Nautilus Parquet เช่นกันหลังขั้นตอนแปลงในเครื่อง ไฟล์ดิบของผู้ให้บริการ
ควรอยู่นอก repo และนอก bucket R2 สาธารณะ

## หลักการตั้งชื่อ {#naming-convention}

```
<source>_<instrument>_<date>_<datatype>.parquet
```

ตัวอย่าง:

- `itch_AAPL_2019-01-30_deltas.parquet`
- `tardis_BTCUSDT_2020-09-01_depth10.parquet`
- `histdata_EURUSD.SIM_2020-01_quotes.parquet`

## Workflow การคัดสรร {#curation-workflow}

### ไฟล์ธรรมดา (ดาวน์โหลดครั้งเดียว) {#simple-files-single-download}

ใช้ `scripts/curate-dataset.sh`:

```bash
scripts/curate-dataset.sh <slug> <filename> <download-url> <license>
```

คำสั่งนี้สร้าง directory ที่มีเวอร์ชัน (`v1/<slug>/`) พร้อมไฟล์, `LICENSE.txt` และ `metadata.json` ที่มีฟิลด์
ที่จำเป็นข้างต้น

### Pipeline ที่ซับซ้อน (parse + แปลง) {#complex-pipelines-parse--transform}

สำหรับชุดข้อมูลที่ต้องแปลงรูปแบบ (เช่น ITCH แบบ binary เป็น Parquet):

1. เขียนฟังก์ชันคัดสรรไว้ใน `crates/testkit/src/<source>/` โดยกั้นไว้หลัง `#[cfg(test)]` หรือ test ที่ติด `#[ignore]`
2. ฟังก์ชันควร: ดาวน์โหลด, parse, กรอง, แปลงเป็น type ของ NautilusTrader และเขียน Parquet
3. ส่ง output เป็นไฟล์ Parquet และ `metadata.json` ไปยัง directory ในเครื่อง
4. อัปโหลดไปยัง R2 ด้วยมือ แล้วเพิ่ม checksum ใน `checksums.json`

### Pipeline แบบ user-fetched (จำกัดการเผยแพร่ซ้ำ) {#user-fetched-pipelines-restricted-redistribution}

สำหรับชุดข้อมูลที่ NautilusTrader เผยแพร่ซ้ำไม่ได้:

1. commit manifest และ `metadata.json` แต่ห้าม commit ข้อมูลจริงของผู้ให้บริการหรือ output Parquet ที่ derive มา
2. จัดเตรียมคำสั่งหรือ script สำหรับดึงข้อมูลในเครื่องที่ใช้ credential, สิทธิ์ หรือไฟล์ข้อมูลย้อนหลังที่ผู้ใช้ซื้อมาเอง
3. แปลงข้อมูลของผู้ให้บริการในเครื่องให้เป็น Nautilus Parquet
4. เก็บไฟล์ผลลัพธ์ไว้ใน path cache ในเครื่องที่ git ignore ไว้
5. ทำให้ test และตัวอย่างเป็นแบบ opt in และควรข้ามอย่างเรียบร้อยเมื่อไม่มีชุดข้อมูล

ลำดับการเผยแพร่ค่าเริ่มต้นสำหรับชุดข้อมูลใหม่คือ:

1. ข้อมูลขนาดเล็กที่ check in ไว้
2. ข้อมูลขนาดใหญ่บน R2 สาธารณะ
3. ข้อมูลแบบ user-fetched

เลือก user-fetched เฉพาะเมื่อสองทางเลือกแรกไม่เป็นที่ยอมรับตามเงื่อนไขของผู้ให้บริการ

ห้าม:

- อัปโหลดชุดข้อมูลของผู้ให้บริการที่มีข้อจำกัดขึ้น bucket R2 สาธารณะ
- commit ไฟล์ Parquet ที่ derive มาจากข้อมูลจริงของผู้ให้บริการเข้า repo เมื่อสิทธิ์ในการเผยแพร่ซ้ำไม่ชัดเจน
- ทำให้ CI ค่าเริ่มต้นขึ้นกับ credential ของผู้ให้บริการหรือการเข้าถึงข้อมูลย้อนหลังแบบเสียเงิน

สามารถดูแล mirror ส่วนตัวสำหรับ CI ภายในหรือพนักงานได้เมื่อ license อนุญาตให้แชร์ภายใน ให้ถือว่าเป็นเส้นทาง
การดำเนินงานแยกต่างหาก ไม่ใช่ส่วนหนึ่งของมาตรฐาน test-data สาธารณะ

## การเพิ่มชุดข้อมูลใหม่ {#adding-a-new-dataset}

1. คัดสรรข้อมูลตาม workflow ข้างต้น
2. เขียน `metadata.json` ให้มีฟิลด์ที่จำเป็นครบ
3. สำหรับข้อมูลขนาดเล็ก: commit เข้า `test_data/<source>/`
4. สำหรับข้อมูลขนาดใหญ่: อัปโหลด Parquet ไปยัง R2 และเพิ่ม checksum ใน `test_data/large/checksums.json`
5. สำหรับข้อมูลแบบ user-fetched: commit เฉพาะ manifest และคำแนะนำการดึงข้อมูล เก็บข้อมูลต้นทางและข้อมูลที่ derive มา
   ไว้นอก repo และนอก bucket R2 สาธารณะ
6. เพิ่มฟังก์ชัน path ของ test-data ที่ใช้ร่วมกันใน `crates/testkit/src/common.rs` เมื่อจำเป็น
7. เขียน test ที่ใช้ชุดข้อมูลนั้น

สำหรับข้อมูลแบบ user-fetched แนะนำโครงสร้างนี้:

```text
test_data/<source>/<slug>/
  metadata.json
  manifest.json
  README.md
```

ใช้ `test_data/local/<source>/<slug>/` เป็น path cache ในเครื่องมาตรฐานสำหรับ artifact ที่สร้างขึ้น และเก็บไฟล์ดิบ
ที่ดาวน์โหลดจากผู้ให้บริการไว้ใน directory `vendor/` ที่อยู่ระดับเดียวกันภายใต้ path cache เดียวกัน เมื่อจำเป็นต้องเก็บไว้ในเครื่อง

manifest ควรอ่านได้ด้วยเครื่องและคงที่ โดยเก็บข้อมูลขั้นต่ำที่จำเป็นในการทำซ้ำขั้นตอนดึงและแปลงข้อมูลบนเครื่องอื่น

`metadata.json` เป็นแหล่งอ้างอิงหลักสำหรับที่มา, license และกฎการเผยแพร่ซ้ำ
`manifest.json` เป็นแหล่งอ้างอิงหลักสำหรับ input ของการดึงข้อมูล, คำสั่ง, ตำแหน่ง cache และไฟล์ output

ฟิลด์ของ manifest ที่แนะนำ:

| Field               | คำอธิบาย                                                      |
| ------------------- | ----------------------------------------------------------- |
| `slug`              | identifier ของชุดข้อมูลที่คงที่                                   |
| `vendor`            | ชื่อผู้ให้บริการหรือ venue                                         |
| `source_type`       | `api`, `portal-download`, `purchased-archive` ฯลฯ            |
| `source_filters`    | symbol, event ID, market ID, ช่วงวันที่ หรือชื่อไฟล์                |
| `target_files`      | ไฟล์ Nautilus Parquet ที่คาดว่าจะได้หลังการแปลง                      |
| `cache_dir`         | ตำแหน่ง output ในเครื่องเทียบกับ `test_data/local/`               |
| `fetch_command`     | คำสั่งหรือ entry point ของ script ที่แนะนำ                         |
| `transform_command` | คำสั่งแปลงข้อมูลในเครื่องที่แนะนำ                                    |
| `env`               | environment variable ที่ต้องใช้                                 |
| `notes`             | หมายเหตุสั้น ๆ ด้านการใช้งานสำหรับผู้ใช้                               |

test ที่อาศัยข้อมูลแบบ user-fetched ควร:

- ถูกทำเครื่องหมายหรือจัดกลุ่มแยกจาก test ของ CI ค่าเริ่มต้น
- ข้ามพร้อมข้อความที่ชัดเจนเมื่อไม่มีชุดข้อมูลในเครื่อง
- หลีกเลี่ยงการเข้าถึงเครือข่าย เว้นแต่ผู้ใช้จะ opt in อย่างชัดเจน
- ใช้ path cache ในเครื่องที่คงที่ เพื่อให้ดึงข้อมูลเพียงครั้งเดียวต่อเครื่อง

หาก CI ค่าเริ่มต้นไม่ได้คาดว่าจะรัน Rust test ที่ต้องเตรียมชุดข้อมูลด้วยมือ ให้ใช้ `#[ignore]`

## การสร้างชุดข้อมูลใหม่ {#regenerating-datasets}

หากการเปลี่ยน schema ทำให้ไฟล์ Parquet ขนาดใหญ่ใช้ไม่ได้ ให้สร้างใหม่จากข้อมูลต้นทางเดิม โดยใช้ curation test ด้านล่าง
หลังสร้างใหม่แล้ว:

1. `sha256sum /tmp/<output_file>.parquet`
1. อัปเดต `test_data/large/checksums.json` ด้วย hash ใหม่
1. อัปเดต `metadata.json` ที่เกี่ยวข้อง (sha256, size_bytes)
1. อัปโหลดไฟล์ Parquet ไปยัง R2
1. แทนที่ไฟล์ใน cache ที่ `test_data/large/` ด้วยไฟล์ที่สร้างใหม่ แล้วรันคำสั่งเตรียมข้อมูลเพื่อตรวจสอบ หากตั้ง
   `TEST_DATA_ROOT_PATH` ไว้ให้ใช้ cache ที่สอดคล้องกันภายใต้ path นั้น
1. commit `checksums.json` และ `metadata.json` (ซึ่งจะล้าง cache ของ CI ไปด้วย)

### ITCH AAPL L3 deltas {#itch-aapl-l3-deltas}

ต้นทาง: `01302019.NASDAQ_ITCH50.gz` (~4.4 GB) จาก NASDAQ EMI

```bash
# Download source (keep a local copy, this is a large file)
wget -O ~/Downloads/01302019.NASDAQ_ITCH50.gz \
  "https://emi.nasdaq.com/ITCH/Nasdaq%20ITCH/01302019.NASDAQ_ITCH50.gz"

# Curation test expects source at /tmp
ln -sf ~/Downloads/01302019.NASDAQ_ITCH50.gz /tmp/01302019.NASDAQ_ITCH50.gz

# Regenerate parquet (output: /tmp/itch_AAPL.XNAS_2019-01-30_deltas.parquet)
cargo test -p nautilus-testkit --lib test_curate_aapl_itch -- --ignored --nocapture
```

### Tardis Deribit BTC-PERPETUAL L2 deltas {#tardis-deribit-btc-perpetual-l2-deltas}

ต้นทาง: `tardis_deribit_incremental_book_L2_2020-04-01_BTC-PERPETUAL.csv.gz` จาก
[Tardis](https://tardis.dev/) ข้อมูลวันแรกของแต่ละเดือนมีให้เป็นตัวอย่างฟรี (ไม่ต้องใช้ API key)

```bash
# Download source (free sample, no API key needed)
wget -O test_data/large/tardis_deribit_incremental_book_L2_2020-04-01_BTC-PERPETUAL.csv.gz \
  "https://datasets.tardis.dev/v1/deribit/incremental_book_L2/2020/04/01/BTC-PERPETUAL.csv.gz"

# Regenerate parquet (output: /tmp/tardis_BTC-PERPETUAL.DERIBIT_2020-04-01_deltas.parquet)
cargo test -p nautilus-tardis --features arrow test_curate_deribit_deltas -- --ignored --nocapture
```

## ข้อมูลทดสอบสำหรับ tutorial {#tutorial-test-data}

tutorial และคู่มือหลายชิ้นโหลด market data ที่ผู้ใช้จัดหาเอง environment variable `NAUTILUS_DATA_DIR` ใช้ override
path ฐานของข้อมูลเหล่านี้ ให้ใช้ `test_data/local/` เป็นตำแหน่งใน repository ที่ถูก ignore สำหรับไฟล์เหล่านี้

### โครงสร้าง directory {#directory-layout}

```text
test_data/local/
  Binance/
    BTCUSDT_T_DEPTH_2022-11-01_depth_snap.csv
    BTCUSDT_T_DEPTH_2022-11-01_depth_update.csv
  Bybit/
    2024-12-01_XRPUSDT_ob500.data.zip
  HISTDATA/
    DAT_ASCII_EURUSD_T_202001.csv.gz
```

directory `test_data/local/` ถูก gitignore ไว้ script ของ tutorial จะหยุดพร้อมข้อความแจ้งว่าข้อมูลขาด เมื่อไม่พบไฟล์
ที่คาดไว้

### การได้มาซึ่งข้อมูล {#obtaining-the-data}

**Depth snapshot ของ Binance** หาได้จาก [Binance public data portal](https://data.binance.vision/) ดาวน์โหลดไฟล์
BTCUSDT T_DEPTH ของวันที่ 2022-11-01 แล้ววาง CSV ของ snap และ update ไว้ใต้ `test_data/local/Binance/`
สำหรับการทดสอบ ใช้ข้อมูลเพียงบางส่วน (เช่น 10,000 แถวแรก) ก็เพียงพอ

**ข้อมูล orderbook ob500 ของ Bybit** หาได้จาก CDN ของ Bybit:

```bash
curl -L "https://quote-saver.bycsi.com/orderbook/linear/XRPUSDT/2024-12-01_XRPUSDT_ob500.data.zip" \
  -o test_data/local/Bybit/2024-12-01_XRPUSDT_ob500.data.zip
```

ไฟล์เต็มมีขนาดประมาณ 360 MB สำหรับการทดสอบ ให้ดึงออกมาเพียงไม่กี่ร้อยบรรทัดแรกแล้วบีบอัดใหม่เป็น zip ขนาดเล็กลง

**ข้อมูล tick ของ HISTDATA** หาได้จาก [histdata.com](https://www.histdata.com/) ดาวน์โหลดข้อมูล tick แบบ ASCII
ของ EUR/USD เดือนใดก็ได้ แล้ววาง CSV (หรือ `.csv.gz`) ไว้ใต้ `test_data/local/HISTDATA/`

## ชุดข้อมูลแบบ legacy {#legacy-datasets}

ชุดข้อมูลเหล่านี้มีมาก่อนนโยบายนี้และใช้รูปแบบดิบของผู้ให้บริการ (CSV/CSV.gz) โดยไม่มี `metadata.json`
ยังใช้ได้กับ test เดิม ส่วนชุดข้อมูลใหม่ควรเป็นไปตามมาตรฐาน Parquet ข้างต้น

| ชุดข้อมูล                     | ต้นทาง    | รูปแบบ            | ตำแหน่ง              | สถานะ    |
| -------------------------- | -------- | ---------------- | ------------------- | -------- |
| Tardis Deribit L2 deltas   | Tardis   | Parquet (large)  | `test_data/large/`  | Curated  |
| ITCH AAPL L3 deltas        | NASDAQ   | Parquet (large)  | `test_data/large/`  | Curated  |
| HISTDATA EURUSD.SIM quotes | HISTDATA | Parquet (large)  | `test_data/large/`  | Migrated |
| Tardis Deribit L2          | Tardis   | CSV (checked in) | `test_data/tardis/` | Legacy   |
| Tardis Binance snapshots   | Tardis   | CSV.gz (large)   | `test_data/large/`  | Legacy   |
| Tardis Bitmex trades       | Tardis   | CSV.gz (large)   | `test_data/large/`  | Legacy   |

catalog `nautechsystems/nautilus_data` เดิมสอดคล้องกับไฟล์ Parquet ของ HISTDATA EURUSD.SIM ข้างต้น
ส่วนไฟล์ CSV ดิบของ HISTDATA ยังเป็นแบบ user-fetched
