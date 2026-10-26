# คู่มือ How-to

สูตรสำเร็จที่มุ่งเป้าหมายสำหรับงานที่พบบ่อย แต่ละคู่มือถือว่าผู้อ่านคุ้นเคยกับแนวคิดของ Nautilus แล้ว
และมุ่งไปที่การทำให้ได้ผลลัพธ์เฉพาะอย่าง

เพิ่งเริ่มใช้ Nautilus? ให้เริ่มจาก[เส้นทาง getting started](../getting_started/)
และ[tutorials](../tutorials/) ก่อน

## Live trading {#live-trading}

| คู่มือ                                                     | คำอธิบาย                                                |
| :------------------------------------------------------ | :---------------------------------------------------- |
| [ตั้งค่า live trading node](configure_live_trading)        | ตั้งค่า LiveNodeConfig, execution engine และ venue       |
| [เริ่มต้นใช้งาน Lighter](get_started_lighter)               | เริ่มใช้งาน Lighter จาก Rust                              |

## Rust {#rust}

| คู่มือ                                              | คำอธิบาย                                                  |
| :----------------------------------------------- | :------------------------------------------------------ |
| [เขียน Actor (Rust)](write_rust_actor)            | สร้าง data actor พร้อม subscription และ handler           |
| [เขียน Strategy (Rust)](write_rust_strategy)      | สร้าง strategy พร้อมการจัดการ order                        |
| [รัน Backtest (Rust)](run_rust_backtest)          | ใช้ BacktestEngine หรือ BacktestNode กับ catalog           |
| [รัน Live Trading (Rust)](run_rust_live_trading)  | เชื่อมต่อกับ venue ด้วย LiveNode                            |
