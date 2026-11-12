"""Self-checks for lunalas classification and daily review. Run:
python3 -m unittest discover -s ai/skills/lunalas/tests
"""

import importlib.util
import json
import os
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))


def load(name):
    spec = importlib.util.spec_from_file_location(name, os.path.join(HERE, "..", "hooks", name + ".py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


classify_mod = load("classify_prompt")
review_mod = load("daily_review")


class ClassifyTests(unittest.TestCase):
    def test_lookup_is_native(self):
        self.assertEqual(classify_mod.classify("find all tsx files in src")[0], "native")

    def test_typo_is_haiku(self):
        self.assertEqual(classify_mod.classify("fix typo in the readme")[0], "haiku")

    def test_feature_is_sonnet(self):
        self.assertEqual(classify_mod.classify("implement the user profile page")[0], "sonnet")

    def test_root_cause_is_opus(self):
        self.assertEqual(classify_mod.classify("debug the race condition in the websocket handler")[0], "opus")

    def test_vietnamese_debug_is_opus(self):
        self.assertEqual(classify_mod.classify("gỡ lỗi nguyên nhân gốc của lỗi này")[0], "opus")

    def test_user_override_wins(self):
        self.assertEqual(classify_mod.classify("debug this, dùng haiku")[0], "haiku")

    def test_no_signal_defaults_to_sonnet(self):
        self.assertEqual(classify_mod.classify("hello there")[0], "sonnet")

    def test_eleven_files_is_sonnet(self):
        files = " ".join(f"src/m{i}.ts" for i in range(11))
        self.assertEqual(classify_mod.classify(f"update {files}")[0], "sonnet")


class ReviewTests(unittest.TestCase):
    def test_render_counts_tiers_for_day(self):
        rows = [
            {"ts": "2026-11-12T10:00:00+00:00", "tier": "haiku", "reasons": ["haiku:fix"]},
            {"ts": "2026-11-12T11:00:00+00:00", "tier": "opus", "reasons": ["opus:debug"]},
            {"ts": "2026-11-11T11:00:00+00:00", "tier": "native", "reasons": []},
        ]
        report = review_mod.render("2026-11-12", [r for r in rows if r["ts"].startswith("2026-11-12")])
        self.assertIn("Prompts routed: 2", report)
        self.assertIn("| haiku | 1 | 50% |", report)

    def test_empty_day(self):
        self.assertIn("No routed prompts", review_mod.render("2000-01-01", []))

    def test_load_skips_bad_lines(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = os.path.join(tmp, "data", "shared", "token-router")
            os.makedirs(folder)
            with open(os.path.join(folder, "log.ndjson"), "w", encoding="utf-8") as f:
                f.write("not json\n")
                f.write(json.dumps({"ts": "2026-11-12T00:00:00+00:00", "tier": "sonnet", "reasons": []}) + "\n")
            old = review_mod.LOG
            review_mod.LOG = os.path.join(folder, "log.ndjson")
            try:
                self.assertEqual(len(review_mod.load("2026-11-12")), 1)
            finally:
                review_mod.LOG = old


if __name__ == "__main__":
    unittest.main()
