#!/usr/bin/env python3
"""
SecureForge Python Pipeline Unit Tests
Tests classify.py, timestamp.py, and report_gen.py
"""

import unittest
import tempfile
import os
import sys
import json
from pathlib import Path

# Add pipeline directory to sys.path
PIPELINE_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(PIPELINE_DIR))

import classify
import timestamp
import report_gen


class TestClassify(unittest.TestCase):
    def test_detect_mime_magic_bytes(self):
        cases = [
            (b"\xff\xd8\xff\xe0\x00\x10JFIF\x00" + b"\x00" * 32, "image/jpeg"),
            (b"\x89PNG\r\n\x1a\n" + b"\x00" * 32, "image/png"),
            (b"%PDF-1.7\n" + b"\x00" * 32, "application/pdf"),
            (b"PK\x03\x04" + b"\x00" * 32, "application/zip"),
            (b"SQLite format 3\x00" + b"\x00" * 32, "application/x-sqlite3"),
            (b"GIF89a" + b"\x00" * 32, "image/gif"),
            (b"\x7fELF" + b"\x00" * 32, "application/x-executable"),
            (b"7z\xbc\xaf\x27\x1c" + b"\x00" * 32, "application/x-7z-compressed"),
        ]
        for data, expected_mime in cases:
            with tempfile.NamedTemporaryFile(delete=False) as tf:
                tf.write(data)
                tf.flush()
                tf_path = tf.name
            try:
                mime = classify.detect_mime(tf_path)
                self.assertEqual(mime, expected_mime, f"Failed for {expected_mime}")
            finally:
                os.remove(tf_path)

    def test_detect_mime_fallback_extension(self):
        with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as tf:
            tf.write(b"plain ascii text")
            tf.flush()
            tf_path = tf.name
        try:
            mime = classify.detect_mime(tf_path)
            self.assertEqual(mime, "text/plain")
        finally:
            os.remove(tf_path)

    def test_get_category(self):
        self.assertEqual(classify.get_category("image/jpeg"), "Media")
        self.assertEqual(classify.get_category("video/mp4"), "Media")
        self.assertEqual(classify.get_category("audio/wav"), "Media")
        self.assertEqual(classify.get_category("application/pdf"), "Documents")
        self.assertEqual(classify.get_category("text/plain"), "Documents")
        self.assertEqual(classify.get_category("application/zip"), "Archives")
        self.assertEqual(classify.get_category("application/x-7z-compressed"), "Archives")
        self.assertEqual(classify.get_category("application/x-sqlite3"), "Databases")
        self.assertEqual(classify.get_category("application/x-executable"), "Executables")
        self.assertEqual(classify.get_category("unknown/blob"), "Unknown")

    def test_compute_sha256(self):
        with tempfile.NamedTemporaryFile(delete=False) as tf:
            tf.write(b"hello world\n")
            tf.flush()
            tf_path = tf.name
        try:
            # SHA-256("hello world\n") = a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447
            digest = classify.compute_sha256(tf_path)
            self.assertEqual(digest, "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447")
        finally:
            os.remove(tf_path)

    def test_hamming_distance(self):
        self.assertEqual(classify.hamming_distance("ffffffffffffffff", "ffffffffffffffff"), 0)
        self.assertEqual(classify.hamming_distance("0000000000000000", "0000000000000001"), 1)
        self.assertEqual(classify.hamming_distance("0000000000000000", "ffffffffffffffff"), 64)
        # Invalid inputs return 64
        self.assertEqual(classify.hamming_distance("", "1234"), 64)
        self.assertEqual(classify.hamming_distance(None, "1234"), 64)

    def test_classify_file(self):
        with tempfile.NamedTemporaryFile(suffix=".pdf", delete=False) as tf:
            tf.write(b"%PDF-1.4\nTest PDF Evidence Content")
            tf.flush()
            tf_path = tf.name
        try:
            rec = classify.classify_file(tf_path)
            self.assertEqual(rec["mime"], "application/pdf")
            self.assertEqual(rec["category"], "Documents")
            self.assertIsNotNone(rec["sha256"])
            self.assertIn("classified_at", rec)
            self.assertEqual(rec["filename"], os.path.basename(tf_path))
        finally:
            os.remove(tf_path)

    def test_classify_directory_scan(self):
        with tempfile.TemporaryDirectory() as td:
            f1 = os.path.join(td, "doc.pdf")
            f2 = os.path.join(td, "img.jpg")
            with open(f1, "wb") as f:
                f.write(b"%PDF-1.4\nsample pdf")
            with open(f2, "wb") as f:
                f.write(b"\xff\xd8\xff\xe0\x00\x10JFIF\x00" + b"\x00" * 20)
            
            results = []
            for root, _, files in os.walk(td):
                for file in files:
                    results.append(classify.classify_file(os.path.join(root, file)))
            
            self.assertEqual(len(results), 2)
            mimes = {r["mime"] for r in results}
            self.assertIn("application/pdf", mimes)
            self.assertIn("image/jpeg", mimes)


