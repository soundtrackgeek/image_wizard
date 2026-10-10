"""Image Wizard's local worker. Requests are files; stdout contains JSON events only."""
from __future__ import annotations

import contextlib
import hashlib
import json
import os
from pathlib import Path
import sys
import time
import urllib.error
import urllib.request

REPOSITORY = "Qwen/Qwen-Image-2.1-Turbo"
REVISION = "d65dbc9a7e8f6b5479e33dee6030eaab2a906509"
SIGMAS = [1.0, .978453, .95418, .926626, .89508, .845148, .704534, .414568]
PROTOCOL_OUT = sys.stdout


class Cancelled(Exception):
    pass


def emit(phase, detail="", completed=None, total=None, **extra):
    record = {"phase": phase, "detail": detail, **extra}
    if completed is not None:
        record["completed"] = completed
    if total is not None:
        record["total"] = total
    print(json.dumps(record, ensure_ascii=False), file=PROTOCOL_OUT, flush=True)


def check_cancel(cancel_path):
    if Path(cancel_path).exists():
        raise Cancelled("Operation cancelled")


def digest(path):
    value = hashlib.sha256()
    with open(path, "rb") as stream:
        for block in iter(lambda: stream.read(4 * 1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def safe_relative(name):
    path = Path(name)
    if path.is_absolute() or ".." in path.parts or "\\" in name or ":" in name:
        raise ValueError("Unsafe model filename")
    return path


def download_file(url, destination, expected_size, expected_hash, cancel_path, progress, opener=urllib.request.urlopen):
    """Resume against an immutable URL; publish only after size/hash validation."""
    destination = Path(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.is_file() and destination.stat().st_size == expected_size:
        if expected_hash and digest(destination) == expected_hash:
            progress(expected_size)
            return
    partial = destination.with_name(destination.name + ".partial")
    offset = partial.stat().st_size if partial.exists() else 0
    if offset > expected_size:
        partial.unlink()
        offset = 0
    if offset < expected_size:
        request = urllib.request.Request(url, headers={"User-Agent": "ImageWizard/0.2.0"})
        if offset:
            request.add_header("Range", f"bytes={offset}-")
        check_cancel(cancel_path)
        with opener(request, timeout=60) as response:
            if offset and response.status == 206:
                content_range = response.headers.get("Content-Range", "")
                if not content_range.startswith(f"bytes {offset}-") or not content_range.endswith(f"/{expected_size}"):
                    raise ValueError("Server returned an unexpected download range")
            elif offset and response.status == 200:
                offset = 0
            elif response.status not in (200, 206):
                raise ValueError("Unexpected download response")
            mode = "ab" if offset else "wb"
            with open(partial, mode) as output:
                progress(offset)
                last_event = 0.0
                while True:
                    check_cancel(cancel_path)
                    block = response.read(1024 * 1024)
                    if not block:
                        break
                    if offset + len(block) > expected_size:
                        raise ValueError("Download exceeds expected file size")
                    output.write(block)
                    offset += len(block)
                    if time.monotonic() - last_event >= .2:
                        progress(offset)
                        last_event = time.monotonic()
    if partial.stat().st_size != expected_size:
        raise ValueError("Incomplete download; retry to resume")
    check_cancel(cancel_path)
    if expected_hash and digest(partial) != expected_hash:
        partial.unlink()
        raise ValueError("Checksum failed; the invalid partial file was removed. Retry download.")
    os.replace(partial, destination)
    progress(expected_size)


def install(request):
    root = Path(request["model_dir"])
    root.mkdir(parents=True, exist_ok=True)
    cancel = request["cancel_path"]
    manifest = json.loads(Path(request["manifest_path"]).read_text(encoding="utf-8-sig"))
    if manifest["repository"] != REPOSITORY or manifest["revision"] != REVISION:
        raise ValueError("Unexpected model manifest")
    files = [entry for entry in manifest["files"] if not entry["path"].startswith("assets/") and entry["path"] != ".gitattributes"]
    total = sum(entry["bytes"] for entry in files)
    completed = 0
    verified = []
    for entry in files:
        check_cancel(cancel)
        destination = root / safe_relative(entry["path"])
        url = f"https://huggingface.co/{REPOSITORY}/resolve/{REVISION}/{entry['path']}"
        emit("downloading", entry["path"], completed, total)
        download_file(url, destination, entry["bytes"], entry.get("sha256"), cancel,
                      lambda current: emit("downloading", entry["path"], completed + current, total))
        emit("verifying", entry["path"], completed + entry["bytes"], total)
        stat = destination.stat()
        verified.append({"path": entry["path"], "size": stat.st_size, "mtime_ns": stat.st_mtime_ns, "sha256": digest(destination)})
        completed += entry["bytes"]
    config = json.loads((root / "model_index.json").read_text())
    if config.get("sample_sigmas") != SIGMAS:
        raise ValueError("Downloaded checkpoint does not contain the official Turbo schedule")
    marker = {"repository": REPOSITORY, "revision": REVISION, "files": verified}
    marker_tmp = root / "verified.json.partial"
    marker_tmp.write_text(json.dumps(marker), encoding="utf-8")
    os.replace(marker_tmp, root / "verified.json")
    (root / "NOTICE").write_text("Qwen-Image-2.1-Turbo checkpoint by Alibaba Qwen.\nSource: https://huggingface.co/Qwen/Qwen-Image-2.1-Turbo\nUsed under the accompanying Qwen Research License (LICENSE).\n", encoding="utf-8")
    emit("downloaded", "Checkpoint verified. Run the engine check to confirm this computer can generate.")


def validate_request(request):
    if request.get("profile") not in ("nf4", "bf16"):
        raise ValueError("Unsupported memory profile")
    width, height = request["width"], request["height"]
    if not isinstance(width, int) or not isinstance(height, int) or min(width, height) < 256 or max(width, height) > 4096:
        raise ValueError("Output dimensions must be between 256 and 4096")
    if width % 16 or height % 16 or width * height > 4_400_000:
        raise ValueError("Output dimensions must be multiples of 16 within the pixel limit")
    if not isinstance(request["seed"], int) or not 0 <= request["seed"] <= 2147483647:
        raise ValueError("Seed is outside the supported range")
    references = request.get("references", [])
    if len(references) > 10:
        raise ValueError("Use at most ten reference images")
    if request.get("mode") in ("transform", "edit") and not references:
        raise ValueError("Add a source image first")
    prompt = request.get("prompt", "").strip()
    if not prompt or len(prompt) > 24000:
        raise ValueError("Enter a prompt of up to 24,000 characters")
    return prompt


def verify_snapshot(root):
    marker = json.loads((root / "verified.json").read_text())
    if marker.get("revision") != REVISION:
        raise ValueError("This checkpoint needs verification")
    manifest = json.loads(Path(__file__).with_name("download-manifest.json").read_text(encoding="utf-8-sig"))
    expected = {entry["path"]: entry["bytes"] for entry in manifest["files"] if not entry["path"].startswith("assets/") and entry["path"] != ".gitattributes"}
    actual = {entry["path"]: entry["size"] for entry in marker["files"]}
    if actual != expected or len(actual) != len(marker["files"]):
        raise ValueError("Checkpoint verification is incomplete; repair the installation")
    for entry in marker["files"]:
        file = root / safe_relative(entry["path"])
        stat = file.stat()
        if stat.st_size != entry["size"] or stat.st_mtime_ns != entry["mtime_ns"]:
            if digest(file) != entry["sha256"]:
                raise ValueError("Model files changed; repair the installation")


_UNREAD_TENSOR = object()


class CheckpointTensors(dict):
    """Materialize one tensor at a time while the pinned loader walks a shard."""
    def __init__(self, pointer, entries=None):
        self.pointer = pointer
        super().__init__(dict.fromkeys(pointer.keys(), _UNREAD_TENSOR) if entries is None else entries)

    def __getitem__(self, key):
        value = super().__getitem__(key)
        return self.pointer.get_tensor(key) if value is _UNREAD_TENSOR else value

    def get(self, key, default=None):
        return self[key] if key in self else default

    def pop(self, key, *default):
        if key not in self:
            if default:
                return default[0]
            raise KeyError(key)
        value = self[key]
        del self[key]
        return value

    def popitem(self):
        if not self:
            raise KeyError("Empty checkpoint")
        key = next(reversed(self))
        return key, self.pop(key)

    def __iter__(self):
        return super().__iter__()

    def items(self):
        for key in self:
            yield key, self[key]

    def values(self):
        for key in self:
            yield self[key]

    def copy(self):
        return CheckpointTensors(self.pointer, {key: dict.__getitem__(self, key) for key in self})


def configure_checkpoint_reader():
    if sys.platform == "win32":
        import functools
        import safetensors
        from safetensors import safe_open
        import safetensors.torch
        # Windows mmap can exhaust copy-on-write commit and crash torch_cpu.dll.
        # Stream tensors instead of materializing a whole shard's CPU dictionary.
        # Keep this compatibility default inside the isolated worker process.
        def read_checkpoint(filename, device="cpu", *, backend="pread"):
            return CheckpointTensors(safe_open(filename, framework="pt", device=device, backend=backend))
        # Metadata-only opens in Transformers also need the safe default.
        safetensors.safe_open = functools.partial(safe_open, backend="pread")
        safetensors.torch.safe_open = safetensors.safe_open
        safetensors.torch.load_file = read_checkpoint


def failure_message(error):
    cause = error
    visited = set()
    while cause is not None and id(cause) not in visited:
        visited.add(id(cause))
        if isinstance(cause, MemoryError) or "paging file is too small" in str(cause).lower():
            return "System memory was exhausted while loading the checkpoint. Close memory-heavy apps and ensure Windows has space for its paging file, then retry."
        cause = cause.__cause__ or cause.__context__
    message = str(error) or type(error).__name__
    if "out of memory" in message.lower():
        message = "GPU memory was exhausted. Try a smaller image or the Memory saver profile. " + message
    return message[:1800]


def generate(request):
    prompt = validate_request(request)
    cancel = request["cancel_path"]
    root = Path(request["model_dir"])
    verify_snapshot(root)
    check_cancel(cancel)
    emit("loading", "Loading local libraries")
    import torch
    configure_checkpoint_reader()
    from PIL import Image
    from diffusers import QwenImage21Pipeline, QwenImage21Transformer2DModel, BitsAndBytesConfig as DiffusersQuant
    from transformers import Qwen3VLForConditionalGeneration, BitsAndBytesConfig as TransformersQuant
    if not torch.cuda.is_available():
        raise RuntimeError("CUDA is unavailable. Install an NVIDIA driver, then repair the runtime.")
    emit("loading", "Loading the model into the selected memory profile")
    options = {"dtype": torch.bfloat16, "local_files_only": True, "low_cpu_mem_usage": True}
    if request["profile"] == "nf4":
        quant = {"load_in_4bit": True, "bnb_4bit_quant_type": "nf4", "bnb_4bit_use_double_quant": True, "bnb_4bit_compute_dtype": torch.bfloat16}
        encoder = Qwen3VLForConditionalGeneration.from_pretrained(root / "text_encoder", quantization_config=TransformersQuant(**quant), device_map="cuda", **options)
        transformer = QwenImage21Transformer2DModel.from_pretrained(root / "transformer", quantization_config=DiffusersQuant(**quant), device_map="cuda", **options)
        pipe = QwenImage21Pipeline.from_pretrained(root, text_encoder=encoder, transformer=transformer, **options)
        pipe.vae.to("cuda")
    else:
        pipe = QwenImage21Pipeline.from_pretrained(root, **options)
        pipe.enable_model_cpu_offload()
    if pipe.sample_sigmas != SIGMAS:
        raise RuntimeError("Runtime did not load the Turbo eight-step schedule")
    pipe.vae.enable_tiling()
    pipe.set_progress_bar_config(disable=True)
    refs = []
    for path in request.get("references", []):
        with Image.open(path) as source:
            source.load()
            refs.append(source.convert("RGBA"))
    effective_prompt = prompt
    if request.get("transparent"):
        effective_prompt += "\nGenerate a transparent background with an alpha channel (RGBA), keep the subject opaque."
    def callback(_pipe, index, _timestep, tensors):
        check_cancel(cancel)
        emit("denoising", f"Step {index + 1} of {len(pipe.sample_sigmas)}", index + 1, len(pipe.sample_sigmas))
        return tensors
    check_cancel(cancel)
    emit("encoding", "Reading the prompt and references")
    torch.cuda.reset_peak_memory_stats()
    with torch.inference_mode():
        result = pipe(prompt=effective_prompt, image=refs or None, width=request["width"], height=request["height"],
                      use_kv_cache=True, generator=torch.Generator("cpu").manual_seed(request["seed"]),
                      callback_on_step_end=callback).images[0]
        if request.get("test_edit"):
            check_cancel(cancel)
            emit("encoding", "Checking image-conditioned editing")
            effective_prompt = "Change the ceramic sphere to pale blue. Keep its shape and the background."
            result = pipe(prompt=effective_prompt, image=[result.convert("RGBA")], width=request["width"], height=request["height"],
                          use_kv_cache=True, generator=torch.Generator("cpu").manual_seed(request["seed"]),
                          callback_on_step_end=callback).images[0]
    check_cancel(cancel)
    emit("saving", "Saving your image")
    output = Path(request["output_path"])
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_name(output.name + ".partial")
    result.save(temporary, format="PNG")
    with Image.open(temporary) as test:
        test.verify()
    alpha = result.mode == "RGBA" and result.getchannel("A").getextrema()[0] < 255
    record = {"path": str(output), "width": result.width, "height": result.height,
         "hasAlpha": alpha, "effectivePrompt": effective_prompt, "peakVramBytes": torch.cuda.max_memory_allocated(),
         "torchVersion": torch.__version__, "modelRevision": REVISION, "diffusersRevision": "1d5d056ecff3a8c1d887a7dd1aa66ca4a0323458", "kvCache": True,
         "checkpointBackend": "pread" if sys.platform == "win32" else "mmap",
         "verification": "text-and-image" if request.get("test_edit") else None}
    artwork = {**request["studioMetadata"], "effectivePrompt": effective_prompt, "hasAlpha": alpha, "provenance": record.copy()}
    journal = output.with_name(output.name + ".metadata.json")
    journal_tmp = journal.with_name(journal.name + ".partial")
    journal_tmp.write_text(json.dumps(artwork), encoding="utf-8")
    os.replace(journal_tmp, journal)
    os.replace(temporary, output)
    emit("succeeded", "Image saved", result={**record, "artwork": artwork})


def probe(_request):
    import torch
    import torchvision
    import inspect
    from diffusers import QwenImage21Pipeline
    if "sample_sigmas" not in inspect.signature(QwenImage21Pipeline).parameters:
        raise RuntimeError("Diffusers lacks Turbo schedule support")
    if not torch.cuda.is_available():
        raise RuntimeError("CUDA is unavailable. Check your NVIDIA driver.")
    emit("succeeded", "Runtime supports CUDA, vision processing, and the Turbo pipeline", result={"device": torch.cuda.get_device_name(0), "torchvisionVersion": torchvision.__version__})


def main():
    request = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8-sig"))
    try:
        with contextlib.redirect_stdout(sys.stderr):
            if request["action"] == "install":
                install(request)
            elif request["action"] == "probe":
                probe(request)
            elif request["action"] == "generate":
                generate(request)
            else:
                raise ValueError("Unknown worker action")
    except Cancelled:
        emit("cancelled", "Stopped. Downloaded partial files are kept for resume.")
        return 2
    except Exception as error:
        emit("failed", failure_message(error))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
