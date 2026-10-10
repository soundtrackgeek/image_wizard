# Qwen-Image-2.1-Turbo research

Verified 2026-10-10 by three Luna research agents and direct official metadata checks. The linked X announcement could not be fetched; the official release repository and model card corroborate the release.

## Exact checkpoint

Local repository: `Qwen/Qwen-Image-2.1-Turbo`. Hosted identifier: `qwen-image-2.1-turbo`. Released 2026-10-09. This is the accelerated Qwen-Image-2.1 checkpoint, not an earlier model, a Turbo LoRA, or a community distillation. [Official Qwen repository](https://github.com/QwenLM/Qwen-Image-2.1).

Turbo loads with `QwenImage21Pipeline`, uses a saved eight-step sampling schedule, defaults to CFG=1, and supports prefix KV caching. Changing `num_inference_steps` alone does not replace that schedule. Explicit `sigmas` overrides are possible but other schedules have not been evaluated for this checkpoint. [Turbo model card](https://huggingface.co/Qwen/Qwen-Image-2.1-Turbo).

## What the app can expose

| User control/workflow | Implementation decision |
| --- | --- |
| Text to image | Prompt with no image inputs |
| Image transformation/edit | Prompt plus one or more ordered condition images |
| Multi-reference composition | Enforce the published maximum of ten inputs |
| Localized edit | Send an annotated reference, or original plus separate mask reference; instruction guided |
| Transparent background/extraction | Use the documented transparency instruction and preserve RGBA PNG |
| Typography/posters | Prompt templates with exact requested text |
| Size and aspect ratio | Supported presets; smaller trial outputs depend on backend validation |
| Seed | Random or saved explicit generator seed |
| Multiple variations | Initially queue one local output at a time to control peak memory |
| Performance profile | Choose a verified precision/offload configuration, not a cosmetic quality label |

Qwen documents transparency, subject extraction, identity/product preservation, multi-reference composition, and region selection using circles, paint, or separate masks. These are model capabilities; strict preservation of every unmasked pixel is not promised. [Official capability blog](https://qwen.ai/blog?id=qwen-image-2.1).

The pipeline accepts ordered image lists, width/height, a generator, `num_images_per_prompt`, and `use_kv_cache`. Negative prompts activate with `true_cfg_scale > 1`, doubling per-step work; keep these advanced experiments out of normal Turbo controls. There is no conventional `strength` argument or dedicated `mask_image` argument in its documented call signature. Expose transformation instructions rather than a misleading denoising-strength slider. Changing KV caching can change a seeded sample, so persist that setting. [Diffusers pipeline reference](https://huggingface.co/docs/diffusers/en/api/pipelines/qwenimage21).

## Official output presets

| Ratio | Width x height |
| --- | --- |
| 1:1 | 2048 x 2048 |
| 4:3 | 2400 x 1792 |
| 3:4 | 1792 x 2400 |
| 3:2 | 2528 x 1696 |
| 2:3 | 1696 x 2528 |
| 16:9 | 2752 x 1536 |
| 9:16 | 1536 x 2752 |

These local model-card presets are distinct from the hosted API's output-area restrictions. [Turbo model card](https://huggingface.co/Qwen/Qwen-Image-2.1-Turbo).

## Download footprint and hardware

Direct [Hugging Face metadata](https://huggingface.co/api/models/Qwen/Qwen-Image-2.1-Turbo?blobs=true) returned revision `d65dbc9a7e8f6b5479e33dee6030eaab2a906509`, with no access gate.

| Component | Bytes |
| --- | ---: |
| Text/image encoder | 17,534,340,584 |
| Transformer shard 1 | 9,968,332,504 |
| Transformer shard 2 | 4,261,951,904 |
| VAE | 675,508,656 |
| Four weight files total | 32,440,133,648 |
| Full repository including other files | 32,493,327,348 |

The four weight files are about 30.21 GiB; the repository is about 30.26 GiB. Disk size is not a peak-memory specification. The 7B visual component does not include the separate encoder. Exact captured file sizes and available SHA256 digests are in [model-manifest.json](model-manifest.json).

Live development hardware: NVIDIA RTX 4060 Ti, 16,380 MiB VRAM, driver 610.88; approximately 31.2 GiB system RAM and 99.3 GiB free on C: at the time of inspection. Windows `AdapterRAM` returned a misleading near-4GB value; `nvidia-smi` established the actual VRAM.

Inference has not been run. Full BF16 weights cannot all reside in this GPU's VRAM. Offloading/quantization and conservative initial resolution need actual Windows testing; Qwen does not publish an official guaranteed minimum. CUDA availability alone is insufficient evidence that a profile can generate successfully.

[Comfy-Org's compatible repository](https://huggingface.co/Comfy-Org/Qwen-Image-2.1/tree/main) includes a separate Turbo INT8 ConvRot transformer and quantized encoder choices. The researched INT8 transformer + W4A8 encoder + BF16 VAE set totals roughly 14.24 GB; with INT8 encoder it is roughly 17.28 GB. These are distinct repack/precision profiles with extra activation and cache memory, not guaranteed 16GB solutions. Download only the selected components, not the entire multi-variant repository.

## Dependencies and runtime

Install a CUDA-compatible PyTorch build in an isolated environment. The Turbo card requires latest Diffusers support including [PR #14950](https://github.com/huggingface/diffusers/pull/14950), `transformers>=5.17.0`, Accelerate, and Pillow. Pin a tested runtime rather than following mutable Git `main` on each launch. [Turbo model card](https://huggingface.co/Qwen/Qwen-Image-2.1-Turbo).

Qwen documents CPU offload. It is an option to validate, not a promise of acceptable latency or RAM use. A first-run readiness probe must test the selected backend, precision, loading, actual generation, and PNG decoding.

## Hosted contract, optional future adapter

Alibaba documents generation/editing through OpenAI-compatible and DashScope endpoints. For 2.1 Turbo, image-conditioned requests accept 1-10 references; output count is 1-6. Controls include seed, size, watermark, prompt extension, and thinking. Agent prompt extension is for text-only generation; direct extension supports editing. Negative prompt is unsupported for hosted 2.1 Turbo.

The hosted size area is 512 x 512 through 2048 x 2048, with ratio 1:8 through 8:1. Uploaded references have format, dimensions, and 10MB/file limits. Region, workspace endpoint, model availability, and key must match.

OpenAI-compatible access is synchronous: no multipart `/images/edits`, dedicated mask field, streaming, or partial images. DashScope async reports job states; only queued jobs can be cancelled. Output URLs expire after 24 hours. A future native adapter must download outputs immediately, keep keys in the OS credential store, and display indeterminate server progress rather than inventing denoising steps. Prompt-extension controls belong to hosted generation; local enhancers would be separate installations. [Alibaba API reference](https://www.alibabacloud.com/help/en/model-studio/qwen-image-generation-and-editing-api-reference).

## License

The [Qwen Research License](https://raw.githubusercontent.com/QwenLM/Qwen-Image-2.1/main/LICENSE) defines non-commercial use as research or evaluation. Commercial use needs a separate license. Redistribution requires the agreement, attribution notice, and modification notices where applicable. The installer should display the actual terms and record their revision; downloadable public weights must not be described as unrestricted commercial weights. Keep the product name Image Wizard and refer to Qwen descriptively.

## Evidence limits

- No weights were downloaded and no inference was attempted during research.
- Windows backend compatibility, measured latency, RAM/VRAM peaks, quantized edit quality, and alpha-channel success remain acceptance checks.
- The preview concepts contain illustrative images and interface copy. They are visual references, not evidence of Qwen output or an operational application.
- Primary source metadata and dependency support can change; refresh and pin them during implementation.
