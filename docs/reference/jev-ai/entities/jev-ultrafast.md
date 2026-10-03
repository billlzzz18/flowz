---
title: Jev Ultrafast และ browser action selection
created: 2026-09-30
updated: 2026-09-30
type: entity
tags: [jev, browser-agent, action-selection, implementation, benchmark, limitations]
sources: [raw/repos/jev-ultrafast-readme.md, raw/repos/jev-ultrafast-performance.md, raw/repos/jev-ultrafast-model.py, raw/repos/jev-ultrafast-agent.py, raw/repos/jev-ultrafast-snapshot.js, raw/repos/jev-ultrafast-browser.py]
confidence: medium
---

# Jev Ultrafast และ browser action selection

## สถาปัตยกรรม

`browser-use/jev-ultrafast` ไม่ให้โมเดลควบคุม browser แบบอิสระ แต่สร้าง snapshot ของ visible DOM เป็นตาราง element ที่มี index แล้วให้ Jev เลือก operation และ target ที่เป็นไปได้ เช่น `CLICK`, `TYPE_TEXT`, `SELECT`, `SCROLL`, `WAIT`, `DONE`, `BLOCKED` ใน request เดียว แนวทางนี้ทำให้ action space dynamic และจำกัดผลลัพธ์ให้เป็นตัวเลือกที่ executor รองรับ

ข้อความที่จะกรอก field ถูกแยกไปให้ small text model เมื่อ operation เป็น `TYPE_TEXT` เท่านั้น ส่วน Jev ทำหน้าที่เลือกการกระทำและ element ไม่ได้สร้าง selector, coordinate, shell command หรือ JavaScript โดยตรง

## Guardrail ที่สำคัญ

โค้ดอ่าน state แบบ atomic, รักษา node identity, ตรวจ freshness ก่อน execute, ตรวจว่ element ยัง visible และไม่ถูกบัง, บันทึก action ก่อนอ่าน state รอบถัดไป และหยุดเมื่อเกิด stale/ambiguous mutation หลักการนี้สำคัญกับ decision plugin เช่นกัน: decision result ต้องผูกกับ input fingerprint และห้ามนำ output เก่ามาใช้กับ context ใหม่

Policy ระบุว่า `DONE` ต้องมี visible evidence ว่าข้อกำหนดทั้งหมดเสร็จ และการรอไม่ใช่ default หากมี control ที่ทำให้ progress ได้ บทเรียนนี้แปลงเป็น decision boundary ได้ว่า “ผ่าน threshold” ต้องมี evidence ครบ ไม่ใช่ดูจากคะแนนเดียว

## หลักฐาน performance

รายงานของ repo ระบุการทดสอบ Google Flights แบบควบคุมเล็ก ๆ 3 คู่ มี median runtime ลดจาก 9.450 เป็น 7.092 วินาที และ browser protocol calls ลดจาก 1,092 เป็น 101 โดยทั้งสองแบบผ่าน 3/3 แต่ผู้เขียนระบุชัดว่า sample เล็ก ไม่ใช่ benchmark ทั่วไป ตัวเลขนี้จึงเป็น engineering evidence ของ loop นี้ ไม่ใช่หลักฐานว่า Jev เร็วเท่ากันทุกงาน

## สิ่งที่นำกลับมาใช้กับ plugin

ใช้ **bounded action space**, **freshness/idempotency guard**, **independent verification**, **structured output validation**, และ **stop condition ที่ชัดเจน** ใน decision-maker/auditor pipeline ดู [[concepts/jev-for-decision-plugin]] และ [[queries/jev-browser-agent-lessons]]
