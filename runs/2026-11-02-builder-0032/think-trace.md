### Step 1/8: PICK TONIGHT'S BUILDER ITEMS

**Context:** Night 2026-11-02, run start 00:32 PT. Tonight's count 0 of 4, so up to 2 items. main CI green, no priority issues. #105 is factory:changes (PR #116 conflicts with main after #112). Ready: #113 (dep #105), #114 (dep #105, #113), #117 (no deps).

**Thought:** §3 order: changes before ready. #105 first. #117 does not depend on #105, directly or indirectly, so it is independent; take it second, one at a time.

**Outcome:** Items: #105 (merge main into PR #116), then #117 (M3-03b).

**Next Action:** Merge main into feat/105-sub-shapes, resolve occt.rs import conflict, push, wait for CI.
- *Rationale:* FACTORY §3 steps 3.3 and 3.4; independence rule in §3 step 3.

**Execution Ref:** `issue:#105`

---

### Step 2/8: RESOLVE PR #116 SEND-BACK

**Context:** git merge origin/main into feat/105-sub-shapes: only conflict is the import block at the top of crates/swx-kernel/src/occt.rs (main added Axis and prepare_revolve; branch added SubShape, TopoKind, validate_adjacent). ffi.rs, lib.rs, shim auto-merged.

**Thought:** Union of both import lists is the correct resolution; no logic overlaps. Verified with swx-kernel occt clippy (clean) and tests (73 passed), plus workspace fmt/clippy/test (170 passed).

**Outcome:** Merge commit 8723c52 pushed; issue labeled factory:review.

**Next Action:** Wait for CI on 8723c52, then record gates.
- *Rationale:* §3 step 3.3: merge main, never rebase.

**Execution Ref:** `issue:#105`

---

### Step 3/8: QA VERDICT FOR #105 / PR #116

**Context:** CI green (3 checks) on 8723c52. 7 ship_check gates verified and passed. Scratch check: plate 7/15/10, edge 99 -> InvalidInput.

**Thought:** All criteria met. Gaps are record-keeping only: export omits command strings, and ship_start/ship_complete returned output-schema errors though the state is correct.

**Outcome:** QA pass with gaps; report posted; ship-report-r2.json on qa-artifacts (6d6fdae).

**Next Action:** Post outcome comments for #105, then start #117.
- *Rationale:* §6b verdict rules: pass only when (e) is None.

**Execution Ref:** `issue:#105`

---

### Step 4/8: APPROACH FOR #117 REVOLVE INPUT CHECKS

**Context:** Issue allows a check in the shim or a small shim query with the check in Rust. The error messages must name the axis and the plane.

**Thought:** Chose one noexcept query (swx_occt_revolve_profile) that returns the plane point and normal plus the min/max of (P-O)·(dir×n). The extremes are exact for lines (end points) and circles (harmonic addition); other curves are sampled. The tolerance logic lives in Rust (check_revolve_profile), so it is unit-testable without OCCT and can build messages with the axis and plane. A non-planar face also maps to InvalidInput. I added a circle-crossing test that only the arc-extreme math catches.

**Outcome:** Commit 4b58b12: 7 new tests (6 occt, 1 pure); swx-kernel occt 75 passed; workspace 169 passed. 334 changed lines (above S, within M).

**Next Action:** Open PR, label factory:review, wait for CI.
- *Rationale:* Keeps unsafe and C++ minimal; puts the decision logic where it can be tested and documented.

**Execution Ref:** `issue:#117`

---

### Step 5/5: QA VERDICT FOR #117 / PR #120

**Context:** CI green (3 checks) on 4b58b12. 8 ship_check records, all verified and passed. Scratch: 500π, bbox ±10/0..5, 3 faces; crossing and tilted-axis errors as specified. Render shows a solid cylinder.

**Thought:** All 9 criteria are met. Failed attempt: one gate command piped cargo test into grep, so the record's exit code was grep's. I re-ran the plain command, and it is disclosed in (e). I also corrected a (d) caption that wrongly implied touching profiles were newly allowed.

**Outcome:** QA pass with gaps; report posted; pr-120 artifacts on qa-artifacts (6957109). Run done: 2 items, tonight 2 of 4.

**Next Action:** Post outcome comments, export think trace to qa-artifacts.
- *Rationale:* §6b: pass only with no gaps; record-keeping gaps exist.

**Execution Ref:** `issue:#117`

---
