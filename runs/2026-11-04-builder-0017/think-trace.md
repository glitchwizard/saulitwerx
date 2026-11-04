### Step 1/6: SELECT WORK FOR BUILDER RUN 2026-11-04 00:17 PT

**Context:** Night 2026-11-04 has 0 builder outcomes (cap 4). main CI green. Ready: #121, #114. #114 depends on #113 (open, in review). No priority or factory:changes issues.

**Thought:** Only #121 qualifies. Owner restored factory:ready after the 11-03 stall. No branch exists for #121, so start fresh from origin/main.

**Outcome:** Take #121 M3-02b only; one item this run.

**Next Action:** Read swx-kernel extrude code and plan the change.
- *Rationale:* FACTORY §3 step 3: skip issues whose Depends on: is open; second item only if independent and qualifying.

**Execution Ref:** `issue:#121`

---

### Step 2/6: APPROACH FOR #121

**Context:** Kernel::extrude only takes an Extent; offset_and_length gives (offset, length) and the shim already supports an arbitrary offset. extrude_feature is generic over Kernel, so it can only use trait methods.

**Thought:** Shim: swx_occt_face_plane(s, index, out[6]) returns plane point and outward normal (reversed when the face is REVERSED), 1 when not planar. Rust validates kind and index first. In extrude_feature, get the workplane from face_plane(profile, Face 0) — same normal convention as swx_occt_extrude — then the target face plane, check parallel (|n0 x m| <= sin 1e-9), signed distance d = (p - p0).n0. ToFace -> Distance(d). BetweenFaces needs a start offset: add Extent::FromTo(start, end), accepted by Kernel::extrude, offset=min, length=|end-start|. Alternative (two prisms + boolean) adds seams and cost.

**Outcome:** Implement face_plane, ToFace, BetweenFaces, plus a small FromTo(start, end) variant; pure helper resolve_face_extent tested without OCCT.

**Next Action:** Write shim function and ffi wrapper.
- *Rationale:* The previous stalled run flagged the missing start offset; a resolved finite extent mirrors how ThroughAll is handled and keeps Kernel::extrude the single sweep path.

**Execution Ref:** `issue:#121`

---

### Step 3/4: QA VERDICT FOR PR #130 (#121)

**Context:** CI green on 45854e1 (3 checks). Server-run gates all exit 0; one extra failed record from a sh-incompatible wrapper (PIPESTATUS) before the plain re-run passed. Scratch program: BetweenFaces cut 18000, ToFace join 22000, side face and Edge errors InvalidInput. Renders show the pocket and the post.

**Thought:** All criteria met and verified by tests and scenarios. Gaps: the FromTo variant is a deviation the Reviewer should accept or reject; the ship report holds one failed record that the re-run supersedes. Pocket depth isn't visible in the iso render, only the volume shows it.

**Outcome:** Verdict: pass with gaps.

**Next Action:** Publish images and ship report to qa-artifacts, post QA report and outcome comments.
- *Rationale:* FACTORY §6b: pass only when (e) is None; the superseded failed gate record and the API deviation are worth listing.

**Execution Ref:** `issue:#121`

---

### Step 4/4: CLOSE BUILDER RUN 2026-11-04 00:17 PT

**Context:** #121 done through PR #130. #114 still depends on open #113; nothing else ready.

**Thought:** No second independent item qualifies, so the run ends after #121 (1 of 4 tonight).

**Outcome:** Run ends with one completed item.

**Next Action:** Export trace, publish qa-artifacts, post comments.
- *Rationale:* FACTORY §3 step 3: a second item only when it qualifies and is independent.

**Execution Ref:** `issue:#121`

---
