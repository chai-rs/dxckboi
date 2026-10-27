---
title: "Migrate Parquet catalog"
---

ใช้ `nautilus catalog migrate-parquet` เพื่อเขียน Parquet catalog ของ Nautilus ที่มีอยู่ใหม่ให้เป็นรูปแบบ Arrow ปัจจุบัน
คำสั่งนี้อ่านจากต้นทางและเขียนไปยังปลายทางแยกต่างหาก ซึ่งต้องเป็นที่ใหม่หรือว่างเปล่า และต้องไม่ทับซ้อนกับต้นทาง
ทั้งสองตำแหน่งเป็น local path หรือ URI ของ object store ที่รองรับก็ได้

## รันการ migrate [#run-the-migration]

ตรวจสอบ schema ของต้นทางก่อน:

```bash
nautilus catalog migrate-parquet /data/catalog-old /data/catalog-new --dry-run
```

Dry run วางแผนการ migrate จาก schema และ layout ของต้นทางเท่านั้น โดยรายงานไฟล์ที่รู้จัก schema ที่ขัดแย้งกัน
และไฟล์ที่อยู่นอกโครงสร้าง catalog ที่รองรับ และไม่สร้างไฟล์หรือ directory ใด ๆ ที่ปลายทาง dry run ไม่ได้ decode
ค่าในแต่ละแถว และไม่ได้พิสูจน์ว่าปลายทางเขียนได้ ให้แก้ schema error ที่รายงานก่อนรันการแปลงจริง:

```bash
nautilus catalog migrate-parquet /data/catalog-old /data/catalog-new
```

เมื่อรันจาก source checkout ให้ใช้:

```bash
cargo run -p nautilus-cli -- catalog migrate-parquet /data/catalog-old /data/catalog-new
```

สำหรับ remote storage ให้ส่งค่าตั้งของ object store แบบ native ด้วย argument `--source-option key=value` และ
`--target-option key=value` ซ้ำได้หลายครั้ง

การทับซ้อนระหว่างสองตำแหน่งจะถูกปฏิเสธก่อนสร้างสิ่งใด และจะไม่มีการเขียนข้อมูลลงปลายทางที่ไม่ว่าง ดังนั้นการรันซ้ำ
กับปลายทางที่ migrate เสร็จแล้วจะล้มเหลวแทนที่จะเขียนทับไฟล์ สำหรับ local path ตัว directory ปลายทางเองอาจถูก
สร้างขึ้นก่อนการตรวจว่าว่างหรือไม่

การ migrate เก็บรักษาไฟล์ต้นทางไว้ คือเพียงอ่านเท่านั้น และตรวจขนาดไฟล์และ version marker ของแต่ละไฟล์ซ้ำอีกครั้ง
ก่อนแปลง

การแปลงยังปฏิเสธไฟล์ที่มีแถวอยู่นอกช่วง timestamp ที่ระบุในชื่อไฟล์ ซึ่ง dry run ที่ดูเฉพาะ schema ตรวจไม่พบ
ให้แก้ช่วงข้อมูลของต้นทางในสำเนาแยก แล้วลองใหม่ไปยังปลายทางใหม่ที่ว่างเปล่า

Report ที่พิมพ์ออกมาแยกแยะไฟล์และแถวที่ migrate แล้ว แถวที่ transcode แถวที่มี identifier ซึ่ง derive จาก path
ไฟล์ที่ข้าม และไฟล์ที่ไม่ได้ migrate ส่วนไฟล์ coverage ที่ว่างจะนับเป็นไฟล์ที่ migrate แล้วโดยมีศูนย์แถว ตรวจดู
report ก่อนสลับไปใช้งานจริง (cut over)

## การแทนค่าใน Arrow [#arrow-representation]

รูปแบบปัจจุบันใช้ Arrow type มาตรฐาน การแทนค่าแต่ละแบบด้านล่างใช้กับกลุ่ม record (record family) ที่ระบุไว้ข้าง ๆ

