### Step 1/6: PICK TONIGHT'S ITEMS

**Context:** Night 2026-11-01, 0 builder items so far, cap 4, run may take 2. Main CI green; no priority, wip or changes issues. Ready: #97 M3-02a, #104 M3-03a, #105 M3-04b, all swx-kernel.

**Thought:** #97 is the lowest roadmap item and its dependency #91 is closed. #104 is next in roadmap order and has no Depends-on line, so it is independent of #97. Careful mode: finish #97 fully before deciding on #104 within the 40-minute start window.

**Outcome:** Took #97 first; #104 is the candidate second item.

**Next Action:** Implement #97 in swx-kernel feature.rs/profile.rs.
- *Rationale:* FACTORY §3 step 3.4 order and independence rule.

**Execution Ref:** `issue:#97`

---

### Step 2/6: APPROACH FOR #97

**Context:** extrude_feature gets a face shape, not a workplane; OcctKernel::extrude sweeps along the face's own normal, so Distance(±L)/Symmetric(2L) express the three directions.

**Thought:** Resolve ThroughAll inside extrude_feature from face/target bounding boxes with the issue's L. Kernel::extrude rejects ThroughAll via offset_and_length. Mode check and extent arithmetic are pure functions so they test without OCCT. Missed cut: compare result and target volumes (relative 1e-9) only for through-all, reported as InvalidInput because the issue does not name a variant and EmptyResult means no solid remains.

**Outcome:** Implemented in 6a1d0b4; all local gates and CI green on first attempt.

**Next Action:** Record ship gates and QA.
- *Rationale:* Smallest change meeting the criteria; no trait or FFI changes.

**Execution Ref:** `issue:#97`

---

### Step 3/6: QA VERDICT FOR #97

**Context:** CI green 3/3 on 6a1d0b4; 7 server-run gates passed; one earlier ship_check failed only in its sh wrapper (PIPESTATUS). Scratch scenarios matched expected volumes; two renders show the holes.

**Thought:** All criteria met and verified. The failed wrapper entry stays in the ship report, so the verdict is pass with gaps and it is disclosed in (b) and (e). ship_start/ship_complete returned output-schema errors but ship_status/export show the task started and completed.

**Outcome:** QA report posted on PR #111, verdict pass with gaps.

**Next Action:** Post #97 outcome comments, then decide on #104 within the time box.
- *Rationale:* FACTORY §6b: pass only when (e) is None.

**Execution Ref:** `issue:#97`

---

### Step 4/6: SECOND ITEM AND APPROACH FOR #104

**Context:** #97 completed at 00:25 PT, inside the 40-minute start window; #104 has no dependency on #97.

**Thought:** Took #104 as the second item. New noexcept shim function wrapping BRepPrimAPI_MakeRevol (TKPrim already linked), a documented unsafe call in ffi, pure-Rust prepare_revolve for validation (clamps 2π+1e-12 to 2π), and Kernel::revolve. To avoid copying extrude_feature, factored feature_target and combine; extrude messages kept identical so its tests are untouched. Both PRs edit feature.rs, so the second to merge may need main merged in.

**Outcome:** Implemented in 9e47ddd; local gates passed first try; PR #112 opened.

**Next Action:** Wait for CI, record gates, QA.
- *Rationale:* FACTORY §3 step 3 allows two independent items per run; smallest change meeting criteria.

**Execution Ref:** `issue:#104`

---

### Step 5/5: QA VERDICT FOR #104 AND RUN CLOSE

**Context:** CI green 3/3 on 9e47ddd; 7 server-run gates verified and passed; scenarios matched (incl. reversed unnormalized axis); two renders show the quarter ring and the ring groove.

**Thought:** Every criterion is met and verified, no gaps, so the verdict is pass. The run took two items (#97, #104), the per-run maximum; tonight's count is 2 of 4. #105 remains for the 2:17am run.

**Outcome:** QA report posted on PR #112, verdict pass. Run ends after outcome comments and trace export.

**Next Action:** Post #104 outcome comments, export think trace to qa-artifacts.
- *Rationale:* FACTORY §6b verdict rule; §3 two items per run.

**Execution Ref:** `issue:#104`

---
