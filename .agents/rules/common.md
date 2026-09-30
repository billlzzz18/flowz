# แนวทางโค้ดและการทดสอบ

ใช้เอกสารนี้เมื่อเพิ่มหรือแก้ implementation และ test

## คำสั่งมาตรฐาน

```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets -- -D warnings
cargo check --all --all-targets
cargo test --all --verbose
python3 -m unittest discover -s tests -p 'test_*.py' -v
python3 scripts/register_decision.py check
```

## กฎการทดสอบ

- เขียน regression test สำหรับ bug หรือ invariant ใหม่ก่อนแก้ behavior

- ทดสอบทั้ง success, input ผิดรูป, ไฟล์หาย, ข้อมูลซ้ำ และการรันซ้ำเมื่อเกี่ยวข้อง

- ตัว parser ต้องทดสอบ Unicode, comma/newline ในค่า และ duplicate ID

- ห้ามรายงานว่า tested โดยไม่มีคำสั่งและผลลัพธ์จากรอบปัจจุบัน

- หากคำสั่งมาตรฐานเดิมล้มเหลวจาก baseline ให้แยกผล baseline กับผลจากงานที่แก้ และอย่าซ่อนข้อผิดพลาด