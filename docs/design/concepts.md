# Visual exploration

Generated 2026-10-10 using the built-in imagegen tool. These are preview-only concepts. Selection is pending; no concept has been adopted as the implementation reference.

## Directions

| Concept | Thesis | Tradeoff |
| --- | --- | --- |
| A: Midnight Atelier | Warm charcoal, copper accents, separate prompt and settings panels, immersive image preview | Clear dedicated controls; denser professional layout |
| B: Paper Studio | Warm ivory, editorial type, integrated composition sheet and contact strip | Approachable and spacious; brighter workspace around the art |
| C: Canvas Lab | Pale sage, canvas-first composition, assets drawer, floating prompt composer | More room for visual editing; less conventional control placement |

The model name, core workflows, prompt, image subject, settings, progress state, and output affordances were held consistent across prompts. Generated copy and controls are illustrative: implementation must align them with verified capability and active job state. The images are not Qwen-generated samples.

After selection, copy the chosen generated image into `docs/design/selected-concept.png` and record the decision here. Images currently remain in Codex's preview image storage.

## Prompt set

The full exact prompts are stored in [concept-prompts.json](concept-prompts.json), one prompt per built-in generation call. No reference images were supplied. Each output was generated with an opaque background.

Use the selected image's hierarchy and visual character as the code reference; text, controls, layout, and icons remain real interface elements. Include first-run model installation, download/resume, loading, error, and interrupted-job states in the finished design.
