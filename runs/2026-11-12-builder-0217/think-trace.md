### Step 1/10: PICK THIS RUN'S ITEMS (BUILDER 2026-11-12 0217)

**Context:** Night 2026-11-12 has 2 builder outcomes (#200, #201); cap 4, so up to 2. Main CI green (d6e0734). No wip/changes/priority. Ready: #190 MB-03a, #194 MB-03b, #199 M4-01c, #202 M4-01d (depends on #199).

**Thought:** Roadmap order puts M4-01 (line 56) before MB-03 (line 88). Lowest is #199. #202 depends on #199, so skip. Next is #190 MB-03a (swx-kernel), independent of #199 (swx-io), different crate.

**Outcome:** Take #199 first, then #190.

**Next Action:** Claim #199 and implement serde derive in swx-io.
- *Rationale:* FACTORY §3 step 3.4 roadmap order; second item independent and in a different crate as §3 prefers.

**Execution Ref:** `issue:#199`

---

### Step 2/10: #199 APPROACH AND QA VERDICT

**Context:** PR #204, head 03eb55e, CI green (3 checks). Gates fmt/clippy/test verified by server. ship_start/ship_complete returned output-schema errors but export shows the task completed with 3 verified checks.

**Thought:** Derived Serialize only; read_manifest kept by hand because derived Deserialize cannot keep the exact FormatError field paths, so it is not simpler. Golden round-trip diff empty; kind=sheet error and extra-field output identical on main and head. Known edge: a hand-built Manifest whose extra map repeats a known key would now let extra win (flatten serializes after fields); before, known fields won. read_manifest never produces that.

**Outcome:** QA verdict: pass with gaps (the flatten key-collision edge for hand-built values is listed in (e)).

**Next Action:** Post QA report on #204, outcome comments, then take #190.
- *Rationale:* All criteria met and verified; one behavioral edge outside the reader path is disclosed rather than hidden.

**Execution Ref:** `issue:#199`

---

### Step 3/10: #190 APPROACH

**Context:** Kernel has revolve (with ffi::revolve_profile side extents) and sweep (ffi::sweep_profile: plane + wire count). OCCT libs needed already linked.

**Thought:** Pure checks in a new helix.rs (prepare_helix, check_helix_profile) with kernel-free tests. OcctKernel: sweep_profile for single face / hole; revolve_profile(origin, dir) for side extents and plane; revolve_profile(origin, n x dir) gives axial extents since (n x d) x n = d. One new shim swx_occt_helical_sweep: helix as Geom2d_Line on Geom_CylindricalSurface (Ax3 origin, dir, radial X) + BRepLib::BuildCurves3d, BRepOffsetAPI_MakePipeShell with SetMode(gp_Dir dir), outer wire, MakeSolid, BRepCheck_Analyzer + exactly one solid.

**Outcome:** One new shim function + ffi wrapper; checks reuse existing shims.

**Next Action:** Write helix.rs, trait method, OcctKernel impl, shim, tests.
- *Rationale:* Smallest change: reuses the revolve/sweep data shims; binormal mode keeps the profile in the rotating meridian plane (screw motion), as the issue specifies.

**Execution Ref:** `issue:#190`

---

### Step 4/6: #190 QA VERDICT

**Context:** PR #205, head 4e5a7bc, CI green (3 checks). 8 ship records: one cargo test record failed on sh syntax (PIPESTATUS) and was re-run plain, passed. Scratch program: volumes within ~1.6e-6 relative; renders show 3-turn and 2-turn coils.

**Thought:** All criteria met and verified by tests plus scenarios. No deviation from the issue; size ~560 lines of which ~330 tests. Nothing deferred.

**Outcome:** QA verdict: pass.

**Next Action:** Post QA report on #205, outcome comments, export think trace.
- *Rationale:* Every criterion has a test and an independent hands-on check; known gaps none.

**Execution Ref:** `issue:#190`

---
