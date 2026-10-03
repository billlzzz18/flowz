# แผนการพัฒนา Decision Service (System One) และระบบติดตั้งโมเดล

> **เป้าหมายสำหรับ Agent:** ใช้ superpowers:subagent-driven-development หรือ superpowers:executing-plans ในการลงมือทำทีละงาน ขั้นตอนใช้รูปแบบ checkbox (`- [ ]`)

**เป้าหมาย:** สร้างและทดสอบระบบการตัดสินใจ System One ที่รองรับการสลับตัวสำรองหลายระดับ (Ollama tev1 -> Local CPU laya -> Cloud Jev) พร้อมสคริปต์เลือกติดตั้งโมเดลลงเครื่อง

**สถาปัตยกรรม:** `crates/decider` ทำหน้าที่เป็นท่อรับส่ง HTTP แบบ latency ต่ำไปยัง endpoint `/v1/systemone` ส่วน `src/service/decision.rs` ทำหน้าที่ควบคุมการสลับระหว่างโมเดลหลักและโมเดลสำรองเมื่อเกิดปัญหา และมีสคริปต์ PowerShell ช่วยดาวน์โหลดโมเดล

**เทคโนโลยีที่ใช้:** Rust (tokio, reqwest, serde, pmcp), PowerShell, Ollama, GGUF/llama.cpp (jevos/laya)

**เอกสารอ้างอิง:** `docs/reference/jev-ai/concepts/jev-for-decision-plugin.md` และ `docs/research/decision-example.html`

## กฎและข้อจำกัดของระบบ
- จำกัดขนาดข้อมูลตอบกลับสูงสุดไม่เกิน 10 MiB (`MAX_RESPONSE_BYTES = 10 * 1024 * 1024`)
- URL ปลายทางต้องไม่มีเครื่องหมาย `?` หรือ `#` และต้องขึ้นต้นด้วย `http://` หรือ `https://`
- ต้องผ่านการตรวจ Clippy 0 warnings: `cargo clippy -p flowz-mcp -- -D warnings`

## จุดสำคัญที่ต้องตรวจสอบ
1. การสลับไปใช้โมเดลสำรองเมื่อโมเดลหลักเกิด timeout หรือติดต่อไม่ได้
2. การปฏิเสธคำขอที่ไม่มีข้อมูล state หรือข้อมูลผิดรูปแบบ
3. การจัดการข้อผิดพลาดเมื่อโมเดลทุกตัวในระบบไม่ตอบสนอง
4. การดักจับข้อมูลตอบกลับที่มีขนาดเกิน 10 MiB
5. การทำงานของสคริปต์ติดตั้งเมื่อเครื่องไม่มีโปรแกรม Ollama

---

### งานที่ 1: ตรวจสอบและทดสอบระบบสลับตัวสำรอง (Fallback) ใน DecisionService

**ไฟล์ที่เกี่ยวข้อง:**
- แก้ไข/เพิ่มเติม: `src/service/decision.rs`
- การทดสอบ: `src/service/decision.rs` (โมดูล unit test ในไฟล์)

**การเชื่อมต่อ:**
- เรียกใช้: `decider::DecisionBackend`, `decider::DecisionQuery`, `decider::DecisionResponse`, `decider::DeciderError`
- ส่งออก: `DecisionService::new()`, `DecisionService::with_backends()`, `DecisionService::decide()`

- [ ] **ขั้นตอนที่ 1: เขียน unit test ตรวจสอบการสลับจากตัวหลักไปตัวสำรอง**
```rust
#[tokio::test]
async fn test_decision_service_falls_back_on_transport_error() {
    // จำลองกรณีตัวหลักเกิด transport error แล้วสลับไปตัวสำรองได้ถูกต้อง
}
```

- [ ] **ขั้นตอนที่ 2: รันการทดสอบเพื่อยืนยันว่าทำงานถูกต้อง**
คำสั่ง: `cargo test -p flowz-mcp --lib service::decision`

- [ ] **ขั้นตอนที่ 3: ตรวจสอบความสะอาดของโค้ดด้วย Clippy**
คำสั่ง: `cargo clippy -p flowz-mcp -- -D warnings`

- [ ] **ขั้นตอนที่ 4: บันทึกงาน (Commit)**
```bash
git add src/service/decision.rs
git commit -m "feat(decision): add multi-tier fallback support to DecisionService"
```

---

### งานที่ 2: ตรวจสอบสคริปต์ติดตั้งโมเดล

**ไฟล์ที่เกี่ยวข้อง:**
- ตรวจสอบ: `scripts/setup-decider.ps1`

**การเชื่อมต่อ:**
- รับค่าพารามิเตอร์: `-Model <tev1|laya|all>`

- [ ] **ขั้นตอนที่ 1: ทดสอบการรันสคริปต์ด้วยตัวเลือก laya**
คำสั่ง: `pwsh -File scripts/setup-decider.ps1 -Model laya`

- [ ] **ขั้นตอนที่ 2: ตรวจสอบว่ามีโฟลเดอร์และไฟล์โมเดลถูกสร้างขึ้นจริง**
คำสั่ง: ตรวจสอบไฟล์ในโฟลเดอร์ `models/`

- [ ] **ขั้นตอนที่ 3: บันทึกงาน (Commit)**
```bash
git add scripts/setup-decider.ps1
git commit -m "feat(scripts): add selectable model setup script for decider"
```
