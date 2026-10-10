# Changelog

## [0.2.0] - 2026-10-10

### Added

- Midnight Atelier Tauri/React Windows desktop app with copper/charcoal styling, dedicated controls, canvas, and responsive layouts.
- Create, Transform, and Edit with ten references, ratios/sizes, seeds, style helpers, transparency requests, comparison, and PNG export.
- Hardware/storage model manager, license acknowledgement, isolated runtime, pinned official weights, resumable verified transfers, repair, cancellation, and text/image engine checks.
- NF4 and BF16 CPU-offload profiles, the official eight-step schedule, and offline model loading.
- SQLite library, retained sources/ancestry, provenance, atomic PNGs, recovery journals, and diagnostic logs.
- Frontend, native, and downloader tests; embedded-assets production executable.

### Fixed

- Managed Python setup verifies the exact interpreter when uv cannot create its minor-version junction.
- Runtime installs the matching Torchvision package required by Qwen's multimodal processor and probes it before downloading weights.
- Late responses/job completion preserve the user's current selection.
- Main actions remain visible at the minimum desktop size.
- Model-manager notifications leave setup and retry actions accessible.
- Native asset scopes use the application data root correctly so imported/generated PNGs display in production.
- Stable user-profile storage and WebView data prevent packaged launchers from redirecting Image Wizard into their own AppData cache.
- Requested transparency is saved separately from actual alpha, preserving settings reuse.
- Windows checkpoint loading uses byte reading and lazy tensors to avoid native memory-mapping failures and whole-shard CPU allocation; runtime failures retain their exit status and useful log tail.

## [0.1.0] - 2026-10-10

### Added

- Initial research baseline for the planned Image Wizard Windows application; application implementation is pending visual direction selection.
- Primary-source research on Qwen-Image-2.1-Turbo generation, reference-image transformations, editing, transparency, sampling, and licensing.
- Hardware-aware local installation plan with resumable downloads, revision and checksum verification, and real inference readiness checks.
- Product brief, implementation acceptance checks, official checkpoint file metadata, and three visual concept prompts.
