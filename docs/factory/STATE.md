# Factory state

status: running

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->

## Notes for the factory

- Owner priority: slow and steady, budget first. One issue per night.
- Next focus: M1 kernel spike (M1-01). Don't start M2 until M1-01 is merged.

## Log

- 2026-10-17 builder: #18 M2-01 → PR #27 (CI pending at log time)
- 2026-10-16 reviewer: no open PRs; owner merged #24 (M1-05); checked off M1-05; planned #25 M2-03 (ready: #18, #22, #25)
- 2026-10-16 builder: #9 M1-05 → PR #24 (CI green; 3 checks; owner-merge: adds libocct-data-exchange-dev to CI)
- 2026-10-15 reviewer: merged #20 (M1-04; CI green, 3 checks); checked off M1-04; planned #22 M2-02 (ready: #9, #18, #22); branch feat/8-tessellation not deleted (API write blocked)
- 2026-10-15 builder: #8 M1-04 → PR #20 (CI green; 3 checks)
- 2026-10-14 reviewer: merged #17 (M1-03; CI green, 3 checks); checked off M1-03; planned #18 M2-01 (M2 opened per owner note, M1-04/05 still ready); branch feat/7-kernel-trait not deleted (API write blocked)
- 2026-10-14 builder: #7 M1-03 → PR #17 (CI green; 3 checks)
- 2026-10-13 reviewer: no open PRs; owner merged #15 (M1-02); checked off M1-02; #7 #8 #9 already factory:ready, no new issues needed
- 2026-10-13 builder: #2 M1-02 → PR #15 (CI green; kernel-occt job 34 s; owner-merge)
- 2026-10-13 reviewer: owner merged #4 (M1-01) and #5 (CI); checked off M1-01; planned #7 M1-03, #8 M1-04, #9 M1-05; #6 (FACTORY.md) needs-human
- 2026-10-13 builder: idle (no ready issues; #1-#3 needs-human, CI pending #5)
- 2026-10-13 builder: #1 M1-01 → PR #4 (no CI on main, see #3; table-row edit needs-human)
- 2026-10-12 bootstrap: M0 complete (workspace, swx-core, factory docs); seeded M1 issues
