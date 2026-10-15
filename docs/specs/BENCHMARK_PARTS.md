# Benchmark parts: modeling real parts through the UI

**Goal:** an agent can model three real, catalog-standard parts from an empty part document, **using only the Saulitwerx UI**, with the result checked automatically in CI. Threads are modeled as real helical geometry, not cosmetic.

This proves the tooling end to end: sketcher, features, parameters, the desktop app, and a way for an agent to see and drive that app. The roadmap items for it are milestone **MB** in `docs/ROADMAP.md`.

## 1. Rules

- **From scratch.** Start from **New part**. No importing STEP, STL or any other file of the part. No mesh features.
- **UI only.** The model is built through UI input: clicks, typed values, viewport and sketch drags, keyboard shortcuts. A script that calls the modeling API directly doesn't count, and neither does an in-app macro that does.
- **Parametric.** Every sketch is fully constrained (M2-05 reports it). Every key dimension in §2 is a named parameter, so changing it in the Parameters dialog rebuilds the part.
- **Reference data.** Dimensions come from the part's catalog drawing (§2) and from the published standards it follows. Record dimensions as numbers in our own words. Never commit a supplier's drawing, image, CAD file or text.
- **Clean room.** The supplier name may appear in this spec as a reference source, as prose. It never appears in code, UI, tests or asset names. Parts are named by function and size, like `hex_nut_m8`.

## 2. The three parts

All three are metric and fit together. The screw and nut mate. The collar fits an 8 mm shaft and carries its own small tapped hole.

**Status: provisional.** The values below are the nominal values from the cited standards. The owner confirms them against the chosen catalog part's dimensional drawing and records its part number (roadmap item MB-00). Where the drawing differs from the standard, the drawing wins.

### BP-1 Hex nut, M8 × 1.25

| | |
|---|---|
| Catalog part number | _to be confirmed by owner (a candidate: a DIN 934 / ISO 4032 class 8 M8 nut)_ |
| Thread | M8 × 1.25, right-hand, internal, tolerance class 6H, modeled full height |
| Width across flats `s` | 13 mm |
| Height `m` | 6.5 mm (DIN 934) or 6.8 mm max (ISO 4032), per the drawing |
| Width across corners `e` | ≥ 14.38 mm (follows from `s` for a regular hexagon: 13 / cos 30° = 15.01 mm before chamfer) |
| Chamfer | 30° conical chamfer on both faces, bearing-face diameter `dw` ≥ 11.63 mm |
| Thread countersink | both ends, diameter 8.0 to 8.64 mm, 90° or 120° per the drawing |

### BP-2 Hex head screw, M8 × 1.25 × 40, fully threaded

| | |
|---|---|
| Catalog part number | _to be confirmed by owner (ISO 4017 / DIN 933 class 8.8)_ |
| Thread | M8 × 1.25, right-hand, external, tolerance class 6g, modeled up to the head |
| Length under head `l` | 40 mm |
| Unthreaded length under head `a` | ≤ 3 P = 3.75 mm |
| Head width across flats `s` | 13 mm |
| Head height `k` | 5.3 mm |
| Head chamfer | 30° conical chamfer on the top face, washer face `dw` ≥ 11.63 mm |
| Point | chamfered end, 45° to the thread minor diameter |

### BP-3 One-piece clamp-on shaft collar, 8 mm bore

| | |
|---|---|
| Catalog part number | _to be confirmed by owner (metric one-piece clamp collar, 8 mm bore)_ |
| Bore | 8 mm |
| Outside diameter | 18 mm |
| Width | 9 mm |
| Slit | one radial slit through one wall, width per the drawing (about 1 mm) |
| Clamp screw | M3 × 0.5 across the slit: a counterbored clearance hole on one side, a tapped M3 hole (6H, modeled) on the other |
| Edge breaks | chamfers on the bore and outside edges, per the drawing |

## 3. Thread geometry (all three parts)

Modeled threads use the ISO 68-1 basic profile, a 60° symmetric thread. Source: ISO 68-1 and ISO 724 basic dimensions, and ISO 965-1 tolerance positions, as tabulated in standard references such as *Machinery's Handbook*.

