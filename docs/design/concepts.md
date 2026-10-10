# Visual exploration

Generated 2026-10-10 using the built-in imagegen tool. The user selected **A: Midnight Atelier**: dark, copper accents, and dedicated controls. [selected-concept.png](selected-concept.png) is the native studio's implementation reference.

## Directions

| Concept | Thesis | Tradeoff |
| --- | --- | --- |
| A: Midnight Atelier | Warm charcoal, copper accents, separate prompt and settings panels, immersive image preview | Clear dedicated controls; denser professional layout |
| B: Paper Studio | Warm ivory, editorial type, integrated composition sheet and contact strip | Approachable and spacious; brighter workspace around the art |
| C: Canvas Lab | Pale sage, canvas-first composition, assets drawer, floating prompt composer | More room for visual editing; less conventional control placement |

The model name, core workflows, prompt, image subject, settings, progress state, and output affordances were held consistent across prompts. Generated copy and controls are illustrative: implementation must align them with verified capability and active job state. The images are not Qwen-generated samples.

The implementation retains separate prompt, canvas, and settings hierarchy, copper actions, warm charcoal panels, and understated typography. Controls and state are actual React/native functionality.

[implemented-studio.png](implemented-studio.png) shows the production desktop app with a real 1024 × 1024 Qwen result, generated locally with the NF4 profile and seed 1234. The verification record separates model output from illustrative design assets.

## Prompt set

The full exact prompts are stored in [concept-prompts.json](concept-prompts.json), one prompt per built-in generation call. No reference images were supplied. Each output was generated with an opaque background.

Use the selected image's hierarchy and visual character as the code reference; text, controls, layout, and icons remain real interface elements. Include first-run model installation, download/resume, loading, error, and interrupted-job states in the finished design.

## Illustrative studio asset

`public/studio-sample.png` was generated with built-in imagegen, opaque background, no reference image, and labelled **Studio sample · illustrative artwork** in the app. It is separate from Qwen results.

Prompt: "Editorial macro still-life photograph of a sculptural translucent glass orchid, pale blush and clear petals with jewel-like droplets and natural refractions, resting on dark burgundy velvet. Warm morning light from upper left, dark background, a square composition with the complete central flower. No UI, text, logos, or watermark."
