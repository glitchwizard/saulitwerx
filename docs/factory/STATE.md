# Factory state

status: running
issues_per_night: 4

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->
<!-- issues_per_night caps how many issues the Builder takes per night, across its runs (FACTORY §3). -->

## Notes for the factory

- Owner priority: one Builder run and one Reviewer run a night, each doing the whole night's work. The Builder takes up to `issues_per_night` (4) issues, one after the other (FACTORY §3); the Reviewer then reviews and merges them (§4).
- Current focus: M3 Part features, in roadmap order.

## Log

- 2026-11-21 reviewer (03:18): merged #231 (M4-01f) (CI green, 3 checks; full diff read, QA report + ship report verified, nothing visual; local fmt + swx-io clippy + 65 swx-io tests on main+#231) and #235 (M4-05a) (CI green, 3 checks; full diff read, QA report + ship report verified, nothing visual; local fmt + swx-io clippy + 65 + 8 swx-io tests on main+#231+#235); roadmap: M4-01 checked; planned #238 M4-04b (no deps; ready: #229, #232, #236, #238; unblocked: #229, #238; #236 waits on #229, #232 on #206; M3 waits on #106); nightly cap of 4 reviews reached, #227 left for the next run; merged branches not deleted (not tried); no retro (not a Monday night)
- 2026-11-21 builder: #216 M4-02b → stalled, needs-human (cylinder volume criterion 0.1 % at deflection 0.01 is stricter than the deflection allows; measured 0.13 %; branch feat/216-export-stl, no PR)
- 2026-11-21 builder: #206 M4-04a → PR #227 (b777c4e; CI green, 3 checks; think-and-ship gates verified; QA pass; send-back fix, merged main)
- 2026-11-21 reviewer (01:18): merged #221 (M4-02a) (CI green, 3 checks; full diff read, QA report + ship report verified, nothing visual; local fmt + swx-io clippy + 56 swx-io tests on main+#221); #227 (M4-04a) sent back, #206 labeled factory:changes (conflicts with main on the swx-io lib.rs crate doc comment after #221; merge main); no roadmap check-off (M4-02 needs M4-02b, #216); planned #236 M4-05c (depends on #228, #229; ready: #216, #229, #232, #236; only #216, #229 unblocked: M3 waits on #106, other M4 slices wait on #206/#228); careful mode: 2 PRs max, #231 and #235 left for the next run; merged branch not deleted (not tried); no retro (not a Monday night)
- 2026-11-21 builder: #207 M4-05a → PR #235 (3e056a3; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-11-21 builder: #206 M4-04a → PR #227 (e24735a; CI green, 3 checks; think-and-ship gates verified; QA pass; send-back fix, merged main)
- 2026-11-20 documenter: M4-01e → PR #234 merged (CI green, 3 checks; 173 pages; no feedback; no issues opened)
- 2026-11-20 reviewer (03:18): merged #226 (M4-03 spec) (CI green, 3 checks; full diff read, QA report + ship report verified, nothing visual; no local run, docs only); #227 (M4-04a) sent back, #206 labeled factory:changes (diff and QA fine, but conflicts with main on the swx-io unpacked.rs import line after #224; merge main); no roadmap check-off (M4-03 needs M4-03a); planned #232 M4-03a (depends on #206; ready: #207, #216, #229, #232; only #207, #229 unblocked, no other unblocked roadmap item: M3 waits on #106, M4 slices wait on #206/#215); nightly cap of 4 reviews reached, #221 and #231 left for the next run; merged branch not deleted (not tried); no retro (not a Monday night)
- 2026-11-20 builder: #228 M4-01f → PR #231 (b0dd299; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-20 builder: #215 M4-02a → PR #221 (9943fd8; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps; send-back fix, no code change)
- 2026-11-20 reviewer: merged #224 (M4-01e) (CI green, 3 checks; full diff read, QA report + ship report verified, nothing visual; no local run, CI covers it); #221 (M4-02a) sent back, #215 labeled factory:changes (no QA report, no ship report, no outcome comments); no roadmap check-off (M4-01 still needs stable IDs, now #228); planned #228 M4-01f, #229 M4-05b (ready: #207, #216, #228, #229; #207, #228, #229 unblocked and independent; #216 waits on #215); careful mode: 2 PRs max, #226 and #227 left for the next run; merged branch not deleted (not tried); no retro (not a Monday night)
- 2026-11-20 builder: #206 M4-04a → PR #227 (871bb2f; CI green, 3 checks; think-and-ship gates verified, 1 shell-syntax record re-run; QA pass with gaps)
- 2026-11-20 builder: #214 M4-03 spec → PR #226 (443f74c; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-19 builder: #223 M4-01e → PR #224 (e5a6720; CI green, 3 checks; think-and-ship gates verified; QA pass; PR opened 11-18, finished 11-19)
- 2026-11-17 documenter: M4-01d → PR #222 merged (CI green, 3 checks; 173 pages; no feedback; opened #223)
- 2026-11-16 reviewer: merged #211 (M4-01d) (CI green, 3 checks; full diff read, QA report + ship report verified; pass with gaps); #212 (MB-03b) not merged, labeled needs-human (the other 11 pitches' `es` values are unverified against ISO 965-1; owner decides); no roadmap check-off (M4-01: "stable IDs" not found in swx-io, not checked); planned #213 M4-02 (STL export) and #214 M4-03 spec (ready: #206, #207, #213, #214; #206 and #207 unblocked by #202 and independent; #213 and #214 independent); retro PR from docs/factory-retro-2026-11-16 (2 changes, needs-human); no Reviewer run-log comment between 2026-11-13 and 2026-11-15
- 2026-11-16 builder: idle (only ready issues #206, #207 depend on open #202; PRs #211, #212 await review since 2026-11-13)
- 2026-11-16 builder: idle (only ready issues #206, #207 depend on open #202; PRs #211, #212 await review since 2026-11-13)
- 2026-11-15 builder: idle (only ready issues #206, #207 depend on open #202; PRs #211, #212 await review since 2026-11-13)
- 2026-11-15 builder: idle (only ready issues #206, #207 depend on open #202; PRs #211, #212 await review since 2026-11-13)
- 2026-11-14 builder: idle (ready #206, #207 depend on open #202; no Reviewer run since 2026-11-12)
- 2026-11-14 builder: idle (ready #206, #207 depend on open #202)
- 2026-11-13 builder: idle (tonight's 2 items already taken by an overlapping run)
- 2026-11-13 builder: #194 MB-03b → PR #212 (ae908b5; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-13 builder: #202 M4-01d → PR #211 (6ee9710; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-13 documenter: rounded-corner plate (backlog) → PR #210 merged (CI green, 3 checks; 170 pages; no feedback; no issues opened)
- 2026-11-12 documenter: MB-03a + M4-01a/b/c + M3-12d → PR #209 merged (CI green, 3 checks; 164 pages; no feedback; no issues opened)
- 2026-11-12 reviewer: merged #204 (M4-01c) (CI green, 3 checks; QA report + ship report verified, nothing visual; no local run, CI covers it) and #205 (MB-03a) (CI green, 3 checks; QA report + ship report + 2 images verified; no local run, independent of #204 and CI covers the occt job); no roadmap check-off (M4-01 needs #202, MB-03 needs the thread slices); planned #206 M4-04a, #207 M4-05a (both depend on #202; ready: #194, #202, #206, #207; #194, #202 unblocked and independent); merged branches not deleted (not tried); no retro (not a Monday night)
- 2026-11-12 builder: #190 MB-03a → PR #205 (4e5a7bc; CI green, 3 checks; think-and-ship gates verified, 1 shell-syntax record re-run; QA pass)
- 2026-11-12 builder: #199 M4-01c → PR #204 (03eb55e; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
