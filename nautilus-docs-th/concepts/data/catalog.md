---
title: "Data catalog"
---

Data catalog จัดเก็บข้อมูลของ NautilusTrader ในไฟล์ [Parquet](https://parquet.apache.org) เพื่อใช้ใน backtest,
live trading และงานวิจัย

## ภาพรวมและสถาปัตยกรรม [#overview-and-architecture]

`ParquetDataCatalog` ทำงานบน catalog ที่เขียนด้วย Rust และ query engine ของ DataFusion crate ด้าน model และ
persistence ของ Rust เป็นตัวกำหนด Arrow schema ของข้อมูลที่มีมาให้ ส่วน custom data ที่ลงทะเบียนไว้จะส่ง schema และ
handler สำหรับ encode/decode ของตัวเองเข้ามาตอน runtime

Timestamp แบบจุดเวลาใช้ `Timestamp(Nanosecond, Some("UTC"))` ส่วน duration ยังคงเป็นจำนวนเต็ม Arrow reader และ
query ของ Nautilus จะคงความละเอียดระดับ nanosecond และ annotation UTC ไว้ SQL reader ที่ map column เหล่านี้เป็น
`TIMESTAMPTZ` ความละเอียดระดับ microsecond อาจตัดค่าที่ละเอียดกว่า microsecond ทิ้ง

Parquet ให้การจัดเก็บแบบ columnar ที่บีบอัดได้และเข้าถึงได้จากหลายภาษา catalog เก็บไฟล์เหล่านี้ไว้ใต้ root เดียว
โดยไม่ต้องมี database service แยกต่างหาก path แบบ local หรือ URI ของ object store เป็นตัวเลือก storage backend

## การ initialize [#initializing]

ส่ง path แบบ local หรือ URI เป็น argument แรกของ constructor

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## Filesystem protocol และ storage option [#filesystem-protocols-and-storage-options]

Catalog รองรับ storage protocol ตามที่ object-store backend ฝั่ง Rust รองรับ

### Filesystem protocol ที่รองรับ [#supported-filesystem-protocols]

| Storage              | URI scheme         | Option key ที่ใช้บ่อย                                                       |
| -------------------- | ------------------ | ------------------------------------------------------------------------- |
| Local filesystem     | Path ธรรมดา, `file` | ไม่มี                                                                      |
| Amazon S3            | `s3`               | `region`, `access_key_id`, `secret_access_key`, `endpoint_url`.           |
| Google Cloud Storage | `gs`, `gcs`        | `service_account_path`, `service_account_key`, `application_credentials`. |
| Azure Blob Storage   | `az`, `abfs`       | `account_name`, `account_key`, `sas_token`.                               |
| HTTP หรือ WebDAV      | `http`, `https`    | `timeout`, `connect_timeout`, `proxy_url`.                                |

Option key คือ configuration key ของ `object_store` สำหรับ URI scheme นั้น:

- Key ของ S3, GCS และ Azure ใช้แบบมี prefix ได้ด้วย เช่น `aws_region`, `google_application_credentials` หรือ
  `azure_storage_sas_token`
- Client option เช่น `timeout` ใช้กับทุก scheme แบบ remote ตั้ง `allow_http` เป็น `true` เพื่อเชื่อมต่อผ่าน `http://` ธรรมดา
- S3 รองรับชื่อแบบเก่า `key` และ `secret` ด้วย
- Key ที่ไม่รู้จักจะล้มเหลวพร้อม error ที่ระบุชื่อ key นั้น

ส่ง credential และค่าตั้งอื่น ๆ ของ backend ผ่าน `storage_options`

หลังส่วนของ bucket, container หรือ host แล้ว base path ของ URI แบบ remote ต้องไม่มีช่องว่างหรืออักขระที่ไม่ใช่ ASCII
และต้องไม่มีอักขระที่ path ของ object-store จะ encode เช่น `~`, `%`, `#`, `?`, `^` หรือ `|` catalog จะปฏิเสธ URI
ลักษณะนี้ตอนเปิด พร้อม error ที่ระบุ URI นั้น

## การบีบอัดและ row group [#compression-and-row-groups]

`DataCatalogConfig` กำหนดวิธีที่ catalog ที่ตั้งค่าไว้อ่านและเขียนไฟล์ข้อมูล:

| Field                | ควบคุม                                       | ค่าเริ่มต้น        |
| -------------------- | ------------------------------------------- | ---------------- |
| `batch_size`         | จำนวน row ต่อ batch ที่ catalog อ่านและเขียน    | 10,000           |
| `compression`        | ชื่อ codec สำหรับไฟล์ข้อมูลที่เขียน              | `zstd` (level 1) |
| `max_row_group_size` | จำนวน row สูงสุดต่อ row group ที่เขียน           | 131,072          |

`ParquetDataCatalog` รับค่า `batch_size` และ `max_row_group_size` แบบเดียวกัน แต่เลือก `compression` ด้วยรหัสตัวเลข
ส่วน `DataCatalogConfig` รับชื่อด้านล่างโดยไม่สนตัวพิมพ์เล็กใหญ่:

| Codec        | ชื่อใน `DataCatalogConfig` | รหัสใน `ParquetDataCatalog` |
| ------------ | ------------------------ | ------------------------- |
| Uncompressed | `uncompressed`           | `0`                       |
| Snappy       | `snappy`                 | `1`                       |
| gzip         | `gzip`                   | `2`                       |
| Brotli       | `brotli`                 | `4`                       |
| LZ4          | `lz4`, `lz4_raw`         | `5`                       |
| zstd         | `zstd`                   | `6`                       |

การสร้างจะล้มเหลวหากใช้ชื่อ compression อื่น หรือตั้ง `batch_size` หรือ `max_row_group_size` เป็นศูนย์
`params` ใช้ส่ง option สำหรับ catalog backend ภายนอกเท่านั้น Parquet catalog จะปฏิเสธ key ใด ๆ ใน `params`

:::info
`lz4` และ `lz4_raw` หมายถึง codec เดียวกัน: catalog เขียนเป็น Parquet `LZ4_RAW` รูปแบบ Parquet เลิกสนับสนุน
codec `LZ4` แบบเก่าที่มี Hadoop framing แล้ว catalog จึงไม่เขียนแบบนั้นเลย รหัส compression `5` เขียนเป็น `LZ4_RAW`
แม้ว่า Parquet จะกำหนดหมายเลขของ `LZ4_RAW` เป็น `7` ไฟล์ที่เคยเขียนด้วย codec แบบ Hadoop framing ยังคงอ่านได้

ไม่รองรับ LZO เพราะ Parquet writer สร้างไม่ได้ ชื่อ `lzo` และรหัส `3` จึงล้มเหลวตอนสร้าง
:::

## การเขียนข้อมูล [#writing-data]

ใช้ writer ที่ตรงกับประเภทข้อมูลจริง instrument definition และ custom data มี writer แยกต่างหาก

Writer สำหรับ market data ที่มีมาให้ได้แก่:

- `write_quote_ticks`
- `write_trade_ticks`
- `write_order_book_deltas`
- `write_order_book_depths`
- `write_bars`
- `write_mark_price_updates`
- `write_index_price_updates`
- `write_option_greeks`

Writer แต่ละตัวรับ `start` และ `end` แบบ optional เพื่อ override เป็น UNIX nanosecond ข้อมูลในการเรียกครั้งเดียวต้องมี
identity เดียว เช่น instrument ID หรือ bar type เดียว และต้องเรียงตาม `ts_init`

การเขียนข้อมูลแบบ typed, instrument และ custom data บน instance เดียวกันจะ serialize การเปลี่ยนแปลงไฟล์ระดับ native
อย่าให้ query, การเขียน record หรือ [catalog operation](#catalog-operations) ทับซ้อนกับการเรียกอื่นบน instance เดียวกัน
การเรียกพร้อมกันอาจเกิด error หากการเรียกตัวใดตัวหนึ่งต้องการสิทธิ์เข้าถึงแบบ exclusive

## การตั้งชื่อไฟล์และการจัดระเบียบข้อมูล [#file-naming-and-data-organization]

Catalog ตั้งชื่อไฟล์ตามช่วง timestamp ด้วยรูปแบบ `{start_timestamp}_{end_timestamp}.parquet` โดยแปลง timestamp
ISO 8601 แต่ละตัวให้ปลอดภัยสำหรับชื่อไฟล์ด้วยการแทน `:` และ `.` ด้วย `-`

ข้อมูลที่มีมาให้จะจัดเป็น directory ตามประเภทข้อมูลและ identifier สำหรับ instrument ID และ bar type catalog จะลบ `/`
และแทน `^` ด้วย `_` เมื่อสร้างชื่อ directory ที่ปลอดภัยสำหรับ URI:

```text
catalog/
├── data/
│   ├── quotes/
│   │   └── EURUSD.SIM/
│   │       └── 2024-01-01T00-00-00-000000000Z_2024-01-01T23-59-59-999999999Z.parquet
│   └── trades/
│       └── BTCUSD.BINANCE/
│           └── 2024-01-01T00-00-00-000000000Z_2024-01-01T23-59-59-999999999Z.parquet
```

Custom data ใช้ `data/custom/<type_name>/` โดยอาจมี path segment ของ identifier ต่อท้าย

Instrument ใช้ directory หนึ่งอันต่อ instrument class จริง เช่น `data/currency_pair/` หรือ `data/equity/`
แทนที่จะใช้ directory `instruments` ร่วมกัน

Query ของ catalog และ backtest จึงอ่าน **เฉพาะ directory มาตรฐานเหล่านี้** query จะไม่อ่าน layout แบบเก่า เช่น
`data/trade_tick/`, `data/quote_tick/` และ `data/custom_<snake_case>/`

หากต้องการอัปเกรด catalog ที่ยังใช้ layout แบบเก่า ให้แปลงด้วย
[nautilus catalog migrate-parquet](../../how_to/migrate_parquet_catalog.md)

:::warning[การเขียนที่ทับซ้อนกัน]
โดยค่าเริ่มต้น การเขียนที่ช่วงเวลาทับซ้อนกันจะเกิด error เพื่อรักษาความถูกต้องของข้อมูล
ตั้ง `skip_disjoint_check=True` เฉพาะเมื่อตั้งใจให้ทับซ้อนเท่านั้น
:::

## การอ่านข้อมูล [#reading-data]

ใช้ typed query เมื่อทราบประเภทผลลัพธ์ที่ต้องการ (เช่น `query_quote_ticks`, `query_trade_ticks`) โดย `start` และ `end`
เป็น UNIX nanosecond

เมื่อเปิด feature `streaming` ให้ลงทะเบียน catalog ผ่าน `LiveNodeConfig.catalogs` ด้วยรายการ `DataCatalogConfig`
historical request จะอ่านช่วงเวลาที่มีอยู่ใน catalog และขอช่วงที่ขาดจาก data client

Request ของ quote และ trade ที่ระบุ `params["schema"]` อย่างชัดเจนจะข้ามการอ่านจาก catalog และการเขียนกลับ
เพราะ schema ต้นทางอาจเปลี่ยนการ sampling หรือ trade ID แต่ key ของ catalog ไม่ได้แยกแยะส่วนนี้

ตั้ง `params={"update_catalog": True}` เพื่อเขียน response ที่สำเร็จจากต้นทางกลับลงใน catalog และใช้ `"catalog_name"`
เพื่อเลือก catalog ที่ลงทะเบียนไว้อย่างชัดเจน

การเขียนกลับต้องใช้ client ที่รายงานการเสร็จสิ้นอย่างชัดเจน เช่น Databento client อื่นจะคืนข้อมูลโดยไม่อัปเดต catalog
และ log เหตุผลที่ข้ามการเขียนกลับ

การเขียนกลับใช้กฎความครอบคลุมเหล่านี้:

- Response ที่ล้มเหลวและที่เป็นระหว่างทางจะข้ามการเขียนกลับ request ที่ถูกแบ่งเป็นหลาย envelope จากต้นทางก็ข้ามเช่นกัน
  รวมถึง request book delta แบบ parent-symbol ที่คืน payload หลายชุด
- Request ที่มีการจำกัดจำนวน record จะข้ามการเขียนกลับ เพราะจำนวน row ที่ decode ได้ไม่สามารถพิสูจน์ได้ว่าต้นทางใช้
  limit นั้นหมดหรือไม่
- Batch ที่มี instrument หรือ bar type นอกเหนือจาก request จะข้ามการเขียนกลับ เพราะไม่ทราบช่วงความครอบคลุมจากต้นทาง
- Response ว่างที่สำเร็จสามารถขยายความครอบคลุมแบบ known-empty ได้ เมื่อมีขอบเขตทั้งสองด้านและ backend รองรับ

ความล้มเหลวในการเขียน catalog จะถูก log ไว้และไม่ทำให้ข้อมูลที่คืนมาถูกทิ้ง

## `BacktestDataConfig`: ข้อมูลสำหรับ backtest [#backtestdataconfig-backtest-data]

`BacktestDataConfig` กำหนดข้อมูลใน catalog ที่ `BacktestNode` จะโหลดสำหรับหนึ่ง run

### พารามิเตอร์หลัก [#core-parameters]

- `data_type` เป็นค่า `NautilusDataType`: `QuoteTick`, `TradeTick`, `Bar`, `OrderBookDelta`, `OrderBookDepth`,
  `MarkPriceUpdate`, `IndexPriceUpdate`, `FundingRateUpdate`, `InstrumentStatus`, `OptionGreeks`, `InstrumentClose`
  หรือ `Instrument` โดย `Instrument` จะโหลดทุก instrument class ที่ catalog มีสำหรับ identifier ที่เลือก
- `catalog_path` ระบุ root ของ catalog
- ต้องระบุ `instrument_id`, `instrument_ids` หรือ `bar_types` อย่างใดอย่างหนึ่ง
- `start_time` และ `end_time` เป็นขอบเขตเวลาแบบ UNIX nanosecond (optional)
- `filter_expr` เป็น predicate แบบ DataFusion SQL (optional)
- `catalog_fs_protocol` ใช้เป็น prefix ของ `catalog_path` สำหรับ remote storage
- `catalog_fs_rust_storage_options` ส่ง option ให้ backend ฝั่ง Rust หากไม่ได้ตั้งค่า `BacktestNode` จะ fallback ไปใช้
  `catalog_fs_storage_options`
- สำหรับ bar, `bar_spec` จะรวมกับ instrument ID เพื่อเลือก bar type แบบ `EXTERNAL` ส่วน `bar_types` ที่ระบุอย่างชัดเจน
  สามารถเลือก bar แบบ internal, external หรือ composite ได้ ตัวอย่างเช่น instrument `AAPL.NASDAQ` กับ bar spec แบบ
  5-MINUTE-LAST จะเลือก `AAPL.NASDAQ-5-MINUTE-LAST-EXTERNAL`
- `optimize_file_loading` จะลงทะเบียนทั้ง directory เมื่อทำได้
- `batch_deltas` (ค่าเริ่มต้น `True`) จะ replay ข้อมูล `OrderBookDelta` เป็น batch `OrderBookDeltas` ที่ปิดด้วย `F_LAST`
  ดู [order book delta replay](../backtesting/apis-and-runs.md#order-book-delta-replay)

### Cloud storage และการกรอง [#cloud-storage-and-filtering]

สำหรับ catalog บน cloud ให้ใช้ `catalog_fs_protocol` (เช่น `s3`) ร่วมกับ `catalog_fs_rust_storage_options` เพื่อส่ง credential
และใช้ `filter_expr` (เช่น `ts_init >= 1704067200000000000`) เพื่อกรองข้อมูล

### การใช้ร่วมกับ BacktestRunConfig [#integration-with-backtestrunconfig]

ส่ง data config เหล่านี้ให้ `BacktestRunConfig` ผ่าน field `data` ร่วมกับ config ของ venue และขอบเขตเวลา `start`/`end`
ของ run

### ขั้นตอนการโหลดข้อมูล [#data-loading-process]

เมื่อ backtest ทำงาน `BacktestNode` จะประมวลผล `BacktestDataConfig` แต่ละตัวดังนี้:

1. สร้าง `ParquetDataCatalog` จาก configuration
1. โหลด instrument definition ที่จำเป็นระหว่างสร้าง engine
1. สร้างและรัน DataFusion query จาก field ใน configuration
1. เรียงข้อมูลที่ merge แล้วตาม `ts_init` และเพิ่มเข้าไปใน backtest engine

## การเข้าถึง catalog โดยตรง [#direct-catalog-access]

- ใช้ `ParquetDataCatalog` เพื่อ query หรือเขียน catalog โดยตรง
- ใช้ `BacktestDataConfig` เมื่อต้องการให้ `BacktestNode` โหลดข้อมูลจาก catalog สำหรับ run
- ใช้ `LiveNodeConfig.catalogs` เพื่อลงทะเบียน catalog สำหรับ historical request ซึ่งไม่ได้ preload ข้อมูลแบบที่
  `BacktestDataConfig` ทำ ส่วน field `streaming` ที่แยกต่างหากใช้กำหนดการเขียน Feather เท่านั้น

## การ query และการกรอง [#querying-and-filtering]

`query` แบบ generic เลือกกลุ่มข้อมูลผ่าน `NautilusDataType` ซึ่ง map ไปยัง directory ของ catalog เช่น `quotes`,
`trades` หรือ `bars` ใช้เมื่อต้องการควบคุม `files` หรือ `optimize_file_loading`

Query method ทั้งหมดนี้ใช้ขอบเขตเวลาแบบ UNIX nanosecond และรับ `where_clause` แบบ DataFusion SQL:

- `query` เป็น interface แบบ generic
- Typed method เช่น `query_quote_ticks`, `query_trade_ticks` และ `query_bars` คืนค่าเป็น model type จริง
- `query_custom_data` resolve custom decoder ผ่าน registry ตอน runtime

:::warning[ฐานข้อมูล time zone ไม่ตรงกัน]
เมื่อใช้ `Cargo.lock` ปัจจุบัน ฟังก์ชันด้านเวลาของ DataFusion SQL จะ resolve time zone แบบมีชื่อด้วยฐานข้อมูลของ
`chrono-tz` 0.10.4 ที่เป็น transitive dependency (IANA 2025b) ส่วนการทำงานด้าน time zone ในแกน Rust ใช้ Jiff 0.2.38
พร้อมฐานข้อมูล IANA 2026e ที่ bundle มา ผลลัพธ์ของ zone อาจต่างกันเมื่อกฎของ zone เปลี่ยน หรือข้อมูลย้อนหลังถูกแก้ไข
หลัง 2025b จนกว่า DataFusion จะ migrate

หาก RustSec ออก advisory ประเภท unmaintained สำหรับ `chrono` หรือ `chrono-tz` ให้คงรายการ ignore ที่มีเอกสารกำกับ
ให้ตรงกันใน `.cargo/audit.toml` และ `deny.toml` จนกว่า DataFusion จะ migrate
:::

## Catalog operation [#catalog-operations]

Catalog operation ใช้เปลี่ยนชื่อ, รวม หรือลบไฟล์ข้อมูล โดยมีตัวเลือกประเภทดังนี้:

- `NautilusDataType`: เลือกกลุ่มข้อมูล `NautilusDataType.Instrument` ครอบคลุมทุก instrument class
- `NautilusRecordType`: เลือกกลุ่ม record
- `NautilusInstrumentType`: เจาะจง instrument class เดียว

`delete_data_range(...)` รับเฉพาะ `NautilusDataType` เพราะมีแต่กลุ่มข้อมูลเท่านั้นที่รองรับการลบเป็นช่วง

### Reset ชื่อไฟล์ [#reset-file-names]

Reset ชื่อไฟล์ Parquet ให้ตรงกับ timestamp ของเนื้อหา เพื่อให้การกรองตามชื่อไฟล์ยังแม่นยำ `reset_all_file_names()`
ประมวลผลทั้ง catalog ส่วน `reset_data_file_names(...)` เจาะจง data path

สำหรับประเภทข้อมูลที่แบ่ง partition ตาม instrument:

- ระบุ instrument ID เพื่อเจาะจง directory ของ instrument นั้น
- หากไม่ระบุ ID การทำงานจะอ่าน directory ของประเภทนั้นแบบ recursive และเปลี่ยนชื่อแต่ละไฟล์ภายใน directory ของ
  instrument ของมันเอง โดยตรวจสอบช่วงเวลาของแต่ละ directory ก่อนเปลี่ยนชื่อไฟล์ใด ๆ

### กู้คืนจากชื่อไฟล์ที่ทับซ้อนกัน [#recover-from-overlapping-file-names]

ชื่อไฟล์ที่ทับซ้อนกันจะบล็อกการเขียนลงใน directory ที่ได้รับผลกระทบ การขยายความครอบคลุมที่ถูกปฏิเสธจะไม่เปลี่ยนชื่อไฟล์เลย
ดังนั้นการทับซ้อนจึงบ่งชี้ว่าเกิดจากการเปลี่ยนชื่อครั้งก่อน, การย้ายไฟล์ด้วยมือ หรือ writer ที่ทำงานพร้อมกัน

เมื่อเนื้อหาของไฟล์ยังไม่ทับซ้อนกัน ให้กู้คืน directory ด้วยการ reset ชื่อไฟล์:

1. หยุด writer ทุกตัวที่เขียนลง catalog
1. สำรอง directory ที่ได้รับผลกระทบ การ reset จะเปลี่ยนชื่อไฟล์ทีละไฟล์ ความล้มเหลวด้าน I/O กลางทางอาจทำให้บางไฟล์
   ถูกเปลี่ยนชื่อไปแล้ว บน object store ที่เปลี่ยนชื่อด้วยการ copy แล้วลบ ความล้มเหลวอาจทำให้ไฟล์มีอยู่ทั้งสองชื่อ
1. รัน `reset_data_file_names(...)` กับ path ที่ได้รับผลกระทบ สำหรับ custom data type ให้ใช้ `reset_all_file_names()`
   แทน ซึ่งครอบคลุมทุก leaf directory รวมถึง layout แบบ custom
1. หากการ reset รายงานว่ามีไฟล์อื่นใช้ชื่อใหม่นั้นอยู่ ให้เปลี่ยนชื่อไฟล์นั้นเป็นชื่อช่วงเวลาที่ยังไม่มีใครใช้และอยู่นอกช่วงข้อมูล
   แล้วรัน reset อีกครั้ง ทำซ้ำจนกว่าจะสำเร็จ
1. ยืนยันว่าไฟล์ที่เปลี่ยนชื่อแต่ละไฟล์ตรงกับช่วงของเนื้อหา จากนั้นเขียนข้อมูลช่วงเวลาถัดไปที่ไม่ทับซ้อน เพื่อยืนยันว่า
   directory รับการเขียนได้อีกครั้ง

การ reset ประมวลผลทีละ directory ก่อนเปลี่ยนชื่อไฟล์ใดใน directory หนึ่ง จะอ่านช่วง `ts_init` ของทุกไฟล์ก่อน และจะล้มเหลว
เมื่อช่วงของเนื้อหาทับซ้อนกัน หรือเมื่อชื่อใหม่ของไฟล์หนึ่งเป็นชื่อปัจจุบันของอีกไฟล์ เมื่อล้มเหลวจะไม่เปลี่ยนชื่อไฟล์ใดใน
directory นั้นเลย ส่วน directory ที่ reset ไปก่อนหน้า directory ที่ล้มเหลวจะคงชื่อใหม่ไว้

ใช้การ reset ชื่อไฟล์เฉพาะกรณีที่เสียหายแค่ชื่อไฟล์เท่านั้น เมื่อเนื้อหาของไฟล์ทับซ้อนกัน การ reset ชื่อไฟล์ไม่สามารถ
กระทบยอดข้อมูลได้และจะล้มเหลว ให้สร้างช่วงที่ได้รับผลกระทบขึ้นใหม่จากต้นทางผ่านกระบวนการที่ตรวจสอบแยกต่างหาก

### รวม catalog [#consolidate-catalog]

รวมไฟล์ Parquet ขนาดเล็กเข้าด้วยกันเพื่อลดจำนวนไฟล์และ overhead ของ query เมื่อไม่ระบุขอบเขต `consolidate_catalog()`
จะประมวลผลทุก leaf data directory ใน catalog ส่วน `consolidate_data(...)` ทำงานกับ directory เดียว ระบุ instrument ID
สำหรับประเภทข้อมูลที่แบ่ง partition ตาม instrument โดยรับขอบเขต `start`/`end` และ option `ensure_contiguous_files` ได้

### รวม catalog ตามช่วงเวลา [#consolidate-catalog-by-period]

แบ่งไฟล์ข้อมูลเป็นช่วงเวลาคงที่ ทั้ง duration และขอบเขตเวลาใช้หน่วย nanosecond ทั้งสอง method
(`consolidate_catalog_by_period` และ `consolidate_data_by_period`) รับขอบเขตแบบ optional และ `period_nanos`
ระบุ identifier ให้กับ method ราย data type สำหรับข้อมูลที่แบ่ง partition ตาม instrument

- **Method ระดับทั้ง catalog**: ประมวลผล quote, trade, order book delta, order book depth, bar, index price, mark price,
  instrument close และ custom type ที่ลงทะเบียนไว้ ประเภทอื่นจะ log warning และข้ามไป
- **Method ราย data type**: ปฏิเสธตัวเลือกแบบ instrument และ record ซึ่งไม่มีการเขียนใหม่ตามช่วงเวลา ให้ใช้
  `consolidate_data(...)` สำหรับกรณีเหล่านั้น

### ลบช่วงข้อมูล [#delete-data-range]

ลบข้อมูลภายในช่วงเวลา โดยอาจจำกัดเฉพาะ data type และ instrument เดียว ระบุ instrument ID สำหรับข้อมูลที่แบ่ง partition
ตาม instrument ขอบเขตเวลาทำงานดังนี้:

- ไม่ระบุ `start` จะขยายช่วงไปถึงจุดเริ่มต้น
- ไม่ระบุ `end` จะขยายช่วงไปถึงจุดสิ้นสุด
- สำหรับ `delete_data_range(...)` การไม่ระบุทั้งสองขอบเขตจะลบข้อมูลที่ตรงเงื่อนไขทั้งหมด

`delete_data_range(...)` รองรับ quote, trade, bar, order book delta, order book depth และ custom type ที่ลงทะเบียนไว้
ส่ง `NautilusDataType.OrderBookDepth` สำหรับ order book depth และ `NautilusDataType.Custom("MarketTickPython")`
สำหรับ custom data type

- `delete_catalog_range(...)` จะทำงานต่อไปหลังเจอ directory ที่ไม่รองรับ โดย log warning และไม่แตะต้องข้อมูลในนั้น
- ใช้ `delete_data_range(...)` เพื่อยืนยันว่าประเภทที่ขอรองรับจริง `NautilusDataType.Instrument` จะถูกปฏิเสธ เพราะ
  instrument definition ไม่รองรับการลบเป็นช่วง

:::danger[ลบข้อมูลถาวร]
การลบไม่สามารถย้อนกลับได้ catalog จะแบ่งไฟล์ที่ทับซ้อนบางส่วนเพื่อรักษาข้อมูลที่อยู่นอกช่วงไว้
:::

## Feather streaming และการแปลง [#feather-streaming-and-conversion]

Runtime สามารถพัก record ไว้ในไฟล์ Feather แบบ local แล้ว promote เข้า catalog ได้ด้วย
`StreamingConfig(writer_path=..., catalog=DataCatalogConfig(...))` record ที่พักไว้จะ query จาก catalog ได้
**หลังจาก promote สำเร็จแล้ว** การ flush ข้อมูลที่พักไว้กับการ commit ลง catalog เป็นคนละขั้นตอนกัน

ค่าเริ่มต้นของ streaming คือ promote ตอนปิด, ไม่มีการ promote ตามช่วงเวลา และเก็บไฟล์ Feather ต้นทางที่ commit แล้วไว้
เมื่อตั้ง `promotion_interval_ms` เป็นค่าบวก:

- Live streaming ใช้การตั้งเวลาตาม wall-clock
- Backtest streaming จะตรวจสอบ clock ที่ส่งเข้ามาระหว่างการเขียนและการ flush

ดู [stream data into a Parquet catalog](../../how_to/stream_parquet_catalog.md) สำหรับค่าเริ่มต้น, configuration,
การมองเห็นข้อมูลใน query และการกู้คืน

`StreamingFeatherWriter` ยังใช้พักข้อมูลโดยตรงได้ และ session ที่เสร็จแล้วสามารถแปลงด้วยมือได้ด้วย
`ParquetDataCatalog.convert_stream_to_data()`