| ค่า                       | การแทนค่าใน Arrow ปัจจุบัน                            | Record family                                                                         |
| ----------------------- | ------------------------------------------------- | ------------------------------------------------------------------------------------- |
| ราคาและขนาด              | `Decimal128(38, 16)`                              | Quote, trade, bar, delta, depth level, mark price และ index price, close              |
| ข้อความทศนิยม             | UTF-8 string                                      | Funding `rate`, instrument, order, field ทางการเงินของ position, snapshot และ report  |
| ค่าวัดแบบ floating-point   | `Float64`                                         | Option greeks และ `realized_return` ของ position                                      |
| Instant timestamp       | `Timestamp(Nanosecond, Some("UTC"))`              | คอลัมน์ `ts_event`/`ts_init` ทุกคอลัมน์และ field ที่เป็นจุดเวลาอื่น ๆ                      |
| Duration และตัวนับ        | Unsigned integer                                  | `duration` ของ position, `interval` ของ funding, sequence, count, order ID, flag     |
| Enum ของ market data    | ชื่อ enum แบบ `Dictionary<Int8, Utf8>`              | Trade aggressor side, delta action/side, close type, status action, greeks convention |
| Enum และ ID ของ record   | UTF-8 string                                      | Enum, side, status และ ID ของ order, position, snapshot และ report                     |
| Payload แบบมีโครงสร้าง     | UTF-8 string พร้อม `arrow.json`                    | Balance/margin/`info` ของ account, `info`/commission/tag ของ order, report            |
| `Money` แบบ custom      | Struct ของยอดทศนิยมและ dictionary ของสกุลเงิน          | Field `Money` ของ custom data จาก Arrow macro                                         |
| ฝั่งของ depth             | List ของ struct ราคา ขนาด จำนวน และ order ID         | Order book depth                                                                      |

ค่าระดับ nanosecond ยังคงแม่นยำ รวมถึง timestamp ที่ต่างกันภายใน microsecond เดียวกัน และ duration
ยังเป็นค่าจำนวนเต็ม

`OrderFilled` แสดงให้เห็นว่า record family ต่างจาก market data อย่างไร คอลัมน์ `last_px`, `last_qty`, `order_side`,
`order_type` และ `liquidity_side` เป็น UTF-8 string ทั้งหมด และมีเพียงคอลัมน์ `info` ที่มี `arrow.json`

Output สำหรับแสดงผลเป็นการแทนค่าแยกต่างหากเพื่อความสะดวก และอาจใช้ราคาและขนาดแบบ floating-point ได้ หากต้องการ
ค่าทศนิยมที่แม่นยำ ให้ใช้ raw output ในกรณีที่มีให้ใช้

### Scale และ precision ในการจัดเก็บ [#storage-scale-and-precision]

ทศนิยมที่จัดเก็บจริงใช้ scale 16 เสมอ ส่วน schema metadata ของแต่ละไฟล์จะเก็บ `price_precision` และ
`size_precision` ของ domain ที่ใช้ตีความค่าเหล่านั้น build แบบ standard-precision จะ rescale raw integer ตอน encode
และตัวอ่านจะปฏิเสธค่าที่การตั้งค่าตัวเลขของตนแทนค่าได้ไม่แม่นยำ แทนการปัดเศษ precision metadata ที่สูงกว่า 16
และสกุลเงินของ `Money` ที่มี precision สูงกว่า 16 จะถูกปฏิเสธ ดังนั้น precision ของ DeFi เช่น wei จึงอยู่นอกการ
แทนค่าของ catalog

### การระบุ schema และความเข้ากันได้ [#schema-identification-and-compatibility]

