# Visual direction

## Role of the visual direction

LocaLog’s visual language was developed through early private studies and is now expressed as a public design contract in this document and `docs/UX.md`. The studies themselves are not part of the repository. Future public images should be screenshots of the implemented application once it represents the project honestly.

The direction is not a literal screen specification. Controls such as avatars, sharing, account UI, recording actions, automatic speaker handling, rich formatting tools, and generic dashboard elements are excluded unless the product specification independently requires them.

## North star

Use the light start page and the first New Meeting setup image for style:

- warm cream surfaces
- generous whitespace
- calm minimalism
- precise, editorial typography
- architectural proportions and grid
- thin dividers and restrained borders
- controls that feel deliberate rather than component-library defaults

Use the denser second New Meeting image only for information architecture: it contains useful fields but is too form-heavy visually.

Relevant influences:

- Pascal's website/editorial architectural language for identity
- Zen Browser for slim sidebar and focus behaviour
- Mercury-like finish for careful states and typography, not dashboard conventions

UI quality is not a polish phase or a disposable shell around the processing pipeline. It is a core product requirement equal to local-first behaviour and data reliability. The Phase 0 shell should establish foundations that can survive into the vertical slice: real navigation behaviour, visual tokens, typography, layout, interaction states, and accessibility.

Before detailed UI implementation, document and review:

1. semantic light and dark colour/surface tokens;
2. typography families or platform-safe stacks, sizes, weights, line heights, and hierarchy;
3. spacing scale, content widths, and layout grid;
4. sidebar sizing, selection, collapse/resizing policy, and workspace behaviour;
5. common controls with default, hover, active, focus, disabled, loading, error, and success states;
6. contextual-inspector and progressive-disclosure rules;
7. visual acceptance criteria for the start, project, new-meeting, transcript-review, protocol-editor, and settings screens.

The authoritative tokens, hierarchy, grid, control states, inspector rules, and screen criteria are consolidated in `docs/UX.md`. Update that contract before detailed UI implementation when visual evidence changes.

## Layout rules

- Prefer planes, columns, spacing, type, and thin rules over nested cards.
- Keep the persistent sidebar narrow and quiet; project names carry more weight than icons.
- Let the central work surface dominate.
- Introduce a contextual inspector only when a task needs it; do not reserve a permanent empty panel.
- Dense screens such as transcript review may tighten spacing, but must preserve the same hierarchy and calmness.
- Use progressive disclosure and sensible defaults. Advanced model/runtime settings never dominate the normal workflow.

## Typography

Barlow is the approved primary application typeface. Use it for navigation, controls, labels, metadata, headings, and other application chrome. Bundle the required font files with the application; never load them from a remote font service at runtime. Define a platform-appropriate system sans-serif fallback for missing or failed assets.

Keep the initial weight set narrow and purposeful. Record the source, licence, version, selected files, and checksums when the font assets are introduced. Exact sizes, weights, line heights, tracking, and text-style tokens are defined in `docs/UX.md` and must be checked for legibility, text scaling, and light/dark rendering.

Phase 0 uses `@fontsource/barlow` 5.3.0 under the SIL Open Font License 1.1. Vite bundles only the Latin WOFF2 files referenced by the stylesheet; there is no runtime font request.

| Asset                           | SHA-256                                                            |
| ------------------------------- | ------------------------------------------------------------------ |
| `barlow-latin-400-normal.woff2` | `b0a8ad37ac45f5fb22ced461576db72e44e295107aad7a9c8a7a4bad728fd03b` |
| `barlow-latin-500-normal.woff2` | `cd759df8ef9efc98fee14307b4eb5ba27f08b1f8f2f3ad2872432e25c89907a8` |
| `barlow-latin-600-normal.woff2` | `4b52ddd4836b592df0e4832b8286956883cdc651b015126bdd18f184b7f90cc3` |

The simple waveform application icon is a Phase 0 scaffold asset derived from the shell's sound mark. It satisfies native build requirements but is not an approved final identity.

No complementary document/editor typeface is selected yet. Add one only when reading and interaction tests demonstrate a real need.

## Colour and surface

Light mode uses warm off-white/cream rather than clinical white. Text is charcoal, secondary text is muted and still accessible. Accent colour is quiet and sparse; bright default blue is not the product identity.

Dark mode is designed, not inverted:

- deep warm charcoal rather than pure black
- subtly distinct sidebar/selection surfaces
- warm off-white primary text
- muted grey secondary text
- restrained borders
- no glow, glass, neon, or “cyber” styling

Accessibility contrast and legibility take precedence over preserving any exploratory colour value.

## Components and states

- Avoid excessive rounded cards, pills, shadows, and filled primary buttons.
- Prefer typographic actions and quiet iconography where affordance remains clear.
- Selection may use a subtle surface change, weight, or fine rule.
- Progress indicators are calm and honest, without “AI magic” animation.
- Local status is informative, not a decorative badge.
- Focus and error states must be unmistakable even in a restrained palette.
- Routine navigation, selection, typing, and editing must remain immediate during fake or real background work.
- Empty, loading, progress, cancellation, failure, retry, interrupted, and recovered states receive the same visual care as the happy path.
- Transcript review and protocol editing require purpose-built interaction design by the vertical slice; the Phase 0 shell may bound their behaviour but must not establish generic placeholder forms as the final pattern.

## Explicitly avoid

- Generic SaaS dashboard composition
- Permanent chat UI
- Bright blue as the default action colour
- Decorative AI imagery, sparkles, gradients, and model branding
- Avatars without a real participant function
- Card grids for document-like content
- Hidden hover-only essential actions
- Generic component-library defaults that override the product's typography, spacing, surface, or interaction character

## Review test

A screen belongs to LocaLog if it feels like a quiet professional writing and project tool, remains legible without colour, and makes the current project, meeting, workflow state, and local-processing status clear.

Review the key screens at representative desktop sizes in light and dark modes. Verify keyboard focus, text scaling, reduced motion, meaningful empty/error/recovery states, and interaction responsiveness alongside visual character. Consistency, calmness, clarity, architectural spacing, and professional finish matter more than fidelity to an earlier study.

When a technical shortcut would materially weaken those qualities, record the trade-off and raise it for review before adopting it.
