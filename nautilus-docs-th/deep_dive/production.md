---
title: "Production checklist"
description: "สิ่งที่ต้องเตรียมก่อนรัน strategy กับเงินจริง: config, secret, logging, monitoring, การ deploy และแผนรับมือเหตุฉุกเฉิน"
---

ระบบที่ผ่าน backtest และ testnet แล้วยังต่างจากระบบที่พร้อมใช้เงินจริงหลายเรื่อง ส่วนใหญ่ไม่เกี่ยวกับ logic การเทรด
แต่เกี่ยวกับสิ่งที่เกิดขึ้นเมื่อมีอะไรผิดพลาดตอนไม่มีใครเฝ้าหน้าจอ ใช้รายการนี้ตรวจก่อนขึ้น mainnet
และตรวจซ้ำทุกครั้งที่อัปเกรด NautilusTrader หรือเปลี่ยน strategy

## โค้ดและเวอร์ชัน [#code-and-versions]

- [ ] dependency ของ NautilusTrader pin ด้วย `rev` เดียวกันทุก crate และ `Cargo.lock` ถูก commit ไว้
- [ ] toolchain ล็อกด้วย `rust-toolchain.toml`
- [ ] binary ที่ deploy build จาก commit ที่ระบุได้ (ใส่ git hash ไว้ใน log ตอนเริ่ม หรือใน ID ของ node)
- [ ] `cargo test --release` และ `cargo build --release --features live` ผ่านใน CI ก่อน deploy ทุกครั้ง
- [ ] การอัปเกรด NautilusTrader ทำเป็นการเปลี่ยนแปลงแยก มี backtest เทียบผลก่อนและหลัง และรันบน testnet ก่อน

## Config และ secret [#config-and-secrets]

