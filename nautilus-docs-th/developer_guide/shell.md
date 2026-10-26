# Shell Scripts

หน้านี้กำหนดว่าเมื่อใดควรสร้าง shell script และควรตั้งชื่อ, เขียน, เรียกใช้ และทดสอบอย่างไร
ค่าเริ่มต้นคือ Bash ใช้ POSIX `sh` เฉพาะเมื่อ caller ที่รองรับไม่สามารถพึ่งพาว่ามี Bash ติดตั้งอยู่

policy นี้ใช้กับไฟล์ shell ทั่วทั้ง repository รวมถึง `scripts/`, `scripts/ci/`,
`.pre-commit-hooks/` และ directory ของ component ต่าง ๆ

## เมื่อใดควรเขียน script {#when-to-write-a-script}

ก่อนเพิ่ม script ให้ค้นหาก่อนว่ามี script, Make target หรือ tool ที่ติดตั้งไว้ซึ่งทำงานนั้นได้อยู่แล้วหรือไม่
เพิ่ม script เมื่อมันทำให้ logic เชิงขั้นตอนมีแหล่งความจริงเดียวที่ทดสอบได้และผ่าน lint

| ตำแหน่ง        | รับผิดชอบ                                                                 | มอบหมายต่อ                                               |
| -------------- | ------------------------------------------------------------------------ | -------------------------------------------------------- |
| GitHub Actions | event, permission, matrix, runner, secret และ hosted action              | พฤติกรรม shell ที่มีหลายขั้นตอน                           |
| `Makefile`     | task ที่ค้นหาได้ง่าย, dependency, ตัวแปร และขีดจำกัดของ concurrency        | control flow ที่ซับซ้อน                                    |
| Shell script   | การ validate, ลำดับคำสั่งที่ใช้ซ้ำได้, การ retry และการแปลงข้อมูล           | การ orchestrate workflow และการจัดการ build dependency   |

ควรใช้ script เมื่อ:

- ลำดับคำสั่งถูกใช้โดยมากกว่าหนึ่ง workflow, Make target หรือ task ของนักพัฒนา
- step แบบ inline ใน GitHub Actions มี branch, loop, retry หรือการจัดการความล้มเหลวที่ควรทดสอบ
  นอก workflow
- recipe ของ Make ต้องการ logic เชิงขั้นตอนมากจนการ quote, การส่งต่อ error หรือพฤติกรรมของแต่ละ
  platform ทำให้ review ได้ยาก
- task ด้าน maintenance หรือ release ที่ทำซ้ำ ๆ ได้ประโยชน์จาก ShellCheck, `shfmt` และการทดสอบแบบเจาะจง

ให้คงคำสั่งไว้แบบ inline เมื่อมันสั้น ใช้ครั้งเดียว และอ่านเข้าใจง่ายกว่าเมื่ออยู่ใน caller อย่าห่อคำสั่ง
ที่เสถียรเพียงคำสั่งเดียวเพียงเพื่อให้มีไฟล์เพิ่มอีกไฟล์

เมื่อแยก logic ออกจาก GitHub Actions ให้คง expression อย่าง `${{ github.ref }}`, permission,
การเลือก secret และการเลือก runner ไว้ใน workflow ส่งค่าทั่วไปให้ script ผ่าน argument หรือ
environment variable และ exit status ของ script ต้องเป็น exit status ของ step นั้น

## เลือก shell และนามสกุลไฟล์ {#choose-the-shell-and-extension}

นามสกุลไฟล์บอกภาษา shell ไม่ได้บอกว่าไฟล์นั้น execute ได้หรือ source ได้

| นามสกุล | Interpreter | ใช้เมื่อ                                                                    |
| ------- | ----------- | --------------------------------------------------------------------------- |
| `.bash` | Bash        | ค่าเริ่มต้นสำหรับ script ใหม่และไฟล์ Bash ที่ source ได้                    |
| `.sh`   | POSIX `sh`  | เฉพาะเมื่อ caller ที่รองรับมีความจำเป็นจริงที่ต้องรันโดยไม่มี Bash          |

ใช้ `#!/usr/bin/env bash` สำหรับไฟล์ `.bash` และ `#!/usr/bin/env sh` สำหรับไฟล์ `.sh` ให้คง shebang ไว้
แม้ว่า Make หรือ GitHub Actions จะเรียกไฟล์ผ่าน `bash` หรือ `sh` ก็ตาม เพราะ tool และ caller ที่เรียกโดยตรง
ใช้ shebang ในการระบุ interpreter

