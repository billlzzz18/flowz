import unittest
import csv
import subprocess
from pathlib import Path

class TestRegisterDecision(unittest.TestCase):
    def setUp(self):
        self.csv_path = Path("docs/decisions.csv")
        self.script_path = Path("scripts/register_decision.py")

    def test_script_execution(self):
        res = subprocess.run(["python", str(self.script_path)], capture_output=True, text=True)
        self.assertEqual(res.returncode, 0, f"Script failed: {res.stderr}")
        self.assertIn("Successfully synced", res.stdout)

    def test_adr_count_and_columns(self):
        self.assertTrue(self.csv_path.exists())
        with open(self.csv_path, "r", encoding="utf-8") as f:
            reader = csv.DictReader(f)
            rows = list(reader)

        expected_columns = ["id", "status", "date", "topic", "decision", "rationale", "source_path"]
        self.assertEqual(reader.fieldnames, expected_columns)
        self.assertEqual(len(rows), 37, f"Expected 37 ADRs, got {len(rows)}")

    def test_unique_sequential_ids(self):
        with open(self.csv_path, "r", encoding="utf-8") as f:
            rows = list(csv.DictReader(f))
        
        ids = [r["id"] for r in rows]
        self.assertEqual(len(ids), len(set(ids)), "Duplicate ADR IDs found")
        for i in range(1, 38):
            expected_id = f"ADR-{i:04d}"
            self.assertIn(expected_id, ids, f"Missing {expected_id}")

    def test_source_paths_exist(self):
        with open(self.csv_path, "r", encoding="utf-8") as f:
            rows = list(csv.DictReader(f))
        
        for r in rows:
            p = Path(r["source_path"])
            self.assertTrue(p.exists(), f"Source path {p} does not exist")

if __name__ == "__main__":
    unittest.main()
