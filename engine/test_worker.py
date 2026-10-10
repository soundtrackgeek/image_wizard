import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import worker


class Response(io.BytesIO):
    def __init__(self, payload, status=200, headers=None):
        super().__init__(payload)
        self.status = status
        self.headers = headers or {}


class DownloadTests(unittest.TestCase):
    def test_resume_uses_exact_range_and_publishes_verified_bytes(self):
        payload = b"verified model bytes"
        with tempfile.TemporaryDirectory() as folder:
            target = Path(folder) / "weights.bin"
            target.with_name("weights.bin.partial").write_bytes(payload[:7])
            def opener(request, **_):
                self.assertEqual(request.headers["Range"], "bytes=7-")
                return Response(payload[7:], 206, {"Content-Range": f"bytes 7-{len(payload)-1}/{len(payload)}"})
            worker.download_file("https://example.invalid/immutable", target, len(payload), hashlib.sha256(payload).hexdigest(), Path(folder)/"cancel", lambda _: None, opener)
            self.assertEqual(target.read_bytes(), payload)
            self.assertFalse(target.with_name("weights.bin.partial").exists())

    def test_ignored_range_restarts_instead_of_appending(self):
        payload = b"complete replacement"
        with tempfile.TemporaryDirectory() as folder:
            target = Path(folder) / "weights"
            target.with_name("weights.partial").write_bytes(b"old")
            worker.download_file("https://example.invalid/immutable", target, len(payload), hashlib.sha256(payload).hexdigest(), Path(folder)/"cancel", lambda _: None, lambda *_args, **_kwargs: Response(payload))
            self.assertEqual(target.read_bytes(), payload)

    def test_wrong_hash_never_publishes_and_can_retry(self):
        with tempfile.TemporaryDirectory() as folder:
            target = Path(folder) / "weights"
            with self.assertRaisesRegex(ValueError, "Checksum"):
                worker.download_file("https://example.invalid/immutable", target, 3, "0"*64, Path(folder)/"cancel", lambda _: None, lambda *_args, **_kwargs: Response(b"bad"))
            self.assertFalse(target.exists())
            self.assertFalse(target.with_name("weights.partial").exists())

    def test_cancel_preserves_partial_without_publishing(self):
        with tempfile.TemporaryDirectory() as folder:
            target = Path(folder) / "weights"
            cancel = Path(folder) / "cancel"
            target.with_name("weights.partial").write_bytes(b"part")
            cancel.touch()
            with self.assertRaises(worker.Cancelled):
                worker.download_file("https://example.invalid/immutable", target, 8, None, cancel, lambda _: None)
            self.assertEqual(target.with_name("weights.partial").read_bytes(), b"part")
            self.assertFalse(target.exists())

    def test_rejects_mismatched_content_range(self):
        with tempfile.TemporaryDirectory() as folder:
            target = Path(folder)/"weights"
            target.with_name("weights.partial").write_bytes(b"part")
            with self.assertRaisesRegex(ValueError, "range"):
                worker.download_file("https://example.invalid/immutable", target, 8, None, Path(folder)/"cancel", lambda _: None, lambda *_args, **_kwargs: Response(b"rest",206,{"Content-Range":"bytes 0-3/8"}))
            self.assertFalse(target.exists())


class BoundaryTests(unittest.TestCase):
    def test_checkpoint_mapping_streams_and_preserves_remapped_values(self):
        class Pointer:
            def __init__(self): self.reads = []
            def keys(self): return ["first", "second", "third"]
            def get_tensor(self, key): self.reads.append(key); return key + "-tensor"
        pointer = Pointer()
        state = worker.CheckpointTensors(pointer)
        self.assertEqual(list(state), ["first", "second", "third"])
        self.assertEqual(pointer.reads, [])
        iterator = iter(state.items())
        self.assertEqual(next(iterator), ("first", "first-tensor"))
        self.assertEqual(pointer.reads, ["first"])
        del iterator
        state["replacement"] = state.pop("second")
        clone = state.copy()
        del state["third"]
        self.assertEqual(clone["replacement"], "second-tensor")
        self.assertEqual(pointer.reads, ["first", "second"])
        self.assertIn("third", clone)
        self.assertNotIn("third", state)
        self.assertEqual(pointer.reads, ["first", "second"])
        self.assertEqual(dict(clone), {"first":"first-tensor", "third":"third-tensor", "replacement":"second-tensor"})
        self.assertEqual(clone.get("missing", "default"), "default")

    def test_empty_memory_error_is_reported_with_recovery_guidance(self):
        message = worker.failure_message(MemoryError())
        self.assertIn("System memory", message)
        self.assertIn("paging file", message)
        self.assertTrue(worker.failure_message(RuntimeError()))
        try:
            try: raise MemoryError()
            except MemoryError as cause: raise OSError("Unable to load weights") from cause
        except OSError as wrapped:
            self.assertIn("System memory", worker.failure_message(wrapped))

    def test_rejects_escaping_model_paths(self):
        for name in ("../outside", "C:/outside", "nested\\outside"):
            with self.assertRaises(ValueError): worker.safe_relative(name)

    def test_changed_snapshot_content_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder); file = root/"weights"; file.write_bytes(b"original")
            entry = {"path":"weights","size":8,"mtime_ns":0,"sha256":hashlib.sha256(b"original").hexdigest()}
            (root/"verified.json").write_text(json.dumps({"revision":worker.REVISION,"files":[entry]}))
            file.write_bytes(b"modified")
            with patch.object(worker.Path, "read_text", side_effect=[json.dumps({"revision":worker.REVISION,"files":[entry]}), json.dumps({"files":[{"path":"weights","bytes":8}]})]):
                with self.assertRaisesRegex(ValueError,"changed"): worker.verify_snapshot(root)

    def test_incomplete_checkpoint_marker_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root/"verified.json").write_text(json.dumps({"revision":worker.REVISION,"files":[]}))
            with self.assertRaisesRegex(ValueError,"incomplete"): worker.verify_snapshot(root)

    def test_edit_requires_source_and_pixel_budget_is_bounded(self):
        request = {"profile":"nf4","width":1024,"height":1024,"seed":0,"mode":"edit","prompt":"change light","references":[]}
        with self.assertRaisesRegex(ValueError,"source"): worker.validate_request(request)
        request["references"] = ["input.png"]
        self.assertEqual(worker.validate_request(request),"change light")
        request["width"] = request["height"] = 4096
        with self.assertRaisesRegex(ValueError,"pixel"): worker.validate_request(request)


if __name__ == "__main__": unittest.main()
