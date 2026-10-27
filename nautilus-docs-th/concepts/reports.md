---
title: "Reports"
---

คู่มือนี้อธิบายว่า report จาก `ReportProvider` ช่วยในการวิเคราะห์ portfolio, การทำบัญชี PnL และการวิเคราะห์ผลหลังรัน
backtest อย่างไร

> หมายเหตุ: ต้นฉบับมีตัวอย่างการสร้าง report เฉพาะ Python (pandas) จึงตัดออก หน้านี้คงไว้เฉพาะโครงสร้างของ report
> และแนวคิดที่ไม่ขึ้นกับภาษา

## ภาพรวม [#overview]

`ReportProvider` แปลง order, fill, position และ account state ที่อยู่ใน cache ให้เป็นตาราง (DataFrame)
สำหรับการวิเคราะห์และการแสดงผล report เหล่านี้ช่วยประเมินผลงานของ strategy วิเคราะห์คุณภาพการ execute
และตรวจสอบการทำบัญชี PnL report ชุดเดียวกันใช้ได้ทั้งใน backtest และ live trading ทำให้การประเมินผลงานและ
การเปรียบเทียบ strategy สอดคล้องกันทั้งสองแบบ

สร้าง report ได้สองแนวทาง:

- **เมธอดของ backtest**: `generate_orders_report()` และเมธอดในกลุ่มเดียวกันของ `BacktestEngine`
  จะอ่านจาก cache ของ engine เอง `BacktestNode` ก็มีเมธอดชุดเดียวกัน โดยรับ ID ของ run config เป็น argument แรก
- **ใช้ `ReportProvider` โดยตรง**: ส่งชุดของ order หรือ position ใดก็ได้ เช่น cache ของ live node
  หรือผลการ query cache ที่กรองแล้ว

ทุกเมธอดจะคืนตารางว่างเมื่อไม่มีข้อมูลที่ตรงเงื่อนไข

## Report ที่มีให้ใช้ [#available-reports]

`ReportProvider` มี static method ที่คืนตาราง (DataFrame) จากข้อมูลการเทรด แต่ละ report มี column และ index
ของตัวเองสำหรับการวิเคราะห์

### Orders report [#orders-report]

สร้างภาพรวมของ order ทั้งหมด column ได้แก่:

| Column            | คำอธิบาย                                            |
| ----------------- | -------------------------------------------------- |
| `client_order_id` | Index - ตัวระบุ order ที่ไม่ซ้ำกัน                        |
| `instrument_id`   | instrument ที่เทรด                                   |
| `strategy_id`     | strategy ที่สร้าง order                              |
| `trader_id`       | ตัวระบุ trader                                       |
| `account_id`      | ตัวระบุ account (ถ้ามีการกำหนด)                         |
| `venue_order_id`  | order ID ที่ venue กำหนด (ถ้าถูก accept)                 |
| `side`            | BUY หรือ SELL                                       |
| `type`            | MARKET, LIMIT ฯลฯ                                   |
| `status`          | สถานะปัจจุบันของ order                                 |
| `quantity`        | ปริมาณเดิมของ order (string)                          |
| `filled_qty`      | ปริมาณที่ fill แล้ว (string)                            |
| `price`           | ราคา limit (string ขึ้นกับประเภท order)                 |
| `avg_px`          | ราคา fill เฉลี่ย (string ถ้ามีการ fill)                   |
| `time_in_force`   | คำสั่ง time-in-force                                 |
| `ts_init`         | timestamp ตอน initialize order (Unix nanoseconds)  |
| `ts_last`         | timestamp ของการอัปเดตล่าสุด (Unix nanoseconds)         |

column เพิ่มเติมจะแตกต่างกันตามประเภท order เช่น `trigger_price` สำหรับ stop order และ `expire_time_ns`
สำหรับ order แบบ GTD

### Order fills report [#order-fills-report]

ให้สรุปของ order ที่ fill แล้ว (**หนึ่งแถวต่อหนึ่ง order**)

