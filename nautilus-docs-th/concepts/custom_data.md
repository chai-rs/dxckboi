# Custom Data

NautilusTrader รองรับ custom data ที่เขียนด้วย Rust โดยใช้ pipeline ด้าน runtime routing, persistence
และ query เดียวกับข้อมูล built-in

เอกสารนี้อธิบายว่า custom data:

- ถูกลงทะเบียนตอน runtime อย่างไร
- ถูก serialize ไปและกลับจาก Arrow/Parquet อย่างไร
- ถูก route ผ่าน actor และ strategy อย่างไร

## เป้าหมาย {#goals}

สถาปัตยกรรมของ custom data ตอบโจทย์ข้อกำหนดต่อไปนี้:

- ให้ custom data ที่นิยามใน Rust ใช้ JSON และ Arrow handler แบบ native ของ Rust ได้
- รองรับ persistence ใน `ParquetDataCatalog` โดยใช้การลงทะเบียน type แบบ dynamic
  แทน schema ที่ hardcode ไว้
- ทำให้ custom data route ผ่าน flow การ subscribe ปกติของ data engine, actor และ strategy ได้

## ภาพรวมระดับสูง {#high-level-model}

| Mode             | รูปแบบการเขียน                                  | เส้นทางการลงทะเบียน                                                 | เส้นทาง encode/decode            | Wrapper backend           |
| ---------------- | ----------------------------------------------- | ----------------------------------------------------------------- | ----------------------------- | ------------------------- |
| Same-binary Rust | type ที่ใช้ `#[custom_data]`                     | `ensure_custom_data_registered::<T>()`                            | Native Rust                   | Native Rust payload       |

custom data ใช้ wrapper `CustomData` ชั้นนอกตัวเดียวกันและโมเดล identity ของ `DataType` แบบเดียวกัน

## Component หลัก {#core-components}

### Registry module {#registry-module}

`crates/model/src/data/registry.rs` เก็บ registry ระดับ process สำหรับ JSON และ Arrow
การลงทะเบียนใช้ operation แบบ atomic ของ `DashMap::entry()` เพื่อไม่ให้การเรียก `register_*` และ `ensure_*`
พร้อมกันเกิด race ขณะจอง entry

module นี้ initialize สถานะของ registry ผ่าน `OnceLock` และเก็บ:

- JSON deserializer โดยใช้ `type_name` เป็น key
- Arrow schema, encoder และ decoder โดยใช้ `type_name` เป็น key

แทนที่จะ hardcode แต่ละ type ไว้ใน binary หลัก NautilusTrader จะ resolve handler ตอน runtime โดยใช้
`type_name` ที่เก็บไว้ใน `DataType` และ Parquet metadata

### `CustomData` {#customdata}

wrapper `CustomData` ชั้นนอกเป็น container กลางที่ห่อ custom data ทุกชนิด
โดยรับ `DataType` เป็นอันดับแรก แล้วตามด้วย payload ภายใน

ประกอบด้วย:

- `DataType`
- custom payload ภายในที่ implement `CustomDataTrait` (ห่อไว้ใน
  `Arc<dyn CustomDataTrait>`)

timestamp (`ts_event`, `ts_init`) ถูก delegate ไปยัง implementation ของ
`CustomDataTrait` ภายใน และเปิดให้เข้าถึงผ่าน wrapper

implementation ของ `PartialEq` ใน Rust จะเปรียบเทียบ `DataType` และ delegate การเปรียบเทียบ payload
ไปยังค่าภายใน

#### JSON envelope ของ `CustomData` {#customdata-json-envelope}

เมื่อ serialize เป็น JSON เช่นสำหรับ `to_json_bytes` / `from_json_bytes`, SQL cache หรือ Redis
`CustomData` จะใช้ envelope มาตรฐานรูปแบบเดียว การ deserialize จึงไม่ขึ้นกับชื่อฟิลด์ของ payload ของผู้ใช้:

- `type`: ชื่อ custom type (จาก `CustomDataTrait::type_name`)
- `data_type`: object ที่มี `type_name`, `metadata` และ `identifier` (optional)
- `payload`: เฉพาะ payload ภายใน (ผลลัพธ์ของ `CustomDataTrait::to_json`
  ที่ parse เป็นค่าแล้ว) deserializer ที่ลงทะเบียนไว้จะส่งเฉพาะค่านี้ไปยัง
  `from_json` ดังนั้น struct ของผู้ใช้จึงใช้ชื่อฟิลด์ใดก็ได้ รวมถึง `value`
  โดยไม่ชนกับ metadata ของ wrapper

