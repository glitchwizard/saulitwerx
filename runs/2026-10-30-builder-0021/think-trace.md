### Step 1/8: BUILDER 2026-10-30 00:21 PT: PICK WORK

**Context:** Nightly count 0 of 2. main CI green (289a0a9). Open ready: #90 M3-01a (size:M, swx-kernel), #91 M3-01b (depends on #90), #92 M3-04a (spec doc, no crate). No changes/priority/wip items.

**Thought:** #90 is the lowest roadmap item. #91 is skipped because #90 is open. #92 is independent of #90 and touches no crate, so it qualifies as a second item, but careful mode says finish the first item fully before starting the second, and §3 forbids starting a new item after 40 minutes.

**Outcome:** Take #90 now; take #92 afterwards only if it can start before 01:01 PT.

**Next Action:** Read the swx-kernel files #90 touches and implement.
- *Rationale:* FACTORY §3 step 3 order and independence rule; careful-mode sequencing.

**Execution Ref:** `issue:#90`

---

### Step 2/8: APPROACH FOR #90

**Context:** swx-kernel: Kernel trait in lib.rs, OcctKernel in occt.rs, unsafe only in ffi.rs, C++ shim occt_shim.cpp. Linked toolkits already include TKTopAlgo/TKPrim (BRepBuilderAPI_*, BRepPrimAPI_MakePrism, BRepCheck_Analyzer).

**Thought:** Put the new types and all input validation in a new non-gated module profile.rs, so it is testable without OCCT. Rust chains each loop (segments may be traversed either way), snaps shared junction points to the midpoint, computes the signed area with Green's theorem, and reorients loops so the outer is counterclockwise about the normal and holes are clockwise. The shim gets flat arrays: plane (9 doubles) and 8 doubles per segment (kind, entry, exit, center, radius); it builds shared vertices, edges (lines, circular arcs on gp_Ax2 with +/- normal), wires, a face on gp_Pln, and rejects it if BRepCheck_Analyzer says invalid. Extrude: shim reads the plane normal from the face, translates by an offset and prisms by a positive length, so negative and symmetric extents never produce inside-out solids.

**Outcome:** Deviation from the suggested OCCT path: shared vertices and BRepBuilderAPI_MakeEdge(gp_Circ, V1, V2) instead of GC_MakeSegment/GC_MakeArcOfCircle, so wires close exactly. Symmetric with a negative length uses its absolute value.

**Next Action:** Write profile.rs with unit tests.
- *Rationale:* Keeps unsafe and OCCT surface minimal and pushes the error cases the issue lists into pure Rust where they can be tested in the default build.

**Execution Ref:** `issue:#90`

---

### Step 3/8: SIZE CHECK FOR #90 (FACTORY §3 STEP 8)

**Context:** Diff: +1103/-2 in swx-kernel only. Tests ~455 lines (profile.rs tests ~265, occt.rs tests ~190); non-test ~650 incl. doc comments and rustfmt expansion. All local gates pass, including the occt CI job commands.

**Thought:** Step 8 says split when an item turns out bigger than size:M. Every acceptance criterion depends on make_face; extrude adds only ~60 lines. A slice without extrude would leave the plate/slot/direction/STEP criteria unmet and need a follow-up of mostly the same tests. Splitting would cost more than it saves.

**Outcome:** Ship as one PR. State the size and the reason in the PR body and outcome comment so the Reviewer can decide.

**Next Action:** Commit, check off criteria, push, open PR.
- *Rationale:* There is no coherent smaller slice that meets the criteria. Flagging the size openly instead of splitting artificially.

**Execution Ref:** `issue:#90`

---

### Step 4/8: QA VERDICT FOR #90 / PR #95

**Context:** CI green on 80f5c3a (3 checks). 7 verified gates. Scratch program: plate with two holes 13595.7523 (expected 13595.7523), L on XZ 3000 with y in [-5, 0], scrambled-direction slot 1514.1593 with 6 faces, and errors for a hole crossing the edge (Backend), a 2e-7 gap (InvalidInput) and Symmetric(0) (InvalidInput). Renders of the plate and slot show the expected shapes.

**Thought:** Every criterion has a named test or scenario. Nothing deferred. The size overrun is a process note, not a QA gap.

**Outcome:** Verdict: pass. Known gaps: None.

**Next Action:** Post QA report on PR #95, check off the QA criterion, post outcome comments.
- *Rationale:* §6b (f): pass requires all criteria met and verified with (e) None.

**Execution Ref:** `issue:#90`

---

### Step 5/10: TAKE SECOND ITEM #92

**Context:** #90 done (PR #95, CI green, QA pass, outcome comments posted) at 00:33 PT, 12 minutes into the run. Nightly count 1 of 2. #92 M3-04a: docs only (docs/specs/M3-04.md), no Depends on, no crate change.

**Thought:** #92 is independent of #90 by the §3 rule (no dependency, no shared crate), and it can start before the 40-minute cutoff. The inputs it needs are FILE_FORMAT.md (for the model.json shape), the swx-core feature/document types (for where TopoRef lives), and the Kernel trait already read for #90. The design: name entities by the generating feature, the sketch-entity id and a role, resolved through OCCT's Generated/Modified history, after Kripac 1997, compared with Capoyleas/Chen/Hoffmann 1996 generic naming and with geometric-signature matching.

**Outcome:** Claim #92 on branch docs/92-topo-naming-spec.

**Next Action:** Read FILE_FORMAT.md and the swx-core feature model, then write the spec.
- *Rationale:* FACTORY §3 step 3 independence rule and time box; nightly cap allows one more.

**Execution Ref:** `issue:#92`

---

### Step 6/10: QA WALK-THROUGH FOR #92 FOUND A GAP

**Context:** QA (c) asks for two paper scenarios: M3-05 fillet and M3-02 to-face. In the to-face scenario, the boss extends to the right half of the top face, split by Cut1. Deleting Cut1 left the fragment's resolution undefined.

**Thought:** Falling back to the whole face would be a silent pick, which goes against the doc's own rule. Strict Missing is consistent and simple.

**Outcome:** Added one rule to Approach: a fragment resolves only while its splitting feature exists, is not suppressed and still splits the face; otherwise Missing. Commit fc7aaf0, pushed; CI re-runs.

**Next Action:** Run §6c gates on fc7aaf0, wait for CI, post QA report.
- *Rationale:* Fix an unmet design case in-run rather than list it as a gap.

**Execution Ref:** `issue:#92`

---

### Step 7/8: QA VERDICT FOR #92 / PR #96

**Context:** Head fc7aaf0, CI green (3 checks), 3 verified gates. The doc has all 9 sections, 3 numeric problem cases, a chosen scheme plus 2 alternatives with public sources, 8 named test scenarios, and slices M3-04b to M3-04g (S/M) with dependencies. The paper walk-throughs (fillet M3-05, to-face M3-02) resolve as designed after the fragment rule was added.

**Thought:** Every criterion is met by the doc's content, which the issue says is verified by reading. QA (c) is the two paper scenarios the issue asks for. Nothing is deferred. The crate-placement proposal is flagged for the owner but is not a gap in this issue's criteria.

**Outcome:** Verdict: pass. Known gaps: None.

**Next Action:** Post QA report on PR #96, check QA box, post outcome comments, export think trace.
- *Rationale:* §6b (f).

**Execution Ref:** `issue:#92`

---
