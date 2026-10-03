---
title: บทเรียนจาก Jev Ultrafast สำหรับ agent architecture
created: 2026-09-30
updated: 2026-09-30
type: query
tags: [browser-agent, action-selection, decision-plugin, security, implementation, limitations]
sources: [raw/repos/jev-ultrafast-readme.md, raw/repos/jev-ultrafast-agent.py, raw/repos/jev-ultrafast-model.py, raw/repos/jev-ultrafast-questions.py, raw/repos/jev-ultrafast-browser.py, raw/repos/jev-ultrafast-snapshot.js]
confidence: high
---

# บทเรียนจาก Jev Ultrafast สำหรับ agent architecture

## สิ่งที่ควรยืม

**จำกัด action space:** ให้ decision-maker เลือกจาก status/choice ที่ schema กำหนด ไม่ปล่อยข้อความอิสระเป็นคำสั่ง

**แยก decision กับ execution:** Jev เลือก operation/target แต่โค้ดเป็นผู้ resolve node, ตรวจ visibility, freshness และ execute เช่นเดียวกับ plugin ที่ decision subagent ไม่ควรใช้ tool

**ตรวจ input freshness:** ผูก decision กับ `session_id`, `run_id`, policy version และ input hash; ถ้าบริบทเปลี่ยนให้ประเมินใหม่

**ตรวจ output อย่างเข้ม:** validate JSON, enum, numeric range, probability sum และ selected choice ก่อน route

**มี independent verification:** `DONE` หรือ `PROCEED` ไม่ใช่หลักฐานว่าผลลัพธ์ถูก ต้องมี auditor หรือ deterministic checker ในงานสำคัญ

**บันทึกก่อน mutation:** เก็บ decision record ก่อนให้ parent agent ทำ action เพื่อให้ failure หลังจากนั้นไม่ทำให้ audit หาย

## สิ่งที่ห้ามตีความเกิน

Jev Ultrafast เป็น demo/implementation สำหรับ browser loop และรายงาน benchmark ที่ sample เล็ก ไม่ได้พิสูจน์ว่า pattern เดียวจะทำให้ decision plugin ถูกต้องขึ้นทุกโดเมน ข้อจำกัดใน repo เช่น shadow DOM, frames, canvas, uploads, tabs และ keyboard widgets ยังสะท้อนว่าการทำ action space แบบ bounded ต้องประกาศ unsupported cases ชัดเจน

## แปลงเป็น acceptance criteria ของ plugin

ก่อนเปิดใช้จริง plugin ต้องตอบได้ว่า: decision นี้อ้าง input snapshot ใด, policy version ใด, option ทั้งหมดคืออะไร, threshold ไหนทำให้ route นี้เกิด, เมื่อ context เปลี่ยนทำอย่างไร, malformed/timeout fallback คืออะไร, และใครเป็นผู้ตรวจผล หากตอบไม่ได้ ระบบกำลังทำ “ขอไปที” ไม่ใช่ autonomous decision system ดู [[concepts/jev-for-decision-plugin]] และ [[concepts/jev-safety-and-evaluation]]
