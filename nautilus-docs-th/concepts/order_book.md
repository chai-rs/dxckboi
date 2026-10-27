---
title: "Order Book"
---

NautilusTrader implement order book ด้วย Rust `OrderBook` ดูแล market depth สาธารณะของ instrument หนึ่งตัว
ส่วน `OwnOrderBook` ติดตาม order ของคุณเองแยกไว้ต่างหาก เพื่อให้ view แบบกรอง (filtered view) หักออกจาก
liquidity สาธารณะได้

:::note
คู่มือนี้ใช้ Rust model API สำหรับการทำงานกับ book ดูรายละเอียด API ได้ที่
[Rust API reference](https://nautilustrader.io/docs/core-latest/)
:::

## Book type [#book-types]

`OrderBook` ถูกดูแลแยกตาม instrument ทั้งใน backtest และ live trading:

- `L3_MBO`: ข้อมูล Level 3 market-by-order (MBO) ติดตามทุก order ในแต่ละ price level โดยใช้ order ID เป็น key
  ในแต่ละฝั่งของ book order ID หนึ่งตัวจะผูกกับ price level เพียงระดับเดียว ถ้า ID เดิมมาซ้ำที่ราคาอื่น order
  จะถูกย้ายไป level ใหม่ input แบบ MBP ใช้ ID ที่สร้างจากราคา และ order ID ที่เป็นศูนย์ก็หมายถึงไม่มี identity
  เช่นกัน ยกเว้น input แบบ top-of-book ที่ใช้ฝั่งของ order เป็น ID
- `L2_MBP`: ข้อมูล Level 2 market-by-price (MBP) รวม order ตาม price level (หนึ่ง entry ต่อหนึ่งราคา)
- `L1_MBP`: ข้อมูล Level 1 market-by-price (MBP) เฉพาะ top-of-book หรือที่เรียกว่า best bid and offer (BBO)
  เก็บเฉพาะราคาที่ดีที่สุด

:::note
ข้อมูล quote, trade และ bar (`QuoteTick`, `TradeTick` และ `Bar`) ก็ใช้ขับเคลื่อน book แบบ `L1_MBP` ได้
:::

## การ subscribe ข้อมูล book [#subscribing-to-book-data]

strategy และ actor subscribe การอัปเดตของ order book ได้สามแบบ:

- book delta แบบ incremental
- depth snapshot (ใช้ค่า default ของ adapter และอยู่ภายใต้ข้อจำกัดของ venue)
- full book snapshot ตามช่วงเวลาที่กำหนด

subscription แต่ละแบบส่งข้อมูลไปยัง handler ที่ตรงกัน (`on_book_deltas`, `on_book_depth` และ `on_book`)

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

### Managed book และ subscription ที่ใช้ร่วมกัน [#managed-books-and-shared-subscriptions]

data engine ดูแล `OrderBook` ที่ cache ไว้หนึ่งตัวต่อ instrument subscription แบบ managed จะเลือกแหล่งอัปเดต
ได้หนึ่งแบบ: `OrderBookDeltas` หรือ `OrderBookDepth` subscription แบบ delta และแบบ interval ใช้ book ที่
managed ด้วย delta ร่วมกันได้ แต่ depth subscription แบบ managed อยู่ร่วมกับ delta แบบ managed หรือ interval
subscription ของ instrument เดียวกันไม่ได้ engine จะปฏิเสธ request ที่ขัดแย้งกัน

ถ้าต้องการรับ callback ของ depth ควบคู่กับ delta แบบ managed หรือ interval book ให้ตั้ง `managed=False` บน depth
subscription callback เหล่านั้นจะไม่อัปเดต book ใน cache ถ้ามีเฉพาะ depth subscription ให้ใช้ `managed=True`
เพื่อดูแล book ใน cache จาก depth snapshot

consumer ที่ใช้แหล่งข้อมูลเดียวกันต้องตกลงกันเรื่อง client, book type, depth และพารามิเตอร์ของ subscription
`depth=None` หมายถึงใช้ค่า default ของ adapter และไม่ถือเป็น wildcard ที่ match กับ depth ที่ระบุชัดเจน client
ต่างกันใช้ config ต่างกันได้เมื่อ consumer ทุกตัวของแหล่งนั้นเป็นแบบ unmanaged การ unsubscribe consumer ตัวหนึ่ง
จะยังคงแหล่งข้อมูลไว้ตราบที่ consumer อื่นยังต้องใช้

การส่งแบบ interval จะ subscribe delta แล้ว publish book ใน cache ตาม timer event `OrderBookDepth` ไม่อัปเดต
book ที่ managed ด้วย delta นั้น รวมถึงระหว่าง backtest ด้วย

- **Replay แบบ depth อย่างเดียว**: ใช้ `subscribe_book_depth` และ `on_book_depth`
- **การส่งแบบ interval**: ป้อน `OrderBookDeltas` โดยแปลง depth snapshot เป็น delta ที่ติด flag snapshot ก่อน
  replay หากจำเป็น

ระหว่างที่ feed ขัดข้องหรือกำลังกู้คืน timer ของ interval อาจยัง publish book ล่าสุดใน cache ต่อไป

## การเข้าถึง book [#accessing-the-book]

`OrderBook` มี accessor สำหรับ top-of-book:

```rust
let best_bid: Option<Price> = book.best_bid_price();
let best_ask: Option<Price> = book.best_ask_price();
let spread: Option<f64> = book.spread();
let midpoint: Option<f64> = book.midpoint();
```

## Method สำหรับวิเคราะห์ [#analysis-methods]

`OrderBook` รองรับการวิเคราะห์ market depth และการจำลอง execution:

```rust
// Average fill price for a given quantity
let avg_fill_px = book.get_avg_px_for_quantity(quantity, OrderSide::Buy);

// Average price, filled quantity, and worst price for a target exposure
let (avg_px, filled_qty, worst_px) =
    book.get_avg_px_qty_for_exposure(target_exposure, OrderSide::Buy);

// Cumulative quantity available at or better than a price
let qty = book.get_quantity_for_price(price, OrderSide::Buy);

// Quantity at a specific price level only
let qty = book.get_quantity_at_level(price, OrderSide::Buy, 2);

// Simulate fills against the book
let fills: Vec<(Price, Quantity)> = book.simulate_fills(&order);

// All crossed levels regardless of order quantity
let levels = book.get_all_crossed_levels(OrderSide::Buy, price, 2);
```

## การตรวจสอบความถูกต้อง (integrity check) [#integrity-checks]

เรียก `book_check_integrity` เพื่อตรวจว่า state ของ book สอดคล้องกับ type ของมัน:

- **L1_MBP**: แต่ละฝั่งมีได้ไม่เกินหนึ่ง level
- **L2_MBP**: แต่ละ price level มีได้ไม่เกินหนึ่ง order
- **L3_MBO**: ไม่มีข้อจำกัดเพิ่มเติมต่อ level หลาย order อยู่ที่ราคาเดียวกันได้
- **ทุก type**: best bid ต้องไม่เกิน best ask (crossed book) ตลาดแบบ locked (bid == ask) ถือว่าถูกต้อง

นี่เป็น **การตรวจแบบ explicit**: การ apply delta จะไม่เรียกฟังก์ชันนี้ method `apply_delta` และ `apply_deltas`
ของ Rust ตรวจ instrument ID ที่เข้ามาเทียบกับ book แยกต่างหาก และคืน `BookIntegrityError::InstrumentMismatch`
เมื่อไม่ตรงกัน

สำหรับ order ID ที่ไม่เป็นศูนย์ delta ที่มี side เป็น `None` จะพยายามหาฝั่งจาก ladder cache ก่อน ถ้าไม่มีฝั่งใน cache
`Add` จะคืน `BookIntegrityError::NoOrderSide` ส่วน `Update` หรือ `Delete` จะถูกข้ามไป ถ้า ID มีอยู่ทั้งสองฝั่ง
`Add` จะคืน `BookIntegrityError::AmbiguousOrderSide` ส่วน `Update` หรือ `Delete` จะถูกข้ามพร้อม warning

delta และ depth snapshot ที่มาผิดลำดับจะ **ถูก apply แทนที่จะถูกปฏิเสธ** ดังนั้น venue ที่ replay หรือสลับลำดับ event
ก็ยังได้ state ตามที่ event เหล่านั้นอธิบาย มีเพียง metadata ของ book ที่ได้รับการปกป้อง: `ts_last` ไม่มีวันถอยหลัง
และ `sequence` ไม่มีวันถอยหลัง ยกเว้นข้าม full clear ตามที่อธิบายด้านล่าง

update ที่ stale จะ log warning หนึ่งครั้งต่อฟิลด์ที่ถอยหลัง โดยนับ `sequence` และ `ts_event` แยกกัน ความถี่ของ
warning ขึ้นกับรูปแบบที่ update มาถึง:

- **Incremental delta**: หนึ่งครั้งต่อ delta ที่ stale
- **Snapshot delta**: หนึ่งครั้งต่อ snapshot ไม่ว่าจะมาเป็น batch `F_SNAPSHOT` หรือเป็น delta `F_SNAPSHOT`
  ตัวเดียว ทุก delta ในการ rebuild ใช้ sequence และ timestamp เดียวกับ snapshot
- **Depth snapshot**: หนึ่งครั้ง เพราะ `OrderBookDepth` แทนที่ book ทั้งหมดใน update เดียว

feed ของบาง venue จะเริ่มนับ sequence ใหม่เมื่อ clear book:

- **Full clear ที่ไม่มี `F_SNAPSHOT`**: clear จะถูกตรวจเทียบกับ sequence high-water เดิม จากนั้น sequence ของมัน
  จะกลายเป็น high-water ใหม่ delta ถัดไปจะถูกเทียบกับค่านั้นแทนค่าที่ได้รับก่อน clear method `clear()` สาธารณะ
  ใช้พฤติกรรมนี้
- **Clear ที่ติด flag snapshot**: คง high-water ปัจจุบันไว้
- **Side clear**: `clear_bids()` และ `clear_asks()` คง high-water ปัจจุบันไว้

การรายงานของ snapshot อธิบาย snapshot ที่เข้ามา จึงไม่ขึ้นกับว่าแต่ละ delta ถึง book หรือไม่ ข้อยกเว้นของทั้งหมดนี้คือ
book แบบ `L1_MBP` ที่ขับเคลื่อนด้วย quote หรือ trade โดย `QuoteTick` หรือ `TradeTick` ที่ stale จะถูกข้ามพร้อม
warning และ book จะไม่เปลี่ยนแปลง

## การแสดงผลแบบอ่านง่าย (pretty printing) [#pretty-printing]

ทั้ง `OrderBook` และ `OwnOrderBook` มี method `pprint` ที่คืน book เป็นตารางที่คนอ่านได้:

```rust
println!("{}", book.pprint(5, None));
println!("{}", book.pprint(5, Some(Decimal::new(1, 2)))); // group_size = 0.01
```

พารามิเตอร์ `group_size` จัด price level เป็นกลุ่มที่หยาบขึ้นสำหรับ instrument ที่มี tick size ละเอียด ผลลัพธ์เป็น
ตารางที่มี bid อยู่ซ้าย ราคาอยู่ตรงกลาง และ ask อยู่ขวา

## Own order book [#own-order-book]

`OwnOrderBook` ติดตาม working order ของคุณเองแยกจาก book สาธารณะ strategy ประเภท market making และ
quoting อื่น ๆ ใช้มันประเมิน liquidity ที่เหลือในแต่ละ price level หลังหัก order ของตัวเองออก

execution engine จะดูแล own book เมื่อเปิด `manage_own_order_books` cache จะอัปเดต own book ที่มีอยู่เมื่อ
order event เปลี่ยน state order ที่มีสิทธิ์ต้องมีราคา ไม่ใช้ time in force แบบ `IOC` หรือ `FOK` และไม่ได้ถูก
order emulator ถือไว้

- **Emulated order**: ไม่เคยวางอยู่ใน book สาธารณะ จึงเข้ามาเฉพาะตอนถูก release
- **Order แบบ quote quantity**: เข้ามาเมื่อ update แปลง quantity เป็นหน่วย base แล้ว
- **Terminal event**: อาจยังล้าง entry ที่มีอยู่ใน own book ได้ แม้ order นั้นจะไม่มีสิทธิ์ถูกติดตามในกรณีปกติ

### Lifecycle ของ order [#order-lifecycle]

`OwnOrderBook` ติดตาม order ตลอด lifecycle:

- **การเพิ่ม**: order ถูกเพิ่มตอน submit หรือสร้างขึ้นจาก reconciliation order ที่ส่งไปยัง external execution
  client จะเข้ามาเมื่อมี order event แรก
- **การอัปเดต**: state ที่ยังไม่ terminal เช่น `OrderStatus::Accepted`, `OrderStatus::PendingUpdate`,
  `OrderStatus::PendingCancel` และ `OrderStatus::PartiallyFilled` จะอัปเดต entry
- **การลบ**: state ที่ปิดแล้วคือ `OrderStatus::Denied`, `OrderStatus::Rejected`, `OrderStatus::Canceled`,
  `OrderStatus::Expired`, `OrderStatus::Filled` และ `OrderStatus::Voided` จะลบ entry ออก

`OwnBookOrder` แต่ละตัวมี:

- `trader_id`: Trader ID ที่เป็นเจ้าของ order
- `client_order_id`: Client order ID ที่ใช้ reconcile own book กับ state ใน cache
- `venue_order_id`: Venue order ID เมื่อได้รับการกำหนดแล้ว
- `side`, `price` และ `size`: ฝั่ง, ราคา และ quantity ที่เหลือ (leaves) ของ order
- `order_type` และ `time_in_force`: metadata ของ order ที่เก็บไว้เพื่อตรวจดู
- `status`: สถานะปัจจุบันของ order เช่น `SUBMITTED`, `ACCEPTED` หรือ `PENDING_CANCEL`
- `ts_last`: timestamp ของ order event ล่าสุดที่ apply กับ order ใน own book นี้
- `ts_accepted`: timestamp ที่ venue accept order หรือศูนย์ถ้ายังไม่ถูก accept
- `ts_submitted`: timestamp ที่ submit order หรือศูนย์ถ้ายังไม่ submit
- `ts_init`: timestamp ที่ initialize order

ฟิลด์ `status` และ `ts_accepted` ใช้ขับเคลื่อน filter แบบ optional ที่อธิบายใน
[การกรองตาม status และเวลา](#status-and-time-filtering)

### การตรวจสอบ (auditing) [#auditing]

method `audit_open_orders` reconcile own book กับชุดของ client order ID ที่ถูกต้อง order ใน own book ที่ไม่อยู่ในชุด
ที่ให้มาจะถูกลบพร้อม warning `Cache::audit_own_order_books` สร้างชุดนี้จาก order ที่ open, in-flight และ
active-local เพื่อให้ entry ที่ยังไม่ terminal คงอยู่ระหว่างการประมวลผล event ตามปกติและช่วง latency ของ venue
ระบบ live สามารถรัน audit นี้เป็นระยะผ่าน own-books audit interval

### การ query [#querying]

```rust
// Check if a specific order is tracked
let in_book = own_book.is_order_in_book(&client_order_id);

// Get all tracked order IDs per side
let bid_ids = own_book.bid_client_order_ids();
let ask_ids = own_book.ask_client_order_ids();

// Aggregated quantities per price level
let bid_qty = own_book.bid_quantity(None, None, None, None, None);
let ask_qty = own_book.ask_quantity(None, None, None, None, None);

// Pretty print
println!("{}", own_book.pprint(5, None));
```

### Filtered view [#filtered-views]

หัก order ของคุณเองออกจาก book สาธารณะเพื่อดู liquidity สุทธิที่เหลือ:

```rust
// Filtered maps of price -> quantity (own orders subtracted)
let net_bids = book.bids_filtered_as_map(Some(10), Some(&own_book), None, None, None);
let net_asks = book.asks_filtered_as_map(Some(10), Some(&own_book), None, None, None);

// Full filtered OrderBook with all analysis methods available
let filtered = book.filtered_view(Some(&own_book), Some(10), None, None, None);
let avg_px = filtered.get_avg_px_for_quantity(quantity, OrderSide::Buy);
```

method `filtered_view` คืน `OrderBook` ใหม่ที่หักขนาด order ของคุณออกแล้ว book สุทธินี้รองรับ method วิเคราะห์
ครบชุด (`spread`, `midpoint`, `get_avg_px_for_quantity` และอื่น ๆ)

### การกรองตาม status และเวลา [#status-and-time-filtering]

filtered view รองรับการกรอง own order ตาม status และเวลาแบบ optional:

```rust
let statuses = AHashSet::from([OrderStatus::Accepted]);

// Only subtract ACCEPTED orders (ignore SUBMITTED, PENDING_CANCEL, etc.)
let filtered = book.filtered_view(Some(&own_book), None, Some(&statuses), None, None);
```

พารามิเตอร์ `accepted_buffer_ns` ให้ช่วงผ่อนผัน (grace period) เมื่อกำหนด `ts_now` view จะรวม own order
เฉพาะเมื่อ `ts_accepted + accepted_buffer_ns <= ts_now` วิธีนี้ตัด order ที่เพิ่งถูก accept ซึ่งอาจยังไม่ปรากฏใน
feed ของ book สาธารณะ

การตรวจเวลามีผลไม่ว่า order จะมี status ใด จึงควรใช้ร่วมกับ status filter เพื่อตัด order ที่ยังไม่ถูก accept ออก
การไม่ระบุ `ts_now` จะปิดการกรองตามเวลา accept และ `accepted_buffer_ns` ที่เป็นบวกต้องมี `ts_now` เสมอ

```rust
// Only subtract orders accepted at least 500ms ago
let filtered = book.filtered_view(
    Some(&own_book),
    None,
    None,
    Some(500_000_000),
    Some(clock.timestamp_ns().as_u64()),
);
```

## Binary market [#binary-markets]

binary market อาจมี instrument ของผลลัพธ์ที่เป็นคู่ตรงข้ามกัน เช่น token YES และ NO ของ Polymarket สำหรับคู่
ที่ทราบว่าเป็นคู่ตรงข้ามกัน parity transform จะแปลงราคา `p` ของผลลัพธ์หนึ่งเป็น `1 - p` ของอีกผลลัพธ์ ภายใต้
transform นี้ NO bid ที่ 0.40 จะกลายเป็น YES ask ที่ 0.60

method `OwnOrderBook::combined_with_opposite` จัดการการแปลงนี้ โดยรวม order จาก instrument ของทั้งสองผลลัพธ์
เข้าเป็น view ของ book แรก:

```rust
let yes_own = own_yes_book
    .cloned()
    .unwrap_or_else(|| OwnOrderBook::new(yes_instrument_id));

let no_own = own_no_book
    .cloned()
    .unwrap_or_else(|| OwnOrderBook::new(no_instrument_id));

// Merge NO orders with the parity price transform (1 - price)
let combined = yes_own.combined_with_opposite(&no_own).unwrap();

// Filter the public YES book using the combined own book
let filtered = book.filtered_view(Some(&combined), None, None, None, None);
```

การแปลงทำงานดังนี้:

- NO ask ที่ราคา `p` จะกลายเป็น bid ที่ราคา `1 - p` ใน book ที่รวมแล้ว
- NO bid ที่ราคา `p` จะกลายเป็น ask ที่ราคา `1 - p` ใน book ที่รวมแล้ว

:::warning
method นี้ปฏิเสธ instrument ID ที่ซ้ำกัน แต่ตรวจไม่ได้ว่า instrument ทั้งสองเป็นคู่ตรงข้ามกันจริง ผู้เรียกต้องส่ง
instrument ฝั่งตรงข้ามที่ถูกต้องเอง own book ที่ได้สามารถใช้กรอง book YES สาธารณะเทียบกับ order ของคุณใน
instrument ของผลลัพธ์ใดก็ได้
:::
