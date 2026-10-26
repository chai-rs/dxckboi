# Commodity

`Commodity` แทนตลาด spot ของ commodity เช่น ทองคำ เงิน น้ำมัน หรือสินทรัพย์จริงอื่น ๆ ที่ quote ราคาเป็นสกุลเงิน
เป็นโมเดลของตลาด spot ไม่ใช่สัญญา futures ที่มีวันครบกำหนด

ตัวอย่างเช่น `XAUUSD.IDEALPRO` และ symbol commodity แบบ cash เฉพาะของแต่ละ venue

## Fields {#fields}

| Field | Rust type | Required/default | หมายเหตุ |
| --- | --- | --- | --- |
| `instrument_id` | `InstrumentId` | Required | ใน Rust เก็บไว้ใน field `id` |
| `raw_symbol` | `Symbol` | Required | symbol ดั้งเดิมของ venue |
| `asset_class` | `AssetClass` | Required | การจัดประเภท asset แบบ commodity |
| `quote_currency` | `Currency` | Required | สกุลเงินที่ใช้กำหนดราคา commodity |
| `price_precision` | `u8` | Required | จำนวนทศนิยมที่อนุญาตสำหรับราคา |
| `size_precision` | `u8` | Required | จำนวนทศนิยมที่อนุญาตสำหรับขนาด order |
| `price_increment` | `Price` | Required | ขั้นราคาที่เล็กที่สุดที่ใช้ได้ |
| `size_increment` | `Quantity` | Required | ขั้นขนาดที่เล็กที่สุดที่ใช้ได้ |
| `ts_event` | `UnixNanos` | Required | timestamp ของ event หน่วยนาโนวินาที |
| `ts_init` | `UnixNanos` | Required | timestamp ตอน initialize หน่วยนาโนวินาที |
| `lot_size` | `Option<Quantity>` | `None` | ขนาด lot หรือ board lot |
| `max_quantity` | `Option<Quantity>` | `None` | ปริมาณ order สูงสุด |
| `min_quantity` | `Option<Quantity>` | `None` | ปริมาณ order ต่ำสุด |
| `max_notional` | `Option<Money>` | `None` | มูลค่า notional ของ order สูงสุด |
| `min_notional` | `Option<Money>` | `None` | มูลค่า notional ของ order ต่ำสุด |
| `max_price` | `Option<Price>` | `None` | ราคา quote หรือ order สูงสุดที่ใช้ได้ |
| `min_price` | `Option<Price>` | `None` | ราคา quote หรือ order ต่ำสุดที่ใช้ได้ |
| `margin_init` | `Option<Decimal>` | `0` | อัตรา initial margin |
| `margin_maint` | `Option<Decimal>` | `0` | อัตรา maintenance margin |
| `tick_scheme` | `Option<Ustr>` | `None` | ชื่อ tick scheme แบบแปรผันที่ลงทะเบียนไว้ |
| `info` | `Option<Params>` | `None` | metadata จาก adapter |

**หมายเหตุ**: builder ใช้ชื่อ `instrument_id` แต่ Rust เก็บค่าเดียวกันนี้ไว้ใน field `id`

## พฤติกรรม {#behavior}

- `Commodity` มี instrument class เป็น `Spot`
- อนุญาตให้ราคาติดลบได้ เพราะตลาด spot บางตลาด เช่น ไฟฟ้าหรือน้ำมัน ซื้อขายต่ำกว่าศูนย์ได้ `RiskEngine`
  ยอมรับราคาติดลบทั้งตอนส่งและแก้ไข order
- ไม่เป็น inverse เลย และ cost currency คือ quote currency
- ไม่มี field สำหรับ activation timestamp, expiry, strike, option kind หรือ settlement currency
- ใช้ `FuturesContract` สำหรับสัญญา commodity futures ที่ซื้อขายในตลาดและมีวันครบกำหนด

## ตัวอย่าง {#example}

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::AssetClass,
    identifiers::{InstrumentId, Symbol},
    instruments::Commodity,
    types::{Currency, Price, Quantity},
};

let gold = Commodity::builder()
    .instrument_id(InstrumentId::from("GOLD.COMEX"))
    .raw_symbol(Symbol::from("GOLD"))
    .asset_class(AssetClass::Commodity)
    .quote_currency(Currency::from("USD"))
    .price_precision(2)
    .size_precision(0)
    .price_increment(Price::from("0.01"))
    .size_increment(Quantity::from("1"))
    .lot_size(Quantity::from("1"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapters {#adapters}

adapter [Interactive Brokers](../../integrations/interactive_brokers.md) สร้างหรือใช้งาน instrument แบบ
`Commodity` สำหรับสัญญา spot commodity และโลหะ

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Futures Contract](futures_contract.md) อธิบาย futures ที่มีวันครบกำหนดซึ่งอ้างอิง commodity
- [Data](../data/) อธิบาย market data ที่อ้างอิงถึง instrument