Schema fingerprint บันทึกชื่อ field ตามลำดับ Arrow type และ nullability registry ของการ migrate ใช้ fingerprint
แบบตรงตัวสำหรับ transcoder ของรูปแบบเดิมที่ลงทะเบียนไว้ การจับคู่ schema ปัจจุบันตรวจคอลัมน์ตามชื่อและ type
ยอมรับ UTF-8 ธรรมดาสำหรับ dictionary string และไม่ต้องการลำดับ field หรือ nullability ที่ตรงกัน schema metadata
ไม่รวมอยู่ใน fingerprint ทั้ง `type_name`, identity และ numeric precision เป็นข้อกำหนดแยกต่างหาก ไม่มี schema
version แบบจำนวนเต็มที่ persist ไว้ และ release version อย่างเดียวไม่ได้ระบุ schema ของไฟล์

Arrow schema ของ catalog v2 ต่างจาก v1 การอัปเดตที่เข้ากันได้จะรักษาความหมายของ field เดิม ค่าตัวเลขที่แม่นยำ
identity และ timestamp การเปลี่ยนเป็น schema ที่ไม่เข้ากันต้องมีการ migrate อย่างชัดเจนและมีเอกสารรองรับต้นทาง
ตัวอ่านต้องไม่ตีความไฟล์เก่าใหม่แบบเงียบ ๆ สำหรับไฟล์ใน layout ของ directory ปัจจุบัน runtime query จะปฏิเสธ
legacy schema ที่รู้จักในไฟล์ที่ query เลือก และแนะนำผู้เรียกให้ไป migrate แม้ SQL predicate จะไม่คืนแถวใดเลย
ขอบเขต timestamp อาจตัดไฟล์ออกก่อนการตรวจ schema ได้ query ยังข้าม layout ของ directory แบบเก่า ทั้งสองกรณี
อาจคืนผลว่างโดยไม่มี schema error ให้ migrate catalog แบบเก่าก่อน query

### Null และค่าที่ถูกปฏิเสธ [#nulls-and-rejected-values]

ราคาและปริมาณที่ไม่ได้กำหนดจะ map เป็น Arrow null ใน family ที่ nullable เช่น delta, close และ mark price และ
index price ส่วน quote, trade และ bar ประกาศคอลัมน์ราคาและขนาดเป็น non-nullable โดยปฏิเสธค่าที่ไม่ได้กำหนดตอน
encode และปฏิเสธ null ตอน decode พร้อมระบุชื่อ field และแถว field ของ depth level เป็น non-nullable การ encode
จะละ level ที่ไม่มีออก และการ decode จะปฏิเสธ level ที่เป็น null และค่าที่เป็น null ค่าที่ไม่ถูกต้องและค่าที่อยู่นอก
ช่วงที่แทนได้จะทำให้เกิด error

### Identity และ layout ของไฟล์ [#identity-and-file-layout]

Schema metadata ของทุกไฟล์ระบุ type ไว้ใต้ `type_name` เช่น `QuoteTick`, `OrderFilled`, `CryptoPerpetual` หรือ
custom type ที่ลงทะเบียนไว้ และเก็บ identity ไว้ใต้ `instrument_id` หรือ `bar_type` ชื่อ directory ซ้ำ identity นั้น
อีกครั้ง ไฟล์ market data รวมถึง funding rate, instrument status และ option greeks เก็บ instrument ID ไว้เฉพาะใน
metadata ส่วนไฟล์ record เก็บเป็นคอลัมน์ด้วย writer จะตัดคอลัมน์ `identifier` แบบ nullable ใน memory ออกก่อน
persist ดังนั้นไฟล์ที่จัดเก็บจึงไม่มีคอลัมน์ `identifier` และไฟล์ Parquet ไม่ได้เก็บ cluster key

Catalog จัดกลุ่มไฟล์ตาม data type และ identifier custom data ใช้ `data/custom/<type>/<identifier>/` ส่วน instrument
ใช้หนึ่ง folder ต่อ instrument type ที่เป็นรูปธรรม ชื่อ folder แบบเก่าจะถูกรู้จักเมื่ออ่านต้นทางของการ migrate
รวมถึง `quote_tick`, `order_book_depth10`, directory ของ instrument ราย class และ `custom_<type>`

