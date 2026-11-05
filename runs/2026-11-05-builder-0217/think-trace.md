### Step 1/8: PICK THIS RUN'S ITEMS

**Context:** Night 2026-11-05, 2:17am Builder. Tonight's count 1 of 4, so up to 2 items. main CI green (549d54c). No priority issues. #122 is factory:changes (PR #128 conflicts with main). Ready: #114, #131, #132.

**Thought:** FACTORY §3 step 3.3 puts factory:changes before new ready work, so #122 goes first. A second item is the lowest ready roadmap item (#114, M3-04d) if independent of #122.

**Outcome:** First item: #122 — merge main into feat/122-fillet-chamfer. Second item decided after #122 is fully done.

**Next Action:** Merge main, resolve conflict, re-run gates.
- *Rationale:* §3 ordering; careful mode takes items one at a time.

**Execution Ref:** `issue:#122`

---

### Step 2/8: RESOLVE THE OCCT.RS MERGE CONFLICT

**Context:** git merge origin/main into feat/122-fillet-chamfer: one conflict in crates/swx-kernel/src/occt.rs — the import lines (EdgeOp/validate_edges vs FacePlane/validate_index) and the end of the test module (fillet tests vs tracked-extrude/face tests).

**Thought:** Both sides are pure additions with no overlap in meaning; union of imports and both test blocks is the correct resolution.

**Outcome:** Resolved by union, merge commit 78b7aaf pushed. Local: clippy and test swx-kernel --features occt (109 passed), workspace fmt/clippy/test all exit 0.

**Next Action:** Wait for CI on 78b7aaf, then run think-and-ship gates.
- *Rationale:* No behaviour change; keeps every test from both sides.

**Execution Ref:** `issue:#122`

---

### Step 3/8: QA VERDICT FOR PR #128 R2

**Context:** Head 78b7aaf, CI green (3 checks). 7 ship_check gates verified exit 0. Scratch example: 4 vertical edges found via sub_shape_bounding_box; fillet volume 5896.991118 = expected, 10 faces; face_plane works on 6 planar faces of the fillet result; chamfer 5940, 7 faces; radius -1 and face entry give InvalidInput. Renders checked.

**Thought:** All criteria still met after the merge. Criterion 4 remains met differently (radius 15 builds on the length-10 edges), as before; that stays a known gap.

**Outcome:** Verdict: pass with gaps.

**Next Action:** Post QA report on PR #128, outcome comments, then consider second item.
- *Rationale:* FACTORY §6b: pass requires (e) None; the criterion-4 deviation is a gap.

**Execution Ref:** `issue:#122`

---

### Step 4/8: SECOND ITEM AND APPROACH FOR #114

**Context:** 02:24 PT, tonight 2 of 4 done, run may take a second item. #114 deps #105, #113 closed; independent of #122. Lowest-numbered ready item.

**Thought:** Public BooleanOp/Operand/OpHistory in topo.rs, ffi enum replaced by the public one. New shim swx_occt_boolean_tracked: per input face (a then b, MapShapes order) images = [] if IsDeleted, else Modified faces found in the result's face map, else itself if in the map; sorted, deduped. Output via counts[n_in] and a flat images buffer with capacity; if too small, shim returns 2 with the needed size and Rust retries once. `generated` left empty: v1 tracks faces only and BOP Generated(face) yields section edges, not faces (documented).

**Outcome:** Take #114 on feat/114-tracked-booleans with that design.

**Next Action:** Implement shim, ffi, types, trait method and tests.
- *Rationale:* Smallest FFI surface following the extrude_tracked pattern; no new deps.

**Execution Ref:** `issue:#114`

---

### Step 5/6: QA VERDICT FOR PR #137

**Context:** Head 32cabfd, CI green (3 checks). 5 new tests pass first try. Scratch run: split cut top face → 2 images with the exact boxes; absorbed boss → all B deleted; embedded boss → bottom deleted, side image z 10..15; cut that misses → A faces map 1:1; cut removing all of a → untracked Ok(0 faces), tracked Ok(0 faces, no images). One cargo test gate record failed only due to my sh wrapper; superseded by the exact command.

**Thought:** All criteria met and verified. Gaps: the shim's buffer-too-small retry path is not exercised by a test (cap 2n+16 is never exceeded in these cases); the failed wrapper record in the ship report; generated left empty by design.

**Outcome:** Verdict: pass with gaps.

**Next Action:** Post QA report on #137, outcome comments, export think trace.
- *Rationale:* §6b: (e) not None → pass with gaps.

**Execution Ref:** `issue:#114`

---
