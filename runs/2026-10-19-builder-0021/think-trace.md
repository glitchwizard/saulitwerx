### Step 1/8: PICK TONIGHT'S BUILDER ITEMS

**Context:** Night 2026-10-19 PT, run start 00:21. No Builder outcomes yet tonight, cap 2. main CI green (3 checks on 7f80405). No priority or factory:changes issues. Ready: #22 M2-02, #25 M2-03, #28 M2-04, #39 QA-01b.

**Thought:** #22 (M2-02) and #39 (QA-01b) both have their dependencies closed (#18, #32). #25 and #28 depend on the M2 chain. #39 changes only .github/workflows/ci.yml; #22 changes only swx-sketch. Neither depends on the other and they share no crate, so they are independent per §3.

**Outcome:** Take #22 and #39, one subagent each in its own worktree
