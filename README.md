# Image Wizard

A planned Windows desktop image studio for **Qwen-Image-2.1-Turbo**, with text prompts, image transformations, instruction-based editing, and model installation from inside the app.

## Project status

Research and visual exploration are complete. Application implementation awaits selection of a visual concept. This baseline contains the product brief, verified model research, download metadata, and design prompts; it does not yet contain a runnable application or installer.

Three Luna research agents checked the exact model's capabilities, local runtime requirements, and integration contracts against primary sources on **2026-10-10**. Three preview concepts were generated with the built-in imagegen tool using the Vibe App Builder workflow.

## Planned experience

- **Create:** describe an image, choose its dimensions and seed, generate it locally, and export PNG.
- **Transform:** drop in reference images and describe a new composition or style.
- **Edit:** describe changes to an existing image, compare versions, and retain the source.
- **Models:** check hardware, choose a storage location, install the runtime and model, follow download progress, resume interruptions, and verify readiness.
- **Library:** keep generated images, prompts, seeds, model revisions, and edit ancestry together on disk.

The proposed stack is **Tauri 2 + Rust + React/TypeScript/Vite**, with SQLite for history and a managed Python inference process. Backend selection and hardware profiles require actual Windows inference validation.

## Read the preparation

- [Product brief and acceptance checks](docs/app-brief.md)
- [Model capabilities, controls, hardware, and sources](docs/research/qwen-image-2.1-turbo.md)
- [Local installation and runtime plan](docs/local-runtime-plan.md)
- [Captured official model file metadata](docs/research/model-manifest.json)
- [Visual concepts and generation prompts](docs/design/concepts.md)
- [Changelog](CHANGELOG.md)

## Model requirements

The [official Turbo checkpoint](https://huggingface.co/Qwen/Qwen-Image-2.1-Turbo) is publicly downloadable. Its complete repository is approximately **32.5 GB**, including the separate encoder and VAE. Additional runtime, cache, working memory, and output space are needed. Qwen has not published a guaranteed minimum VRAM specification; an installation must pass a real generation check before the app marks it ready.

The weights use the [Qwen Research License](https://github.com/QwenLM/Qwen-Image-2.1/blob/main/LICENSE), which defines non-commercial use as research or evaluation and requires a separate license for commercial use. The application will show the model's terms before installation and preserve required notices.

## Development

Run, test, and packaging commands will be documented when the application scaffold is added. No weights, inference runtime, or paid hosted API calls have been installed or run as part of this research baseline.

Follow [AGENTS.md](AGENTS.md): update documentation and version history for changes, then commit and push.
