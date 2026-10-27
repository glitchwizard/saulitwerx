### Step 1/5: PICK TONIGHT'S BUILDER ITEMS

**Context:** Night 2026-10-27, 00:19 PT run; 0 builder outcome comments tonight; cap 2. No priority, no factory:changes. Ready: #71 M2-07b, #73 M2-09, #79 M2-08b (depends on open #72).

**Thought:** #79 skipped (dep #72 open). Lowest roadmap ready is #71 (M2-07b). Second candidate #73 changes swx-sketch, and #71 also changes swx-sketch (new params module + set_dimension_value), so they share a crate and are not independent under §3.

**Outcome:** Take only #71 this run.

**Next Action:** Claim #71, branch feat/71-dimension-params, implement.
- *Rationale:* FACTORY §3 step 3: second item only if independent and no shared crate.

**Execution Ref:** `issue:#71`

---

### Step 2/3: APPROACH FOR #71

**Context:** Sketch owns constraints + resolved rows + row_owners. swx-core Document has add_model_param/param/set_param/rename_param.

**Thought:** Store bindings as Vec<(ConstraintId, String)> on Sketch (private field, accessible from child module params). set_dimension_value re-resolves the edited dimension (dimension rows depend only on indices + value) and overwrites its row in place. apply_params validates all values before mutating so it is all-or-nothing. rename_bound returns a count instead of a Result since an unbound name is not an error for this sketch. New SketchError variants UnknownConstraint/NotADimension; ParamError wraps SketchError and adds NotADimension(Constraint) and UnknownParameter(name).

**Outcome:** Implemented in e212add; 7 new tests; local gates exit 0; CI 3/3 green.

**Next Action:** QA scenarios and plots.
- *Rationale:* Smallest change in swx-sketch only; no change to swx-core needed. rename_bound returning usize is a minor deviation (issue gave no return type).

**Execution Ref:** `issue:#71`

---

### Step 3/3: QA VERDICT FOR PR #83

**Context:** Scratch program outside the repo built the 40x25 rectangle, bound d0/d1, edited d0 to 60, renamed d0->width in the doc only (error), then rename_bound, and tried binding Parallel. Plots rect-d0-before/after rendered with swx-qa plot().

**Thought:** All outputs matched expectations: corners exact to 1e-9, error names `d0`, constraint untouched after failed apply, Parallel rejected with param count unchanged. Plots auto-fit each sketch, so the before/after views are scaled independently; the dimension labels (40 vs 60) show the change. All criteria verified by tests or scenarios; nothing left uncovered.

**Outcome:** Verdict: pass.

**Next Action:** Publish qa-artifacts, post QA report and outcome comments.
- *Rationale:* FACTORY §6b: pass when every criterion is met and verified and (e) is None.

**Execution Ref:** `issue:#71`

---