- [ ] พารามิเตอร์ของ strategy อยู่ใน config ไม่ได้เขียนตายตัวในโค้ด และ config ของ production ถูกเก็บใน version control
- [ ] API key อ่านจาก environment variable หรือ secret manager ไม่อยู่ใน repository, log หรือ command line
- [ ] API key ของ production ให้สิทธิ์เท่าที่ต้องใช้ (เทรดได้ แต่ **ถอนเงินไม่ได้**) และจำกัด IP ถ้า exchange รองรับ
- [ ] key ของ testnet กับ mainnet แยกกันชัดเจน (adapter Bybit ใช้ชื่อตัวแปรต่างกันอยู่แล้ว ดู [ไปสู่ live](live.md#credentials))
- [ ] `StrategyId`, `TraderId` และ `AccountId` คงที่ข้ามการ deploy

## ขีดจำกัดความเสี่ยง [#risk-limits]

- [ ] ขนาดต่อ order จำกัดสองชั้น: ใน config ของ strategy และใน risk engine ([Order และความเสี่ยง](orders_and_risk.md#risk-engine))
- [ ] rate limit ของ risk engine ต่ำกว่า rate limit ของ API exchange พอสมควร
- [ ] strategy ถอยเมื่อ order ถูก deny หรือ reject ไม่ส่งซ้ำทุกแท่ง
- [ ] position sizing เคารพ `min_quantity`, `max_quantity` และ increment ของ instrument
- [ ] กำหนดขาดทุนสูงสุดต่อวันไว้ และมีวิธีหยุดการเทรดเมื่อถึงขีดจำกัด (ทั้งในโค้ดและเป็นขั้นตอนที่คนทำได้)
- [ ] รู้ว่า exchange จัดการ position อย่างไรเมื่อ margin ไม่พอ (liquidation, ADL) และระบบรับ event เหล่านี้ได้

strategy ในบทนี้ยังไม่มีขีดจำกัดขาดทุนรายวัน เพิ่มได้โดยสะสม `realized_pnl` จาก `on_position_closed`
แล้วตรวจก่อนเข้าเทรดใหม่ใน `on_bar` แบบเดียวกับ cooldown

## Logging [#logging]

- [ ] log ระดับ `INFO` ขึ้นไปถูกเขียนลงไฟล์ หรือส่งเข้าระบบเก็บ log ไม่ได้อยู่แค่บนหน้าจอ
  (`LoggerConfig` มี `fileout_level` และ `file_config` สำหรับเขียนไฟล์ ส่ง config ให้ node ผ่าน `with_logging`)
- [ ] log ของแต่ละ node แยกกันได้ด้วย trader ID
- [ ] บรรทัด `WARN` และ `ERROR` ถูกนับและแจ้งเตือน ไม่ใช่รอให้คนเปิดอ่านเอง
- [ ] log ไม่มี API key หรือข้อมูลบัญชีที่ไม่ควรเปิดเผย

## Monitoring และการแจ้งเตือน [#monitoring]

| สัญญาณ | ตรวจอย่างไร | ความหมายเมื่อผิดปกติ |
| --- | --- | --- |
| process ยังทำงานอยู่ | process supervisor หรือ health check | node ตาย |
| บรรทัด status ตาม timer | log `Status:` ต้องมาตามรอบ ([timer](strategy.md#timers)) | node ค้าง หรือไม่ได้รับข้อมูล |
| bar ล่าสุด | ตัวนับ `bars` ต้องเพิ่มขึ้นทุกรอบ status | data feed หลุด |
| order ถูก reject | ตัวนับ `rejections` และบรรทัด `WARN` | config ไม่ตรงกฎของ exchange |
| position เทียบกับ exchange | ดูจากหน้าเว็บหรือ API ของ exchange เป็นระยะ | reconciliation ทำงานผิด |

ตัวนับที่ strategy ในบทนี้ log ไว้ (`bars`, `trades_closed`, `rejections`, `flat`) คือข้อมูลขั้นต่ำสำหรับ monitoring
ถ้าต้องการกราฟและ alert ที่จริงจังขึ้น ให้ส่งค่าเหล่านี้ออกไปยังระบบ metrics ที่ใช้อยู่

## การ deploy และการ restart [#deploy]

- [ ] node รันภายใต้ process supervisor (เช่น systemd หรือ container orchestrator) ที่ restart ให้เมื่อ process ตาย
- [ ] supervisor ส่ง **SIGTERM** ก่อน และรอนานกว่า `delay_post_stop` ก่อนจะ kill
  เพื่อให้ node ยกเลิก order และรับ event สุดท้ายได้ครบ ([หยุด node อย่างปลอดภัย](live.md#shutdown))
- [ ] ทดสอบแล้วว่า restart ระหว่างมี position เปิดอยู่ทำงานถูกต้อง (reconciliation และ
  [external order claim](live.md#external-orders))
- [ ] นาฬิกาของเครื่อง sync ด้วย NTP เวลาที่คลาดเคลื่อนทำให้ request ที่ต้องลงเวลาถูก exchange ปฏิเสธ
- [ ] เครื่องอยู่ใกล้ exchange ในแง่ network ถ้า strategy ไวต่อ latency
- [ ] มีขั้นตอน rollback: กลับไปใช้ binary และ config ชุดก่อนหน้าได้ภายในไม่กี่นาที

## เมื่อเกิดเหตุฉุกเฉิน [#incident]

เขียนขั้นตอนเหล่านี้ไว้ล่วงหน้า และซ้อมบน testnet อย่างน้อยหนึ่งครั้ง:

1. **หยุด strategy** ส่ง SIGTERM ให้ node แล้วรอให้ shutdown เสร็จ
2. **ตรวจที่ exchange โดยตรง** ว่ายังมี order หรือ position ค้างอยู่หรือไม่ อย่าเชื่อ log ของ node อย่างเดียว
3. **ปิดด้วยมือ** ถ้ายังมีของค้าง ปิดผ่านหน้าเว็บหรือ API ของ exchange
4. **ปิดสิทธิ์ของ API key** ถ้าสงสัยว่า key รั่วหรือระบบทำงานผิดแบบควบคุมไม่ได้
5. **เก็บหลักฐาน** log, config และ commit hash ที่ใช้ เพื่อหาสาเหตุก่อนเปิดระบบอีกครั้ง

## Backtest กับ live ต้องสอดคล้องกัน [#parity]

เมื่อรัน live ไประยะหนึ่ง ให้รัน backtest ด้วยข้อมูลช่วงเดียวกันแล้วเทียบผล:

- จำนวนเทรดใกล้เคียงกันไหม ถ้า live เทรดน้อยกว่ามาก แปลว่ามีสัญญาณที่ถูกบล็อกใน live (reject, cooldown, ข้อมูลหาย)
- ราคา fill ใน live แย่กว่า backtest เท่าไร ส่วนต่างนี้คือต้นทุนจริงที่ควรนำกลับไปตั้ง fill model และ fee model
- ผลต่างที่สะสมไปเรื่อย ๆ คือสัญญาณว่าสมมติฐานของ backtest ไม่ตรงกับตลาดจริง ต้องแก้ก่อนเพิ่มขนาดการเทรด

ดูความแตกต่างที่ framework ระบุไว้ได้ที่ [Backtest and live differences](../concepts/live.md#backtest-and-live-differences)
