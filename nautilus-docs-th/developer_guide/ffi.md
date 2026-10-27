---
title: "สัญญาด้านหน่วยความจำของ FFI"
---

NautilusTrader เปิด C foreign function interface (FFI) จากเพียง `nautilus-core` และ
`nautilus-model` เท่านั้น ทั้งสอง crate ซ่อน interface นี้ไว้หลัง Cargo feature `ffi` และเก็บ
module ที่ export ไว้ใน `crates/core/src/ffi/` และ `crates/model/src/ffi/`

crate อื่นใน workspace ใช้ Rust API แทน ส่วน crate `nautilus-plugin` ที่แยกต่างหากกำหนด
ABI สาธารณะสำหรับ guest plug-in และไม่ได้ใช้สัญญาด้านหน่วยความจำนี้ร่วมกัน

กฎด้านล่างนี้เข้มงวด การละเมิดอาจทำให้เกิด undefined behavior รวมถึง double free,
memory leak และการเข้าถึง pointer ที่ไม่ถูกต้อง

## การจัดการ panic [#panic-handling]

Rust panic ต้องไม่ unwind ข้ามฟังก์ชัน `extern "C"` เด็ดขาด ฟังก์ชันที่ export และอาจ panic ได้
ต้องส่งการทำงานผ่าน `nautilus_core::ffi::abort_on_panic` ซึ่งจะ log panic
และ abort process ก่อนที่การ unwind จะข้ามขอบเขตของ C

## ความเป็นเจ้าของ `CVec` [#cvec-ownership]

`CVec` แทน metadata ของการจัดสรรหน่วยความจำ (allocation) ของ vector ใน Rust ในรูปแบบที่เข้ากันได้กับ C
`CVec` ที่สร้างจาก `Vec<T>` จะโอนความเป็นเจ้าของ allocation แบบผู้เดียว (unique ownership) ให้กับผู้เรียกฝั่ง foreign
ใน Rust มันถูกออกแบบให้ไม่เป็นทั้ง `Copy` และ `Clone` โดยเจตนา แต่ฝั่ง C ยังคง copy ฟิลด์ของมันได้ ดังนั้น
ผู้เรียกต้องบังคับใช้กฎความเป็นเจ้าของแบบครั้งเดียวเท่านั้น (exactly-once) แบบเดียวกัน

| ขั้น | เจ้าของ  | การกระทำ                                                                                     |
| --- | ------- | ------------------------------------------------------------------------------------------- |
| 1   | Rust    | แปลง `Vec<T>` เป็น `CVec` ซึ่งเป็นการโอน allocation ให้ผู้เรียก                                      |
| 2   | Foreign | อ่าน element โดยไม่เปลี่ยนแปลง `ptr`, `len` หรือ `cap`                                            |
| 3   | Foreign | เรียกฟังก์ชัน `vec_drop_*` ที่ตรงกับ type นั้นเพียงครั้งเดียวเท่านั้นเพื่อคืน allocation                      |

หากลืม drop จะทำให้ allocation รั่ว (leak) การ drop allocation เดียวกันมากกว่าหนึ่งครั้งอาจทำให้
allocator เสียหายและ process crash ได้

ค่า `CVec` ที่ว่างจะมี `len == 0` และ `cap == 0` pointer ของมันเป็น sentinel แบบ opaque และห้าม
dereference ผู้ใช้ฝั่ง Rust ต้องใช้ `CVec::into_vec` ซึ่งจัดการกรณีว่างก่อนจะ
ตรวจสอบ pointer ผู้ใช้ที่ยืม (borrow) ข้อมูลต้องใช้ `CVec::as_slice` ด้วยเหตุผลเดียวกัน

ทั้งสอง method เป็น unsafe เพราะ metadata สาธารณะไม่สามารถพิสูจน์ที่มาของ allocation (provenance), alignment,
การ initialize หรือความเป็นเจ้าของแบบผูกขาดได้ ฟังก์ชันที่ export ใดๆ ที่รับ `CVec` จากผู้เรียก
และเรียก method ใด method หนึ่งในสองตัวนี้ต้อง:

- เป็น `unsafe extern "C" fn`
- ระบุหน้าที่ของผู้เรียกไว้ในส่วน `# Safety` ของเอกสาร
- ตรวจสอบ invariant ของ `len`, `cap` และ null pointer ก่อนสร้างข้อมูลกลับคืนหรือยืมข้อมูล
- ใช้ element type แบบ concrete ที่ตรงกับ allocation ของ `Vec<T>` ต้นทาง