class TestTimestamp(unittest.TestCase):
    def test_create_timestamp_request_invalid_hash(self):
        with self.assertRaises(ValueError):
            timestamp.create_timestamp_request("invalid_hex", "http://example.com")
        with self.assertRaises(ValueError):
            timestamp.create_timestamp_request("a" * 63, "http://example.com")  # short
        with self.assertRaises(ValueError):
            timestamp.create_timestamp_request("z" * 64, "http://example.com")  # non-hex

    def test_verify_token_invalid_hash(self):
        with tempfile.NamedTemporaryFile(suffix=".tsr") as tf:
            tf.write(b"dummy_token_data" * 4)
            tf.flush()
            res = timestamp.verify_token(tf.name, "not_valid_hex")
            self.assertFalse(res)

    def test_verify_token_nonexistent_file(self):
        res = timestamp.verify_token("/tmp/nonexistent_file_secureforge.tsr", "a" * 64)
        self.assertFalse(res)

    def test_verify_token_empty_file(self):
        with tempfile.NamedTemporaryFile(suffix=".tsr") as tf:
            tf.flush()
            res = timestamp.verify_token(tf.name, "a" * 64)
            self.assertFalse(res)


class TestReportGen(unittest.TestCase):
    def test_sanitize_context_allowlist(self):
        dirty = {
            "case_id": "case-001",
            "method": "dod3",
            "__proto__": "polluted",
            "malicious_eval": "os.system('id')",
            "device": {"path": "/dev/sdb"},
        }
        clean = report_gen.sanitize_context(dirty)
        self.assertIn("case_id", clean)
        self.assertIn("method", clean)
        self.assertIn("device", clean)
        self.assertNotIn("__proto__", clean)
        self.assertNotIn("malicious_eval", clean)

    def test_latex_escape(self):
        raw = "Analysis & Testing #1: 50% $100 _root_ {x} ~test ^caret"
        escaped = report_gen.latex_escape(raw)
        self.assertIn(r"\&", escaped)
        self.assertIn(r"\#", escaped)
        self.assertIn(r"\%", escaped)
        self.assertIn(r"\$", escaped)
        self.assertIn(r"\_", escaped)
        self.assertIn(r"\{", escaped)
        self.assertIn(r"\}", escaped)

    def test_chunk_hash(self):
        h = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        chunked = report_gen.chunk_hash(h)
        self.assertEqual(len(chunked.split()), 8)
        self.assertEqual(chunked.split()[0], "e3b0c442")

    def test_render_html_templates(self):
        template_dir = str(PIPELINE_DIR / "templates")
        data = {
            "case_id": "test-case-123",
            "date": "2026-09-16",
            "investigator": "Lead Auditor",
            "target": "/dev/sdb",
            "method": "dod3",
            "status": "completed",
            "verified": True,
            "passes_completed": 3,
            "chain_verified": True,
            "chain_tip": "a" * 64,
            "device": {"path": "/dev/sdb", "model": "Test Disk", "serial": "SF-123"},
            "entries": [],
            "files": [],
        }

        for template in ["erasure", "recovery", "custody"]:
            html_out = report_gen.render_html(data, template, template_dir)
            self.assertTrue(len(html_out) > 500)
            self.assertIn("test-case-123", html_out)

    def test_compute_report_hash(self):
        with tempfile.NamedTemporaryFile() as tf:
            tf.write(b"SecureForge Forensic Report Content")
            tf.flush()
            digest = report_gen.compute_report_hash(tf.name)
            self.assertIsNotNone(digest)
            self.assertEqual(len(digest), 64)
            self.assertTrue(all(c in "0123456789abcdef" for c in digest))


if __name__ == "__main__":
    unittest.main()
