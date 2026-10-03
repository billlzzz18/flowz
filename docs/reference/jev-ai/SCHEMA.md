# Wiki Schema

## Domain

ความรู้ภาษาไทยเกี่ยวกับ JEV AI / Jev / System One Models ตั้งแต่ primitive, API, local alternative, browser action selection ไปจนถึงการนำไปใช้เป็นชั้นตัดสินใจใน decision-audit plugin

## Conventions

- ชื่อไฟล์ภาษาอังกฤษตัวพิมพ์เล็ก ใช้ขีดกลาง
- ทุกหน้าใน `entities/`, `concepts/`, `comparisons/`, `queries/` มี YAML frontmatter
- ทุกหน้าต้องมี `[[wikilinks]]` อย่างน้อย 2 ลิงก์
- แยกข้อเท็จจริง, การอนุมาน, คำแนะนำ และข้อจำกัดอย่างชัดเจน
- ตัวเลข benchmark ต้องระบุผู้รายงาน ขอบเขตการทดสอบ และไม่สรุปเกิน sample
- แหล่งดิบใน `raw/` เป็น immutable และมี `source_url`, `ingested`, `sha256`
- ข้อกล่าวอ้างจากผู้พัฒนาใช้ `confidence: medium` จนกว่าจะมีการทดสอบซ้ำอิสระ
- ทุกการเปลี่ยนแปลงต้องบันทึกใน `log.md` และเพิ่มหน้าใหม่ใน `index.md`

## Tag Taxonomy

`jev`, `system-one`, `typesafe`, `decision-model`, `probability`, `calibration`, `api`, `local-inference`, `browser-agent`, `action-selection`, `security`, `decision-plugin`, `audit`, `evaluation`, `benchmark`, `limitations`, `implementation`

## Page Types

- `entity`: โครงการ ผู้ให้บริการ โมเดล หรือ repository
- `concept`: แนวคิด primitive pattern หรือ integration technique
- `comparison`: เปรียบเทียบทางเลือก
- `query`: คำตอบเชิงปฏิบัติที่ควรเก็บไว้
- `summary`: ภาพรวมสังเคราะห์

## Quality Rules

ห้ามใช้ Jev เป็นผู้ตัดสินใจอิสระในเรื่องที่ต้อง reasoning เปิดกว้างหรือมีผลกระทบสูงโดยไม่มี policy, threshold, audit และ escalation ชัดเจน การมี probability ไม่ได้แปลว่า calibrated ในทุกโดเมน ต้องทำ shadow evaluation และวัด false positive/false negative ก่อนเปิดใช้จริง