| Quantity | Formula | M8 × 1.25 | M3 × 0.5 |
|---|---|---|---|
| Fundamental triangle height `H` | 0.866025 P | 1.0825 | 0.4330 |
| Pitch diameter `d2 = D2` | d − 0.649519 P | 7.188 | 2.675 |
| Internal minor diameter `D1` | D − 1.082532 P | 6.647 | 2.459 |
| External minor diameter `d3` | d − 1.226869 P | 6.466 | 2.387 |
| External fundamental deviation, 6g `es` | ISO 965-1 | −0.028 | −0.020 |
| Internal fundamental deviation, 6H `EI` | ISO 965-1 | 0 | 0 |

External threads are modeled at the basic profile shifted by `es`. Internal threads are modeled at the basic profile (`EI` = 0). This leaves real clearance between mating threads, so booleans between them aren't degenerate.

## 4. How "through the UI" is checked

1. **UI test harness (MB-01).** The real `swx-app` runs headless in a test. The test sends only UI input, addressing widgets by their accessible label, and can take screenshots.
2. **Agent driver (MB-02).** The same harness accepts one UI action at a time over stdin and replies with what's on screen: a summary of the accessibility tree and a screenshot path. An agent explores the UI this way. The driver records the session as a replayable UI script, `benchmarks/<part>/build.ui.json`.
3. **Replay in CI (MB-04).** CI replays each script against the current app, saves the `.sltwx`, exports STL, and uploads the screenshots as artifacts. A script that stops working is a red build, so later UI changes can't silently break the benchmark.

## 5. Acceptance checks per part

Each part's test replays its UI script, then checks:

1. **Matches an oracle.** A reference model of the same part, built in the test through the document API (not the UI), has the same volume within 0.1 % and the same bounding box within 0.01 mm. The oracle is a check on the UI path, never a substitute for it.
2. **Matches the drawing.** The bounding box matches the §2 dimensions within 0.05 mm. Every §2 key dimension exists as a named parameter with the drawing's value.
3. **Fully constrained.** Every sketch reports fully constrained.
4. **Parametric edit.** Through the UI, the test changes one key dimension in the Parameters dialog (the nut's height, the screw's length, the collar's width), and the part rebuilds without errors with the expected new bounding box.
5. **Threads are real.** For BP-1 and BP-2 together: the nut placed on the screw at the matching thread phase intersects it with a volume under 0.1 % of the nut's volume. The same nut shifted axially by P/2 intersects it with a volume over 1 % of the nut's volume. This fails for cosmetic threads.

## 6. Stretch goal: three random objects from Thingiverse

After BP-1 to BP-3 pass, repeat the exercise with three objects picked at random from Thingiverse. This is milestone item MB-08, and it is an owner step until its table is filled in.

**Selection procedure (reproducible):**

1. Draw thing IDs in order from `random.Random(20261015).randint(1, 7_300_000)` in Python.
2. Skip an ID if it doesn't resolve to a public thing, or if the thing fails any eligibility rule below. Record each skipped ID and the reason.
3. Take the first three eligible things.

**Eligibility:**

- License CC0, public domain, or CC BY. Skip NonCommercial, NoDerivatives and ShareAlike licenses, since the recreated model lives in an MIT/Apache repository.
- A single solid part that a feature-based modeler can make: prismatic, turned, swept or lofted shapes. Skip sculpted, scanned or organic shapes, lattices, and parts whose main feature is text or a logo.
- No brand names or replicas of branded products (CLEAN_ROOM.md).
- Roughly 30 features or fewer.

**For each pick, record:** thing ID, title, designer, license, URL, the bounding box, volume and genus measured from the published STL, and a design brief with the key dimensions. Don't commit the STL. The brief carries everything the factory needs, so factory runs never need access to Thingiverse.

| Pick | Thing | Designer | License | Brief |
|---|---|---|---|---|
| T-1 | _not yet selected_ | | | |
| T-2 | _not yet selected_ | | | |
| T-3 | _not yet selected_ | | | |

The same checks as §5 apply, except the oracle comparison is replaced by the measured bounding box (within 0.5 mm per axis), volume (within 2 %) and genus (equal).