report นี้มีเฉพาะ order ที่ `filled_qty > 0` มี column เหมือน orders report แต่กรองเฉพาะ order ที่ execute แล้ว
และแปลง `ts_init` กับ `ts_last` เป็น datetime สำหรับการวิเคราะห์

### Fills report [#fills-report]

แสดงรายละเอียดของ fill event แต่ละรายการ (**หนึ่งแถวต่อหนึ่ง fill**) column ได้แก่:

| Column            | คำอธิบาย                                  |
| ----------------- | ---------------------------------------- |
| `client_order_id` | Index - ตัวระบุ order                      |
| `trade_id`        | ตัวระบุ trade/fill ที่ไม่ซ้ำกัน                  |
| `venue_order_id`  | order ID ที่ venue กำหนด                     |
| `instrument_id`   | instrument ที่เทรด                         |
| `strategy_id`     | strategy ที่สร้าง order                    |
| `account_id`      | ตัวระบุ account                            |
| `position_id`     | position ID ที่เกี่ยวข้อง (ถ้ามี)               |
| `order_side`      | BUY หรือ SELL                             |
| `order_type`      | ประเภท order (MARKET, LIMIT ฯลฯ)           |
| `last_px`         | ราคาที่ fill (string)                        |
| `last_qty`        | ปริมาณที่ fill (string)                      |
| `currency`        | สกุลเงินของ fill                            |
| `liquidity_side`  | MAKER หรือ TAKER                          |
| `commission`      | จำนวนและสกุลเงินของค่า commission (string)   |
| `ts_event`        | timestamp ของ fill (datetime)             |
| `ts_init`         | timestamp ตอน initialize (datetime)       |

report นี้ตัด column `type` ออก

### Positions report [#positions-report]

การวิเคราะห์ position รวมถึง snapshot ด้วย เมื่อสร้างจาก cache โดยตรง ให้ส่ง position snapshot ไปด้วย
เพราะจำเป็นสำหรับยอดรวมของ OMS แบบ `NETTING` (report จากการรัน backtest ที่เสร็จแล้วจะรวม snapshot ให้อัตโนมัติ)
column ได้แก่:

| Column             | คำอธิบาย                                               |
| ------------------ | ----------------------------------------------------- |
| `position_id`      | Index - ตัวระบุ position ที่ไม่ซ้ำกัน                        |
| `instrument_id`    | instrument ที่เทรด                                      |
| `strategy_id`      | strategy ที่ดูแล position                               |
| `trader_id`        | ตัวระบุ trader                                          |
| `account_id`       | ตัวระบุ account                                         |
| `opening_order_id` | order ID ที่เปิด position                                |
| `closing_order_id` | order ID ที่ปิด position                                 |
| `entry`            | ฝั่งที่เข้า (BUY หรือ SELL)                                  |
| `side`             | ฝั่งของ position (LONG, SHORT หรือ FLAT)                  |
| `quantity`         | ขนาด position ปัจจุบัน (string)                           |
| `peak_qty`         | ขนาดสูงสุดที่เคยถึง (string)                                |
| `avg_px_open`      | ราคาเข้าเฉลี่ย (decimal string)                           |
| `avg_px_close`     | ราคาออกเฉลี่ย (decimal string ถ้าปิดแล้ว)                   |
| `commissions`      | ค่า commission ที่จ่าย หนึ่งรายการต่อสกุลเงิน (list)              |
| `realized_pnl`     | กำไร/ขาดทุนที่ realize แล้วใน cost currency (string)          |
| `realized_return`  | ผลตอบแทนที่ realize แล้วเป็นอัตราส่วน (float) เช่น `0.05` คือ 5% |
| `ts_init`          | timestamp ตอน initialize position (Unix nanoseconds)  |
| `ts_opened`        | timestamp ตอนเปิด (datetime)                           |
| `ts_last`          | timestamp ของการอัปเดตล่าสุด (Unix nanoseconds)            |
| `ts_closed`        | timestamp ตอนปิด (datetime หรือ NA)                     |
| `duration_ns`      | ระยะเวลาของ position หน่วยนาโนวินาที                         |
| `is_snapshot`      | เป็น snapshot ในอดีตหรือไม่                                |

