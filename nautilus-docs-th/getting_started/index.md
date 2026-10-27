---
title: "เริ่มต้นใช้งาน"
---

เอกสารชุดนี้เน้นการใช้ NautilusTrader จาก Rust โดย Nautilus มี implementation ของ Rust ครบถ้วนอยู่ใน
directory `crates/` ทำให้เขียน actor, strategy, รัน backtest และเทรดแบบ live ได้ด้วย Rust ล้วน

## 1. ติดตั้ง [#1-install]

ติดตั้ง Rust toolchain (rustup และ cargo) และ dependency สำหรับการ build ตามคู่มือ
[การติดตั้ง](installation.md) ซึ่งอธิบายแพลตฟอร์มที่รองรับและการ build จาก source

จากนั้นเพิ่ม crate ของ Nautilus ลงใน `Cargo.toml` ของโปรเจกต์ ดูรายการ crate และ feature flag ได้ที่หัวข้อ
[การตั้งค่าโปรเจกต์](../concepts/rust.md#project-setup) ในคู่มือแนวคิด Rust

## 2. รัน backtest แรก [#2-run-the-quickstart]

คู่มือ [รัน backtest (Rust)](../how_to/run_rust_backtest.md) พาไปตั้งแต่ dependency ใน `Cargo.toml`
จนถึงการรัน backtest ด้วยทั้ง `BacktestEngine` และ `BacktestNode`

## 3. เลือกเส้นทางของคุณ [#3-choose-your-path]

- **เขียน strategy และ actor** - เริ่มจาก [Write an Actor (Rust)](../how_to/write_rust_actor.md) แล้วต่อด้วย
  [Write a Strategy (Rust)](../how_to/write_rust_strategy.md)
- **Backtesting** - ดู API ทั้งสองระดับด้านล่าง แล้วศึกษา [tutorial](../tutorials/) สำหรับตัวอย่างรูปแบบ strategy
- **Live trading** - ดู [รัน live trading (Rust)](../how_to/run_rust_live_trading.md),
  how-to [Configure a live trading node](../how_to/configure_live_trading.md)
  และ [Integrations](../integrations/) สำหรับ venue ที่รองรับ
- **Data workflow** - ดู [how-to guide](../how_to/) สำหรับการตั้งค่า Parquet data catalog
- **ภาพรวมของ Rust API** - ดูคู่มือแนวคิด [Rust](../concepts/rust.md)
- **สร้าง adapter และร่วมพัฒนา** - ดู [Developer guide](../developer_guide/) และแนวทางการเขียน
  [Rust](../developer_guide/rust.md) ของโปรเจกต์

## ระดับของ Backtesting API [#backtesting-api-levels]

NautilusTrader มี API สำหรับ backtest สองระดับ:

| ระดับ API        | Entry point      | เหมาะกับ                                                                  |
| :-------------- | :--------------- | :----------------------------------------------------------------------- |
| Low-level API   | `BacktestEngine` | เข้าถึง component โดยตรง, การพัฒนา library                                  |
| High-level API  | `BacktestNode`   | workflow ระดับ production, ย้ายไป live trading ได้ง่ายกว่า (แนะนำ)               |

API แบบ high-level ต้องใช้ data catalog ที่เป็น Parquet ส่วน API แบบ low-level ใช้ข้อมูลในหน่วยความจำได้
แต่ไม่มีเส้นทางไปสู่ live trading ดูวิธีใช้ทั้งสองแบบจาก Rust ได้ที่ [รัน backtest (Rust)](../how_to/run_rust_backtest.md)

:::warning[หนึ่ง node ต่อหนึ่ง process]
ไม่รองรับการรัน instance ของ `BacktestNode` หรือ `LiveNode` หลายตัวพร้อมกันใน process เดียวกัน เนื่องจากมี
state แบบ global singleton แต่รองรับการรันต่อกันตามลำดับ โดย dispose อย่างถูกต้องระหว่างแต่ละรอบ การสร้าง
`LiveNode` ตัวใหม่แทนตัวเดิมบน thread เดียวกันก็ต้อง drop node ตัวก่อนหน้าก่อนสร้างด้วย

ดูรายละเอียดได้ที่ [Processes and threads](../concepts/architecture.md#processes-and-threads)
:::

ดูคู่มือแนวคิด [Backtesting](../concepts/backtesting/) เพื่อช่วยเลือกระดับ API

## ตัวอย่างใน repository [#examples-in-the-repository]

เอกสารออนไลน์แสดงตัวอย่างเพียงบางส่วน ดูชุดเต็มได้ใน repository บน GitHub:

| Directory                                                                        | เนื้อหา                                                  |
| :------------------------------------------------------------------------------- | :------------------------------------------------------ |
| [crates/](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates) | implementation ของ Rust ทั้งหมด                           |
| [docs/tutorials/](../tutorials/)                                                 | tutorial ที่สาธิต workflow ที่ใช้บ่อย                         |
| [docs/concepts/](../concepts/)                                                   | คู่มือแนวคิดพร้อมตัวอย่างโค้ดประกอบ                           |