## สิ่งที่แปลงได้ [#what-converts]

| รูปแบบต้นทาง                                     | การแปลง                                                                |
| ---------------------------------------------- | ---------------------------------------------------------------------- |
| Market data แบบ fixed-width, field 8 หรือ 16 byte | Normalize เป็นทศนิยม, timestamp แบบ UTC, enum dictionary, UTF-8 ธรรมดา    |
| Depth แบบ flat 10 level และแบบ fixed-list         | สร้างใหม่เป็น list ที่ depth ยืดหยุ่น ทิ้ง level ที่เป็น null และ ID ที่ขาดเป็นศูนย์   |
| Record ที่มี metadata `type` แบบเก่า              | จัดให้ตรงกับ schema ของ record ที่ลงทะเบียนไว้ รวมถึง `info`                   |
| Instrument ที่มี metadata `class` แบบเก่า          | Decode แล้ว encode ใหม่เป็น schema แบบ string ราย class                   |
| รูปแบบเก่าที่รู้จักของ status, funding และ close      | แปลงผ่าน transcoder ที่ลงทะเบียนไว้สามตัว                                   |
| Custom data รูปแบบสุดท้ายที่มี `type_name`         | ส่งผ่านตามเดิม และเปลี่ยนชื่อ `custom_<type>` เป็น `custom/<type>`          |
| Custom data แบบเก่าที่ไม่มี `type_name`            | อนุมาน type จาก directory `custom_<type>` แล้วเปลี่ยนชื่อ                   |
| ไฟล์ coverage ที่ว่าง                              | คัดลอกเป็นไฟล์ว่างภายใต้ layout ที่เปลี่ยนชื่อแล้ว                             |

จากนั้นการ migrate จะ decode batch ปลายทางแบบ built-in ทุกตัวแล้ว encode ใหม่ด้วย encoder ปัจจุบัน ดังนั้นไฟล์ที่
migrate แล้วจะมี schema และ metadata เดียวกับไฟล์ที่การเขียนครั้งต่อไปสร้างขึ้น และรวม (consolidate) กันได้ แถวที่
decoder ปัจจุบันปฏิเสธจะทำให้การ migrate ล้มเหลว

คอลัมน์ `ts_event` และ `ts_init` ของ custom data ที่เก็บเป็น nanosecond แบบ `uint64` จะแปลงเป็น
`timestamp("ns", tz="UTC")` คอลัมน์ string แบบ dictionary-encoded รุ่นเก่าจะกลายเป็น UTF-8 ธรรมดา
ส่วนคอลัมน์ custom data อื่นที่รองรับจะส่งผ่านโดยไม่เปลี่ยนแปลง
ไฟล์ custom ที่ timestamp เป็น Arrow timestamp ระดับ nanosecond อยู่แล้วจะส่งผ่านโดยไม่เปลี่ยนแปลง
ไฟล์ที่ timestamp ใช้ physical type อื่น (โดยเฉพาะ `int64` จาก catalog รุ่นเก่าที่เขียนด้วย pandas) จะไม่ผ่าน
preflight ให้ cast ใหม่เป็น nanosecond แบบ `uint64` ก่อน migrate

การอนุมานชื่อ type จาก directory `custom_<type>` เป็นการแปลง snake_case เป็น PascalCase แบบ best-effort
ตัวย่อ (acronym) จะไม่คงรูปเดิม ดังนั้นควรตรวจ directory ปลายทางที่อนุมานได้ก่อน cut over

รูปแบบต้นทางแบบ fixed-depth ไม่มี order ID และการ migrate จะคง ID ที่เป็นศูนย์ตามที่ตัวอ่านคืนมา รูปแบบปลายทาง
จะเก็บ order ID สำหรับการเขียนครั้งต่อไป

### สิ่งที่แปลงไม่ได้ [#what-does-not-convert]

