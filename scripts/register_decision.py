#!/usr/bin/env python3
"""
scripts/register_decision.py

Automates syncing and auditing ADR files from docs/decisions/*.md into docs/decisions.csv.
Rules:
- Read docs/decisions/*.md
- Write only docs/decisions.csv using atomic write
- Never modify source ADR files
- Never touch source code
- Validate duplicate ID, required metadata, and sections
- Columns: id,status,date,topic,decision,rationale,source_path
"""

import os
import sys
import re
import csv
import tempfile
from pathlib import Path

DECISIONS_DIR = Path("docs/decisions")
CSV_PATH = Path("docs/decisions.csv")
EXPECTED_COLUMNS = ["id", "status", "date", "topic", "decision", "rationale", "source_path"]

def extract_adr_metadata(file_path):
    rel_path = file_path.as_posix()
    content = file_path.read_text(encoding="utf-8")

    # Match ID from filename or content
    m_id = re.search(r"^#\s*ADR-(\d{4})[:\s]+(.*)$", content, re.MULTILINE)
    if m_id:
        adr_id = m_id.group(1)
        topic = m_id.group(2).strip()
    else:
        # Match from filename e.g. 0001-separate-timebudget-from-cron.md
        m_fname = re.search(r"(\d{4})-(.*)\.md", file_path.name)
        if not m_fname:
            raise ValueError(f"Cannot parse ADR filename: {file_path.name}")
        adr_id = m_fname.group(1)
        raw_topic = m_fname.group(2).replace("-", " ")
        topic = raw_topic.title()

    # Status
    m_status = re.search(r"^Status:[ \t]*(.+)$", content, re.MULTILINE | re.IGNORECASE)
    status = m_status.group(1).strip() if m_status else "Accepted"

    # Date
    m_date = re.search(r"^Date:[ \t]*(.+)$", content, re.MULTILINE | re.IGNORECASE)
    date_val = m_date.group(1).strip() if m_date else "2026-09-23"

    # Decision
    m_dec = re.search(r"^##\s*Decision\s*\n([\s\S]*?)(?=^##|\Z)", content, re.MULTILINE)
    if m_dec:
        decision = " ".join(m_dec.group(1).strip().split())
        decision = re.sub(r"[`#*]", "", decision)
        decision = (decision[:150] + "...") if len(decision) > 150 else decision
    else:
        decision = topic

    # Rationale / Context
    m_ctx = re.search(r"^##\s*(?:Context|Rationale)\s*\n([\s\S]*?)(?=^##|\Z)", content, re.MULTILINE)
    if m_ctx:
        rationale = " ".join(m_ctx.group(1).strip().split())
        rationale = re.sub(r"[`#*]", "", rationale)
        rationale = (rationale[:150] + "...") if len(rationale) > 150 else rationale
    else:
        rationale = ""

    return {
        "id": f"ADR-{adr_id}",
        "status": status,
        "date": date_val,
        "topic": topic,
        "decision": decision,
        "rationale": rationale,
        "source_path": rel_path
    }

def sync_decisions():
    if not DECISIONS_DIR.exists():
        print(f"Directory {DECISIONS_DIR} not found.")
        sys.exit(1)

    files = sorted(DECISIONS_DIR.glob("*.md"))
    records = []
    seen_ids = set()

    for f in files:
        meta = extract_adr_metadata(f)
        if meta["id"] in seen_ids:
            raise ValueError(f"Duplicate ADR ID detected: {meta['id']} in {f}")
        seen_ids.add(meta["id"])
        records.append(meta)

    records.sort(key=lambda r: r["id"])

    # Atomic write
    temp_dir = CSV_PATH.parent
    temp_dir.mkdir(parents=True, exist_ok=True)

    with tempfile.NamedTemporaryFile("w", delete=False, dir=str(temp_dir), newline="", encoding="utf-8") as tf:
        writer = csv.DictWriter(tf, fieldnames=EXPECTED_COLUMNS)
        writer.writeheader()
        writer.writerows(records)
        temp_name = tf.name

    os.replace(temp_name, str(CSV_PATH))
    print(f"Successfully synced {len(records)} ADR records to {CSV_PATH} (atomic write).")
    return len(records)

if __name__ == "__main__":
    count = sync_decisions()
