# Tutorials

tutorial เหล่านี้ให้คำแนะนำทีละขั้นตอนสำหรับฟีเจอร์และ workflow ของ NautilusTrader

tutorial ฝั่ง Rust ใช้คำสั่งที่แสดงไว้ในแต่ละหน้า

:::note

- **Latest**: เอกสารที่ build จาก branch `master` สำหรับ release ที่เสถียร
  ดู <https://nautilustrader.io/docs/latest/tutorials/>
- **Nightly**: เอกสารที่ build จาก branch `develop` สำหรับฟีเจอร์ทดลอง
  ดู <https://nautilustrader.io/docs/nightly/tutorials/>

:::

## ลำดับที่แนะนำ {#recommended-order}

หากเพิ่งเริ่มใช้ NautilusTrader ให้ทำตามลำดับนี้:

1. [รัน backtest ด้วย Rust](../how_to/run_rust_backtest.md)
1. [เขียน strategy ด้วย Rust](../how_to/write_rust_strategy.md)
1. [รัน live trading ด้วย Rust](../how_to/run_rust_live_trading.md)
1. เลือก tutorial ตามหัวข้อที่สนใจด้านล่าง

## Data workflow {#data-workflows}

สำหรับสูตรการทำงานกับข้อมูลแบบเน้นงาน ดู[คู่มือ how-to](../how_to/)

## รูปแบบ strategy {#strategy-patterns}

| Tutorial                                                                            | คำอธิบาย                                       | ข้อมูล              |
| :---------------------------------------------------------------------------------- | :-------------------------------------------- | :---------------- |
| [Mean Reversion with Proxy FX Data (AX Exchange)](fx_mean_reversion_ax)             | Bollinger Band mean reversion บน EURUSD-PERP  | TrueFX proxy      |
| [Gold Perpetual Book Imbalance (AX Exchange)](gold_book_imbalance_ax)               | Order book imbalance บน XAU-PERP              | Databento API key |
| [On-Chain Grid Market Making with Short-Term Orders (dYdX)](grid_market_maker_dydx) | Grid MM บน perpetual ของ dYdX v4               | Live API          |

## Options {#options}

| Tutorial                                                                | คำอธิบาย                                        | ข้อมูล     |
| :---------------------------------------------------------------------- | :--------------------------------------------- | :------- |
| [Options Data and Greeks (Bybit)](options_data_bybit)                   | สตรีม greeks และ snapshot ของ option chain       | Live API |
| [Delta-Neutral Options Strategy (Bybit)](delta_neutral_options_bybit)   | Short strangle พร้อม delta hedging ด้วย perpetual | Live API |
| [Delta-Neutral Options Strategy (Derive)](delta_neutral_options_derive) | ตัว hedge strangle ETH บน Derive พร้อมการเข้าด้วย premium | Live API |

## Rust {#rust}

| Tutorial                                                                                    | คำอธิบาย                                              | ข้อมูล                |
| :------------------------------------------------------------------------------------------ | :--------------------------------------------------- | :------------------ |
| [Book Imbalance Backtest (Betfair)](backtest_book_imbalance_betfair)                        | actor แบบ book imbalance บนข้อมูล L2 ของ Betfair       | User-provided       |
| [Composite Market Making on Lighter RWA with Databento EQUS NVDA](lighter_rwa_composite_mm) | MM ที่เอียงตาม signal บน NVDA-PERP                     | Databento + Lighter |
| [Hurst/VPIN Directional Strategy (Kraken Futures)](hurst_vpin_kraken)                       | strategy แบบ informed-flow ที่กรองด้วย regime บน PF_XBTUSD | Tardis.dev          |