- **Feather tree** — Feather tree ของ `backtest` และ `live` อยู่นอกขอบเขตของคำสั่งนี้ ให้แปลงการรันที่ stage ไว้
  ผ่าน[streaming workflow](stream_parquet_catalog.md)แทน
- **Path ที่ไม่ได้ migrate** — directory `portfolio_snapshot`, leaf ที่ไม่ใช่ Parquet และ directory ของ catalog
  ที่ไม่รู้จัก จะถูกรายงานว่าไม่ได้ migrate และไม่ถูกแปลงเลย
- **Preflight ล้มเหลว** — schema ที่ขัดแย้ง, metadata `type_name` ที่หายไปนอก directory `custom_<type>` แบบเก่า,
  instrument ที่ไม่มี metadata `class` หรือ `type_name`, คอลัมน์ custom แบบ fixed-binary, custom timestamp ที่
  ไม่ใช่ทั้ง `uint64` และ Arrow timestamp ระดับ nanosecond และ fingerprint ที่ไม่รู้จัก จะไม่ผ่าน preflight โดย
  แสดงปัญหาทั้งหมด ก่อนมีการเขียนลงปลายทางใด ๆ

### ต้นทางที่ผ่านการตรวจสอบและการตรวจก่อน cut over [#verified-sources-and-cutover-checks]

ความครอบคลุมของต้นทางจาก release จำกัดอยู่ที่ fixture ต่อไปนี้จากเวอร์ชัน **1.231.0**:

| Writer ต้นทาง                    | การตั้งค่าตัวเลข          | Family ที่ตรวจแล้ว                                |
| ------------------------------- | --------------------- | --------------------------------------------- |
| Python Linux wheel ที่ release   | 128-bit               | Quote, `CurrencyPair`, Binance mark update    |
| Rust source ที่ tag ของ release  | 64-bit                | Quote และ `CurrencyPair`                      |
| Python adapter writer ที่ release | ไม่ขึ้นกับ precision     | Binance mark update ใน fixture catalog ทั้งสองชุด |

Fixture ของ Binance คือ `BinanceFuturesMarkPriceUpdate` ซึ่งราคาและ funding rate เป็น string และไฟล์จาก adapter
ที่ release ไฟล์เดียวกันถูกทดสอบกับการตั้งค่าตัวเลขทั้งสองแบบ การทดสอบเปรียบเทียบ field ของ quote และ instrument
ที่ decode แล้ว, Arrow field ทั้งหมดของ adapter, partition identity, timestamp ระดับ nanosecond, coverage ที่ทราบว่า
ว่าง และ byte ของต้นทาง การตรวจสอบ adapter ครอบคลุมการ migrate และ raw Arrow query แต่ไม่ได้รับประกันการ decode
แบบมี type เมื่อ schema ของ adapter เปลี่ยน instrument writer ของ Python ที่ release ไม่ได้เขียน tick scheme ดังนั้น
การ migrate จึงกู้คืน tick scheme ไม่ได้ ที่มาของ fixture และวิธีทำซ้ำอยู่คู่กับข้อมูลต้นทางใน
`test_data/nautilus/catalog_1_231_0/`

ตารางนี้ไม่ได้ยืนยันการรองรับทุก release ของ 1.x ทุก instrument class หรือทุก adapter release และ family อื่น
ต้องมี fixture ต้นทางและการเปรียบเทียบก่อน cut over ของตัวเอง fingerprint ที่ไม่รู้จักและ custom timestamp type
ที่ไม่รองรับยังคงเป็น preflight error

ความครอบคลุมแบบ end-to-end เพิ่มเติมมาจาก develop commit `1602043deb` ที่ build โดยเปิด feature `high-precision`
ครอบคลุม quote, trade, bar, order book แบบ fixed-depth, instrument แบบ `CurrencyPair`, record ของ account state
และ custom data type แบบทั่วไป การทดสอบเปรียบเทียบค่าปลายทางที่ decode แล้วกับค่าต้นฉบับ และตรวจว่าไฟล์ต้นทาง
ยังเหมือนเดิมทุก byte fixture นี้ไม่ได้กำหนด release ขั้นต่ำที่รองรับ และไม่ได้ครอบคลุมรูปแบบ catalog ในอดีตทั้งหมด

