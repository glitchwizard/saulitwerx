# Product spec: what Saulitwerx should feel like

Saulitwerx is a desktop, history-based parametric mechanical CAD system (parts, assemblies, drawings). It is **not** a renderer. Photoreal rendering is a late, optional module.

**Reference behavior:** where mainstream MCAD tools differ, follow the workflow model of **Autodesk Inventor**. That is the owner's explicit choice. Match how things *work*; never copy names, icons, artwork, help text, or a pixel-for-pixel layout (see `CLEAN_ROOM.md`). In code and UI, use only the generic terms in the table at the bottom of this file.

**Behavior lookups:** when a spec here is ambiguous, agents may read the vendor's public online help (https://help.autodesk.com/view/INVNTOR/2027/ENU/) to understand how a command behaves: its options, extents, and edge cases. Rules for that: paraphrase only. Never copy help text, screenshots, icons, or examples into the repo, and never reference the vendor in code, UI, tests, or commit messages. Write down what you learned as behavior in an issue or in `docs/specs/`, in our own words.

Every agent run reads this file. Each issue's acceptance criteria should trace back to a section here.

## Hard limits (tell users plainly)

- No reading or writing other vendors' native files (`.ipt`, `.iam`, `.idw`, `.sldprt`, …). Migration goes through **STEP**, with DXF for 2D.
- Behavior can be familiar; branding can't.

## 1. Document types

Separate files, one extension per kind. The format is specified in `docs/specs/FILE_FORMAT.md`: a documented ZIP container with JSON inside, plus cached geometry in open formats.

| Kind | Extension | Contains |
|---|---|---|
| Part | `.sltwx` | One part: parameters, sketches, features, one or more solid bodies |
| Assembly | `.sltwxa` | References to parts/subassemblies + joints/constraints + BOM data + **its exploded views** |
| Drawing | `.sltwxd` | 2D sheets with views of parts/assemblies, dimensions, parts lists |

Exploded views live **inside the assembly file**, not in a separate document (a deliberate difference from the reference tool's separate exploded-view file). They can't get separated from their assembly, and there's one file to manage.

## 2. Part modeling workflow

1. **New part** starts with an origin: 3 planes, 3 axes, a center point. The first sketch goes on an origin plane; the origin is projected into the sketch automatically.
2. **Every dimension is a parameter.** Each sketch or feature dimension creates a *model parameter* auto-named `d0`, `d1`, … in creation order. Users can rename it and type an expression with units (`d3 = d1 / 2 + 5 mm`). *User parameters* are created separately. One **Parameters dialog** lists both, with name, unit, expression, value, and comment.
3. **Sketch → feature consumption:** a feature consumes its sketch and the sketch hides. A sketch can be *shared* so several features use it.
4. **Model browser** (feature tree): origin folder → features in order → an **end-of-part marker**. Dragging the marker up rolls the model back; new features insert above it.
5. **Work features:** work planes, axes, and points, with many construction methods (offset, angle, 3 points, through edge, tangent, midplane, …).
6. **Sketched features:** extrude (distance, to face, through all, between; join / cut / intersect / new solid; symmetric and asymmetric), revolve, sweep, loft, coil, rib, emboss.
7. **Placed features:** fillet (constant, variable, face, full-round), chamfer (distance, distance-angle, two distances), hole (simple, counterbore, countersink, tapped, with a thread table), shell, face draft, thread, split, combine.
8. **Patterns:** rectangular, circular, along a path, and mirror, for features or bodies.
9. **Multi-body parts:** extrude and similar features can create a new solid; combine handles join/cut/intersect between bodies.
10. **Units** are set per document. The default is millimeters.

## 3. Sketching

- Line tool with inline tangent arcs (drag from an endpoint), plus circle, arc (3-point, center, tangent), rectangle (2-point, 3-point, center), slot, polygon, spline, sketch fillet/chamfer, trim/extend/split/offset, project geometry/edges, and construction/centerline styles.
- **Constraints are inferred while drawing** (coincident, horizontal, vertical, tangent, perpendicular, parallel), with glyphs that can be shown and hidden.
- Live DOF feedback: geometry changes color when fully constrained, and the status bar shows "N dimensions needed".
- Dimensions are placed by clicking geometry and edited inline. Driven (reference) dimensions are supported.

## 4. Assemblies (joint-first)

- The first placed component is **grounded**.
- **Joints are the primary tool**, because they define motion: rigid, rotational, slider, cylindrical, planar, and ball, with optional limits.
- **Constraints are the secondary tool:** mate (including the flush variant), angle, tangent, insert, and symmetry. Motion constraints (rotation–rotation, rotation–translation) come later.
- Dragging a component shows its remaining degrees of freedom. Interference checking is included.
- Component patterns and mirrors; subassemblies, which can be flexible; replace component; edit a part in place within the assembly.
- Bill-of-materials structure (normal, phantom, reference, purchased).

## 5. Drawings

- Base, projected, auxiliary, section, detail, and break-out views from parts, assemblies, or exploded views.
- Retrieve model dimensions, add drawing dimensions, center marks and centerlines, and annotations (surface finish, GD&T frames, notes).
- Parts list and balloons driven by the BOM. Sheet formats and title blocks come from templates.
- Export PDF and DXF.

## 6. Exploded views (inside the assembly)

An assembly can hold any number of named exploded views. Component *tweaks* (translate or rotate along an axis), with trail lines. Ordered steps can be played as an animation. Drawings can use an exploded view.

## 7. Later modules (in rough priority order)

1. **Parameters automation:** rules written in a sandboxed embedded scripting language (permissively licensed, e.g. Rhai). Rules can read and write parameters, suppress features, and run on parameter change.
2. **Part families:** table-driven variants (rows of parameter values become family members).
3. **Sheet metal:** face, contour flange, flange, hem, bend, and flat pattern.
4. **Structural frames:** profile members placed on skeleton sketches, with miters and notches.
5. **Standard parts library:** fasteners, washers, nuts and similar, generated from published standard dimension tables.
6. **Design calculators:** gears, shafts, bolted connections.
7. Simulation (stress, dynamics), photoreal rendering, and a browser (WASM) build.

## 8. UI conventions

- **Ribbon** with one tab set per environment (part, sketch, assembly, drawing). Exploded-view editing is a mode inside the assembly environment. Sketch mode shows its own contextual tab and a *Finish sketch* button.
- **Model browser** on the left. Feature dialogs are **property panels** with live preview and OK / Apply / Cancel.
- **Right-click radial menu**, with frequent commands placed around the cursor, plus a mini-toolbar near the selection.
- **Orientation cube** top-right with a home view; view and zoom controls at the screen edge.
- **Mouse navigation default:** wheel zooms toward the cursor, middle-drag pans, Shift + middle-drag orbits, and there is an orbit key. Users can switch to other presets or remap.
- **Single-letter command aliases** (e.g. line, circle, dimension, extrude, revolve, hole), all user-remappable.

## Generic terms (use these in code and UI)

| Use | Don't use vendor-branded equivalents such as |
|---|---|
| orientation cube | ViewCube |
| rules / parameters automation | iLogic |
| part family / table-driven part | iPart, iAssembly, configurations |
| standard parts library | Content Center, Toolbox |
| structural frame | Frame Generator, Weldments |
| design calculator | Design Accelerator |
| exploded view (stored in the assembly) | presentation / .ipn |
| radial menu | marking menu |
| property panel | PropertyManager |
| model browser / feature tree | FeatureManager |