แถวที่เป็น snapshot ใช้ index เป็น ID ที่สร้างขึ้นจาก position ID เดิม ให้ใช้ `is_snapshot` แทน index
ในการแยกรอบที่ archive ไว้ออกจาก position ที่ยังเปิดอยู่ report นี้ตัด `signed_qty`, `base_currency`,
`quote_currency` และ `settlement_currency` ออก

### Account report [#account-report]

ติดตามการเปลี่ยนแปลงของ balance และ margin ของ account ตามเวลา

เมื่อสร้างจาก `BacktestEngine` ต้องระบุ `venue` หรือ `account_id` อย่างใดอย่างหนึ่ง หากไม่ระบุทั้งคู่จะเกิด error
หากระบุทั้งคู่ `account_id` จะมีลำดับความสำคัญกว่า และ account ที่ไม่รู้จักจะได้ตารางว่าง

column ได้แก่:

| Column          | คำอธิบาย                                      |
| --------------- | -------------------------------------------- |
| `ts_event`      | Index - timestamp ที่ account state เปลี่ยน     |
| `account_id`    | ตัวระบุ account                                |
| `account_type`  | ประเภท account (เช่น CASH, MARGIN)             |
| `base_currency` | base currency ของ account                     |
| `total`         | ยอด balance ทั้งหมด (string)                    |
| `free`          | balance ที่ใช้ได้ (string)                        |
| `locked`        | balance ที่ถูก lock ไว้ใน order (string)          |
| `currency`      | สกุลเงินของ balance                             |
| `reported`      | balance ถูกรายงานโดย venue หรือไม่               |
| `margins`       | ข้อมูล margin (list ถ้ามี)                       |
| `info`          | ข้อมูลเพิ่มเติมเฉพาะของ venue                     |

แต่ละแถวแทนรายการ balance หนึ่งรายการ account ที่มีหลายสกุลเงินจะได้หลายแถวต่อ account state event หนึ่งครั้ง

## ข้อควรพิจารณาในการทำบัญชี PnL [#pnl-accounting-considerations]

การทำบัญชี PnL ขึ้นอยู่กับการคำนวณ position, สกุลเงิน และ snapshot

### PnL ที่อิงตาม position [#position-based-pnl]

- **Realized PnL**: คำนวณเมื่อ position ถูกปิดบางส่วนหรือทั้งหมด
- **Unrealized PnL**: mark-to-market ด้วยราคาปัจจุบัน `Position.unrealized_pnl(last)` จะ mark position
  ที่เปิดอยู่ด้วย `Price` ที่กำหนด
- **ผลของ commission**: นับรวมเฉพาะเมื่ออยู่ใน cost currency ของ position ดู [Positions](positions.md)
  ว่า commission ที่เป็น base currency ของคู่ spot จะไปปรับขนาด position แทนอย่างไร