envelope นี้สร้างโดยการ serialize `CustomData` ใน Rust และถูกใช้โดย
registry module ตอน deserialize custom data จาก JSON

### `DataType` {#datatype}

`DataType` ระบุตัวตนของ custom data สำหรับการ route และ persistence

ประกอบด้วย:

- `type_name`
- `metadata` (optional)
- `identifier` (optional) สำหรับ path ของ persistence และการค้นหาใน cache database
  ไม่มีผลต่อการ route, equality หรือ hashing

**Equality, hashing และการ route topic** คำนวณจาก `type_name` และ `metadata` เท่านั้น ค่า `DataType`
สองค่าที่มี type name และ metadata เดียวกันแต่ identifier ต่างกันจะถือว่าเท่ากันและ publish ไปยัง
topic เดียวกันบน message bus

`identifier` ใช้เลือก path ใน catalog ภายใต้ `data/custom/<type_name>/<identifier...>` และใช้ในการกรองของ
PostgreSQL และ Redis

persistence จะเก็บ `DataType` ทั้งหมดไว้กับค่า `CustomData` แต่ละค่าและกู้คืนเมื่อ query ส่วนการค้นหา
handler ใช้ `type_name` ดังนั้น type เชิงตรรกะเดียวกันจึงมี metadata หรือ identifier ต่างกันได้ และยัง
decode ผ่าน handler ที่ลงทะเบียนไว้ตัวเดียวกัน

## สถาปัตยกรรมการลงทะเบียน {#registration-architecture}

### การลงทะเบียน Rust แบบ same-binary {#same-binary-rust-registration}

สำหรับ Rust type ที่ compile อยู่ใน process:

1. `#[custom_data]` สร้าง implementation ของ trait และ JSON
   รวมถึง implementation ของ Arrow โดย default
1. `ensure_custom_data_registered::<T>()` ใส่ handler schema/encoder/decoder แบบ native
   ลงใน registry ระดับ process

เส้นทางนี้เป็น native Rust ทั้งหมดสำหรับ encode/decode

### ลำดับความสำคัญของการลงทะเบียน {#registration-precedence}

การลงทะเบียนแบบ idempotent ของ `ensure_*` จะไม่เขียนทับ handler ที่มีอยู่ สำหรับแต่ละ type name การลงทะเบียน
Arrow ครั้งแรกจะคง schema, encoder และ decoder ของตนไว้

## Wrapper backend {#wrapper-backends}

### Native same-binary Rust payload {#native-same-binary-rust-payload}

สำหรับ Rust type ที่ compile อยู่ใน process payload ภายในคือ Rust type ตัวจริง
และสามารถ downcast ได้โดยตรงจาก `Arc<dyn CustomDataTrait>`

## สถาปัตยกรรม persistence {#persistence-architecture}

### ทำไมต้องลงทะเบียน Arrow แบบ dynamic {#why-dynamic-arrow-registration-is-needed}

data type แบบ built-in ของ NautilusTrader มี schema และ encoder ที่ Rust binary รู้จักแบบ static แต่ custom data
ไม่เป็นเช่นนั้น persistence layer จึง resolve custom data แบบ dynamic โดยใช้ `type_name` ที่ลงทะเบียนไว้

### Flow การเขียน catalog {#catalog-write-flow}

`ParquetDataCatalog` คาดหวังให้การเขียน custom data เข้ามาเป็นค่า `CustomData`

เส้นทางการเขียน custom data:

1. นำ `type_name` จาก payload ภายใน และ `metadata` กับ `identifier` จาก
   `DataType` ของค่าแรก
1. ตรวจสอบว่า Arrow schema ที่ลงทะเบียนไว้มี `ts_init` และฟิลด์ `ts_event` หรือ
   `ts_init` ใช้ `timestamp("ns", tz="UTC")`
