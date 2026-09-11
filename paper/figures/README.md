# TDSC manuscript figures

This directory contains the complete ten-figure visual narrative requested for the manuscript. The running-example figure is intentionally excluded. Figures 1–9 are architecture/semantics figures grounded in the inspected code. Figure 10 is an explicitly marked internal data-required preview because no complete reproducible evaluation series was found.

## Contents

- `src/`: editable SVG source with selectable text.
- `pdf/`: vector PDF exports at intended physical width.
- `png/`: 600 dpi PNG exports plus color and grayscale contact sheets.
- `style/`: palette and line-semantics specification.
- `captions.md`, `alt_text.md`: manuscript captions and accessibility text.
- `figure_manifest.md`: provenance, implementation status, and unresolved questions.
- `data/`: header-only Figure 10 data schemas; no fabricated values.
- `scripts/`: deterministic generation, export, and validation.

## Regeneration

From the repository root:

```bash
bash paper/figures/scripts/regenerate.sh
```

The generator uses Python’s standard library, Pillow for the contact sheet, and the installed `librsvg-2`/`libcairo` shared libraries for native PDF and 600 dpi PNG export. It does not alter project production files.

To test Figure 10 data readiness:

```bash
python3 paper/figures/scripts/fig10_results.py
```

That command intentionally exits nonzero while any required CSV is header-only. Once real data exist, implement the preregistered aggregation and uncertainty method before replacing the draft.

## Scientific status

Solid artwork represents mechanisms verified in source. Dotted artwork marks target architecture or pending integration. Most importantly, the current figures do not call linked proofs “aggregated,” do not invent in-circuit signatures, do not claim that the analyzer proves whole-system correctness, and do not conflate algebraic witness recoverability with a general failure of formal zero knowledge.
