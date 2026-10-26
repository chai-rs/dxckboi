# Simulation Modules

Simulation module มีทั้งรูปแบบ enum และรูปแบบ shared handle ตามที่อธิบายไว้ใน
[การออกแบบ behavioral model](../behavioral_models.md) ขอบเขตของการตั้งค่า (configuration boundary)
เป็นตัวกำหนดว่าจะเก็บรูปแบบใด:

| ขอบเขต                                         | รูปแบบที่เก็บ             | implementation ที่รับได้                     |
| ---------------------------------------------- | ------------------------ | -------------------------------------------- |
| `BacktestVenueConfig` แบบ declarative          | `SimulationModuleAny`    | โมดูลในตัวและ language bridge                |
| `SimulatedVenueConfig` และ `SimulatedExchange` | `SimulationModuleHandle` | implementation ของ Rust trait ใดก็ได้ที่ link เข้ามา |

:::warning[state ของโมดูลที่ใช้ร่วมกัน]
`SimulationModuleHandle` ถือ `Rc<dyn SimulationModule>` ไว้ ดังนั้นการ clone handle จะเป็นการแชร์โมดูลและ
state ของมัน ส่วนการ clone ค่า enum ของโมดูลในตัวจะคัดลอก state ออกมา venue หรือการรันที่ต้องการ
state แยกกันจึงต้องใช้ instance ของโมดูลที่แยกกัน
:::

## Lifecycle {#lifecycle}

exchange จะรันแต่ละโมดูลผ่าน lifecycle ดังนี้:

1. `pre_process` รันก่อนที่ exchange จะประมวลผล market data แต่ละรายการที่รองรับ
1. `process` รันกับแต่ละโมดูลตามลำดับ บน snapshot ของ exchange แบบอ่านอย่างเดียวชุดเดียวกัน
   หลังจากที่ command ของ timestamp นั้นเสร็จสิ้นแล้ว การประมวลผลจะหยุดเมื่อเจอความล้มเหลวครั้งแรก
   และ exchange จะไม่นำ adjustment ใด ๆ จาก timestamp นั้นไปใช้
1. สำหรับผลลัพธ์ที่เสร็จสมบูรณ์แต่ละรายการตามลำดับ exchange จะนำ batch ของมันไปใช้เป็น adjustment
   แบบ `Money` ตามลำดับ จากนั้นเรียก `acknowledge` ของโมดูลนั้นเพียงครั้งเดียวพร้อมผลลัพธ์ที่ตรงกัน
   แม้ batch จะว่างก็ตาม

## การจัดการความล้มเหลว {#failure-handling}

- **ความล้มเหลวใน lifecycle**: ความล้มเหลวจาก `pre_process`, `process`, `acknowledge` หรือ `reset` จะทำให้
  exchange อยู่ในสถานะ error จนกว่าทุกโมดูลจะ reset สำเร็จ เพื่อป้องกันไม่ให้ acknowledgement ที่ล้มเหลว
  นำ adjustment ที่ account อาจมีอยู่แล้วกลับมาใช้ซ้ำ
- **ความล้มเหลวเชิงวินิจฉัย (diagnostic)**: จะส่งกลับไปยัง engine พร้อม index ของโมดูลและชื่อ hook
  โดยไม่เปลี่ยนสถานะ error ของ exchange

## โมดูลในตัว {#built-in-modules}

โมดูล FX rollover และ CFD swap ที่มีมาในตัวใช้ flow แบบ acknowledgement ต่อ batch ที่เสร็จสมบูรณ์

อัตรา CFD swap กำหนดต่อ instrument เป็นสัดส่วนรายวันแบบ `Decimal` ที่มีเครื่องหมาย (signed)
ของ settlement notional โดยแยกค่าฝั่ง long และ short เวลา rollover (UTC) และวันในสัปดาห์ที่คิด
triple-roll ตั้งค่าได้ สำหรับ account แบบสกุลเงินเดียว โมดูลจะแปลง adjustment เป็นสกุลเงินหลักของ account
ด้วยอัตราแลกเปลี่ยน mid ที่อยู่ใน cache

โมดูล CFD swap จะเลื่อนการประมวลผลทั้ง batch ออกไปเมื่อขาดข้อมูลอย่างใดอย่างหนึ่งต่อไปนี้:

- matching engine
- settlement price
- อัตราแลกเปลี่ยน

โมดูลจะ log warning หนึ่งครั้งต่อวันที่บันทึกบัญชี, instrument และประเภทความล้มเหลว ก่อนจะ retry
แบบเงียบลง

Perpetual funding ยังคงเป็นส่วนหนึ่งของ `SimulatedExchange` และไม่ได้เป็น simulation module
