# Image Wizard product brief

Prepared 2026-10-10. Midnight Atelier selected and implemented in version 0.2.0. See [verification.md](verification.md) for actual checks and limits.

## User and job

Give a Windows user a beautiful, understandable workspace for making images with the exact Qwen-Image-2.1-Turbo model. A person should be able to install it from inside the app, create from text, transform reference images, make instruction-based edits, and export results without assembling command-line workflows.

The core journey is **open app -> check this computer -> choose install location -> install -> verify -> write prompt -> generate -> export**. Each step should remain understandable during slow downloads, model loading, and generation failures.

## Platform

Use Tauri 2 with Rust for native operations, React/TypeScript/Vite for the interface, SQLite for local history, and an isolated managed Python process for inference. This is a local Windows application. Hosted generation is a possible future adapter, rather than a prerequisite for the requested local flow.

## Required surfaces

| Surface | User outcome |
| --- | --- |
| Create | Prompt, size/ratio, seed, PNG transparency request, generate, export |
| Transform | Add and order references, describe a transformation, retain originals |
| Edit | Choose source, describe change, compare before/after, create a new version |
| Models | Device check, disk estimate, model profile, storage picker, install/resume/repair |
| Library | Browse outputs, reopen settings, reuse seeds, see edit ancestry, reveal/export files |

Keep useful progress visible while the user visits another surface. During local denoising, report completed steps out of the loaded schedule. Download percentages must use expected file bytes. Runtime installation, model loading, and final image decoding need phase labels and elapsed time when no reliable percentage exists.

## Data and recovery

The installed model manifest plus verified files determine installation status. A UI flag cannot declare a model installed. A successful runtime probe and generation determine readiness; downloading files alone does not.

Persist model repository/revision, runtime versions, profile/precision, install path, file verification, download state, and last readiness result. Persist generation requests and job states with stable IDs. On restart, interrupted downloads become resumable and running generation becomes interrupted; completed outputs remain usable.

Save original input images without modifying them. Store each generated/edit result as a new PNG with its original prompt, effective prompt, dimensions, seed, model revision, runtime version, profile, KV-cache setting, and parent/reference IDs. Different software or precision may change a seeded result; reproducibility metadata should describe what was used without promising universal identical pixels.

## First implementation sequence

1. Confirm a visual concept; save the chosen reference and prompt in the repository.
2. Scaffold the desktop shell, design tokens, typed native contracts, and isolated inference protocol.
3. Complete hardware preflight, one managed download profile, resume/verification, and a small real generation probe.
4. Complete one prompt-to-PNG flow through the real runtime, persistence, history, and export.
5. Add reference-based transformations and instruction editing through the same contract.
6. Add comparisons, repeat/reuse, error recovery, accessibility, and packaging.

## Acceptance checks

- First launch explains model setup and provides a useful hardware/disk report.
- Cancellation stops downloads at a safe boundary; restart resumes verified bytes without discarding completed files.
- Failed checksum or unsupported runtime prevents readiness and offers a clear repair action.
- Create produces a real locally generated PNG with persisted metadata and a working export.
- Transform and Edit pass source/reference images to the actual inference pipeline and create new outputs.
- Transparency requests preserve any generated alpha channel; the app checks the result rather than assuming the request guaranteed alpha.
- Running work remains visible across navigation and cannot replace another job's selected result.
- A failed generation preserves the request and inputs, with actionable out-of-memory/dependency recovery.
- History and files remain available after restart.
- Wide and narrow windows support keyboard navigation, legible controls, and visible focus.
- Frontend and Rust checks pass; a production GUI executable launches with embedded assets and without a development server.

## Boundaries

No conventional strength slider, arbitrary model substitutions, or advertised pixel-perfect inpainting. Alternate sigma schedules, guidance/negative prompts, strict masked compositing, LoRAs, prompt-enhancer checkpoints, and hosted adapters need their own verified capability gates. A quantized/repacked profile must be identified separately from official BF16 weights.
