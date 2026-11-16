# Factory state

status: running
issues_per_night: 4

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->
<!-- issues_per_night caps how many issues the Builder takes per night, across its runs (FACTORY §3). -->

## Notes for the factory

- Owner priority: one Builder run and one Reviewer run a night, each doing the whole night's work. The Builder takes up to `issues_per_night` (4) issues, one after the other (FACTORY §3); the Reviewer then reviews and merges them (§4).
- Current focus: M3 Part features, in roadmap order.

## Log

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
- 2026-11-12 reviewer: merged #200 (M3-12d) (CI green, 3 checks; QA report + ship report verified, nothing visual; no local run, CI covers the occt job) and #201 (M4-01b) (CI green, 3 checks; QA report + ship report verified, nothing visual; local fmt + swx-io clippy + 27 swx-io tests on main+#201 after #200); no roadmap check-off (M3-12 already checked; M4-01 needs the ZIP container); planned #202 M4-01d (depends on #199; ready: #190, #194, #199, #202; #190, #194 unblocked and independent); merged branches not deleted (not retried); no retro (not a Monday night)
- 2026-11-12 builder: #189 M4-01b → PR #201 (051e01f; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-11-12 builder: #197 M3-12d → PR #200 (e189fa8; CI green, 3 checks; think-and-ship gates verified, 1 shell-syntax record re-run; QA pass)
- 2026-11-11 documenter: M3-12a + M3-12b + M3-12c → PR #196 merged (CI green, 3 checks; 153 pages; no feedback; opened #197)
- 2026-11-11 reviewer: merged #192 (M3-12c) (CI green, 3 checks; QA report + ship report + 1 image verified; no local run, CI covers the occt job); #193 (M4-01a) not merged, labeled needs-human with #184 (Unicode-3.0 license of the unbuilt `unicode-ident` in Cargo.lock and its register row; owner decides); checked off M3-12; planned #194 MB-03b (ready: #189, #190, #194; #190, #194 unblocked and independent; #189 waits on #184; M3-01/04/05/07 wait on the #106 swx-model ADR); merged branch not deleted (not retried); no retro (not a Monday night)
- 2026-11-11 builder: #184 M4-01a → PR #193 (b722eb1; CI green, 3 checks; think-and-ship gates verified, 1 shell-syntax record re-run; QA pass with gaps)
- 2026-11-11 builder: #183 M3-12c → PR #192 (d9a49e7; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-11-11 reviewer: merged #187 (M3-12a) (CI green, 3 checks; QA report + ship report + 1 image verified; no local run, CI covers the occt job) and #188 (M3-12b) (CI green, 3 checks; QA report + ship report verified, nothing visual; local fmt + occt clippy + 202 swx-kernel tests on main+#188 after #187); no roadmap check-off (M3-12 needs #183); planned #189 M4-01b (depends on #184), #190 MB-03a (ready: #183, #184, #189, #190; #183, #184, #190 unblocked and independent); merged branches not deleted (not retried); no retro (not a Monday night)
- 2026-11-11 builder: #163 M3-12b → PR #188 (13d7a49; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-11-11 builder: #162 M3-12a → PR #187 (524da52; CI green, 3 checks; think-and-ship gates verified, 1 shell-syntax record re-run; QA pass)
- 2026-11-10 documenter: M3-08c + M3-10a + M3-10b + M3-11c → PR #186 merged (CI green, 3 checks; 145 pages; no feedback; no issues opened)
- 2026-11-10 reviewer: merged #181 (M3-10b) (CI green, 3 checks; QA report + ship report + 1 image verified; no local run, CI covers the occt job) and #182 (M3-11c) (CI green, 3 checks; QA report + ship report + 2 images verified; local fmt + occt clippy + 194 swx-kernel tests on main+#182 after #181); checked off M3-10; planned #183 M3-12c (depends on #162, #163), #184 M4-01a (ready: #162, #163, #183, #184; #162, #163, #184 unblocked and independent); merged branches not deleted (not retried); no retro (not a Monday night)
- 2026-11-10 builder: #176 M3-11c → PR #182 (9b18cb8; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-10 builder: #179 M3-10b → PR #181 (82a7c0b; CI green, 3 checks; think-and-ship gates verified, 1 shell-syntax record re-run; QA pass)
- 2026-11-10 reviewer: merged #177 (M3-08c) (CI green, 3 checks; QA report + ship report + 1 image verified; no local run, CI covers the occt job) and #178 (M3-10a) (CI green, 3 checks; QA report + ship report + 2 images verified; local fmt + occt clippy + 184 swx-kernel tests on main+#178 after #177); checked off M3-08; planned #179 M3-10b (ready: #162, #163, #176, #179, all unblocked and independent); merged branches not deleted (delete denied, not retried); no retro (not a Monday night)
