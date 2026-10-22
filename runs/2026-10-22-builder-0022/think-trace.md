### Step 1/6: PICK WORK FOR BUILDER RUN 2026-10-22 00:22 PT

**Context:** main CI green (run 76, c4554e0). Nightly count 0/2. No priority or factory:changes issues. Ready with deps closed: #25 M2-03, #49 M2-05; #28 waits on #25.

**Thought:** Lowest roadmap item is #25. #49 also qualifies but both change crate swx-sketch, so §3 says they are not independent.

**Outcome:** Take #25 only.

**Next Action:** Read swx-sketch geometry and solver API, implement constraints.
- *Rationale:* FACTORY §3 step 3: second item only if independent and different crate.

**Execution Ref:** `issue:#25`

---

### Step 2/6: APPROACH FOR #25

**Context:** swx-sketch has Sketch (flat params + slots) and solver::Residuals. Rectangle test needs line endpoints as points.

**Thought:** Added PointRef (point/line start/end/center) so coincident works on line ends. Each constraint resolves at add time into rows over parameter indices (Equal, Value, Distance); Sketch implements Residuals with an analytic Jacobian. Added HorizontalPoints/VerticalPoints variants for the two-point form. One clippy failure (type_complexity) fixed with a type alias.

**Outcome:** Commit 4845689, PR #51. Local gates exit 0, swx-sketch 30 passed.

**Next Action:** Wait for CI, record ship gates, run QA scenarios.
- *Rationale:* Resolving once keeps residual evaluation simple and makes invalid references an error at add time, never a panic in the solver.

**Execution Ref:** `issue:#25`

---

### Step 3/3: QA VERDICT FOR #25 / PR #51

**Context:** CI 3/3 green on 4845689. Ship gates fmt/clippy/test verified and passed. Scratch scenarios: 3-4-5 triangle converged (|AC|=5), conflicting Fixed stagnates without panic, invalid kinds/NaN rejected. Distance from exactly coincident start stagnates (zero gradient).

**Thought:** Every criterion is met and tested. Gaps: no visual (swx-qa sketch, QA-02, doesn't exist yet); distance from exactly coincident points can't pick a direction. Neither is an acceptance criterion.

**Outcome:** Verdict: pass with gaps.

**Next Action:** Publish ship report and think trace to qa-artifacts, post QA report and outcome comments.
- *Rationale:* FACTORY §6b: gaps in (e) mean pass with gaps.

**Execution Ref:** `issue:#25`

---
