# ADR-0030: Learning Cron (HarnessBank Pattern)

Status: Accepted

## Context
HarnessBank (arXiv 2607.13683) พิสูจน์ว่า task agent และ evolver agent ต้องแยกกัน และต้องมี statistical rigor ในการ admit patch

## Decision
Cron 1: flowz-learning-cron — รันทุกวัน 03:00 (Asia/Bangkok), misfire: RunOnce
7 ขั้นตอน: Collect (24h) -> Diagnose -> Generate -> Screen (4 gates) -> Admit (Gene Bank) -> Version -> Report