## ฟังก์ชัน drop เฉพาะ type [#type-specific-drop-functions]

ไม่มี `cvec_drop` แบบ generic การสร้างทุก allocation กลับเป็น `Vec<u8>` จะทำให้ allocator ได้
layout ของ element ที่ผิดสำหรับ type อื่น vector ที่มีเจ้าของแต่ละตัวที่ข้ามขอบเขตต้องมีฟังก์ชัน drop
สำหรับ element type ที่ตรงกันพอดี เช่น `vec_drop_book_levels`, `vec_drop_book_orders` หรือ
`vec_drop_fills`

ให้เพิ่มฟังก์ชัน drop ไว้ข้างๆ ฟังก์ชันที่สร้างข้อมูล (producer) เพื่อให้การ review ตรวจสอบทั้งคู่ไปพร้อมกันได้ test ต้องครอบคลุม
sentinel ของค่าว่างและการตรวจสอบ metadata ที่ฝั่งผู้ใช้ implement ไว้

## Buffer ที่ยืมมาจากฝั่ง foreign [#borrowed-foreign-buffers]

หน่วยความจำที่จัดสรรนอก Rust ต้องไม่ถูกสร้างกลับเป็น `Vec<T>` ให้ยืมผ่าน
`CVec::as_slice` และ copy ด้วย `to_vec()` เมื่อ Rust ต้องการพื้นที่เก็บที่เป็นเจ้าของเอง ผู้เรียกฝั่ง foreign ยังคง
เป็นเจ้าของและต้องคืน buffer ต้นฉบับด้วย allocator ที่สร้างมันขึ้นมา

## Opaque pointer [#opaque-pointers]

model object ที่ layout ไม่ได้เป็น `repr(C)` จะข้าม ABI ในรูปของ opaque pointer ที่เป็นเจ้าของ (owning)
header ที่ generate ขึ้นจะ forward-declare type นั้นไว้ ผู้เรียกฝั่ง foreign จึงจัดการมันได้ผ่านฟังก์ชันที่ export เท่านั้น:

- constructor คืนค่า `*mut T` ที่เป็นเจ้าของ จาก `Box::into_raw`
- accessor แบบ pure รับ reference แบบ `&T` หรือ `&mut T`
- ทุก constructor ต้องมีฟังก์ชัน drop ที่คู่กัน

ตัวอย่างเช่น:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn orderbook_new(id: InstrumentId, book_type: BookType) -> *mut OrderBook {
    Box::into_raw(Box::new(OrderBook::new(id, book_type)))
}

/// # Safety
///
/// `book` must be a live owning pointer returned by [`orderbook_new`], and must not
/// be used after this call.
///
/// # Panics
///
/// Panics if `book` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn orderbook_drop(book: *mut OrderBook) {
    abort_on_panic(|| {
        assert!(!book.is_null(), "`book` was NULL");
        // SAFETY: Caller guarantees `book` was allocated by `orderbook_new`
        drop(unsafe { Box::from_raw(book) });
    });
}
```

เจ้าของฝั่ง foreign ต้องเรียกฟังก์ชัน drop เพียงครั้งเดียวเท่านั้น ห้าม copy pointer แล้วนำทั้งสองสำเนา
ไปใช้ (consume) และห้ามใช้ pointer หลังจาก drop แล้ว

## Checklist สำหรับการ review [#review-checklist]

สำหรับ FFI export ที่เพิ่มใหม่หรือมีการเปลี่ยนแปลงแต่ละตัว:

- เก็บ implementation ไว้ใน `nautilus-core` หรือ `nautilus-model`
- ใช้ `repr(C)` กับทุก type ที่ layout ข้ามขอบเขต
- ป้องกันไม่ให้ panic unwind ข้ามขอบเขต
- จับคู่ allocation ที่มีเจ้าของแต่ละตัวกับเส้นทางการคืนหน่วยความจำเฉพาะ type หนึ่งเส้นทาง
- ระบุหน้าที่ด้าน pointer และความเป็นเจ้าของให้ครบถ้วนในเอกสาร `# Safety`
- เพิ่ม test ที่เจาะจงสำหรับกฎความเป็นเจ้าของและการตรวจสอบความถูกต้อง
