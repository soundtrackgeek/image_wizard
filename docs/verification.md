# Verification — 0.2.0

Date: 2026-10-10. Windows x64, NVIDIA GeForce RTX 4060 Ti (16,380 MiB), approximately 33.5 GB RAM, driver 610.88.

## Automated checks

Passed: TypeScript/Vite build, 3 Vitest checks, Rust formatting/Clippy with warnings denied, 6 Rust validation/recovery/instance tests, and 11 Python downloader/snapshot/lazy-reader/error-reporting tests.

## Native and visual checks

The real Tauri WebView was driven with Playwright/CDP. Model manager reads actual GPU, memory, and drive storage. Runtime installation reproduced a Windows Python junction failure, repaired by verifying the exact pinned interpreter.

Wide (1440 × 960), minimum (920 × 640), smaller content height (920 × 600), and mobile preview (390 × 844) were inspected. Desktop actions remain visible without page overflow; mobile has no horizontal overflow. Expanded preview traps Tab on its close action and closes with Escape. Browser/native consoles reported zero errors. Native import created an owned 1254 × 1254 PNG copy from the sample.

An actual transfer was cancelled after 8,609,857,536 bytes of the encoder. Its partial remained and resume continued beyond that offset. The completed 32,451,622,214-byte inference snapshot (excluding demonstration assets) passed weight checksums and configuration verification. Repair installed Torchvision 0.24.1/CUDA 12.8 and reverified weights.

The release executable launched at `http://tauri.localhost/` with port 1438 stopped. Embedded assets and native commands work. Windows GUI subsystem 2 was verified. Readiness and installation persisted across restart.

NF4 engine check passed real text generation followed by an image-conditioned edit at 512 × 512, seed 42, changing an orange ceramic sphere to pale blue. Total operation time including library/model loading: approximately 136 seconds. Peak PyTorch allocation: 13,707,190,272 bytes (12.77 GiB), which excludes unrelated GPU applications and is not a universal memory requirement. RGBA PNG, actual alpha, pinned model/runtime revisions, KV cache, and completed metadata were inspected. Diagnostics are kept outside the creative library.

Normal Create passed at 1024 × 1024, seed 1234, in approximately 84 seconds including model loading, with peak PyTorch allocation about 10.62 GiB. The resulting glass orchid on burgundy velvet was inspected. Edit passed at 512 × 512, seed 5678, in approximately 122 seconds: the velvet became teal. The source retained SHA256 `0CBD42603470FF81D3CDFDB8B1BFDA8B1F63BE5E4FB5D7D34870FA74B9339D44`. The edit records the source as parent/reference and is a separate PNG.

During Edit, a different selection was made in Library. Completion added the edit without replacing the selected original on the canvas. History persisted across release restarts. Native input/output previews load after correcting the asset scope's app-data root. The latest release console has zero errors. Async native import and the imported PNG preview were verified.

Native export produced a PNG with exactly the source hash above; a second export to that destination was rejected and the file remained intact. The exporter command was tested directly through native IPC; operating-system file-picker interaction was not automated.

Transform passed the image-conditioned execution and transparent PNG checks at 512 × 512, seed 9012, in approximately 131 seconds. Its RGBA alpha ranges from 0 to 255, with 17.55% of pixels fully transparent. Native preview displays the checkerboard through the transparent background. The model did not fully follow the requested change from glass to porcelain; this is a recorded quality limitation of that prompt/profile/size. Its PNG and exported copy share SHA256 `1D873221B4D89FEBB5517F8E0644A4358C05E2A0E5D99C9978C712A90A9441D1`.

Create, Edit, and Transform retain distinct PNGs, seeds, requested transparency, references, and parent IDs. Reusing Create settings leaves the transparency request off even though its PNG contains alpha.

Canonical path inspection found that the original AppData location resolved inside Codex's package cache. Windows documents this class of [packaged-app data redirection](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes). The final release uses `%USERPROFILE%\.image-wizard` for both backend files and its WebView profile. The installed runtime, checkpoint, images, and history were preserved there; all four copied weights passed SHA256 verification, and history/reference/recovery-journal paths were checked. The copied isolated runtime reports Python 3.12.12, PyTorch 2.9.1/CUDA 12.8, Torchvision 0.24.1/CUDA 12.8, and CUDA availability.

Relocated input/output previews and native import/export were verified. The relocated Create PNG and transparent exported PNG still match their original hashes. Exactly four redundant old weight files were removed only after both copies matched the manifest hashes; the new verified checkpoint and original small history/image recovery files were preserved.

Post-relocation checking exposed Windows paging-file exhaustion and a `torch_cpu.dll` access violation (`0xC0000005`) during checkpoint loading. A [primary upstream report](https://github.com/safetensors/safetensors/issues/693) describes the same Windows mmap failure class; the pinned Transformers source already uses `pread` for its main Windows weight path. The worker now defaults metadata opens and Diffusers tensor reads to `pread`. A dictionary-compatible lazy reader supplies one tensor at a time as it is quantized, rather than materializing a whole 10 GB shard in CPU memory. Tests verify lazy metadata queries, values, copying, and key remapping. Failure reporting includes process status and the useful log tail, and empty/wrapped memory errors provide recovery guidance.

The final in-app NF4 text-and-edit check passed from the stable home folder in 113.3 seconds, enabled readiness, and kept the three creative images in Library. The worker's separate full text-and-edit check also passed with `checkpointBackend: pread`; its pale-blue sphere was inspected. Model-manager notification placement was checked at 1440 × 844 and 920 × 640 so setup/retry actions remain unobscured.

The final executable retained NF4 readiness and all three creative images after restart. Production previews all loaded, Generate was enabled, the setup retry action was unobscured, and the browser console reported zero errors. [The implementation screenshot](design/implemented-studio.png) captures that native release with the real 1024-pixel Create result.

## Limits

Device inference was tested with NF4 at 512 and 1024 pixels. BF16 CPU offload, native 2K inference, and simultaneous use of ten references were not exercised on this machine. Sizes/reference counts depend on memory. Each profile is enabled only after its actual text and image check. No cross-device performance guarantee, commercial model license, hosted adapter, pixel-exact inpainting, or MSI/NSIS installer is included.
