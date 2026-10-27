---
title: "ข้อมูลและ venue สำหรับ backtest"
---

## ข้อมูล [#data]

ข้อมูลย้อนหลังเป็นตัวเดินนาฬิกาของ backtest, อัปเดตสถานะตลาดจำลอง และขับเคลื่อน callback ของ strategy
`book_type` ของ venue เป็นตัวกำหนดว่าข้อมูลประเภทใดบ้างที่อัปเดต book ที่ใช้ matching ได้ ดังนั้น config
จึงต้องสอดคล้องกับข้อมูลที่มีอยู่

ข้อมูล order book ให้รายละเอียดด้าน execution มากกว่า quote, trade หรือ bar แต่แม้จะเป็น book ที่บันทึกไว้
ก็ไม่สามารถบอกได้ว่า order จำลองจะเปลี่ยนแปลงตลาดอย่างไร NautilusTrader รองรับข้อมูลต่อไปนี้
เรียงจากละเอียดมากไปน้อย:

```mermaid
flowchart LR
    L3["L3 Order Book<br/>(market-by-order)"]
    L2["L2 Order Book<br/>(market-by-price)"]
    L1["L1 Quotes<br/>(top of book)"]
    T["Trades"]
    B["Bars"]

    L3 --> L2 --> L1 --> T --> B

    style L3 fill:#2d5a3d,color:#fff
    style L2 fill:#3d6a4d,color:#fff
    style L1 fill:#4d7a5d,color:#fff
    style T fill:#5d8a6d,color:#fff
    style B fill:#6d9a7d,color:#fff
```

ข้อมูลที่ละเอียดกว่าจะเผยให้เห็นคิวและความลึกที่บันทึกไว้ได้มากกว่า ส่วนข้อมูลที่ละเอียดน้อยกว่าต้องอาศัย
สมมติฐานในการจำลองมากขึ้น

- **ข้อมูล L3 order book (market-by-order)**: order แต่ละตัวในทุกระดับราคาที่บันทึกไว้
- **ข้อมูล L2 order book (market-by-price)**: ขนาดรวมในแต่ละระดับราคาที่บันทึกไว้
- **L1 quote tick (market-by-price)**: ราคาและขนาดของ best bid และ best ask
- **Trade tick**: การ execute ที่บันทึกไว้
- **Bar**: ราคาและ volume ที่ aggregate ในช่วงเวลาคงที่

### การเลือกข้อมูล: ต้นทุนเทียบกับความแม่นยำ [#choosing-data-cost-vs-accuracy]

ข้อมูล bar อาจเพียงพอสำหรับการพัฒนา strategy ในช่วงแรก และมักหาได้ง่ายและถูกกว่าข้อมูล tick หรือ
order book แต่ bar ไม่สามารถบอกลำดับราคาภายใน bar, spread, ความลึก หรือตำแหน่งในคิวได้ strategy
ที่อ่อนไหวต่อ execution จึงต้องตรวจสอบด้วยข้อมูลที่ละเอียดกว่า

:::tip
เริ่มด้วย bar เพื่อทดสอบ signal หลักเมื่อเหมาะสม แล้วจึงเปลี่ยนไปใช้ข้อมูล quote, trade หรือ depth
ก่อนจะเชื่อถือผลลัพธ์ที่ขึ้นกับ spread, ลำดับราคาภายใน bar ที่แน่นอน, การออกจาก position ที่แคบ หรือ
ตำแหน่งในคิว
:::

## Venue [#venues]

เมื่อ initialize venue สำหรับ backtest ให้ระบุ `book_type` ของ order book ภายในที่ใช้ประมวลผล execution:

- `L1_MBP`: Level 1 market-by-price (ค่าเริ่มต้น) เก็บเฉพาะระดับบนสุดของ order book
- `L2_MBP`: Level 2 market-by-price เก็บความลึกของ order book โดยรวม order เป็นหนึ่งตัวต่อหนึ่งระดับราคา
- `L3_MBO`: Level 3 market-by-order เก็บความลึกของ order book โดยติดตาม order ทุกตัวตามที่ข้อมูลให้มา

`book_type` เป็นตัวกำหนดว่าข้อมูลใดจะอัปเดตสถานะ book และขับเคลื่อนการ matching สำหรับข้อมูลที่ไม่เกี่ยวกับ
book ที่เลือก:

- Matching engine จะไม่นำไปใช้อัปเดต book และราคา
- นาฬิกาชั้นนอกของ backtest ยังคงเดินต่อ
- Strategy ยังคงได้รับข้อมูลที่ subscribe ไว้ผ่าน data engine

การตรวจสอบ precision ขึ้นกับเส้นทางของการ matching เช่น bar ที่ถูก venue แบบ L2 หรือ L3 ละเลย
จะ return ก่อนถึงขั้นตรวจสอบ precision ของ bar ที่ใช้ execute

| Data type         | L1_MBP            | L2_MBP            | L3_MBO            |
| ----------------- | ----------------- | ----------------- | ----------------- |
| `QuoteTick`       | อัปเดต book        | *ละเลย*           | *ละเลย*           |
| `TradeTick`       | Trigger matching  | Trigger matching  | Trigger matching  |
| `Bar`             | อัปเดต book        | *ละเลย*           | *ละเลย*           |
| `OrderBookDelta`  | *ละเลย*           | อัปเดต book        | อัปเดต book        |
| `OrderBookDeltas` | *ละเลย*           | อัปเดต book        | อัปเดต book        |
| `OrderBookDepth`  | อัปเดต book        | อัปเดต book        | อัปเดต book        |

ความละเอียดของข้อมูลต้องตรงกับ `book_type` ของ order Nautilus **ไม่สามารถสร้างข้อมูลที่ละเอียดกว่า**
(L2 หรือ L3) จากข้อมูลระดับต่ำกว่า เช่น quote, trade หรือ bar ได้

:::warning[ข้อมูล book แบบ L2 และ L3]
หากระบุ `L2_MBP` หรือ `L3_MBO` เป็น `book_type` ของ venue, quote และ bar จะไม่อัปเดต book
ต้องป้อนข้อมูล order book delta ให้ด้วย มิฉะนั้น order อาจดูเหมือนไม่เคยถูก fill เลย
:::

:::warning[L1 ละเลย order book delta]
เมื่อใช้ `L1_MBP` (ค่าเริ่มต้น) matching engine จะละเลย order book delta หาก subscribe order book delta
ให้ตั้ง `book_type` ของ venue เป็น `L2_MBP` หรือ `L3_MBO` เรื่องนี้ใช้กับ sandbox execution ด้วย
เนื่องจาก matching engine ที่นั่นใช้ config `book_type` แบบเดียวกัน
:::