Bash เป็นตัวเลือกที่แนะนำสำหรับการพัฒนาทั่วไป, Make และ script ใน CI เพราะ repository พึ่งพา Bash อยู่แล้ว
และฟีเจอร์ของมันทำให้โค้ด shell ที่ไม่เล็กน้อยอ่านง่ายขึ้น ฟีเจอร์เหล่านี้ได้แก่ `pipefail`, `[[ ... ]]`,
array, process substitution และตัวแปรภายใน function

script เล็ก ๆ สำหรับ bootstrap หรือ wrapper อาจมี interface ที่รองรับซึ่งต้องหลีกเลี่ยงการพึ่งพา Bash
ใช้ POSIX `sh` เฉพาะกับ script ลักษณะนี้เท่านั้น และทดสอบ script ภายใต้ `/bin/sh` เป้าหมาย syntax ที่เรียบง่าย
เพียงอย่างเดียวไม่ได้พิสูจน์ความเข้ากันได้กับ POSIX

ชื่อไฟล์ที่มีอยู่เดิมเกิดขึ้นก่อน policy เรื่องนามสกุลนี้ ไฟล์ `.sh` บางไฟล์จึงมีโค้ด Bash อยู่ ให้ถือ shebang
ของไฟล์เหล่านั้นเป็นแหล่งความจริง อย่าเปลี่ยนชื่อ script ที่มีอยู่เพียงเพื่อเปลี่ยนนามสกุล ให้ใช้ policy นี้กับ
ไฟล์ใหม่ และกับการเปลี่ยนชื่อในขอบเขตที่ได้อัปเดตทุกจุดที่เรียกใช้และทุกเอกสารแล้ว

## กำหนดเป้าหมายด้าน portability {#define-the-portability-target}

script ต้องรองรับทุก platform ที่ caller ของมันรันอยู่ หากจุดประสงค์ของ script ไม่ได้ระบุเป้าหมายที่แคบกว่า
ให้เขียนสำหรับ Linux, macOS และ Windows ผ่าน Git Bash, MSYS2 หรือ WSL

ใช้ Bash 3.2 เป็นระดับภาษาขั้นต่ำโดยค่าเริ่มต้น เพราะมีอยู่บนระบบ macOS ที่รองรับ หลีกเลี่ยงฟีเจอร์ของ
Bash 4+ เว้นแต่ทุก caller จะจัดเตรียมเวอร์ชันที่ใหม่กว่าไว้ ฟีเจอร์ Bash 4+ ที่พบบ่อยและทางเลือกที่ portable ได้แก่:

| ฟีเจอร์                             | เวอร์ชัน Bash | ทางเลือกที่ portable                  |
| ---------------------------------- | ------------ | ------------------------------------ |
| Associative array (`declare -A`)   | 4.0+         | ไฟล์, array ธรรมดา หรือ function     |
| `readarray` / `mapfile`            | 4.0+         | loop แบบ `while read`                |
| `${var,,}` / `${var^^}`            | 4.0+         | `tr` สำหรับแปลงตัวพิมพ์เล็ก/ใหญ่      |

script ที่ใช้เฉพาะใน CI อาจใช้ Bash เวอร์ชันใหม่กว่าหรือ tool เฉพาะ platform ได้ เมื่อทุก workflow ที่เรียกใช้
รับประกัน environment นั้น ให้บันทึกข้อจำกัดไว้ใกล้กับโค้ดที่พึ่งพามัน path ภายใต้ `scripts/ci/` ไม่ได้ทำให้
script เป็น Linux-only โดยอัตโนมัติ เพราะ CI ก็ใช้ runner ที่เป็น macOS และ Windows ด้วย

### System utility {#system-utilities}

ควรใช้ option ที่รองรับทั้ง utility ของ GNU และ BSD เมื่อไม่มีรูปแบบที่ใช้ร่วมกันได้ ให้ตรวจจับความสามารถ
หรือระบบปฏิบัติการ แล้ว implement ทั้งสองรูปแบบ

