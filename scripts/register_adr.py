#!/usr/bin/env python3
"""
Flowz ADR Auto-Registration Tool
Automatically registers new Architecture Decision Records (ADRs)
into docs/decisions.csv and creates the corresponding markdown file in docs/decisions/.
Usage:
    python scripts/register_adr.py --id 0038 --topic "My Topic" --decision "My Decision" --rationale "Why"
    python scripts/register_adr.py --interactive
    python scripts/register_adr.py --scan (scans docs/decisions/*.md and syncs CSV)
"""

import os
import sys
import csv
import re
import argparse
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DOCS_DIR = REPO_ROOT / "docs"
DECISIONS_DIR = DOCS_DIR / "decisions"
CSV_PATH = DOCS_DIR / "decisions.csv"

def load_csv():
    if not CSV_PATH.exists():
        return []
    with open(CSV_PATH, "r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        return list(reader)

def save_csv(rows):
    fieldnames = ["id", "status", "topic", "decision", "rationale"]
    with open(CSV_PATH, "w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for r in rows:
            writer.writerow(r)

def register_adr(adr_id, topic, decision, rationale, status="✅", content=None):
    adr_id_str = f"{int(adr_id):04d}" if str(adr_id).isdigit() else str(adr_id)
    rows = load_csv()
    
    # Check if id already exists
    existing = [r for r in rows if r["id"] == adr_id_str]
    if existing:
        print(f"[!] ADR {adr_id_str} already exists in CSV. Updating entry...")
        for r in rows:
            if r["id"] == adr_id_str:
                r["status"] = status
                r["topic"] = topic
                r["decision"] = decision
                r["rationale"] = rationale
    else:
        rows.append({
            "id": adr_id_str,
            "status": status,
            "topic": topic,
            "decision": decision,
            "rationale": rationale
        })
        # Sort rows by id
        rows.sort(key=lambda x: x["id"])

    save_csv(rows)
    print(f"[+] Updated {CSV_PATH} with ADR {adr_id_str}")

    # Ensure markdown file exists
    DECISIONS_DIR.mkdir(parents=True, exist_ok=True)
    slug = re.sub(r"[^a-zA-Z0-9]+", "-", topic.lower()).strip("-")
    md_filename = f"{adr_id_str}-{slug}.md"
    md_path = DECISIONS_DIR / md_filename

    # If md doesn't exist, create it
    if not list(DECISIONS_DIR.glob(f"{adr_id_str}-*.md")):
        if not content:
            content = f"""# ADR-{adr_id_str}: {topic}

Status: Accepted
Date: 2026-09-24

## Context
{rationale}

## Decision
{decision}

## Consequences
- Maintained in docs/decisions.csv automatically
"""
        with open(md_path, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"[+] Created markdown: {md_path}")
    else:
        print(f"[i] Markdown file for ADR {adr_id_str} already exists.")

def scan_and_sync():
    """Scans all docs/decisions/*.md and updates CSV if missing."""
    rows = load_csv()
    existing_ids = {r["id"] for r in rows}
    
    md_files = sorted(DECISIONS_DIR.glob("*.md"))
    added = 0
    for mf in md_files:
        m = re.match(r"^(d{4})-(.*).md$", mf.name)
        if not m:
            continue
        adr_id = m.group(1)
        if adr_id not in existing_ids:
            # Parse title from file
            text = mf.read_text(encoding="utf-8")
            topic_match = re.search(r"^#s*(?:ADR-d+:s*)?(.*)$", text, re.MULTILINE)
            topic = topic_match.group(1).strip() if topic_match else m.group(2).replace("-", " ")
            rows.append({
                "id": adr_id,
                "status": "✅",
                "topic": topic,
                "decision": f"Refer to {mf.name}",
                "rationale": "Extracted from markdown record"
            })
            added += 1
            existing_ids.add(adr_id)

    if added > 0:
        rows.sort(key=lambda x: x["id"])
        save_csv(rows)
        print(f"[+] Synced {added} missing ADR(s) to {CSV_PATH}")
    else:
        print("[i] All markdown ADRs are already synced in CSV.")

def main():
    parser = argparse.ArgumentParser(description="Flowz ADR Auto-Registration")
    parser.add_argument("--id", help="ADR ID (e.g. 0038)")
    parser.add_argument("--topic", help="Topic title")
    parser.add_argument("--decision", help="Decision summary")
    parser.add_argument("--rationale", help="Rationale summary")
    parser.add_argument("--status", default="✅", help="Status symbol (default: ✅)")
    parser.add_argument("--scan", action="store_true", help="Sync all existing markdown files to CSV")
    
    args = parser.parse_args()

    if args.scan:
        scan_and_sync()
        return

    if not args.id or not args.topic or not args.decision or not args.rationale:
        parser.print_help()
        sys.exit(1)

    register_adr(args.id, args.topic, args.decision, args.rationale, args.status)

if __name__ == "__main__":
    main()
