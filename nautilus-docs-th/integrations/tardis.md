# Tardis

Tardis ให้บริการ market data ของ cryptocurrency แบบละเอียด รวมถึง snapshot และ update ของ order book
แบบ tick-by-tick, trade, open interest, funding rate, option summary และ liquidation

NautilusTrader เชื่อมต่อกับ Tardis API, WebSocket server ของ Tardis Machine และรูปแบบ CSV ของ Tardis
ความสามารถของ adapter นี้ได้แก่:

- ฟังก์ชันโหลดและ stream CSV ที่อ่านไฟล์รูปแบบ Tardis เป็นข้อมูลของ Nautilus แบบรวดเดียวหรือเป็น chunk
  ที่มีขอบเขต
- `run_tardis_machine_replay` replay ข้อมูลย้อนหลังและเขียนเป็นไฟล์ Parquet catalog ของ Nautilus
- `TardisDataClientConfig` และ `TardisDataClientFactory` เชื่อม Nautilus node เข้ากับ historical replay
  หรือ real-time stream ของ Tardis Machine ที่ตั้งค่าไว้
- Rust มี `TardisMachineClient` และ `TardisHttpClient` สำหรับเข้าถึง normalized stream และ metadata
  ของ instrument ในระดับล่าง

:::info
ต้องมี `TARDIS_API_KEY` สำหรับการเรียก metadata ของ instrument ใน Nautilus Tardis Machine ใช้
`TM_API_KEY` สำหรับวันที่ในอดีตที่อยู่นอกวันแรกของแต่ละเดือนซึ่งใช้ได้ฟรี ดูเพิ่มเติมที่
[environment variable](#environment-variables)
:::

## ภาพรวม {#overview}

adapter นี้ implement ด้วย Rust component ของมันถูก compile รวมอยู่ใน NautilusTrader จึงไม่ต้องติดตั้ง
client library ของ Tardis แยกต่างหาก ดู API, รูปแบบข้อมูล และ server ต้นทางได้ที่
[เอกสารของ Tardis](https://docs.tardis.dev/)

Tardis Node เวอร์ชันต่ำสุดที่ทดสอบแล้วซึ่ง Tardis Machine ใช้คือ **18.2.2** release นี้แก้ปัญหาการ normalize
order book ของ Deribit เมื่อ update ที่อยู่ในคิวซึ่ง snapshot ครอบคลุมแล้วมาถึงหลัง snapshot
NautilusTrader ไม่ได้ pin เวอร์ชันใน deployment ของ Tardis Machine ของผู้ใช้

## รูปแบบที่รองรับ {#supported-formats}

Tardis ให้ market data แบบ *normalized* ซึ่งเป็นรูปแบบเดียวกันที่สอดคล้องกันทุก exchange ที่รองรับ
การ normalize นี้ทำให้ parser ตัวเดียวจัดการข้อมูลจาก[exchange ใดก็ได้ที่ Tardis รองรับ](#venues)
adapter นี้ของ NautilusTrader ไม่รองรับ market data ของ Tardis ในรูปแบบดั้งเดิมของแต่ละ exchange

NautilusTrader รองรับรูปแบบ normalized ของ Tardis Machine ต่อไปนี้ ดู schema ของ field ได้ที่
[Tardis data type reference](https://docs.tardis.dev/tardis-machine/data-types) อย่างเป็นทางการ

| รูปแบบของ Tardis     | Nautilus data type                                              |
| :------------------ | :-------------------------------------------------------------- |
| `book_change`       | `OrderBookDeltas`                                               |
| `book_snapshot_*`   | `OrderBookDepth` หรือ `OrderBookDeltas`                         |
| `quote`             | `QuoteTick`                                                     |
| `quote_10s`         | `QuoteTick`                                                     |
| `trade`             | `TradeTick`                                                     |
| `trade_bar_*`       | `Bar`                                                           |
| `derivative_ticker` | `FundingRateUpdate`, `MarkPriceUpdate` หรือ `IndexPriceUpdate`  |
| `option_summary`    | `OptionGreeks` และ `QuoteTick` (ตัวเลือกเสริม) จาก field BBO    |
| `disconnect`        | ถูกเพิกเฉย                                                      |

**หมายเหตุ**:

- Tardis ระบุว่า `quote` เป็น alias ของ `book_snapshot_1_0ms`
- Tardis ระบุว่า `quote_10s` เป็น alias ของ `book_snapshot_1_10s`
- `quote`, `quote_10s` และ snapshot ระดับเดียวจะถูก parse เป็น `QuoteTick`
- data client จะ emit update ของ funding rate, mark price และ index price จาก message `derivative_ticker`
  เฉพาะเมื่อค่าเปลี่ยนแปลง pipeline การ replay ไปยัง catalog ไม่ได้เขียน update เหล่านี้
- message `option_summary` ของ Tardis มี field best bid/offer Nautilus จะ map feed นี้เป็น `OptionGreeks`
  เสมอ ตั้ง `extract_bbo_as_quotes` เป็น `true` เพื่อให้ emit `QuoteTick` จาก field BBO เหล่านั้นด้วย
- adapter ไม่ได้ parse รูปแบบ normalized `book_ticker`, `liquidation` หรือ `error` ของ Tardis

:::info
ดูเพิ่มเติมที่ [Tardis Machine quickstart](https://docs.tardis.dev/tardis-machine/quickstart) ของ Tardis
:::

## Bar {#bars}

adapter แปลง interval และ suffix ของ trade bar ใน Tardis เป็น `BarType` ของ Nautilus
ได้แก่:

| Suffix ของ Tardis | ความหมาย      | Nautilus bar aggregation     |
| :--------------- | :----------- | :--------------------------- |
| `ms`             | มิลลิวินาที    | `MILLISECOND`                |
| `s`              | วินาที        | `SECOND`                     |
| `m`              | นาที          | `MINUTE`, `HOUR` หรือ `DAY`  |
| `ticks`          | จำนวน tick    | `TICK`                       |
| `vol`            | ขนาด volume   | `VOLUME`                     |

interval หน่วยนาทีที่หารเป็นชั่วโมงหรือวันได้ลงตัวจะใช้ aggregation `HOUR` หรือ `DAY` มาตรฐานของ Nautilus

## Symbology และการ normalize {#symbology-and-normalization}

integration ของ Tardis จะ normalize symbol อย่างสม่ำเสมอเพื่อให้เข้ากันได้กับ adapter ของ crypto exchange
ใน NautilusTrader โดยปกติ NautilusTrader จะใช้ชื่อดั้งเดิมของ exchange ที่ Tardis ให้มา สำหรับบาง exchange
จะปรับ symbol ดิบให้ตรงกับ symbology ของ Nautilus ดังนี้:

### กฎทั่วไป {#common-rules}

- symbol ทั้งหมดถูกแปลงเป็นตัวพิมพ์ใหญ่
- บาง exchange จะต่อท้าย suffix ของประเภทตลาดด้วยขีด (hyphen)
- symbol ดั้งเดิมของ exchange ถูกเก็บไว้ใน field `raw_symbol` ของ instrument definition ใน Nautilus

### การ normalize เฉพาะ exchange {#exchange-specific-normalizations}

- **Binance**: Nautilus ต่อท้าย suffix `-PERP` ให้ symbol ของ perpetual จาก `binance`,
  `binance-futures`, `binance-us`, `binance-dex` และ `binance-jersey`
- **Bybit**: Nautilus ใช้ suffix ตามหมวดผลิตภัณฑ์ ได้แก่ `-SPOT`, `-LINEAR`, `-INVERSE` และ `-OPTION`
- **dYdX v3**: Nautilus ต่อท้าย suffix `-PERP` ให้ symbol ของ perpetual จาก `dydx`
- **Gate.io**: Nautilus ต่อท้าย suffix `-PERP` ให้ symbol ของ perpetual จาก `gate-io-futures`
- **MEXC**: Nautilus ต่อท้าย suffix `-PERP` ให้ symbol ของ perpetual จาก `mexc-futures`

เอกสาร symbology โดยละเอียดของแต่ละ exchange:

- [Binance symbology](./binance.md#symbology)
- [Bybit symbology](./bybit.md#symbology)
- [dYdX symbology](./dydx.md#symbology)

## Venue {#venues}

exchange บางแห่งบน Tardis ถูกแบ่งออกเป็นหลาย venue
ตารางด้านล่างแสดงการจับคู่ระหว่าง venue ของ Nautilus กับ exchange ที่สอดคล้องกันใน Tardis:

| Venue ของ Nautilus  | Exchange ของ Tardis                                                                                         |
| :----------------- | :----------------------------------------------------------------------------------------------------------- |
| `ASCENDEX`         | `ascendex`                                                                                                   |
| `BINANCE`          | `binance`, `binance-dex`, `binance-european-options`, `binance-futures`, `binance-jersey`, `binance-options` |
| `BINANCE_DELIVERY` | `binance-delivery` (*สัญญาแบบ COIN-margined*)                                                                |
| `BINANCE_US`       | `binance-us`                                                                                                 |
| `BITFINEX`         | `bitfinex`, `bitfinex-derivatives`                                                                           |
| `BITFLYER`         | `bitflyer`                                                                                                   |
| `BITGET`           | `bitget`, `bitget-futures`                                                                                   |
| `BITMEX`           | `bitmex` (*ข้อมูลย้อนหลังเท่านั้น*)                                                                            |
| `BITNOMIAL`        | `bitnomial`                                                                                                  |
| `BITSTAMP`         | `bitstamp`                                                                                                   |
| `BLOCKCHAIN_COM`   | `blockchain-com`                                                                                             |
| `BYBIT`            | `bybit`, `bybit-options`, `bybit-spot`                                                                       |
| `COINBASE`         | `coinbase`                                                                                                   |
| `COINBASE_INTX`    | `coinbase-international`                                                                                     |
| `COINFLEX`         | `coinflex` (*ข้อมูลย้อนหลังเท่านั้น*)                                                                          |
| `CRYPTO_COM`       | `crypto-com`                                                                                                 |
| `CRYPTOFACILITIES` | `cryptofacilities`                                                                                           |
| `DELTA`            | `delta`                                                                                                      |
| `DERIBIT`          | `deribit`                                                                                                    |
| `DYDX`             | `dydx`                                                                                                       |
| `DYDX_V4`          | `dydx-v4`                                                                                                    |
| `FTX`              | `ftx`, `ftx-us` (*ข้อมูลย้อนหลังเท่านั้น*)                                                                     |
| `GATE_IO`          | `gate-io`, `gate-io-futures`                                                                                 |
| `GEMINI`           | `gemini`                                                                                                     |
| `HITBTC`           | `hitbtc`                                                                                                     |
| `HUOBI`            | `huobi`, `huobi-dm`, `huobi-dm-linear-swap`, `huobi-dm-options`                                              |
| `HUOBI_DELIVERY`   | `huobi-dm-swap`                                                                                              |
| `HYPERLIQUID`      | `hyperliquid`                                                                                                |
| `KRAKEN`           | `kraken`                                                                                                     |
| `KUCOIN`           | `kucoin`, `kucoin-futures`                                                                                   |
| `LIGHTER`          | `lighter`                                                                                                    |
| `MANGO`            | `mango`                                                                                                      |
| `MEXC`             | `mexc`, `mexc-futures`                                                                                       |
| `OKCOIN`           | `okcoin`                                                                                                     |
| `OKEX`             | `okex`, `okex-futures`, `okex-options`, `okex-spreads`, `okex-swap`                                          |
| `PHEMEX`           | `phemex`                                                                                                     |
| `POLONIEX`         | `poloniex`                                                                                                   |
| `SERUM`            | `serum` (*ข้อมูลย้อนหลังเท่านั้น*)                                                                             |
| `STAR_ATLAS`       | `star-atlas`                                                                                                 |
| `UPBIT`            | `upbit`                                                                                                      |
| `WOO_X`            | `woo-x`                                                                                                      |

exchange ID บางตัวเป็น venue ที่เลิกให้บริการแล้วแต่ยังเก็บไว้สำหรับข้อมูลย้อนหลัง ดูความพร้อมใช้งานและ
สถานะการเลิกให้บริการได้ที่ [historical data details](https://docs.tardis.dev/historical-data-details) อย่างเป็นทางการ

Tardis Node [18.2.3](https://github.com/tardis-dev/tardis-node/releases/tag/18.2.3) ยกเลิกการ stream แบบ real-time
ของ BitMEX แต่การ replay ข้อมูลย้อนหลังของ BitMEX, channel และการ normalize ยังรองรับอยู่

## Environment variable {#environment-variables}

Tardis และ NautilusTrader ใช้ environment variable ต่อไปนี้

- `TM_API_KEY`: API key ที่ส่งให้ process ของ Tardis Machine สำหรับเข้าถึงข้อมูลย้อนหลัง
- `TARDIS_API_KEY`: API key สำหรับ request metadata ของ instrument ใน Nautilus
- `TARDIS_MACHINE_WS_URL` (ตัวเลือกเสริม): base URL ของ WebSocket ของ Tardis Machine
- `NAUTILUS_PATH` (ตัวเลือกเสริม): directory แม่ที่มี subdirectory `catalog/` สำหรับ output ของการ replay

instruments metadata API ของ Tardis ต้องยืนยันตัวตนด้วย bearer token และใช้ได้กับ subscription ของ Tardis
ระดับ pro และ business ที่ active อยู่

## การรัน historical replay ของ Tardis Machine {#running-tardis-machine-historical-replays}

[Tardis Machine Server](https://docs.tardis.dev/tardis-machine/quickstart) เป็น server ที่รันในเครื่องได้
และมีการ cache ข้อมูลในตัว มันให้ market data ของ cryptocurrency ทั้งแบบย้อนหลังระดับ tick และแบบ real-time
ที่รวมแล้ว ผ่าน HTTP และ WebSocket API

:::warning
หาก cancel historical WebSocket replay ก่อนเสร็จ Tardis Machine 18.2.2 อาจหยุดทำงาน กรณีนี้เกิดกับ
Tardis Node 18.2.3 บน Node.js 26.7.0 การ replay อาจทิ้ง handle ของไฟล์ที่ cache ไว้เปิดค้าง ทำให้เกิด error
`ERR_INVALID_STATE` เมื่อ garbage collection ปิดมัน การที่ server ปิดตัวยังขัดจังหวะ client อื่นที่เชื่อมต่ออยู่ด้วย

ให้ปล่อยให้ replay ทำงานจนเสร็จเมื่อทำได้ server เดียวกันบน Node.js 24.19.0 หลีกเลี่ยงการ crash ที่พบได้
แต่ยังแสดง warning `DEP0137` และไม่ได้แก้ปัญหา file-handle รั่วที่ต้นทาง
:::

สามารถรัน WebSocket replay ของ Tardis Machine แบบสมบูรณ์จาก Rust และเขียนผลลัพธ์เป็นรูปแบบ Parquet
ของ Nautilus ได้

ฟังก์ชัน data pipeline แบบ end-to-end `run_tardis_machine_replay` ใช้[configuration](#configuration)
ที่กำหนดเพื่อทำขั้นตอนต่อไปนี้:

- เชื่อมต่อกับ server ของ Tardis Machine
- request และ parse instrument definition ทั้งหมดของ exchange ที่ตั้งค่าไว้จาก instruments metadata API ของ Tardis
- stream instrument และ data type ทั้งหมดที่ร้องขอในช่วงเวลาที่กำหนดจาก Tardis Machine
- สำหรับแต่ละ data type และวัน (UTC) เขียนไฟล์ `.parquet` ที่เข้ากันได้กับ catalog แยกตาม instrument หรือ bar type
- จบ stream และ flush ข้อมูลที่เหลือลง disk

### ไฟล์ output {#output-files}

ไฟล์ถูกเขียนหนึ่งไฟล์ต่อวัน (UTC) และ instrument หรือต่อ bar type โดยใช้ช่วง timestamp แบบ ISO 8601:

- **รูปแบบ**: `{start_timestamp}_{end_timestamp}.parquet`
- **ตัวอย่าง**: `2023-10-01T00-00-00-000000000Z_2023-10-01T23-59-59-999999999Z.parquet`
- **Relative path**: `{data_type}/{instrument_id}/{filename}` หรือ `bars/{bar_type}/{filename}` สำหรับ bar

รูปแบบนี้เข้ากันได้กับการ query, การรวมไฟล์ (consolidation) และการจัดการ data catalog ของ Nautilus

:::note
สามารถ request ข้อมูลของวันแรกของแต่ละเดือนได้โดยไม่ต้องมี API key ของ Tardis Machine วันอื่นต้องใช้ `TM_API_KEY`
:::

กระบวนการนี้ปรับให้เหมาะกับการเขียน output ลง Parquet data catalog ของ Nautilus โดยตรง
ตั้ง `NAUTILUS_PATH` เป็น directory แม่ที่มี subdirectory `catalog/` ไฟล์ Parquet จะถูกเขียนไว้ใต้
`<NAUTILUS_PATH>/catalog/data/` ใน subdirectory แยกตาม data type และ instrument หรือ bar type

หากไม่ได้ระบุ `output_path` และไม่ได้ตั้ง `NAUTILUS_PATH` output จะไปอยู่ที่ working directory ปัจจุบันโดยค่าเริ่มต้น

### ขั้นตอน {#procedure}

:::warning
อย่า publish port ของ Tardis Machine บน host address `0.0.0.0` Docker
[publish port บนทุก interface ของ host โดยค่าเริ่มต้น](https://docs.docker.com/engine/network/port-publishing/)
เมื่อ mapping ไม่ได้ระบุ host address บน Linux Docker
[เบี่ยง traffic ของ container ที่ publish ไว้ก่อนที่ `ufw` จะใช้กฎของมัน](https://docs.docker.com/engine/network/packet-filtering-firewalls/#docker-and-ufw)
ซึ่งอาจข้ามข้อจำกัดของ firewall ที่คาดไว้ ให้ bind ทั้งสอง port ไว้กับ `127.0.0.1` เว้นแต่จำเป็นต้องเข้าถึง
จากระยะไกลและมีการรักษาความปลอดภัยแยกต่างหาก
:::

สำหรับวันที่อยู่นอกวันแรกของแต่ละเดือนซึ่งใช้ได้ฟรี ให้ตั้ง `TM_API_KEY` ใน environment ของ host จากนั้น
เริ่ม Docker container `tardis-machine`:

```bash
docker run \
  -p 127.0.0.1:8000:8000 \
  -p 127.0.0.1:8001:8001 \
  -e TM_API_KEY \
  -d tardisdev/tardis-machine
```

คำสั่งนี้เริ่ม server `tardis-machine` โดยไม่มี cache ถาวรในเครื่อง ซึ่งอาจกระทบประสิทธิภาพ
เพื่อประสิทธิภาพการ replay ที่ดีขึ้น ให้รันด้วย persistent volume

### Configuration {#configuration}

ถัดไป ให้เตรียมไฟล์ configuration แบบ JSON ไว้

**Field ของ configuration JSON**

- `tardis_ws_url` (`str | null`): WebSocket URL ของ Tardis Machine ค่าเริ่มต้นคือ `TARDIS_MACHINE_WS_URL`
- `tardis_http_url` (`str | null`): base URL ของ Tardis HTTP API ค่าเริ่มต้นคือ `https://api.tardis.dev/v1`
- `normalize_symbols` (`bool | null`): ใช้การ normalize symbol ของ Nautilus ค่าเริ่มต้นคือ `true`
- `output_path` (`str | null`): directory output สำหรับข้อมูล Parquet หากไม่ได้ตั้ง จะใช้
  `<NAUTILUS_PATH>/catalog/data` ถ้าตั้ง `NAUTILUS_PATH` ไว้ หรือไม่ก็ working directory ปัจจุบัน
- `book_snapshot_output` (`"deltas" | "depth" | null`): รูปแบบ output ของ snapshot ค่าเริ่มต้นคือ `"deltas"`
- `extract_bbo_as_quotes` (`bool | null`): เขียนข้อมูล `QuoteTick` จาก field best bid/offer ใน message
  `option_summary` ของ Tardis Machine ด้วย ค่าเริ่มต้นคือ `false`
- `compression` (`"zstd" | "snappy" | "uncompressed" | null`): codec การบีบอัดของ Parquet
  ค่าเริ่มต้นคือ `"zstd"` level 3
- `proxy_url` (`str | null`): URL ของ proxy สำหรับ HTTP request ไปยัง Tardis ค่าเริ่มต้นคือไม่ใช้ proxy
- `options` (`JSON[]`): object ของ option สำหรับ replay request (จำเป็น)

มีตัวอย่างไฟล์ configuration อยู่ที่ `crates/adapters/tardis/bin/example_config.json`:

```json
{
  "tardis_ws_url": "ws://localhost:8001",
  "output_path": null,
  "options": [
    {
      "exchange": "bitmex",
      "symbols": [
        "xbtusd",
        "ethusd"
      ],
      "data_types": [
        "trade"
      ],
      "from": "2019-10-01",
      "to": "2019-10-02"
    }
  ]
}
```

### Output ของ book snapshot {#book-snapshot-output}

option `book_snapshot_output` ควบคุมวิธีแปลงและจัดเก็บ message `book_snapshot_*` ของ Tardis

| ค่า      | Nautilus type     | Directory output     | คำอธิบาย                                    |
| :------- | :---------------- | :------------------- | :----------------------------------------- |
| `deltas` | `OrderBookDeltas` | `order_book_deltas/` | delta แบบ clear และ add สำหรับแต่ละ snapshot |
| `depth`  | `OrderBookDepth`  | `order_book_depths/` | snapshot ที่มีระดับราคาทั้งหมด              |

**ควรใช้รูปแบบใดเมื่อใด**:

- **`deltas` (ค่าเริ่มต้น)**: ใช้เมื่อต้องสร้าง state ของ book ขึ้นใหม่หรือรวม snapshot เข้ากับข้อมูล
  `book_change` แต่ละ snapshot จะกลายเป็น clear delta ตามด้วย add delta สำหรับแต่ละระดับราคา
- **`depth`**: ใช้เมื่อ strategy ต้องการ depth snapshot เป็นระยะ แต่ละ snapshot เป็น record เดียวที่มี
  ระดับราคาทั้งหมดจาก message ของ venue

**การหลีกเลี่ยงการเขียนทับไฟล์**:

เมื่อดาวน์โหลดทั้งข้อมูล `book_snapshot_*` และ `book_change` สำหรับ instrument และช่วงวันเดียวกัน
`depth` จะเขียน snapshot ไปที่ `order_book_depths/` และหลีกเลี่ยงการเขียนทับ `order_book_deltas/`

ตัวอย่าง configuration ที่ระบุรูปแบบชัดเจน:

```json
{
  "tardis_ws_url": "ws://localhost:8001",
  "book_snapshot_output": "depth",
  "options": [
    {
      "exchange": "binance-futures",
      "symbols": ["btcusdt"],
      "data_types": ["book_snapshot_5_100ms", "book_change"],
      "from": "2024-01-01",
      "to": "2024-01-02"
    }
  ]
}
```

### การดึง BBO จาก option summary {#option-summary-bbo-extraction}

ตั้ง `extract_bbo_as_quotes` เป็น `true` เมื่อ request ข้อมูล `option_summary` จาก Tardis Machine และ
backtest ต้องใช้ BBO quote ของ option ด้วย Nautilus ยังคงเขียน `OptionGreeks` จากทุก message
`option_summary` เมื่อ field best bid/offer ครบทั้งหมดและขนาดถูกต้อง มันจะเขียน `QuoteTick` สำหรับ
instrument และ timestamp เดียวกันด้วย

option นี้ใช้กับ message `option_summary` ของ Tardis Machine ทั้งแบบ replay และ stream เท่านั้น ไม่ได้เปลี่ยน
การโหลด CSV ของ Tardis

```json
{
  "tardis_ws_url": "ws://localhost:8001",
  "extract_bbo_as_quotes": true,
  "options": [
    {
      "exchange": "deribit",
      "symbols": ["BTC-28JUN24-70000-C"],
      "data_types": ["option_summary"],
      "from": "2024-01-01",
      "to": "2024-01-02"
    }
  ]
}
```

### การ replay ด้วย Rust {#rust-replays}

หากต้องการรัน replay ด้วย Rust ให้เปิด feature flag `replay` ของ `nautilus-tardis` และสร้าง binary
ในลักษณะต่อไปนี้:

```rust
use std::path::PathBuf;

use nautilus_tardis::replay::run_tardis_machine_replay_from_config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    nautilus_common::logging::ensure_logging_initialized();

    let config_filepath = PathBuf::from("YOUR_CONFIG_FILEPATH");
    run_tardis_machine_replay_from_config(&config_filepath).await?;
    Ok(())
}
```

logging มีค่าเริ่มต้นเป็นระดับ INFO หากต้องการเปิด debug logging ให้ export environment variable ต่อไปนี้:

```bash
export NAUTILUS_LOG=debug
```

มีตัวอย่าง binary ที่ใช้งานได้อยู่ที่ `crates/adapters/tardis/bin/example_replay.rs`

สามารถรันด้วย cargo ได้เช่นกัน:

```bash
cargo run -p nautilus-tardis --features replay --bin tardis-replay <path_to_your_config>
```

### Catalog สำหรับ backtest แบบ option chain {#option-chain-backtest-catalog}

backtest แบบ option chain เริ่มหลังจากที่ Tardis replay เขียนข้อมูลลง catalog ของ Nautilus แล้ว
backtest loader ไม่ได้ request ข้อมูล Tardis ที่ขาดหายระหว่างการรัน ดังนั้น catalog ต้องมี:

- option instrument จาก instrument metadata API ของ Tardis
- ข้อมูล `QuoteTick` จาก option book snapshot ระดับเดียว, ข้อมูล quote หรือการดึง BBO จาก `option_summary`
- ข้อมูล `OptionGreeks` จาก message `option_summary` ของ Tardis

ใช้ทั้ง `QuoteTick` และ `OptionGreeks` ในรายการ `BacktestDataConfig` สำหรับ option instrument ID เดียวกัน
option-chain manager จะรวม BBO และ Greeks ที่ replay แล้วเป็น snapshot `OptionChainSlice` ใช้
`snapshot_interval_ms=None` สำหรับการ publish แบบดิบ หรือตั้ง interval เป็นมิลลิวินาทีเพื่อ publish
snapshot ที่ลดความถี่ลง

strategy สามารถเลือกสัญญาตาม moneyness ด้วยช่วง strike แบบอิงกับ ATM หรือเปอร์เซ็นต์จาก ATM, ตาม delta ด้วย
`StrikeRange.delta(target, tolerance)` หรือตาม strike คงที่ด้วย `StrikeRange.fixed([...])` การ matching
ของ option order ใน backtest ขับเคลื่อนด้วย quote order ที่ marketable จะ fill เป็น taker กับ BBO ฝั่งตรงข้าม
ส่วน passive limit อาจ fill เป็น maker เมื่อ BBO update ที่ตามมา trade ทะลุราคา limit

ตั้งค่าค่าธรรมเนียมของ option อย่างชัดเจนบน simulated venue ด้วย structural fee model เช่น
`CappedOptionFeeModel` หรือ `TieredNotionalOptionFeeModel` ไม่มีการ map จาก exchange ของ Tardis ไปยัง
fee model โดยอัตโนมัติ

### การแปลง CSV ของ option chain เป็น catalog {#option-chain-csv-catalog-conversion}

สำหรับ option chain ย้อนหลังจากไฟล์ CSV ของ Tardis ที่ดาวน์โหลดได้ ให้ใช้ `convert_tardis_options_chain_csv(...)`
เพื่อแปลงแถว `options_chain` เป็นข้อมูล catalog ของ Nautilus เส้นทางนี้ไม่ได้เรียกทั้ง Tardis Machine และ
instrument metadata API ใช้กับไฟล์ CSV ของ Tardis ที่มีอยู่ หรือเพื่อเริ่มสร้าง catalog จากข้อมูลที่ดาวน์โหลดมา
โดยไม่ต้องมี API key

ตัวแปลงจะเขียน `OptionGreeks` สำหรับทุกแถวที่เลือก เมื่อใช้ค่าเริ่มต้น `extract_bbo_as_quotes=True`
แถวที่มี best bid/offer ครบจะเขียน `QuoteTick` ด้วย ให้เปิดค่านี้ไว้สำหรับ backtest แบบ option chain
catalog ที่มีแต่ Greeks ไม่มี quote ดังนั้น chain manager จึงไม่สามารถ publish snapshot `OptionChainSlice`
ที่มีข้อมูลสำหรับ strike ที่ไม่มีข้อมูล BBO ได้

การสร้าง instrument รองรับเฉพาะ option ของ Deribit สำหรับ venue ของ option อื่น ให้ตั้ง `write_instruments=False`
ก่อนแปลง และโหลด instrument จากแหล่งอื่นก่อน backtest การปล่อยให้เปิดไว้กับไฟล์ที่ไม่ใช่ Deribit อาจล้มเหลว
หลังจากที่ไฟล์ข้อมูลถูกเขียนลง catalog ไปแล้ว

ส่ง path ของ CSV `options_chain` รายวันตามลำดับเวลา filter `underlyings` จะจับคู่ prefix ของ symbol
เช่น `["BTC-"]` หากต้องการเก็บแถวสุดท้ายต่อ instrument ต่อ interval ภายในแต่ละไฟล์ input ให้ตั้ง
`snapshot_interval_ms` หรือใช้ `None` เพื่อเขียนทุกแถวที่เลือก เมื่อลดความถี่ แถวภายในแต่ละไฟล์ต้อง
เรียงตาม `local_timestamp`

ให้ระบุ `price_precision` และ `size_precision` อย่างชัดเจนเพื่อให้ metadata ของ quote เป็นแบบ deterministic
precision ที่อนุมานได้อาจเพิ่มขึ้นเมื่ออ่านแถวที่ตามมา ดังนั้นข้อมูลที่เขียนไว้ก่อนหน้าในไฟล์อาจมี metadata
ของ precision ที่ต่ำกว่า

> หมายเหตุ: ต้นฉบับมีเฉพาะตัวอย่าง Python จึงตัดออก

## การโหลดข้อมูล CSV ของ Tardis {#loading-tardis-csv-data}

สามารถโหลดข้อมูล CSV รูปแบบ Tardis ด้วย Rust ได้ loader จะอ่านข้อมูล CSV แบบ text จาก disk และ parse
เป็นข้อมูลของ Nautilus

สามารถระบุ parameter `limit` ให้ฟังก์ชัน `load_*` เพื่อควบคุมจำนวนแถวสูงสุดที่โหลดได้

loader และ streamer ของ delta จะข้ามแถวใด ๆ ที่อยู่ก่อนแถว snapshot แรกในไฟล์ ตาม
[Tardis FAQ](https://docs.tardis.dev/faq/order-books) CSV ที่ถูกตัดหรือแบ่งจนไม่ได้เริ่มต้นด้วยแถว snapshot
อีกต่อไปจะได้ delta เป็นศูนย์

:::note
การโหลดไฟล์ CSV ที่มีหลาย instrument ปนกันทำได้ยากเพราะข้อกำหนดด้าน precision และไม่แนะนำ
ให้ใช้ไฟล์ CSV ที่มี instrument เดียวแทน

ฟังก์ชัน `load_tardis_options_chain`, `stream_tardis_options_chain` และ `convert_tardis_options_chain_csv`
เป็นข้อยกเว้น: ไฟล์ `options_chain` ของ Tardis เป็นไฟล์ chain ที่มีหลาย instrument และเส้นทางเหล่านี้ติดตาม
precision แยกตาม instrument แต่ยังแนะนำให้ระบุ precision อย่างชัดเจนเพื่อให้ได้ output แบบ deterministic
:::

### การโหลดข้อมูล CSV ด้วย Rust {#loading-csv-data-in-rust}

สามารถโหลดข้อมูล CSV รูปแบบ Tardis ด้วย Rust โดยใช้ฟังก์ชันโหลดใน `crates/adapters/tardis/src/csv/mod.rs`
เมื่อโหลดข้อมูล สามารถระบุ instrument ID, price precision และ size precision ได้ (ตัวเลือกเสริม) การระบุ
instrument ID ช่วยให้โหลดได้เร็วขึ้น price precision และ size precision จะถูกอนุมานจาก CSV หากไม่ระบุ
แต่แนะนำให้ระบุค่าอย่างชัดเจนเพื่อให้ได้ output แบบ deterministic โดยเฉพาะกับไฟล์ขนาดใหญ่

ดูตัวอย่างแบบสมบูรณ์ได้ที่ `crates/adapters/tardis/bin/example_csv.rs`

หากต้องการโหลดข้อมูล สามารถใช้โค้ดในลักษณะต่อไปนี้:

```rust
use std::path::Path;

use nautilus_model::identifiers::InstrumentId;
use nautilus_tardis::csv::load_deltas;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optionally specify precisions and the CSV filepath
    let price_precision = Some(1);
    let size_precision = Some(0);
    let filepath = Path::new("YOUR_CSV_DATA_PATH");

    // Optionally specify an instrument ID and/or limit
    let instrument_id = InstrumentId::from("BTC-PERPETUAL.DERIBIT");
    let limit = None;

    let _deltas = load_deltas(
        filepath,
        price_precision,
        size_precision,
        Some(instrument_id),
        limit,
    )?;
    Ok(())
}
```

## การ stream ข้อมูล CSV ของ Tardis {#streaming-tardis-csv-data}

integration ของ Tardis สามารถโหลดและประมวลผลไฟล์ CSV ขนาดใหญ่เป็น chunk ที่ตั้งค่าได้ ทำให้ประมวลผลไฟล์
ขนาดหลาย gigabyte ได้โดยไม่ต้องโหลดทั้งไฟล์เข้าหน่วยความจำหรือใช้หน่วยความจำของระบบจนหมด

มีฟังก์ชัน `stream_*` สำหรับข้อมูล CSV ต่อไปนี้:

- Order book deltas
- Order book depth snapshots (จากไฟล์ snapshot 5 และ snapshot 25)
- Quote ticks
- Trade ticks
- Funding rates
- แถว options chain

### การใช้หน่วยความจำ {#memory-use}

การ stream จำกัดจำนวน record ที่ parse แล้วซึ่งถูกเก็บไว้ในแต่ละช่วงเวลา:

- **ควบคุมการใช้หน่วยความจำได้**: โหลดเพียง chunk เดียวในหน่วยความจำต่อครั้ง
- **ประมวลผลไฟล์ขนาดใหญ่ได้**: iterator ประมวลผลไฟล์ที่ใหญ่กว่า RAM ที่มีได้
- **ตั้งขนาด chunk ได้**: ปรับ `chunk_size` ในช่วง `[1, 1_000_000]` ตามหน่วยความจำและความต้องการด้าน
  ประสิทธิภาพของระบบ (ค่าเริ่มต้น 100,000)

:::warning
เมื่อใช้การ stream ร่วมกับการอนุมาน precision ค่า precision ที่อนุมานได้อาจต่างจากการโหลดทั้งไฟล์รวดเดียว
การอนุมาน precision ทำงานภายในขอบเขตของ chunk และ chunk ที่ต่างกันอาจมีค่าที่ต้องการ precision ต่างกัน
เพื่อให้พฤติกรรมของ precision เป็นแบบ deterministic ให้ระบุ parameter `price_precision` และ `size_precision`
อย่างชัดเจน
:::

### การ stream ข้อมูล CSV ด้วย Rust {#streaming-csv-data-in-rust}

ฟังก์ชันการ stream เบื้องหลัง implement ด้วย Rust และใช้งานได้โดยตรง:

```rust
use std::path::Path;

use nautilus_model::identifiers::InstrumentId;
use nautilus_tardis::csv::stream_trades;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filepath = Path::new("large_trades_file.csv");
    let chunk_size = 100_000;
    let price_precision = Some(1);
    let size_precision = Some(0);
    let instrument_id = Some(InstrumentId::from("BTC-PERPETUAL.DERIBIT"));

    // Stream trades in chunks
    let stream = stream_trades(
        filepath,
        chunk_size,
        price_precision,
        size_precision,
        instrument_id,
    )?;

    for chunk in stream {
        let chunk = chunk?;
        println!("Processing chunk with {} trades", chunk.len());
        // Process chunk
    }

    Ok(())
}
```

## Metadata ของ instrument {#instrument-metadata}

pipeline การ replay และ data client จะ request metadata ของทุก exchange ใน option ของ Tardis ที่ตั้งค่าไว้
ก่อนเชื่อมต่อกับ Tardis Machine โดยใช้
[instruments metadata API ของ Tardis](https://docs.tardis.dev/api/instruments-metadata-api) เพื่อ parse
metadata ของ instrument เป็น definition ของ Nautilus data client ยัง publish definition เหล่านั้นไปยัง
data engine ของ Nautilus ด้วย

:::note
ต้องมี `TARDIS_API_KEY` ของ subscription Tardis ระดับ pro หรือ business ที่ active อยู่ การ bootstrap
อัตโนมัติจะ request metadata ของ instrument ทั้งหมดสำหรับแต่ละ exchange ของ Tardis ที่ตั้งค่าไว้
:::

ผู้ใช้ Rust ยังสามารถ request instrument definition ได้โดยตรงด้วย `TardisHttpClient`
client รับ argument `api_key`, `base_url`, `timeout_secs`, `normalize_symbols` และ `proxy_url`
(ตัวเลือกเสริม) มันดึงได้ทั้ง symbol เดียวหรือทุก instrument ของ exchange หนึ่ง ให้ใช้ exchange ID ของ Tardis
แบบ lower-kebab เช่น `binance-futures`

### การ request instrument ด้วย Rust {#requesting-instruments-in-rust}

ดูตัวอย่างแบบสมบูรณ์ได้ที่ `crates/adapters/tardis/bin/example_http.rs`

```rust
use nautilus_tardis::{
    common::enums::TardisExchange,
    http::TardisHttpClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    nautilus_common::logging::ensure_logging_initialized();

    let client = TardisHttpClient::new(None, None, None, true, None)?;

    // Tardis instrument definitions
    let info = client
        .instruments_info(TardisExchange::Bitmex, Some("XBTUSD"), None)
        .await?;
    println!("Received: {info:?}");

    // Nautilus instrument definitions
    let instruments = client
        .instruments(
            TardisExchange::Bitmex,
            Some("XBTUSD"),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;
    println!("Received: {instruments:?}");
    Ok(())
}
```

## Data client ของ Nautilus {#nautilus-data-client}

`TardisDataClientConfig` และ `TardisDataClientFactory` เชื่อม stream ของ Tardis Machine ที่ตั้งค่าไว้
เข้ากับ Nautilus node configuration จะเลือกโหมดใดโหมดหนึ่ง:

- รายการ `options` ที่ไม่ว่างจะเชื่อมต่อกับ endpoint ย้อนหลัง `ws-replay-normalized`
- เมื่อ `options` ว่าง รายการ `stream_options` ที่ไม่ว่างจะเชื่อมต่อกับ endpoint แบบ real-time
  `ws-stream-normalized` และ reconnect อัตโนมัติหลังการขัดจังหวะที่กู้คืนได้

ต้องมีรายการใดรายการหนึ่งที่ไม่ว่าง หากตั้งทั้งสองไว้ `options` จะเลือกโหมด historical replay request option
เหล่านี้เป็นตัวกำหนด exchange, symbol และ data type ต้นทาง command การ subscribe ของ Nautilus ไม่ได้เพิ่ม
หรือนำข้อมูลออกจาก WebSocket ของ Tardis Machine

เมื่อ `options` ว่าง entry ของ BitMEX ใน `stream_options` จะล้มเหลวก่อนมี network request ใด ๆ
ให้ใช้ `options` แบบย้อนหลังสำหรับการ replay BitMEX การปิด connection จาก Tardis Machine ด้วยเหตุผล
exchange ที่ไม่รองรับจะยุติ live stream โดยไม่ reconnect

data client จะเพิ่ม `derivative_ticker` ให้ทุก request ที่ตั้งค่าไว้ เพื่อให้ publish funding rate,
mark price และ index price ได้เมื่อค่าเปลี่ยน มันยังรองรับ output อื่นใน[รูปแบบที่รองรับ](#supported-formats)
รวมถึง `OptionGreeks` และข้อมูล BBO `QuoteTick` (ตัวเลือกเสริม) จาก message `option_summary`

ดู `crates/adapters/tardis/examples/node_data_tester.rs` สำหรับตัวอย่าง replay client ฝั่ง Rust แบบสมบูรณ์
รวมถึงการลงทะเบียน factory และ config กับ live node

config ของ data client ฝั่ง Rust สามารถตั้ง `book_snapshot_output` เป็น `depth` ได้ configuration JSON
ของการ replay แบบ standalone รองรับทั้งสองค่า

## การสร้าง trade ID {#trade-id-derivation}

trade tick ใช้ trade ID ที่ venue ให้มาใน message หรือแถว CSV ของ Tardis เป็น `TradeId` หาก venue ไม่ได้ให้
trade ID (เป็น string ว่างหรือ null ในบาง exchange) parser ทั้งสองจะใช้ hash แบบ FNV-1a ที่ deterministic
hash ประกอบด้วย symbol, timestamp, ราคา, จำนวน และ side ราคาและจำนวนถูก hash ตามค่าทศนิยม ดังนั้น parser
ทั้งสองจะได้ ID เดียวกันสำหรับ event เดียวกัน โดยมีเงื่อนไขว่าค่าใน CSV แปลงไปกลับผ่าน `f64` ได้โดยไม่เปลี่ยน
event เดียวกันจาก venue จะได้ trade ID เดียวกันในทุกการ replay ทำให้การตัดข้อมูลซ้ำในขั้นถัดไปยังถูกต้อง

## ข้อจำกัดและข้อควรพิจารณา {#limitations-and-considerations}

`TardisDataClient` ไม่ได้ implement data request ของ Nautilus รวมถึง request ของ instrument, order book,
quote, trade, funding rate และ bar ให้ตั้งค่า historical replay ผ่าน `options` หรือใช้
`run_tardis_machine_replay` สำหรับ workflow ของ catalog

## การทดสอบความเข้ากันได้กับ Tardis Node {#testing-tardis-node-compatibility}

regression test ของ Deribit จะป้อน snapshot, update ในคิวที่ snapshot ครอบคลุมแล้ว และ update ใหม่
ผ่าน normalizer ของ Tardis Node ที่ติดตั้งไว้ จากนั้นตรวจ order book ของ Nautilus ทั้งหมด ต้องใช้ Node.js
ที่เข้ากันได้กับ package `tardis-dev` ที่ทดสอบแล้ว และรันแยกจาก Rust test ค่าเริ่มต้น
จาก root ของ repository:

```bash
tardis_test_dir=$(mktemp -d)
npm install --prefix "$tardis_test_dir" --ignore-scripts --no-audit --no-fund tardis-dev@18.2.2
TARDIS_NODE_MODULE="$tardis_test_dir/node_modules/tardis-dev" \
  CARGO_BUILD_JOBS=4 NEXTEST_TEST_THREADS=4 cargo nextest run --locked -p nautilus-tardis \
  --cargo-profile nextest --run-ignored only -E 'test(test_deribit_snapshot_queued_updates_with_tardis_node)'
rm -rf "$tardis_test_dir"
```

## การมีส่วนร่วม {#contributing}

:::info
สำหรับฟีเจอร์เพิ่มเติมหรือการมีส่วนร่วมพัฒนา adapter ของ Tardis โปรดดู
[contributing guide](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)
:::