| การทำงาน            | รูปแบบ GNU      | รูปแบบ BSD หรือ macOS | แนวทางที่ portable                                              |
| ------------------- | -------------- | -------------------- | -------------------------------------------------------------- |
| `sed` แบบแก้ในไฟล์   | `sed -i`       | `sed -i ''`          | ใช้ backup suffix เช่น `sed -i.bak` แล้วลบไฟล์ backup ทิ้ง      |
| ขนาดไฟล์            | `stat -c '%s'` | `stat -f '%z'`       | ลองหรือเลือกรูปแบบที่รองรับ                                    |
| SHA-256             | `sha256sum`    | `shasum -a 256`      | ตรวจจับคำสั่ง และจัดการ output ให้เหมือนกัน                     |
| Canonical path      | `readlink -f`  | ไม่มีรูปแบบร่วม       | หลีกเลี่ยง หรือ resolve จาก directory ที่ทราบด้วย `pwd`         |
| Extended matching   | `grep -P`      | ไม่มีรูปแบบร่วม       | ใช้ `grep -E` เมื่อเขียน pattern เดียวกันได้                    |
| เวลาระดับนาโนวินาที  | `date +%N`     | ไม่มีรูปแบบร่วม       | ใช้ run ID ที่มีอยู่หรือ `$RANDOM` สำหรับ cache busting          |

quote path และ expansion เพื่อไม่ให้ช่องว่างเปลี่ยนขอบเขตของ argument อย่าสมมติว่า path ของ filesystem
แยกตัวพิมพ์เล็ก/ใหญ่ ใช้ path แบบ relative กับ repository ได้ก็ต่อเมื่อ resolve root ของ repository จาก
ตำแหน่งของ script แล้วเท่านั้น ไม่ใช่จาก working directory ของ caller

ใช้เฉพาะคำสั่งที่ติดตั้งโดยขั้นตอน setup ของ development หรือ runner ที่มีเอกสารกำกับ หากจำเป็นต้องใช้คำสั่ง
ที่เป็นตัวเลือกเสริม ให้ตรวจสอบด้วย `command -v` และแจ้งวิธีติดตั้งหรือทางเลือกทดแทน repository เก็บไฟล์
text ด้วย line ending แบบ LF ผ่าน `.gitattributes` อย่าเพิ่ม line ending เฉพาะ platform

## ตำแหน่งและการตั้งชื่อ script {#place-and-name-scripts}

- วางคำสั่งสำหรับ development และ maintenance ทั่วไปไว้ใต้ `scripts/`
- วางคำสั่ง build, test, publication และ verification ที่เฉพาะกับ workflow ไว้ใต้ `scripts/ci/`
- วางการตรวจสอบ repository ที่ pre-commit เรียกใช้ไว้ใต้ `.pre-commit-hooks/`
- เก็บ script ที่เฉพาะกับ component ไว้ข้าง ๆ component นั้น เมื่อการย้ายไปไว้ใน `scripts/` จะทำให้
  มองไม่เห็นว่าใครเป็นเจ้าของ

- ใต้ `scripts/` และใน directory ของ component ให้ใช้ kebab-case ตัวพิมพ์เล็ก ตั้งชื่อ regression script
  เป็น `test-<script-name>.bash` หรือ `test-<script-name>.sh` เพื่อให้คู่ที่ถูกทดสอบอยู่ด้วยกัน
- ใต้ `.pre-commit-hooks/` ให้ใช้ snake_case ตัวพิมพ์เล็ก และให้ตรงกับกลุ่มชื่อ `check_*` และ
  `test_check_*` ที่มีอยู่

ให้แต่ละ script เน้นงานเดียว และขยาย script ที่มีอยู่เมื่อ logic ใหม่มีหน้าที่เดียวกัน

## จัดโครงสร้าง script ให้รันได้อย่างเชื่อถือได้ {#structure-scripts-for-reliable-execution}

Bash script แบบ standalone ควรเริ่มต้นด้วย:

```bash
#!/usr/bin/env bash
set -euo pipefail
```

ใช้ `set -eu` ใน POSIX `sh` script แบบ standalone POSIX ไม่ได้กำหนด `pipefail` ไว้ ดังนั้นให้ตรวจสอบ
พฤติกรรมของ pipeline เองเมื่อต้องการให้ความล้มเหลวส่งต่อออกมา หาก script ไม่สามารถใช้ option เหล่านี้ได้
ให้อธิบาย control flow ที่ทำให้ option นั้นไม่ปลอดภัย

ทำตามข้อกำหนดต่อไปนี้:

- validate argument และ environment variable ที่จำเป็นก่อนเปลี่ยน state แสดงข้อความ usage
  แบบกระชับ และ exit ด้วยค่าที่ไม่ใช่ศูนย์เมื่อ input ไม่ถูกต้อง
- quote parameter expansion ใช้ Bash array สำหรับรายการ argument แทนการต่อ string เป็นคำสั่ง
  และอย่าใช้ `eval`
- ให้ output ที่เครื่องอ่านได้อยู่บน standard output และข้อความวินิจฉัยอยู่บน standard error
  เมื่อ caller ดักจับผลลัพธ์
