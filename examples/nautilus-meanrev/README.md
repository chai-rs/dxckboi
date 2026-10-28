# nautilus-meanrev

ตัวอย่างประกอบบท [Deep Dive: จากแนวคิดสู่การใช้งานจริง](../../nautilus-docs-th/deep_dive/index.md)
ของเอกสาร NautilusTrader ภาษาไทย: strategy แบบ Bollinger Band mean reversion เขียนด้วย Rust
พร้อม backtest, test และ binary สำหรับรันบน Bybit testnet

ใช้ NautilusTrader ที่ commit `04f766b4` (pin ไว้ใน `Cargo.toml`) และ Rust 1.99.1

```bash
cargo test --release                              # unit test + integration test
cargo run --release -- run                        # backtest หนึ่งรอบพร้อม log
cargo run --release -- run random                 # รอบเดียวกันบนตลาด random walk (control)
cargo run --release -- sweep                      # parameter sweep
cargo run --release --features live --bin live    # Bybit testnet (ต้องตั้ง BYBIT_TESTNET_API_KEY/SECRET)
```

ข้อมูลใน backtest เป็นข้อมูลสังเคราะห์ ผลลัพธ์ยืนยันได้แค่ว่าโค้ดทำงานถูกต้อง ไม่ใช่คำแนะนำการลงทุน
