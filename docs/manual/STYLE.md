# Manual style guide

The rules that keep the Saulitwerx user manual reading as one book, however many nights wrote it. FACTORY §6d sets the requirements; this file sets the conventions. The template (`template.typ`) implements every visual rule here, so chapters use its helpers instead of styling text by hand.

## Reader and voice

- The reader is a capable adult who has never used CAD. Developers are a second audience, served in sections marked `#for-developers`.
- Plain words, short sentences, active voice, second person ("you"). One task per section.
- Explain each idea as: what it is, why it matters, a small example, then how to do it.
- Say what exists. Never describe an unbuilt feature as working; plans go only in **Coming next**, and in "In Saulitwerx today:" lines that say plainly what is missing.
- No vendor or product names, and no imitation of another vendor's manual (`CLEAN_ROOM.md`). Use the generic terms in `docs/specs/PRODUCT.md`.

## Terms and capitalization

- Saulitwerx (never SaulitWerx). OpenCascade on first use in a chapter, then "the geometry engine" or "the kernel". STEP in capitals.
- Feature and constraint names are lowercase in running text (extrude, coincident) and capitalized in table cells.
- Units: "mm", with a space after the number (40 mm). Areas mm², volumes mm³. Use × for sizes (40 × 25 mm).
- Define a term with `#term[...]` where it first appears; this italicizes and indexes it. Add it to the glossary in `chapters/11-back.typ`. Index other key spots with `#idx("...")`.

## Structure (outline)

1. Front matter: cover (commit, build date, status), *How to use this manual*, contents.
2. Part I Getting started: 1 Welcome, 2 Installing, 3 Your first part (the ~15-minute tutorial).
3. Part II Concepts: 4 How parametric CAD works (solids, units, sketches, constraints, dimensions, degrees of freedom, parameters, features, history).
4. Part III Tasks: 5 Sketching, 6 Solids and STEP files, 7 Extruding a profile (`chapters/06a-extrude.typ`), 8 Revolving a profile (`chapters/06b-revolve.typ`), 9 Pictures of parts and sketches, 10 Parameters and history. Later: part features, files, assemblies, drawings.
5. Part IV Reference: 11 Reference (commands, build options, kernel operations, sketch entities and constraints, solver, degree-of-freedom report, formats and units, keyboard and mouse).
6. Part V Help: 12 Help (troubleshooting by symptom, every error message, questions, known limitations).
7. Back matter: glossary, What's new (newest first, dated by night), Coming next, quick-reference card, credits and licenses, index.

When the desktop app arrives (M5), Part I's tutorial moves to the app and the command-line tutorial moves into Part III for developers.

## Tasks

Each task section has, in order: a one-paragraph purpose, `#task-intro(goal:, needs:, time:)`, `#steps(...)`, `#result[...]`, then tips and common mistakes. Each step starts with a verb and says what you should see. Commands go in `#term-block("...")` (a string, so spacing is exact), program output in `#term-block(label: "Output", "...")`, copied from a real run in this edition.

## Cross-references

Label chapters `<ch-…>`, sections `<sec-…>`, figures `<fig-…>`. `@label` prints "Chapter N", "Section N.M" or "Figure N.M".

## Visual conventions

- Page: US Letter, mirrored margins. Body Libertinus Serif 10.5 pt; headings and labels Inter; code DejaVu Sans Mono.
- Colour: steel blue `#3d5a80` (accent, matches rendered parts); green for tips and results; rust for cautions; purple for developer sections. Callouts never break across pages.
- Pictures: tool-made only (`swx-qa render` for parts, `swx-qa sketch` or `swx_qa::sketch::plot` for sketches; app screenshots from M5). PNG in `img/`, under 1 MB, at least 200 dpi at printed size (a full-width figure is about 6.5 in, so at least 1300 px wide). The command that made each image sits in a comment next to its use. Annotations (operator signs, markers, labels) are drawn by the template, never painted into the image. Every figure has a caption and alt text. Sketch pictures are 600x420 px, shown at 40 % width alone or two across, so their built-in labels stay legible in print.
- Picture rows: `#image-equation` for "this and that give this" (operator signs between images); `#image-row` for one start and several outcomes side by side, each with its own label and alt text.
- Task intros (`#task-intro`) never break across pages.
- Tables use `#ref-table`; status uses `#now` / `#later` pills.
