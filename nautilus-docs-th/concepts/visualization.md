# Visualization

NautilusTrader มี tearsheet แบบ HTML ที่โต้ตอบได้สำหรับวิเคราะห์ผล backtest ผ่านระบบ visualization
ที่ต่อขยายได้ซึ่งสร้างบน Plotly สามารถสร้างรายงานได้ด้วยโค้ดเพียงเล็กน้อย และเพิ่ม chart กับ theme ของตัวเองได้

:::note
ต้นฉบับอธิบายระบบ visualization นี้ผ่าน API ฝั่ง Python เท่านั้น (ติดตั้งผ่าน extra `visualization`
ซึ่งพ่วง Pandas, Plotly และ Kaleido มาด้วย) ตัวอย่างโค้ด Python ทั้งหมดในหน้านี้จึงถูกตัดออก
คงไว้เฉพาะแนวคิด รายการ chart, theme และตัวเลือก config
:::

## ภาพรวม {#overview}

ระบบ visualization ประกอบด้วยสามส่วน:

- **Chart registry**: นิยาม chart ที่แยกเป็นอิสระจากกัน และต่อขยายด้วย visualization ของตัวเองได้
- **ระบบ theme**: การจัดรูปแบบที่สม่ำเสมอ ทั้ง theme ในตัวและ theme ที่กำหนดเอง
- **Configuration**: การระบุแบบ declarative ว่าจะ render อะไรและแสดงผลอย่างไร

tearsheet ถูกเขียนออกมาเป็น **ไฟล์ HTML ที่สมบูรณ์ในตัวเอง** เปิดดูได้ในเบราว์เซอร์สมัยใหม่ทุกตัว
แชร์ให้ผู้เกี่ยวข้อง หรือเก็บไว้อ้างอิงภายหลังได้ หาก output path มีนามสกุลของไฟล์ภาพนิ่ง (เช่น `.png`
หรือ `.pdf`) ระบบจะ export เป็นภาพนิ่งผ่าน Kaleido

## Tearsheet {#tearsheets}

tearsheet คือรายงาน performance ที่รวม chart และสถิติหลายอย่างไว้ใน visualization แบบโต้ตอบได้ชิ้นเดียว
tearsheet จะถูกสร้างหลังจาก backtest รันเสร็จ และให้ผลตอบรับเชิงภาพเกี่ยวกับ performance ของ strategy ได้ทันที

### เริ่มต้นอย่างรวดเร็ว {#quick-start}

การสร้าง tearsheet ด้วยค่าเริ่มต้นจะได้ไฟล์ HTML ที่มี chart ค่าเริ่มต้นครบทุกตัว ใช้ light theme และจัด
layout อัตโนมัติ เปิดไฟล์ผลลัพธ์ (เช่น `backtest_results.html`) ในเบราว์เซอร์เพื่อดู tearsheet แบบโต้ตอบได้

### Input จากผล backtest {#backtest-result-input}

สามารถส่ง `BacktestResult` ที่ได้จาก backtest ที่รันเสร็จแล้วเข้าไปโดยไม่ต้องมี node เพื่อสร้าง tearsheet
จากผลลัพธ์อย่างเดียวได้

หากต้องการแสดงยอดคงเหลือเริ่มต้นของ account จากรายงานของ node ต้องเก็บ state ของ node ไว้ด้วย
และ node ยังจำเป็นเมื่อ tearsheet ที่ตั้งค่าไว้มี chart ที่อาศัยข้อมูลจาก cache เช่น `bars_with_fills`

