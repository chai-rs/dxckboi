---
title: "NautilusTrader Docs (ภาษาไทย)"
description: "เอกสาร NautilusTrader ฉบับแปลภาษาไทย เน้น Rust"
---

เอกสารชุดนี้แปลเป็นภาษาไทยจาก [เอกสาร NautilusTrader](https://nautilustrader.io/docs/latest/)
โดยตัดเนื้อหาเฉพาะ Python ออกและใช้ Rust เป็นหลัก โค้ด Rust คงไว้ตามต้นฉบับ ส่วนศัพท์เทคนิคที่นักพัฒนาไทยใช้ทับศัพท์อยู่แล้ว
(order, position, backtest ฯลฯ) คงเป็นภาษาอังกฤษ

## เริ่มจากตรงไหน [#where-to-start]

- [เริ่มต้นใช้งาน](getting_started/index.md) และ [การติดตั้ง](getting_started/installation.md)
- [รัน backtest ด้วย Rust](how_to/run_rust_backtest.md)
- [เขียน strategy ด้วย Rust](how_to/write_rust_strategy.md) และ [เขียน actor ด้วย Rust](how_to/write_rust_actor.md)
- [รัน live trading ด้วย Rust](how_to/run_rust_live_trading.md)

## หมวดเนื้อหา [#sections]

- [แนวคิด (Concepts)](concepts/index.md): สถาปัตยกรรม, data, order, execution, backtesting
- [How-to](how_to/index.md): คู่มือทีละขั้นสำหรับงานเฉพาะ
- [Tutorials](tutorials/index.md): ตัวอย่าง strategy แบบครบวงจร
- [Integrations](integrations/index.md): venue และผู้ให้บริการข้อมูลที่รองรับ
- [Developer guide](developer_guide/index.md): สำหรับผู้พัฒนา NautilusTrader เอง

Rust API reference (ภาษาอังกฤษ): <https://nautilustrader.io/docs/core-latest/>

## ต้นฉบับและ license [#source-and-license]

แปลจากโฟลเดอร์ `docs/` ของ [nautechsystems/nautilus_trader](https://github.com/nautechsystems/nautilus_trader)
ที่ commit `04f766b4` ลิขสิทธิ์ต้นฉบับเป็นของ Nautech Systems Pty Ltd และผู้ร่วมพัฒนา เผยแพร่ภายใต้ LGPL-3.0
ฉบับแปลนี้เผยแพร่ภายใต้สัญญาอนุญาตเดียวกัน
