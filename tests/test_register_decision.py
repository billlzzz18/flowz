import unittest
import csv
import subprocess
import sys
import tempfile
from pathlib import Path

class TestRegisterDecision(unittest.TestCase):
    def setUp(self):
        self.csv_path = Path("docs/decisions.csv")
        self.script_path = Path("scripts/register_decision.py")

    def test_script_execution(self):
        with tempfile.TemporaryDirectory() as td:
            output = Path(td) / "decisions.csv"
            res = subprocess.run(
                [sys.executable, str(self.script_path), "--output", str(output)],
                capture_output=True,
                text=True,
            )
            self.assertEqual(res.returncode, 0, f"Script failed: {res.stderr}")
            self.assertIn("Successfully synced", res.stdout)
            self.assertTrue(output.exists())

    def test_adr_count_and_columns(self):
        self.assertTrue(self.csv_path.exists())
        with open(self.csv_path, "r", encoding="utf-8") as f:
            reader = csv.DictReader(f)
            rows = list(reader)

        expected_columns = ["id", "status", "date", "topic", "decision", "rationale", "source_path"]
        self.assertEqual(reader.fieldnames, expected_columns)
        actual_files_count = len(list(Path("docs/decisions").glob("*.md")))
        self.assertEqual(len(rows), actual_files_count, f"Expected {actual_files_count} ADRs, got {len(rows)}")

    def test_unique_sequential_ids(self):
        with open(self.csv_path, "r", encoding="utf-8") as f:
            rows = list(csv.DictReader(f))
        
        ids = [r["id"] for r in rows]
        self.assertEqual(len(ids), len(set(ids)), "Duplicate ADR IDs found")

    def test_source_paths_exist(self):
        with open(self.csv_path, "r", encoding="utf-8") as f:
            rows = list(csv.DictReader(f))
        
        for r in rows:
            p = Path(r["source_path"])
            self.assertTrue(p.exists(), f"Source path {p} does not exist")

    def test_check_command(self):
        res = subprocess.run([sys.executable, str(self.script_path), "check"], capture_output=True, text=True)
        self.assertEqual(res.returncode, 0, f"Check failed: {res.stderr}")
        self.assertIn("OK:", res.stdout)

    def test_parse_decision_single_pass(self):
        sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
        from scripts.register_decision import parse_decision

        with tempfile.TemporaryDirectory() as td:
            sample = Path(td) / "0099-test-adr.md"
            sample.write_text(
                "# ADR-0099: Test Title\n\n"
                "Status: Proposed\n\n"
                "## Decision\n"
                "We decide " + ("something " * 25) + "\n\n"
                "## Context\n"
                "Some context\n",
                encoding="utf-8",
            )
            decision = parse_decision(sample)
            self.assertEqual(decision.id, "ADR-0099")
            self.assertEqual(decision.status, "Proposed")
            self.assertEqual(decision.date, "2026-09-23")
            self.assertEqual(decision.topic, "Test Title")
            self.assertTrue(decision.decision.endswith("..."))
            self.assertEqual(len(decision.decision), 153)
            self.assertEqual(decision.rationale, "Some context")

if __name__ == "__main__":
    unittest.main()
