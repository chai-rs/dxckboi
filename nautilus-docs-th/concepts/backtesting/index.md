# Backtesting

Backtesting คือการจำลองการเทรดกับข้อมูลย้อนหลัง โดยใช้ component หลักชุดเดียวกับ live trading ได้แก่
engine ที่มีมาในตัว, `Cache`, [MessageBus](../message_bus.md), `Portfolio`,
[Actors](../actors.md), [Strategies](../strategies.md), [Execution Algorithms](../execution/algorithms.md)
และโมดูลที่ผู้ใช้กำหนดเอง

`BacktestEngine` จะประมวลผล stream ของข้อมูลย้อนหลัง เมื่อข้อมูลใน stream หมดแล้ว engine
จะสร้างผลลัพธ์และ performance metric สำหรับนำไปวิเคราะห์ NautilusTrader มี API สำหรับ backtest
สองระดับ:

| ระดับ API  | ใช้เมื่อ                                                                     |
| ---------- | ---------------------------------------------------------------------------- |
| High-level | ต้องการใช้ `BacktestNode`, config object, data catalog และการรันแบบ batch    |
| Low-level  | ต้องการควบคุม `BacktestEngine` โดยตรงและตั้งค่า component เอง               |

ส่วนนี้อธิบาย backtest engine ฝั่ง Rust

## ลำดับการอ่าน {#reading-guide}

sidebar ที่สร้างอัตโนมัติอาจเรียงหน้าเหล่านี้ตามตัวอักษร หากต้องการอ่านทั้งส่วนตั้งแต่ต้นจนจบ
ให้อ่านตามลำดับนี้:

| ขั้น | หน้า                                                       | ใช้สำหรับ                                                   |
| ---- | ---------------------------------------------------------- | ----------------------------------------------------------- |
| 1    | [API และการรันซ้ำ](apis-and-runs.md)                       | เลือกระดับ API, โหลดข้อมูล และรันแบบ batch                  |
| 2    | [ข้อมูลและ venue](data-and-venues.md)                      | จับคู่ความละเอียดของข้อมูลกับ `book_type` ของ venue         |
| 3    | [ลำดับการ execution](execution-flow.md)                    | ทำความเข้าใจลำดับการทำงาน, timer และ trade ID               |
| 4    | [ราคา fill และการ matching](fill-prices-and-matching.md)   | ทำความเข้าใจพฤติกรรมการ matching แบบ deterministic          |
| 5    | [Trade execution](trade-execution.md)                      | ใช้ trade tick, aggressor side และคิว                       |
| 6    | [Bar execution](bar-execution.md)                          | ใช้ bar, ลำดับ OHLC และจังหวะเวลาของ bar                    |
| 7    | [Fill model](fill-models.md)                               | ตั้งค่า slippage และ fill แบบอิงความน่าจะเป็น               |
| 8    | [Account และ margin](accounts-and-margin.md)               | ตั้งค่า funding, balance และ margin model                   |

## Simulation module {#simulation-modules}

[Simulation module](simulation-modules.md) อธิบายการตั้งค่าโมดูล, lifecycle, การจัดการความล้มเหลว
และพฤติกรรม FX rollover และ CFD swap ที่มีมาในตัว

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Strategies](../strategies.md): พัฒนา strategy เพื่อนำไป backtest
- [Visualization](../visualization.md): สร้าง tearsheet จากผล backtest
- [Reports](../reports.md): วิเคราะห์ข้อมูล performance ของ backtest
