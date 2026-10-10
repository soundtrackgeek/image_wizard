# Local runtime implementation

Version 0.2.0, 2026-10-10. Tauri/Rust owns paths, SQLite, process lifetime, import/export, progress, and model status. Managed Python owns installation/inference and emits JSON over stdout; library diagnostics go to per-job logs. No inference HTTP server is exposed.

## Installation

The app verifies the pinned uv download checksum, installs Python 3.12.12 under `%USERPROFILE%\.image-wizard`, probes the exact interpreter, and creates an isolated environment. Backend files and the WebView profile share this stable user folder. The exact interpreter avoids a Windows uv minor-version junction failure. Pinned CUDA PyTorch and Qwen-compatible Diffusers/dependencies are installed without Git or global Python changes.

The checkpoint manifest pins repository, revision, sizes, and weight hashes. Immutable Hugging Face URLs supply files. `.partial` files resume with HTTP Range; mismatched Content-Range is rejected and ignored Range restarts the file. Completed weights are rehashed; invalid checksums never publish. Configuration/license files use the same revision. A complete marker and the official sigma configuration are required. An attribution NOTICE accompanies LICENSE.

Preflight considers remaining download bytes plus working space. Cancellation terminates the worker tree; partial weights survive. Closing the Windows app kills owned subprocesses through a Job Object. Restart marks unfinished work interrupted and offers resume. Downloaded and ready are distinct states.

## Inference

- **NF4 memory saver:** quantize the Qwen3-VL encoder and QwenImage21 transformer at load time, with double quantization and BF16 computation. Original checkpoints stay on disk.
- **BF16 original precision:** official model with CPU model offload. Requires substantial host memory and a separate check.

`QwenImage21Pipeline` uses the saved official eight-step schedule, KV cache, and VAE tiling. Dimensions are bounded, divisible by 16, and limited to 4.4 million pixels. Seeds are concretized before execution. References are imported with file/pixel bounds; generation accepts only owned paths. Transform and Edit use Qwen's image-conditioned instruction pipeline. Alpha is requested in the effective prompt and inspected in the result.

On Windows, the isolated worker sets Safetensors 0.8.0's metadata-open and checkpoint-loader defaults to `pread` before importing Diffusers and Transformers. The main Transformers Windows weight path already selects that backend. A dictionary-compatible lazy reader supplies Diffusers tensors as they are loaded and quantized, avoiding a whole-shard CPU dictionary. This avoids whole-file copy-on-write mapping pressure; installed library files are not modified. Output provenance records the backend. The exact Diffusers version is pinned because this compatibility path follows its loading contract.

**Check engine** runs text generation followed by an image-conditioned edit in the same loaded pipeline. Only completion enables a profile. Changing profile/location clears readiness. No guaranteed minimum VRAM or universal seeded pixel identity is claimed.

## Durable state

One operation runs at a time. Rust stores progress in SQLite WAL. Each request has a UUID and is saved in `operations/`. The worker verifies a temporary PNG, writes metadata, then publishes PNG atomically. Rust commits history. On startup, matching valid PNG/metadata recover an interrupted operation idempotently; incomplete images remain interrupted.

History saves prompts, seed, size, alpha, precision, references, parent, model revision, torch version, Diffusers revision, and KV cache. Polling discards stale responses and selects completion only while the user still awaits that job. Export copies to a new PNG filename; existing files are not overwritten.

Tests exercise resume, ignored/mismatched ranges, corruption, cancellation, incomplete snapshots, escaping paths, bounded requests, job serialization, interruption, and completed-image recovery. Actual device/native/production checks are in [verification.md](verification.md).
