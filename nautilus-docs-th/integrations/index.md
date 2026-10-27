---
title: "Integrations"
---

NautilusTrader ใช้ *adapter* แบบ modular ในการเชื่อมต่อกับ trading venue และผู้ให้บริการข้อมูล adapter
จะแปลง API ดิบให้เป็น interface เดียวกันและ domain model ที่ normalize แล้ว

integration ที่มีให้ใช้หรืออยู่ในแผนมีดังนี้:

| ชื่อ                                                        | ID                    | ประเภท                   | สถานะ                                                   | เอกสาร                           |
| :--------------------------------------------------------- | :-------------------- | :---------------------- | :------------------------------------------------------ | :------------------------------ |
| [AX Exchange](https://architect.exchange)                  | `AX`                  | Derivatives Exchange    | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](architect_ax.md)        |
| [Betfair](https://betfair.com)                             | `BETFAIR`             | Sports Betting Exchange | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](betfair.md)             |
| [Binance](https://binance.com)                             | `BINANCE`             | Crypto Exchange (CEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](binance.md)             |
| [Blockchain](blockchain.md)                                | `BLOCKCHAIN`          | DeFi Data Provider      | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](blockchain.md)          |
| [Bybit](https://www.bybit.com)                             | `BYBIT`               | Crypto Exchange (CEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](bybit.md)               |
| [CoinAPI](https://www.coinapi.io/?ref=nautilus)            | `COINAPI`             | Data Provider           | ![status](https://img.shields.io/badge/beta-yellow)     | [คู่มือ](coinapi.md)             |
| [Coinbase](https://coinbase.com)                           | `COINBASE`            | Crypto Exchange (CEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](coinbase.md)            |
| [Databento](https://databento.com)                         | `DATABENTO`           | Data Provider           | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](databento.md)           |
| [Deribit](https://www.deribit.com)                         | `DERIBIT`             | Crypto Exchange (CEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](deribit.md)             |
| [Derive](https://www.derive.xyz)                           | `DERIVE`              | Crypto Exchange (DEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](derive.md)              |
| [dYdX](https://dydx.trade)                                 | `DYDX`                | Crypto Exchange (DEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](dydx.md)                |
| [FinFeed](https://www.finfeedapi.com/?ref=nautilus)        | `FINFEED`             | Data Provider           | ![status](https://img.shields.io/badge/building-orange) | -                               |
| [Hyperliquid](https://hyperliquid.xyz)                     | `HYPERLIQUID`         | Crypto Exchange (DEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](hyperliquid.md)         |
| [Interactive Brokers](https://www.interactivebrokers.com)  | `INTERACTIVE_BROKERS` | Brokerage (multi-venue) | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](interactive_brokers.md) |
| [Kalshi](https://kalshi.com)                               | `KALSHI`              | Prediction Market (CEX) | ![status](https://img.shields.io/badge/building-orange) | -                               |
| [Kraken](https://kraken.com)                               | `KRAKEN`              | Crypto Exchange (CEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](kraken.md)              |
| [Lighter](https://lighter.xyz)                             | `LIGHTER`             | Crypto Exchange (DEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](lighter.md)             |
| [Lighter on Robinhood](https://robinhoodchain.lighter.xyz) | `LIGHTER_ROBINHOOD`   | Crypto Exchange (DEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](lighter.md)             |
| [OKX](https://okx.com)                                     | `OKX`                 | Crypto Exchange (CEX)   | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](okx.md)                 |
| [Polymarket](https://polymarket.com)                       | `POLYMARKET`          | Prediction Market (DEX) | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](polymarket.md)          |
| [Tardis](https://tardis.dev)                               | `TARDIS`              | Crypto Data Provider    | ![status](https://img.shields.io/badge/stable-green)    | [คู่มือ](tardis.md)              |

- **ID**: client ID ค่าเริ่มต้นของ client ใน adapter ของ integration นั้น
- **ประเภท**: ประเภทของ integration (มักเป็นประเภทของ venue)

สำหรับ Lighter on Robinhood นั้น `LIGHTER_ROBINHOOD` คือ venue และ client ID ที่ต้องลงทะเบียนอย่างชัดเจน
ส่วน factory ของ Lighter ที่ใช้ร่วมกันยังคงใช้ `LIGHTER` เป็นค่าเริ่มต้นเพื่อความเข้ากันได้

## สถานะ [#status]

- `planned`: อยู่ในแผนการพัฒนาในอนาคต
- `building`: อยู่ระหว่างการสร้างและน่าจะยังใช้งานไม่ได้
- `beta`: เสร็จจนทำงานได้ในระดับขั้นต่ำ และอยู่ในช่วงทดสอบแบบ 'beta'
- `stable`: ชุด feature และ API มีความเสถียร นักพัฒนาและผู้ใช้ทดสอบ integration ไปในระดับที่สมเหตุสมผลแล้ว
  แต่อาจยังมี bug หลงเหลืออยู่บ้าง

## เป้าหมายของการ implement [#implementation-goals]

NautilusTrader มุ่งให้เป็นระบบเทรดแบบรวมศูนย์สำหรับ integration ที่หลากหลาย เพื่อรองรับ strategy การเทรด
ให้กว้างที่สุด การพัฒนาจึงให้ความสำคัญกับความสามารถ*มาตรฐาน*ก่อน:

- การ request market data ย้อนหลัง
- การ stream market data แบบ live
- การทำ reconciliation ของสถานะการ execute
- การส่ง order ประเภทมาตรฐานพร้อมคำสั่งการ execute มาตรฐาน
- การแก้ไข order ที่มีอยู่ (ถ้า exchange รองรับ)
- การ cancel order

การ implement แต่ละ integration มุ่งให้เป็นไปตามเกณฑ์ต่อไปนี้:

- component ของ client ระดับ low-level ควรตรงกับ API ของ exchange ให้มากที่สุด
- ความสามารถทั้งหมดของ exchange (ส่วนที่ใช้กับ NautilusTrader ได้) ควรได้รับการรองรับ*ในที่สุด*
- data type เฉพาะของ exchange จะรองรับความสามารถและ return type ที่ผู้ใช้คาดหวังได้อย่างสมเหตุสมผล
- action ที่ exchange หรือ NautilusTrader ไม่รองรับจะถูก log เป็น warning หรือ error เมื่อถูกเรียก

::::warning[Trace logging และ credential]

log ระดับ TRACE อาจมี payload ของ WebSocket ขาออกแบบดิบ ซึ่งสำหรับบาง venue อาจมีข้อมูลการยืนยันตัวตนอยู่
ใช้ TRACE เฉพาะการ debug บนเครื่องตัวเอง และลบข้อมูลสำคัญออกจาก log ระดับ TRACE ก่อนแชร์

::::

## การรวม API ให้เป็นมาตรฐานเดียวกัน [#api-unification]

integration ทั้งหมดต้องเป็นไปตาม system API ของ NautilusTrader ซึ่งต้องมีการ normalize และทำให้เป็นมาตรฐาน:

- symbol ควรใช้รูปแบบ symbol ดั้งเดิมของ venue เว้นแต่ต้องแยกให้ชัดเจน (เช่น Binance Spot กับ Binance Futures)
- timestamp ต้องใช้ UNIX epoch nanoseconds ถ้าใช้ milliseconds ชื่อฟิลด์/property ควรลงท้ายด้วย `_ms` อย่างชัดเจน
