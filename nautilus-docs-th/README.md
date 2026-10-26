# NautilusTrader Docs (ภาษาไทย, เน้น Rust)

เอกสารชุดนี้แปลเป็นภาษาไทยจาก [เอกสาร NautilusTrader](https://nautilustrader.io/docs/latest/)
โดยตัดเนื้อหาเฉพาะ Python ออก และใช้ Rust เป็นหลัก

- ต้นฉบับ: [nautechsystems/nautilus_trader](https://github.com/nautechsystems/nautilus_trader) โฟลเดอร์ `docs/`
  ที่ commit `04f766b4d434d67fc8672026fefa25135bdc01be` (26 ต.ค. 2026)
- Rust API reference (ภาษาอังกฤษ): <https://nautilustrader.io/docs/core-latest/>

## สิ่งที่เปลี่ยนจากต้นฉบับ

- แปลเนื้อหาเป็นภาษาไทย ศัพท์เทคนิคที่นักพัฒนาไทยใช้เป็นภาษาอังกฤษอยู่แล้ว (order, position, backtest ฯลฯ) คงไว้ตามเดิม
- โค้ด Rust คงไว้ตามต้นฉบับทุกตัวอักษร
- ตัดโค้ดตัวอย่าง Python, คำสั่ง `pip`/`uv` และหัวข้อที่ใช้ได้เฉพาะ Python ออก
- ไม่รวมหน้าเหล่านี้: `api_reference/` (Python API), `concepts/python.md`, `developer_guide/python.md`,
  `developer_guide/python_adapters.md` และ tutorial/notebook ที่เป็นไฟล์ `.py`
- หัวข้อแปลเป็นภาษาไทยและมี anchor ภาษาอังกฤษเดิมกำกับ (`{#original-slug}`) เพื่อให้ลิงก์ข้ามหน้ายังใช้ได้

## สารบัญ

- [เริ่มต้นใช้งาน](getting_started/index.md) · [การติดตั้ง](getting_started/installation.md)
- [แนวคิด (Concepts)](concepts/index.md) · [Rust](concepts/rust.md) · [สถาปัตยกรรม](concepts/architecture.md)
- [How-to](how_to/index.md)
  - [รัน backtest ด้วย Rust](how_to/run_rust_backtest.md)
  - [รัน live trading ด้วย Rust](how_to/run_rust_live_trading.md)
  - [เขียน strategy ด้วย Rust](how_to/write_rust_strategy.md)
  - [เขียน actor ด้วย Rust](how_to/write_rust_actor.md)
- [Tutorials](tutorials/index.md)
- [Integrations](integrations/index.md)
- [Developer guide](developer_guide/index.md) · [Rust](developer_guide/rust.md)

## License

ต้นฉบับเผยแพร่ภายใต้ GNU Lesser General Public License v3.0 (LGPL-3.0)
งานแปลนี้เป็นฉบับดัดแปลงจากต้นฉบับและเผยแพร่ภายใต้สัญญาอนุญาตเดียวกัน ดู [LICENSE](LICENSE)
ลิขสิทธิ์ต้นฉบับเป็นของ Nautech Systems Pty Ltd และผู้ร่วมพัฒนา NautilusTrader