- อย่าปิดท้าย output สถานะตามปกติด้วยจุด (period) ให้คงเครื่องหมายวรรคตอนไว้เมื่อ output เป็นประโยค
  อธิบายหรือประโยควินิจฉัยที่สมบูรณ์
- สร้างไฟล์ชั่วคราวด้วย `mktemp`, ลงทะเบียนการ cleanup ด้วย `trap` และจำกัดการ cleanup ไว้เฉพาะ
  path ที่ script สร้างขึ้นเท่านั้น
- จำกัดจำนวนการ retry, รายงานความล้มเหลวครั้งสุดท้าย และคืนสถานะที่ไม่ใช่ศูนย์เมื่องานที่ร้องขอ
  ไม่เสร็จสมบูรณ์
- อย่าพิมพ์ secret หรือเปิด command tracing รอบ ๆ credential ส่ง secret ผ่าน environment หรือช่องทาง
  รับ secret ที่ tool รองรับ
- ใช้ comment เฉพาะสำหรับข้อจำกัดหรือพฤติกรรมที่ตัวคำสั่งไม่ได้แสดงให้เห็นชัด

ให้สิทธิ์ execute แก่ script แบบ standalone เมื่อผู้ใช้หรือ tool เรียกมันแบบ `./path` ไฟล์ที่ใช้ source
ไม่จำเป็นต้องมีสิทธิ์ execute ควรรัน script ใน child process ให้ source ไฟล์เฉพาะเมื่อ caller ต้องใช้
function หรือ shell state ร่วมกัน

ไฟล์ Bash ที่ source ได้ต้องไม่เรียก `exit` หรือเปลี่ยน shell option ของ caller ให้คืน error จาก function
และให้ caller เลือก error policy เอง ใช้ `${BASH_SOURCE[0]}` แทน `$0` เพื่อ resolve ตำแหน่งของไฟล์ที่ถูก source

หาก script ต้องมีหลาย function:

1. นิยาม `main` ไว้ก่อน
1. วาง function ที่ถูกเรียกไว้ใต้ caller ของมัน
1. เรียก `main "$@"` หลังจากนิยามทั้งหมดแล้ว

วิธีนี้ทำให้เห็นงานหลักอยู่ด้านบน ในขณะที่ยังรับประกันว่าทุก function มีอยู่ก่อนเริ่มรัน

## การทำงานร่วมกับ Make และ GitHub Actions {#integrate-with-make-and-github-actions}

Make target ควรเป็นคำสั่งที่เสถียรและค้นหาได้ง่ายสำหรับนักพัฒนา ให้คง dependency ของ target,
ตัวแปรของ build และขีดจำกัดของ concurrency ไว้ใน Makefile แล้วเรียก script ด้วย interpreter ที่ระบุชัดเจน
ตัวอย่างเช่น target `check-generated-drift` มอบงานเชิงขั้นตอนให้ `scripts/ci/check-generated-drift.bash`

GitHub Actions ควรส่ง context ของ workflow ผ่าน environment variable ที่ตั้งชื่อไว้ และเรียก script เดียวกับ
ที่ใช้ในเครื่องเมื่อทำได้ build workflow ทำตามขอบเขตนี้: workflow เป็นเจ้าของเงื่อนไขของ matrix และ
`TARGET_DIR` แล้วจึงเรียก `scripts/ci/check-generated-drift.bash`

หาก script เป็นคำสั่งที่ใช้ในเครื่องด้วย ให้เก็บไฟล์ output เฉพาะของ GitHub ไว้ที่ขอบเขตของ workflow
เช่น `$GITHUB_OUTPUT` และ `$GITHUB_ENV` script ที่ทำขึ้นเฉพาะสำหรับ GitHub Actions อาจเขียนไฟล์เหล่านี้ได้
เมื่อการเชื่อมต่อนั้นเป็นจุดประสงค์ที่ระบุไว้ของมัน

## Format และ lint {#format-and-lint}

การตั้งค่า pre-commit จะ format ไฟล์ shell ด้วย `shfmt` โดยใช้การย่อหน้าสองช่องว่าง, ย่อหน้า branch
ของ case, ระยะห่างของ redirect ที่สม่ำเสมอ และ parser ของ Bash จากนั้น ShellCheck จะตรวจการ quote,
expansion, control flow, portability และข้อผิดพลาดของคำสั่งที่พบบ่อย hook ที่ pin ไว้ใน
`.pre-commit-config.yaml` เป็นแหล่งความจริงของเวอร์ชันและ option ของ tool

รันทั้งสอง hook กับ script ที่เปลี่ยนแปลงจริง:

