# Crypto Future

`CryptoFuture` แทนสัญญา crypto futures ที่มีวันครบกำหนด โดยอ้างอิง underlying ที่เป็น crypto,
quote ราคาด้วย quote currency, settle ด้วย settlement currency และหมดอายุ ณ timestamp ที่กำหนดไว้

ตัวอย่างเช่น futures ของ BTC หรือ ETH แบบมีวันครบกำหนดบน venue ซื้อขาย crypto derivatives

## Fields {#fields}

| Field | Rust type | Required/default | หมายเหตุ |
| --- | --- | --- | --- |
| `instrument_id` | `InstrumentId` | Required | ใน Rust เก็บไว้ใน field `id` |
| `raw_symbol` | `Symbol` | Required | symbol ดั้งเดิมของ venue |
| `underlying` | `Currency` | Required | crypto asset ที่สัญญาอ้างอิง |
| `quote_currency` | `Currency` | Required | สกุลเงินที่ใช้ quote ราคา |
| `settlement_currency` | `Currency` | Required | สกุลเงินที่ใช้ settle PnL และค่าธรรมเนียม |
| `is_inverse` | `bool` | Required | เป็น true เมื่อการคิดขนาด/ต้นทุนเป็นแบบ inverse |
| `activation_ns` | `UnixNanos` | Required | timestamp ที่สัญญาเริ่มใช้งาน |
| `expiration_ns` | `UnixNanos` | Required | timestamp ที่สัญญาหมดอายุ |
| `price_precision` | `u8` | Required | จำนวนทศนิยมที่อนุญาตสำหรับราคา |
| `size_precision` | `u8` | Required | จำนวนทศนิยมที่อนุญาตสำหรับขนาด order |
| `price_increment` | `Price` | Required | ขั้นราคาที่เล็กที่สุดที่ใช้ได้ |
| `size_increment` | `Quantity` | Required | ขั้นขนาดที่เล็กที่สุดที่ใช้ได้ |
| `multiplier` | `Quantity` | `1` | ตัวคูณของสัญญา |
| `lot_size` | `Quantity` | `1` | ขนาด lot หรือ board lot |
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
| `ts_event` | `UnixNanos` | Required | timestamp ของ event หน่วยนาโนวินาที |
| `ts_init` | `UnixNanos` | Required | timestamp ตอน initialize หน่วยนาโนวินาที |

**หมายเหตุ**: builder ใช้ชื่อ `instrument_id` แต่ Rust เก็บค่าเดียวกันนี้ไว้ใน field `id`

## พฤติกรรม {#behavior}

- `CryptoFuture` มี asset class เป็น `Cryptocurrency` และ instrument class เป็น `Future`
- ราคาเป็นศูนย์หรือติดลบได้ ยกเว้น inverse futures
- ใช้ `CryptoPerpetual` สำหรับ crypto derivatives ที่ไม่มีวันหมดอายุ

**รูปแบบการ settle**: ชุดสกุลเงินที่กำหนดเป็นตัวบอกรูปแบบการ settle

- **Linear**: โดยทั่วไปตั้ง `is_inverse=False` และ settle ด้วย quote currency
- **Inverse**: ตั้ง `is_inverse=True` และโดยทั่วไป settle ด้วยสกุลเงินของ underlying
- **Quanto**: settle ด้วยสกุลเงินที่สามซึ่งต่างจากทั้ง underlying และ quote

## ตัวอย่าง {#example}

```rust tab="Rust"
use jiff::Timestamp;
use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Symbol},
    instruments::CryptoFuture,
    types::{Currency, Money, Price, Quantity},
};

let activation: Timestamp = "2024-01-08T00:00:00Z".parse().unwrap();
let expiration: Timestamp = "2024-03-29T00:00:00Z".parse().unwrap();

let btcusdt_future = CryptoFuture::builder()
    .instrument_id(InstrumentId::from("BTCUSDT-240329.BINANCE"))
    .raw_symbol(Symbol::from("BTCUSDT-240329"))
    .underlying(Currency::from("BTC"))
    .quote_currency(Currency::from("USDT"))
    .settlement_currency(Currency::from("USDT"))
    .is_inverse(false)
    .activation_ns(UnixNanos::from(activation))
    .expiration_ns(UnixNanos::from(expiration))
    .price_precision(2)
    .size_precision(6)
    .price_increment(Price::from("0.01"))
    .size_increment(Quantity::from("0.000001"))
    .max_quantity(Quantity::from("9000.0"))
    .min_quantity(Quantity::from("0.000001"))
    .min_notional(Money::from("10.00 USDT"))
    .max_price(Price::from("1000000.00"))
    .min_price(Price::from("0.01"))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```

## Adapters {#adapters}

**adapter ที่สร้างหรือใช้งาน instrument แบบ `CryptoFuture` ได้แก่**:

- [Bybit](../../integrations/bybit.md) สำหรับตลาด crypto futures
- [Deribit](../../integrations/deribit.md) สำหรับ crypto futures แบบมีวันครบกำหนด
- [OKX](../../integrations/okx.md) สำหรับ crypto futures แบบมีวันครบกำหนด
- [Tardis](../../integrations/tardis.md) สำหรับ metadata ของ crypto futures

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Crypto Perpetual](crypto_perpetual.md) อธิบาย crypto perpetual futures
- [Futures Contract](futures_contract.md) อธิบายสัญญา futures ที่ไม่ใช่ crypto