Regression test ของ synthetic instrument ยังครอบคลุม string แบบธรรมดาและแบบ dictionary-encoded ในไฟล์
`CryptoPerpetual` ที่มี metadata `class` แบบเก่า ทั้ง timestamp แบบ `uint64` และ `timestamp("ns", tz="UTC")`
และทั้งแบบมีและไม่มีคอลัมน์ `maker_fee` และ `taker_fee` ที่ถูกนำออกไปแล้ว
การ migrate จะทิ้งคอลัมน์ fee เหล่านั้นเพราะ instrument model ปัจจุบันไม่เก็บแล้ว runtime query จะปฏิเสธไฟล์
instrument แบบเก่าเหล่านี้และแนะนำผู้ใช้ไปยังคำสั่ง migrate โดยไม่แปลงไฟล์ระหว่างอ่าน

ตรวจปลายทางอย่างระมัดระวังเมื่อต้นทางมีสิ่งต่อไปนี้:

- ข้อมูลที่เขียนโดย Nautilus release รุ่นเก่า
- Instrument type อื่นนอกจาก `CurrencyPair` และ `CryptoPerpetual`
- Custom data ของ adapter นอกเหนือจาก fixture ของ Binance ข้างต้น เช่น Betfair, Deribit และ Hyperliquid
- Delta, mark price และ index price, close, greeks, funding รูปแบบปัจจุบัน หรือ record family อื่นนอกจาก
  `account_state`

ตรวจสอบปลายทางด้วยการ query แล้วเปรียบเทียบค่าที่ decode แล้ว identity, timestamp และช่วง coverage กับต้นทาง
จำนวนไฟล์และจำนวนแถวอย่างเดียวไม่เพียงพอ

## การกู้คืนจากการ migrate ที่ล้มเหลว [#recover-from-a-failed-migration]

ไฟล์ต้นทางแต่ละไฟล์จะถูกอ่านเข้า memory ทั้งไฟล์ก่อนแปลง ทีละไฟล์ ไฟล์ถูกเขียนตามลำดับด้วยการเขียนแบบ create-only
ดังนั้นความล้มเหลวอาจทิ้งปลายทางที่เขียนไว้บางส่วนไว้

:::warning[ลองใหม่ไปยังปลายทางใหม่ที่ว่างเปล่า]
ไม่มีการ resume ต่อในปลายทางที่เขียนไว้บางส่วน ให้แก้สาเหตุแล้วลองใหม่ไปยังปลายทางใหม่ที่ว่างเปล่า การลองใหม่ไปยัง
ปลายทางที่เขียนไว้บางส่วนจะไม่ผ่านการตรวจว่าปลายทางว่าง
:::

การ rollback ใช้ต้นทางที่เก็บรักษาไว้ โดยชี้ application กลับไปที่ catalog ต้นทางพร้อมใช้ application เวอร์ชันที่
เข้ากันได้ ไม่มีการ migrate ย้อนกลับ ดังนั้นข้อมูลที่เขียนลงปลายทางใหม่เท่านั้นหลัง cut over จะไม่ถูกนำกลับไปยังต้นทาง

## อ่านผลลัพธ์ [#read-the-result]

ชี้ `ParquetDataCatalog`, config ข้อมูลของ backtest หรือ config catalog ของ live node ไปที่ปลายทาง
ใช้ Nautilus เวอร์ชันปัจจุบันในการเขียนข้อมูลเพิ่มเติม PyArrow, pandas และ Polars อ่านค่า Arrow แบบเปิดได้
โดยไม่ต้องมี binary decoder เฉพาะของ Nautilus

สำหรับข้อมูลที่ stream เข้ามาใหม่ ดู [Parquet streaming](stream_parquet_catalog.md)