1. ค้นหา Arrow encoder ใน registry ระดับ process
1. encode ค่าต่าง ๆ เป็น `RecordBatch`
1. เพิ่มคอลัมน์ `data_type` ที่มี `DataType` ที่ persist ไว้
1. แนบ `type_name` และ metadata ไปกับ Arrow schema
1. เขียน batch ลง Parquet ภายใต้ path ของ custom data

layout ของ path คือ `data/custom/<type_name>/<identifier...>`

identifier จะถูก normalize ก่อนกลายเป็นส่วนหนึ่งของ path

### Flow การอ่าน catalog {#catalog-read-flow}

เมื่อ query:

1. catalog อ่านไฟล์ Parquet ที่ตรงกัน
1. ดึง `type_name` จาก schema metadata
1. ขอ decoder จาก registry ระดับ process
1. decode `RecordBatch` เป็น `Vec<Data>`
1. สร้าง `CustomData` ขึ้นใหม่ด้วย `DataType` เดิม

ทำให้การ resolve ตอน query custom data สมมาตรกับการลงทะเบียนตอนเขียน

ระหว่างการแปลง Feather เป็น Parquet (เช่นหลัง backtest) branch ของ custom data จะแปลง Arrow
batch และเขียนผลลัพธ์ไปยัง path ของ custom data ที่ตรงกันโดยตรง:

- ส่ง data class เป็น `custom/{TypeName}` โดยใช้ type name ที่ลงทะเบียนไว้ตามตัวอักษร session ของ Feather เก็บ
  custom data ไว้ใต้ `data/custom/{TypeName}` ไม่ใช่ใต้ไดเรกทอรีแบบ snake_case
- ตั้ง `promote_on_close=False` เมื่อต้องการแปลงเอง ค่า default คือ promote ตอนปิด
  การแปลง session ที่ promote ไปแล้วจะเขียนช่วง `ts_init` เดิมซ้ำภายใต้ชื่อไฟล์อื่น
  ซึ่ง catalog จะปฏิเสธเพราะช่วงเวลาไม่ disjoint กัน

การอ่านใช้เฉพาะ `data/custom/{TypeName}` เท่านั้น catalog ที่ยังใช้ layout เดิม `data/custom_<snake_case>`
ที่เขียนโดยฝั่ง Python (เช่น `data/custom_binance_bar`) ให้แปลงด้วย
[nautilus catalog migrate-parquet](../how_to/migrate_parquet_catalog.md)

## การสร้างข้อมูลขึ้นใหม่ตอน query {#reconstruction-on-query}

เมื่อโหลด custom data กลับจาก catalog Rust type แบบ same-binary จะ decode เป็นค่า Rust แบบ native โดยตรง

## การ integrate กับ runtime {#runtime-integration}

custom data มีส่วนร่วมใน runtime routing ของ NautilusTrader ด้วย

integration ที่เกี่ยวข้องได้แก่:

- `crates/data/src/engine/dispatch.rs` publish `CustomData` ผ่าน message
  bus
- `crates/common/src/msgbus/switchboard.rs` สร้าง custom topic จาก
  `DataType`
- `crates/common/src/actor/*` route custom data ไปยัง subscription ของ actor
- `crates/backtest/src/engine.rs` ถือว่า `Data::Custom` เป็น
  input ที่ data engine ส่งมอบ ไม่ใช่ข้อมูลที่ route ผ่าน exchange

custom type ที่ลงทะเบียนแล้วสามารถ persist, query, subscribe และใช้งานได้
ผ่าน runtime interface เดียวกับกลุ่มข้อมูล built-in

## การ integrate กับ cache database {#cache-database-integration}

implementation ของ cache database แบบ PostgreSQL และ Redis รองรับ `CustomData`

- PostgreSQL เก็บ custom data ไว้ในตาราง `custom`
- record ที่เก็บประกอบด้วย `data_type`, `metadata`, `identifier` และ JSON payload
  ทั้งหมด
- การอ่านจะสร้าง `CustomData` ขึ้นใหม่ด้วย `CustomData::from_json_bytes(...)`
- Redis cache เก็บ custom data ภายใต้ key
  `custom:<ts_init_020>:<uuid>` โดยมี JSON ของ `CustomData` ทั้งหมดเป็นค่า
- `add_custom_data` และ `load_custom_data` ของ Redis กรองตาม `DataType` (type_name, metadata, identifier) และ
  return ผลลัพธ์เรียงตาม `ts_init`
