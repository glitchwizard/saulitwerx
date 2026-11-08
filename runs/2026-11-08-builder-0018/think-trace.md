### Step 1/4: #147 SEND-BACK: CAUSE OF INTERMITTENT TEST FAILURE

**Context:** PR #150 sent back for an intermittent failure in the variable-fillet tests.

**Thought:** Parallel suite runs failed 27/100 (variable-fillet tests only), serial runs 0/40, so OCCT's variable-radius builder is not safe to run concurrently.

**Outcome:** Added a process-wide Mutex around the shim call in ffi.rs plus an 8-thread regression test (fails 9/10 without the lock). Full occt suite 0/100 parallel, 0/40 serial. Merged main. CI green on a4b3b83.

**Next Action:** Post QA report r3, outcome comments, then take the next item.
- *Rationale:* Serializing one call is the smallest change that removes the shared-state race without loosening the test.

**Execution Ref:** `issue:#147`

---

### Step 2/3: #132 M3-06B: PICK AND APPROACH

**Context:** Second item of the run. All ready issues are swx-kernel, so roadmap order applies: #132 M3-06b is lowest.

**Thought:** Mirror the shell pattern: validate inputs in occt.rs, one noexcept shim over BRepOffsetAPI_DraftAngle (TKOffset already linked), ffi wrapper with SAFETY comment. OCCT's positive angle already shrinks the solid along the pull direction, so no sign flip.

**Outcome:** PR #157 at c3fd548, 5 new occt tests pass, CI green (3 checks), 7 gates verified. QA: pass with gaps (no hands-on check of curved faces, out of scope).

**Next Action:** Post QA report and outcome comments, export trace, end run.
- *Rationale:* Smallest change that meets the criteria, matching existing kernel ops.

**Execution Ref:** `issue:#132`

---