:::warning
position snapshot จะเก็บ PnL ในอดีตไว้เมื่อ position ที่ปิดแล้วถูกเปิดใหม่ภายใต้ ID เดิม ทั้งใน OMS แบบ `NETTING`
และ `HEDGING` **ให้รวม snapshot ไว้ใน report** เพื่อให้คำนวณ PnL รวมได้ถูกต้อง ดู
[Position snapshotting](positions.md#position-snapshotting) สำหรับพฤติกรรมการเปิดใหม่และการกลับฝั่ง
:::

### การทำบัญชีหลายสกุลเงิน [#multi-currency-accounting]

เมื่อทำงานกับหลายสกุลเงิน:

- แต่ละ position ติดตาม PnL ใน cost currency ของตัวเอง: quote สำหรับสัญญา linear, base สำหรับสัญญา inverse
  และ settlement สำหรับสัญญา quanto
- การรวมระดับ portfolio ต้องแปลงสกุลเงิน `Portfolio` ทำได้ด้วย `realized_pnls` โดยระบุ target currency
  ซึ่งใช้ exchange rate ใน cache ดู [Supported conversions](portfolio.md#supported-conversions)
- สกุลเงินของ commission อาจต่างจาก cost currency ของ position

### ข้อควรพิจารณาเรื่อง snapshot [#snapshot-considerations]

สำหรับ OMS แบบ `NETTING` ยอดรวมที่ถูกต้องของ instrument คือการบวก realized PnL ของแต่ละรอบที่ archive ไว้
เข้ากับ position ที่ยังเปิดอยู่ (รอบที่ archive ไว้ถูกเก็บภายใต้ ID ของ position ที่ยังเปิดอยู่ และควรรวมแยกตามสกุลเงิน)
ดู [Position snapshotting](positions.md#position-snapshotting) ว่า execution engine archive รอบที่ปิดแล้วอย่างไร

## การวิเคราะห์หลังรัน backtest [#backtest-post-run-analysis]

เมื่อ backtest จบ สามารถวิเคราะห์ผลได้จาก result statistics และ report ที่สร้างขึ้น
หรือเข้าถึง order, position และ position snapshot ใน cache ของ engine โดยตรงเพื่อวิเคราะห์ตามต้องการ

### สถิติของ portfolio [#portfolio-statistics]

ผลลัพธ์ของ backtest ให้ metric ด้านผลงาน แบ่งเป็นกลุ่ม `stats_pnls` (key ด้วยรหัสสกุลเงิน แล้วตามด้วยชื่อสถิติ),
`stats_returns` (key ด้วยชื่อสถิติ) และ `stats_general` (key ด้วยชื่อสถิติ)

สถิติแต่ละตัวจะมีส่วนในกลุ่มที่มันมีการคำนวณรองรับ ได้แก่ realized PnL, returns หรือ positions
สถิติที่เขียนเองสามารถอยู่ได้มากกว่าหนึ่งกลุ่ม

:::info
ดู [คู่มือ Portfolio](portfolio.md#portfolio-statistics) สำหรับชุดสถิติเริ่มต้น วิธีคำนวณของแต่ละกลุ่ม
และความแตกต่างระหว่าง position returns กับ portfolio returns
:::

:::info
report ถูกสร้างจากโครงสร้างข้อมูลในหน่วยความจำ สำหรับการวิเคราะห์ขนาดใหญ่หรือระบบที่รันต่อเนื่องเป็นเวลานาน
ควรพิจารณาบันทึก report ลงฐานข้อมูลเพื่อให้ query ได้อย่างมีประสิทธิภาพ ดูตัวเลือกด้าน persistence ได้ที่
[คู่มือ Cache](cache.md)
:::

## การทำงานร่วมกับ component อื่น [#integration-with-other-components]

`ReportProvider` ทำงานร่วมกับ component หลายตัวในระบบ:

- **Cache**: แหล่งข้อมูลการเทรดทั้งหมด (order, position, account) สำหรับ report
- **Portfolio**: คำนวณสถิติของตัวเองจากข้อมูลชุดเดียวกันใน cache อย่างอิสระ ไม่ได้คำนวณจาก report เหล่านี้
- **BacktestEngine**: มีเมธอดสร้าง report ที่ใช้วิเคราะห์ผลหลังรันและแสดงผล
- **Position snapshots**: จำเป็นสำหรับการรายงาน PnL ที่ถูกต้องใน OMS แบบ `NETTING`

## คู่มือที่เกี่ยวข้อง [#related-guides]

- [Visualization](visualization.md) - tearsheet และกราฟแบบ interactive จากผล backtest
- [Portfolio](portfolio.md) - สถิติของ portfolio และ metric ด้านผลงาน
- [Backtesting](backtesting/) - การรัน backtest ที่สร้าง report
- [Cache](cache.md) - ระบบ cache ที่เก็บข้อมูลสำหรับ report
