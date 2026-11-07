### Step 1/8: BUILDER 2026-11-07 0218: PICK ITEMS

**Context:** Tonight 2 of 4 items done by the 12:17am run; main CI green on b6d0841; no priority issues. #147 (PR #150) is factory:changes because it conflicts with main after #146. Ready: #132, #143, #144, #151.

**Thought:** §3 order: factory:changes first, so #147. A second item may follow if independent; #151 (swx-core) is independent of #147 and in a different crate, which §3 prefers. Decide the second only after finishing #147.

**Outcome:** Item 1: #147 M3-05c, merge main into PR #150 branch.

**Next Action:** Claim #147 (factory:wip) and merge origin/main into its branch.
- *Rationale:* FACTORY §3 step 3.3 and the run cap min(2, 4-2)=2.

**Execution Ref:** `issue:#147`

---

### Step 2/8: HANDLE A GATE FAILURE ON #147 R2
![Confidence](https://img.shields.io/badge/confidence-70%25-blue)

**Context:** Merge of main into PR #150 resolved by keeping both shim/ffi functions (verified identical to each side). CI green on 8415f7d. ship_check `cargo test -p swx-kernel --features occt` failed once: fillet_variable_too_large_is_an_error_and_keeps_the_input panicked at the valid 2->4 unwrap (occt.rs:1102). The gate command also used PIPESTATUS, which sh rejects ("Bad substitution").

**Thought:** Reran: 10/10 single-test runs pass, 25/25 full-suite runs pass (my earlier Bash run and CI also passed). So it is intermittent, about 1 in 36 full runs here. The backend has no lock and cargo runs tests in parallel threads, so a thread-safety issue in OCCT's fillet path is the likely cause, but proving it needs extensive exploration, which §5 forbids. The code change in this round is only a conflict resolution, so it does not cause the flake; it is in the PR's own test though.

**Outcome:** Re-run the gate once with a plain command (no PIPESTATUS); report the failed record and the flake rate in QA (b) and (e); recommend a follow-up to investigate OCCT concurrency in tests. Not blocking: CI and all reruns are green.

**Next Action:** ship_check `cargo test -p swx-kernel --features occt`
- *Rationale:* Honest reporting per §6a/§6b; a single intermittent failure that does not reproduce in 35 reruns is a gap, not an unmet criterion.

**Execution Ref:** `issue:#147`

---

### Step 3/8: QA VERDICT #147 R2

**Context:** PR #150 head 8415f7d, CI green 3 checks, gates verified (one failed occt test record, re-run passed). Scenario: shell after variable fillet fails in backend; constant fillet shells fine.

**Thought:** All criteria still met; the flake and the shell interaction are gaps, not unmet criteria. QA report r2 posted.

**Outcome:** pass with gaps; #147 labeled factory:review.

**Next Action:** Post outcome comments, then decide item 2 (#151).
- *Rationale:* §6b (f) definitions.

**Execution Ref:** `issue:#147`

---

### Step 4/8: PICK ITEM 2

**Context:** Tonight 3 of 4 done, 7 minutes into the run. Ready: #132 M3-06b, #143 M3-06c, #144 M3-08a (kernel/geometry), #151 M3-09a (swx-core).

**Thought:** §3: second item is the next in order and must be independent; prefer a different crate. #151 has no Depends-on line, touches only swx-core, size:S. #132 is lowest roadmap ID but same crate as #147 (swx-kernel).

**Outcome:** Item 2: #151 M3-09a, branch feat/151-end-of-part from origin/main.

**Next Action:** Claim #151 and read swx-core lib.rs.
- *Rationale:* §3 different-crate preference; avoids a second kernel conflict with #150.

**Execution Ref:** `issue:#151`

---

### Step 5/5: QA VERDICT #151

**Context:** PR #153 head 2b44946: swx-core only, 4 new tests, 33 existing unchanged; gates verified; CI green 3 checks; scratch scenarios match the issue.

**Thought:** Implemented exactly as specified: field end_of_part, insert at marker, active-only inputs and dirty sets. No deviation from the issue. Only gaps: marker persistence is out of scope, ship tool schema errors.

**Outcome:** pass with gaps; #151 labeled factory:review. Run done: 2 items (tonight 4 of 4).

**Next Action:** Outcome comments, export think trace to qa-artifacts.
- *Rationale:* §6b (f).

**Execution Ref:** `issue:#151`

---
