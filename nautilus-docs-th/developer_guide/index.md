# คู่มือนักพัฒนา (Developer Guide)

แนวทางสำหรับการพัฒนาและต่อยอด NautilusTrader หรือการมีส่วนร่วมกลับไปยังโปรเจกต์

NautilusTrader มี **core เขียนด้วย Rust**:

- **Rust** รับผิดชอบงาน networking, การ parse ข้อมูล, order matching และงานอื่น ๆ ที่ต้องการประสิทธิภาพสูง

แนวทางนี้ให้ทั้งประสิทธิภาพและความปลอดภัยของหน่วยความจำ (memory safety) ตามแบบของ Rust

## เนื้อหา {#contents}

- [การตั้งค่าสภาพแวดล้อม](environment_setup.md)
- [หลักการออกแบบ](design_principles.md)
- [ข้อตกลงการ dispatch callback](callback_dispatch.md)
- [ข้อตกลงความสอดคล้องของ runtime](runtime_conformance.md)
- [มาตรฐานการเขียนโค้ด](coding_standards.md)
- [Shell](shell.md)
- [Rust](rust.md)
- [Testing](testing.md)
- [Test Datasets](test_datasets.md)
- [รูปแบบการเขียนเอกสาร](docs.md)
- [รูปแบบ Markdown](markdown_style.md)
- [Release Notes](releases.md)
- [สถาปัตยกรรมด้านความปลอดภัย](security.md)
- [Adapters](adapters.md)
- [Data Testing Spec](spec_data_testing.md)
- [Execution Testing Spec](spec_exec_testing.md)
- [Benchmarking](benchmarking.md)
- [ข้อตกลงหน่วยความจำของ FFI](ffi.md)
- [Plugins](plugins.md)