ให้ทำตาม[การตั้งค่า `BacktestNode`](backtesting/apis-and-runs.md#high-level-api) ให้ครบ
โดยตั้ง `dispose_on_completion=False` ใน `BacktestRunConfig` จากนั้นส่งผลลัพธ์ที่รันเสร็จแล้วพร้อม node
ที่เก็บไว้เข้าไป

:::warning
การส่ง node ที่ run configuration ที่ตรงกันเปิดการ dispose ไว้จะทำให้เกิด error เพราะ cache และรายงาน
ของ node นั้นไม่มีอยู่แล้ว
:::

### การปรับแต่ง {#customization}

กำหนดได้ว่าจะให้ chart ใดปรากฏและจัดรูปแบบอย่างไร ผ่าน `TearsheetConfig` (รายการ chart, theme,
ความสูง เป็นต้น)

### การกรองตามสกุลเงิน {#currency-filtering}

สำหรับ backtest ที่มีหลายสกุลเงิน สามารถกรองสถิติให้เหลือเฉพาะสกุลเงินที่ต้องการได้ด้วยพารามิเตอร์ `currency`

เมื่อ `currency` เป็น `None` (ค่าเริ่มต้น) สถิติของทุกสกุลเงินจะแสดงแยกกันใน tearsheet

สำหรับ input แบบ `BacktestEngine` chart ที่อิงผลตอบแทน (return) ต้องมีสกุลเงินเดียว chart เหล่านี้คำนวณจาก
snapshot ของ equity ใน portfolio โดย fallback ไปใช้รายงานของ account และสร้างไม่ได้สำหรับ account ที่มีหลาย
สกุลเงินปนกันหากไม่กรอง ดังนั้นสำหรับ backtest หลายสกุลเงินให้ส่ง `currency` เพื่อให้ chart ผลตอบแทนใช้
สกุลเงินที่เลือก

สำหรับ input แบบ `BacktestResult` พารามิเตอร์ `currency` จะกรองสถิติ PnL และยอดคงเหลือของ account
ส่วน **ชุดข้อมูลผลตอบแทนที่เก็บไว้ในผลลัพธ์จะไม่เปลี่ยนแปลง**

## Chart ที่มีให้ใช้ {#available-charts}

tearsheet สามารถรวม chart ในตัวต่อไปนี้ได้ในทุกรูปแบบการผสม:

| ชื่อ Chart          | ประเภท       | คำอธิบาย                                                 |
| ----------------- | ----------- | ------------------------------------------------------- |
| `run_info`        | Table       | metadata ของการรันและยอดคงเหลือของ account                  |
| `stats_table`     | Table       | สถิติ performance (PnL, ผลตอบแทน, ตัวชี้วัดทั่วไป)              |
| `equity`          | Line        | ผลตอบแทนสะสมตามเวลา พร้อม benchmark (ถ้ามี)                 |
| `drawdown`        | Area        | เปอร์เซ็นต์ drawdown จากจุดสูงสุดของ equity                   |
| `monthly_returns` | Heatmap     | เปอร์เซ็นต์ผลตอบแทนรายเดือนของ portfolio จัดเรียงตามปี          |
| `distribution`    | Histogram   | การกระจายของค่าผลตอบแทนแต่ละค่า                              |
| `rolling_sharpe`  | Line        | Sharpe ratio แบบ rolling 60 วัน                            |
| `yearly_returns`  | Bar         | เปอร์เซ็นต์ผลตอบแทนรายปี                                     |
| `bars_with_fills` | Candlestick | price bar (OHLC) โดยมี fill ของ order ซ้อนเป็น marker        |

chart registry เก็บ chart ทั้งหมดไว้ ส่วน chart object ใน `TearsheetConfig.charts` ใช้กำหนดค่า chart
เหล่านั้น (chart object แต่ละตัวจับคู่กับชื่อ chart ในตัวหนึ่งชื่อ)

### ตารางข้อมูลการรัน {#run-information-table}

chart `run_info` แสดง metadata สำคัญของการรัน backtest:

- Run ID, เวลาเริ่ม, เวลาสิ้นสุด
- ช่วงเวลาของ backtest (วันเริ่ม/วันสิ้นสุด)
- จำนวน iteration ทั้งหมดที่ประมวลผล
- จำนวน event, order และ position
- ยอดคงเหลือเริ่มต้นและสิ้นสุดของ account (แยกตามสกุลเงิน)

ตารางนี้จะอยู่ตำแหน่งบนซ้ายโดยค่าเริ่มต้น

### ตารางสถิติ performance {#performance-statistics-table}

chart `stats_table` แสดงตัวชี้วัด performance แบ่งเป็นหมวด:

- **PnL Statistics** (แยกตามสกุลเงิน): Total PnL, win rate, profit factor ฯลฯ
- **Returns Statistics**: Sharpe ratio, Sortino ratio, max drawdown ฯลฯ
- **General Statistics**: จำนวน trade ทั้งหมด, ระยะเวลาถือ trade เฉลี่ย ฯลฯ

ตารางนี้จะอยู่ตำแหน่งบนขวาโดยค่าเริ่มต้น

### Equity curve {#equity-curve}

chart `equity` พล็อตผลตอบแทนสะสมตลอดช่วง backtest เมื่อส่ง `benchmark_returns` เข้าไปใน
`create_tearsheet()` ระบบจะซ้อน benchmark ไว้เพื่อเปรียบเทียบ (เช่น ผลตอบแทนของดัชนี S&P 500 โดย index
ต้องเป็น datetime และสอดคล้องกับ timeframe ของผลตอบแทนของ strategy)

ชุดข้อมูล benchmark จะถูกพล็อตตามที่ให้มาโดยไม่ปรับแต่ง ควรตรวจสอบให้ index ตรงกับวันที่ของผลตอบแทน
ของ strategy เพื่อให้การเปรียบเทียบแม่นยำ

### ผลตอบแทนรายเดือนและรายปี {#monthly-and-yearly-returns}

chart `monthly_returns` และ `yearly_returns` ใช้ผลตอบแทนแบบทบต้น (time-weighted) เป็นค่าเริ่มต้น แต่ละช่อง
วัดกำไรของช่วงนั้นเทียบกับยอดคงเหลือ ณ ต้นช่วง และผลตอบแทนของแต่ละช่วงทบต้นรวมกันได้เป็นผลตอบแทนรวม

ตั้ง `compounding=False` เพื่อรายงานผลตอบแทนแบบธรรมดา (ไม่ทบต้น) ที่วัดเทียบกับเงินทุนเริ่มต้นคงที่
แต่ละช่องจะวัดกำไรของช่วงนั้นเป็นเปอร์เซ็นต์ของเงินทุนเริ่มต้น ดังนั้นผลตอบแทนของแต่ละช่วงจะ *บวก* รวมกันได้
เป็นผลตอบแทนรวมแทนที่จะทบต้น นี่คืออัตราผลตอบแทนแบบ nominal ซึ่งเป็นแนวปฏิบัติสำหรับ strategy ที่ใช้
เงินทุนคงที่ เทรดด้วยขนาดคงที่ และถอนกำไรออก

ฟังก์ชันแบบ standalone `create_monthly_returns_heatmap()` และ `create_yearly_returns()` รับอาร์กิวเมนต์
`compounding` แบบเดียวกัน เพื่อให้ตัวเลขแบบไม่ทบต้นสะท้อนเงินทุนคงที่ได้อย่างถูกต้อง ควรกำหนดขนาด position
เป็นจำนวนคงที่ แทนที่จะเป็นสัดส่วนของ equity ปัจจุบัน มิฉะนั้นช่วงหลัง ๆ จะดูสูงเกินจริงเมื่อยอดคงเหลือเติบโตขึ้น

## Theme {#themes}

theme ควบคุมการจัดรูปแบบของ chart ทั้งสี ฟอนต์ และพื้นหลัง NautilusTrader มี theme ในตัวสี่แบบ:

| ชื่อ Theme        | คำอธิบาย                                        | กรณีใช้งาน                       |
| --------------- | --------------------------------------------- | ------------------------------ |
| `plotly_white`  | light theme เรียบสะอาด หัวตารางสีเทาเข้ม            | ค่าเริ่มต้น, รายงานแบบมืออาชีพ        |
| `plotly_dark`   | พื้นหลังสีเข้มกับสีมาตรฐานของ Plotly                  | สภาพแวดล้อมแสงน้อย                 |
| `nautilus`      | light theme ใช้สีแบรนด์ของ NautilusTrader           | light mode อย่างเป็นทางการ         |
| `nautilus_dark` | dark theme ใช้สีเอกลักษณ์ teal/cyan                | dark mode อย่างเป็นทางการ          |

### การเลือก theme {#selecting-a-theme}

ระบุ theme ใน `TearsheetConfig` ผ่านฟิลด์ `theme` (เช่น `"nautilus_dark"`)

### Theme แบบกำหนดเอง {#custom-themes}

สามารถลงทะเบียน theme ของตัวเองเพื่อให้แบรนด์สม่ำเสมอในทุก visualization โดยระบุชื่อ, Plotly template ฐาน
และชุดสี (`primary`, `positive`, `negative`, `neutral`, `background`, `grid` และสีของตารางแบบไม่บังคับ
`table_section`, `table_row_odd`, `table_row_even`, `table_text`)

ระบบ theme จะคำนวณสี `table_*` ค่าเริ่มต้นจากสี `background` และ `grid` ค่าเริ่มต้นเหล่านี้ช่วยให้ยังเข้ากันได้
กับ theme ที่ลงทะเบียนไว้ก่อนที่จะมีสีเฉพาะสำหรับตาราง

## Configuration {#configuration}

`TearsheetConfig` ให้การควบคุมแบบ declarative ในการสร้าง tearsheet: chart ที่จะรวม, theme, ชื่อเรื่อง,
ความสูง, benchmark และ layout ของ grid (`GridLayout` ที่ระบุจำนวนแถว/คอลัมน์ ความสูงของแต่ละแถว และระยะห่าง)

### พารามิเตอร์ของ configuration {#configuration-parameters}

| Parameter           | Type                   | Default          | คำอธิบาย                              |
| ------------------- | ---------------------- | ---------------- | ----------------------------------- |
| `charts`            | `list[TearsheetChart]` | Built-ins        | chart ที่จะรวม เรียงตามลำดับ             |
| `theme`             | `str`                  | `"plotly_white"` | ชื่อ theme สำหรับการจัดรูปแบบ             |
| `layout`            | `GridLayout`           | `None`           | layout ของ subplot grid แบบกำหนดเอง   |
| `title`             | `str`                  | Auto-generated   | ชื่อเรื่องของ tearsheet                  |
| `include_benchmark` | `bool`                 | `True`           | แสดง benchmark เมื่อมีการส่งเข้ามา        |
| `benchmark_name`    | `str`                  | `"Benchmark"`    | ชื่อที่แสดงของ benchmark                |
| `height`            | `int`                  | `1500`           | ความสูงรวมหน่วยพิกเซล                   |
| `show_logo`         | `bool`                 | `True`           | สงวนไว้สำหรับการแสดงโลโก้ในอนาคต           |

เมื่อ `layout` เป็น `None` จำนวน chart จะเป็นตัวกำหนดขนาด grid และความสูงของแถว chart ค่าเริ่มต้น 8 ตัว
ใช้ grid 4x2 โดยมีความสูง `[0.50, 0.22, 0.16, 0.12]` ซึ่งให้พื้นที่กับตารางแถวบนสุดมากกว่า

## Chart แบบกำหนดเอง {#custom-charts}

รูปแบบ registry ช่วยให้เพิ่ม chart ของตัวเองได้ chart คือฟังก์ชันที่ render trace ลงบน figure object ของ Plotly

### การลงทะเบียน chart แบบกำหนดเอง {#registering-a-custom-chart}

ฟังก์ชัน chart แบบกำหนดเองลงทะเบียนด้วย `register_chart` เพื่อใช้แบบ standalone (ผ่าน `get_chart()` /
`list_charts()`) โดย signature ของฟังก์ชันควรสอดคล้องกับฟังก์ชัน chart ในตัวเพื่อความสม่ำเสมอ

### การผนวกเข้ากับ tearsheet {#tearsheet-integration}

หากต้องการผนวกเข้ากับ tearsheet ให้วางตำแหน่งใน grid ได้ถูกต้อง ให้ใช้ `register_tearsheet_chart` ต่างจาก
`register_chart` (ซึ่งลงทะเบียนฟังก์ชัน standalone ที่คืน figure ของตัวเอง) renderer ของ tearsheet จะวาด trace
ลงบนช่องของ subplot grid ที่ใช้ร่วมกันโดยตรง signature ของมันจึงรับ `fig` เป้าหมาย พร้อม `row` และ `col`
ที่จะ render ลงไป จากนั้นชื่อที่ลงทะเบียนไว้จะใช้ใน `TearsheetConfig.charts` ได้

ฟังก์ชัน renderer จะได้รับข้อมูลที่จำเป็นทั้งหมด (ผลตอบแทน, สถิติ, theme configuration) และ render ลงบนตำแหน่ง
subplot ที่กำหนดโดยตรง

## การวิเคราะห์แบบ offline {#offline-analysis}

ในกรณีที่มีสถิติที่คำนวณไว้ล่วงหน้าแต่ไม่มี instance ของ `BacktestEngine` ให้ใช้ API ระดับล่าง
`create_tearsheet_from_stats()` ซึ่งรับ `stats_pnls` (แยกตามสกุลเงิน), `stats_returns`, `stats_general` และชุด
ผลตอบแทนรายวัน

key ของ dictionary ควรตรงกับค่าที่ได้จาก `engine.get_result().stats_pnls`,
`engine.get_result().stats_returns` และ `engine.get_result().stats_general`

แนวทางนี้มีประโยชน์สำหรับ:

- วิเคราะห์ผลจากการรัน backtest หลายครั้งที่เก็บแยกกันไว้
- เปรียบเทียบ strategy โดยใช้ตัวชี้วัดที่คำนวณไว้ล่วงหน้า
- ผนวกเข้ากับ pipeline วิเคราะห์ภายนอก

## แนวปฏิบัติที่ดี {#best-practices}

### การเลือก chart {#chart-selection}

- ใช้ chart ค่าเริ่มต้นสำหรับการวิเคราะห์เชิงสำรวจเพื่อดูตัวชี้วัดทั้งหมดที่มี
- ปรับแต่ง chart เมื่อรู้แล้วว่าตัวชี้วัดใดสำคัญกับ strategy ของตน
- ตัด chart ที่ไม่เกี่ยวข้องออกเพื่อลดความรกทางสายตาและขนาดไฟล์

### การใช้ theme {#theme-usage}

- ใช้ `plotly_white` สำหรับรายงานและการนำเสนอแบบมืออาชีพ
- ใช้ `nautilus_dark` สำหรับเอกสารทางการหรือการดูในที่แสงน้อย
- สร้าง theme ของตัวเองให้ตรงกับแนวทางภายในองค์กรหรือความชอบส่วนตัว

### ข้อควรพิจารณาด้าน performance {#performance-considerations}

- ไฟล์ HTML ของ tearsheet ฝังข้อมูลทั้งหมดไว้ในตัว และอาจมีขนาดหลายเมกะไบต์สำหรับ backtest ที่ยาว
- พิจารณาสร้าง tearsheet แยกสำหรับช่วงเวลาวิเคราะห์ที่ต่างกัน
- สำหรับชุดข้อมูลขนาดใหญ่มาก ให้ใช้ฟังก์ชัน chart แยกตัวแทน tearsheet เต็มรูปแบบ

### การผนวกสถิติแบบกำหนดเอง {#custom-statistics-integration}

ใช้ chart แบบกำหนดเองคู่กับสถิติที่ส่งผ่าน dictionary `stats_pnls`, `stats_returns` และ `stats_general`
ชุดเดียวกับที่ chart ในตัวของ tearsheet ใช้

สำหรับ input แบบ `BacktestEngine` ค่าเหล่านี้มาจาก `engine.get_result()` ดังนั้นสถิติที่ลงทะเบียนด้วย
`Portfolio.register_statistic()` จะไปถึง tearsheet ได้โดยไม่ต้องเชื่อมต่ออะไรเพิ่ม ดู
[สถิติแบบกำหนดเอง](portfolio.md#custom-statistics)

สำหรับการวิเคราะห์แบบ offline ให้ส่ง dictionary ที่เข้ากันได้ (เช่น `"Sharpe Ratio (252 days)"` หรือ
`"Custom Volatility Score"` ใน `stats_returns`) เข้าไปใน `create_tearsheet_from_stats()` โดยตรง

## ระดับของ API {#api-levels}

ระบบ visualization มี API สองระดับ รวมถึงฟังก์ชัน chart แบบ standalone:

### API ระดับสูง {#high-level-api}

แนะนำสำหรับกรณีใช้งานส่วนใหญ่: `create_tearsheet(engine=..., config=...)`

ดึงข้อมูลจาก `BacktestEngine` หรือ `BacktestResult` โดยอัตโนมัติ สร้าง chart ทั้งหมดที่ตั้งค่าไว้ และได้ tearsheet
HTML ที่สมบูรณ์

### API ระดับล่าง {#low-level-api}

สำหรับการปรับแต่งขั้นสูงหรือการวิเคราะห์แบบ offline: `create_tearsheet_from_stats()` ซึ่งรับ `stats_pnls`,
`stats_returns`, `stats_general`, `returns`, `run_info`, `account_info` และ `config`

ให้การควบคุม input ข้อมูลอย่างละเอียด และรองรับการวิเคราะห์สถิติที่คำนวณไว้ล่วงหน้า

### ฟังก์ชัน chart แบบ standalone {#standalone-chart-functions}

ฟังก์ชัน chart แต่ละตัวใช้งานแยกกันได้อย่างอิสระ เพื่อสร้าง visualization HTML สำหรับจุดประสงค์เดียว
หรือ figure ของ Plotly สำหรับ workflow การวิเคราะห์แบบกำหนดเอง

#### Price bar พร้อม fill {#price-bars-with-fills}

ฟังก์ชัน `create_bars_with_fills` สร้าง candlestick chart ที่มี fill ของ order ซ้อนอยู่ ซึ่งมีประโยชน์ในการวิเคราะห์
การ execution ของ strategy เทียบกับ price action ด้วยสายตา ใช้แบบ standalone หรือรวมไว้ใน tearsheet ได้
(ผ่าน chart object `TearsheetBarsWithFillsChart` และใส่ได้หลายตัวใน tearsheet เดียวสำหรับหลาย instrument)

visualization จะแสดง candlestick ของ price action แบบ OHLC พร้อม marker รูปสามเหลี่ยมแทน fill ของ order
(สามเหลี่ยมชี้ขึ้นสำหรับฝั่งซื้อ และชี้ลงสำหรับฝั่งขาย ใช้สี positive และ negative ของ theme) chart ที่ต้องการ
configuration เพิ่มเติม (เช่น `bar_type`) จะรับพารามิเตอร์เหล่านั้นโดยตรงบน chart object (ตัวอย่างเช่น
`TearsheetBarsWithFillsChart(bar_type=...)`)

ฟังก์ชัน chart แยกตัวอื่น ๆ ได้แก่ `create_equity_curve`, `create_drawdown_chart`,
`create_monthly_returns_heatmap` และอื่น ๆ

## คู่มือที่เกี่ยวข้อง {#related-guides}

- [Backtesting](backtesting/) - เรียนรู้วิธีรัน backtest ที่ให้ผลลัพธ์สำหรับ tearsheet
- [Reports](reports.md) - ทำความเข้าใจสถิติเบื้องหลังที่แสดงใน tearsheet
- [Portfolio](portfolio.md) - สำรวจการติดตาม portfolio และตัวชี้วัด performance