```bash
prek run shfmt --files scripts/ci/test-wheel.bash
prek run shellcheck --files scripts/ci/test-wheel.bash
```

`shfmt` จะแก้ไฟล์ในที่ ให้ review การเปลี่ยนแปลงของมันก่อนรัน ShellCheck ShellCheck เลือกภาษาจาก
shebang ดังนั้นไฟล์ `.sh` ที่มี `#!/usr/bin/env sh` จะได้รับการตรวจแบบ POSIX แม้ว่า formatter จะใช้
parser ของ Bash ร่วมกันก็ตาม

ให้การ suppress ของ ShellCheck อยู่ในบรรทัดที่แคบที่สุดที่ใช้ได้ หากโค้ดไม่ได้แสดงข้อจำกัดให้เห็นชัด ให้เพิ่ม
เหตุผลไว้ใกล้ ๆ อย่าปิดการตรวจทั้ง repository เพื่อให้ script ตัวเดียวเงียบ

repository ยังปฏิเสธไฟล์ที่ execute ได้แต่ไม่มี shebang, line ending ที่ปนกัน, ช่องว่างท้ายบรรทัด และ
merge marker ที่ยังไม่ได้ resolve การตรวจเหล่านี้เสริม ShellCheck แต่ไม่ได้แทนการทดสอบขณะรันจริง

## ทดสอบพฤติกรรม {#test-behavior}

รันการทดสอบที่เล็กที่สุดที่ครอบคลุม branch และเส้นทางความล้มเหลวที่เปลี่ยนแปลง เพิ่ม shell test คู่กัน
สำหรับ logic ที่อาจ regress ได้โดยไม่ขึ้นกับ workflow ซึ่งรวมถึงการ parse, การตัดสินใจหลาย branch,
การ retry และ cleanup, การตรวจ policy และผลข้างเคียงภายนอกที่สำคัญ

suite ระดับ domain อาจครอบคลุม script ที่ทำงานร่วมกันได้ wrapper บาง ๆ ไม่จำเป็นต้องมี test แบบหนึ่งต่อหนึ่ง
เมื่อ suite นั้นเรียกใช้และพิสูจน์พฤติกรรมของมันแล้ว

test คู่กันแต่ละตัว:

- สร้าง state ที่แยกออกมาภายใต้ `mktemp -d` และลบทิ้งเมื่อ exit
- ใส่คำสั่งภายนอกปลอมผ่าน `PATH` ชั่วคราว แทนการแก้โค้ด production
- ใช้ input ที่แตกต่างกัน และ assert output, exit status และผลข้างเคียงแบบตรงตัว
- ครอบคลุมกรณีสำเร็จ, input ไม่ถูกต้อง, dependency ล้มเหลว และ cleanup เมื่อมีเส้นทางเหล่านั้น
- ล้มเหลวเมื่อไม่มีคำสั่งที่ test ต้องใช้ การ skip แล้วผ่านไม่ได้เป็นการ validate พฤติกรรม
- รันจาก `make test-scripts` ซึ่งเป็นรายการ script test ที่ CI ใช้

เมื่อ script มี caller บนหลายระบบปฏิบัติการ ให้ทดสอบการเปลี่ยนแปลงที่อ่อนไหวต่อ platform บน CI matrix
ที่เกี่ยวข้องของ caller แต่ละตัว การทดสอบบน Linux บวกกับผล ShellCheck ที่สะอาดไม่ได้พิสูจน์พฤติกรรม
บน macOS หรือ Windows

## Checklist สำหรับ review {#review-checklist}

- พฤติกรรมนั้นยังไม่มีให้ใช้จาก script, Make target, tool ที่ติดตั้งไว้ หรือฟีเจอร์ native ง่าย ๆ ของ workflow
- GitHub Actions, Make และ script รับผิดชอบส่วนที่ถูกต้องของ workflow
- นามสกุลไฟล์, shebang, สิทธิ์ execute และเป้าหมาย portability ที่ระบุไว้สอดคล้องกัน
- script รันได้โดยไม่ขึ้นกับ working directory ของ caller
- argument, ความล้มเหลว, ไฟล์ชั่วคราว, การ retry, output และ secret มีการจัดการที่ชัดเจน
- test พฤติกรรมแบบเจาะจง, `shfmt` และ ShellCheck ผ่านสำหรับไฟล์สุดท้ายที่เปลี่ยนแปลง
- ทุกจุดที่เรียกใช้, path filter ของ workflow และเอกสารใช้ชื่อไฟล์สุดท้าย
