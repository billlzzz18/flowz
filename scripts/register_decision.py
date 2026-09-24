#!/usr/bin/env python3
"""Synchronize ADR Markdown files into a deterministic CSV index.

Markdown files under docs/decisions/ are the source of truth.  The generated CSV
is an index for humans and tools; it must never be edited manually.
"""
from __future__ import annotations

import argparse
import csv
import io
import os
import re
import sys
import tempfile
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path

CSV_FIELDS = ("id", "status", "date", "topic", "decision", "rationale", "source_path")
TITLE_RE = re.compile(r"^#\s*ADR-(?P<id>\d{4})[:\s]+(?P<title>.+?)\s*$", re.IGNORECASE)
STATUS_RE = re.compile(r"^\*{0,2}Status:\*{0,2}\s*(?P<value>.+?)\s*$", re.IGNORECASE)
DATE_RE = re.compile(r"^\*{0,2}Date:\*{0,2}\s*(?P<value>\d{4}-\d{2}-\d{2})", re.IGNORECASE)
HEADING_RE = re.compile(r"^##\s+(?P<name>.+?)\s*$")


class DecisionError(ValueError):
    """Raised when an ADR cannot be indexed safely."""


@dataclass(frozen=True)
class Decision:
    id: str
    status: str
    date: str
    topic: str
    decision: str
    rationale: str
    source_path: str

    def row(self) -> dict[str, str]:
        return asdict(self)


def _clean(text: str) -> str:
    text = re.sub(r"[`*_]+", "", text)
    text = re.sub(r"\s+", " ", text)
    return text.strip(" -:\n\t")


def _truncate(text: str) -> str:
    cleaned = _clean(text)
    return cleaned[:150] + "..." if len(cleaned) > 150 else cleaned


def parse_decision(path: Path) -> Decision:
    adr_id: str | None = None
    topic: str | None = None
    raw_status: str | None = None
    raw_date: str | None = None
    current_section: str | None = None
    sections: dict[str, list[str]] = {
        "decision": [],
        "context": [],
        "consequences": [],
    }

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line:
            continue

        heading_match = HEADING_RE.match(line)
        if heading_match:
            current_section = heading_match.group("name").strip().lower()
            continue

        if current_section is None:
            if adr_id is None:
                title_match = TITLE_RE.match(line)
                if title_match:
                    adr_id = f"ADR-{title_match.group('id')}"
                    topic = _clean(title_match.group("title"))
                    continue
            if raw_status is None:
                status_match = STATUS_RE.match(line)
                if status_match:
                    raw_status = status_match.group("value")
                    continue
            if raw_date is None:
                date_match = DATE_RE.match(line)
                if date_match:
                    raw_date = date_match.group("value")
                    continue
        else:
            if current_section in sections:
                sections[current_section].append(line)

    if not adr_id or not topic:
        raise DecisionError(f"{path}: missing '# ADR-NNNN: title' heading")

    decision = _truncate(" ".join(sections["decision"]))
    if not decision:
        raise DecisionError(f"{path}: missing non-empty '## Decision' section")

    context = _truncate(" ".join(sections["context"]))
    consequences = _truncate(" ".join(sections["consequences"]))
    rationale = context or consequences or decision

    return Decision(
        id=adr_id,
        status=_clean(raw_status) if raw_status else "Accepted",
        date=raw_date if raw_date else "2026-09-23",
        topic=topic,
        decision=decision,
        rationale=rationale,
        source_path=path.as_posix(),
    )


def collect_decisions(directory: Path) -> list[Decision]:
    if not directory.is_dir():
        raise DecisionError(f"decision directory does not exist: {directory}")
    decisions = [parse_decision(path) for path in sorted(directory.glob("*.md"))]
    if not decisions:
        raise DecisionError(f"no ADR Markdown files found in {directory}")
    duplicates = sorted(item for item, count in Counter(decision.id for decision in decisions).items() if count > 1)
    if duplicates:
        raise DecisionError(f"duplicate ADR id(s): {', '.join(duplicates)}")
    return sorted(decisions, key=lambda decision: decision.id)


def render_csv(decisions: list[Decision]) -> str:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=CSV_FIELDS, lineterminator="\n")
    writer.writeheader()
    writer.writerows(decision.row() for decision in decisions)
    return output.getvalue()


def atomic_write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent, text=True)
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="") as stream:
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    except Exception:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass
        raise


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Generate or verify docs/decisions.csv from ADR Markdown files.")
    parser.add_argument("command", choices=("sync", "check"), nargs="?", default="sync")
    parser.add_argument("--decisions-dir", type=Path, default=Path("docs/decisions"))
    parser.add_argument("--output", type=Path, default=Path("docs/decisions.csv"))
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        decisions = collect_decisions(args.decisions_dir)
        generated = render_csv(decisions)
        if args.command == "check":
            current = args.output.read_text(encoding="utf-8") if args.output.exists() else None
            if current != generated:
                print(f"out of date: {args.output}", file=sys.stderr)
                return 1
            print(f"OK: {args.output} matches {args.decisions_dir}/*.md")
            return 0
        atomic_write(args.output, generated)
        print(f"Successfully synced {len(decisions)} ADR records to {args.output} (atomic write).")
        return 0
    except (DecisionError, OSError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
