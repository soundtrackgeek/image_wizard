# Local installation and generation plan

Proposed implementation, 2026-10-10. These behaviors are not implemented in the research baseline.

## Native boundaries

React sends typed requests to Rust. Rust owns app-data paths, SQLite, installation records, child-process lifecycle, validated file selection, output publication, and model status. Python owns the selected inference adapter and reports structured JSON events. Prefer JSON over stdio to avoid an exposed local service; do not run inference inside the WebView.

Create an app-owned isolated environment rather than installing packages into the user's global Python. A packaged runtime or managed Python distribution must have a verified source, version, and integrity check. Use an argument-vector process API; user prompts and paths must never become shell commands. Hide helper console windows on Windows while retaining actionable logs.

## Model profiles

The manifest identifies the exact source repository, immutable revision, required files, hashes, precision, compatible runtime, and eight-step sampling schedule. The official BF16 profile is distinct from a Comfy-Org INT8/quantized profile. A low-memory profile is offered as ready only after its own text and image-input probes pass on the selected device.

Begin with the documented Diffusers adapter as the reference implementation. Evaluate compatible Comfy-Org or other explicit quantization adapters for the development machine's 16GB GPU. Do not silently exchange Turbo for the base 2.1 model or install a Turbo LoRA as though it were the checkpoint.

The final default profile remains an implementation decision requiring a measured successful run. The UI should explain when the reference BF16 profile is unsuitable and show a verified alternative when available.

## Installation flow

1. Detect available NVIDIA device, actual VRAM, CUDA usability, system RAM, free bytes on the chosen target volume, and existing runtime/model files. Keep the user able to choose another drive.
2. Present source, precision/profile, expected download bytes, storage overhead, and the model's actual license terms.
3. Download and verify the managed runtime and pinned dependencies. Probe required pipeline/configuration support before downloading large weights where possible.
4. Resolve the selected immutable model manifest. Download only required files into a staging snapshot. Use tested Hugging Face Hub cache/resume behavior or an equally verified ranged downloader.
5. Persist file-level progress and transfer validators. Resume without treating missing connectivity as corruption; reject stale manifest/ETag combinations.
6. Verify every weight size and SHA256. Verify small configuration files against pinned source metadata. Reject unexpected paths and paths outside the owned install root.
7. Publish the complete snapshot atomically, then load the selected runtime profile and perform a small prompt-to-PNG check.
8. Run an image-conditioned readiness check before enabling Transform/Edit for that profile. Record dimensions, alpha result, device, elapsed time, and peak memory where supported.

The [official Hub download API](https://huggingface.co/docs/huggingface_hub/en/package_reference/file_download) provides snapshot revisions, dry runs, file filters, caching, and progress facilities. Installation progress must cover runtime/dependencies, download, verification, loading, and test generation separately.

## State contracts

Model state: `absent -> preflight -> installing_runtime -> downloading -> verifying -> downloaded -> loading -> testing -> ready`. Any active phase can produce a structured failure. Paused/interrupted download remains resumable. Downloaded and ready are distinct states.

Generation state: `queued -> loading -> encoding -> denoising -> decoding -> saving -> succeeded`. Cancellation is acknowledged by actual worker state. Failures retain the request and inputs. A late completion updates its job ID and library entry; it does not replace another job currently selected in the interface.

One local generation runs at a time initially. The step denominator is taken from the loaded schedule; phases outside denoising show elapsed activity. Pause is a download feature; generation exposes cancellation at a documented safe boundary, not a fake pause.

## Requests and output authority

Each generation request includes job ID, model-install ID, mode, original/effective prompt, ordered references, output width/height, seed, alpha request, and immutable profile settings. The native boundary validates source paths and dimensions; image parsing checks actual content and bounded size. A random seed becomes a concrete saved seed before execution.

Store inputs and edited versions separately. Save output to an owned temporary PNG, decode/validate it, then publish atomically and commit its history record. If the process dies between file publication and history commit, recover the job using its journal rather than leaving a permanent orphan or declaring success prematurely.

Exporter copies a completed artifact to a user-selected path with explicit overwrite handling. Preserve RGBA PNG. JPEG export, if added, needs a user-selected background because JPEG cannot preserve alpha.

## Minimum verification

- Interrupt a staged transfer and confirm resumed bytes and correct final hash.
- Simulate a wrong hash, insufficient disk, incompatible dependency, failed GPU load, and out-of-memory generation.
- Check model state recovery and interrupted jobs after restart.
- Test two jobs with navigation between them; completion must preserve the current selection.
- Generate a real local text image, an image transformation, and an edit; validate exported files and retained originals.
- Verify the low-memory profile on the actual 16GB development device before advertising it as supported.
- Run frontend checks, Rust formatting/Clippy/tests, a real native startup, and release executable launch without a dev server. Verify Windows GUI subsystem.
