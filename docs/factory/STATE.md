# Factory state

status: running
issues_per_night: 4

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->
<!-- issues_per_night caps how many issues the Builder takes per night, across its runs (FACTORY §3). -->

## Notes for the factory

- Owner priority: every run does real work. Two issues per Builder run, one after the other, so up to `issues_per_night` (4) a night across the 12:17am and 2:17am runs (FACTORY §3). Each Reviewer run merges what the Builder run before it opened.
- Current focus: M3 Part features, in roadmap order.

## Log

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
- 2026-11-10 builder: #169 M3-10a → PR #178 (b6baca3; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-11-10 builder: #173 M3-08c → PR #177 (b3cb1ac; CI green, 3 checks; think-and-ship gates verified, 1 shell-syntax record re-run; QA pass)
- 2026-11-09 documenter: M3-11a + M3-11b + M3-08b → PR #175 merged (CI green, 3 checks; 138 pages; no feedback; opened #176)
- 2026-11-09 reviewer: merged #167 (M3-11b) (CI green, 3 checks; QA report r2 + ship report r2 + 2 images verified; no local run) and #172 (M3-08b) (CI green, 3 checks; QA report + ship report + 1 image verified; local fmt + occt clippy + 174 swx-kernel tests on main+#172 after #167); checked off M3-11; planned #173 M3-08c (ready: #162, #163, #169, #173, all unblocked and independent); merged branches not deleted (not retried); retro already done this night (PR #170)
- 2026-11-09 builder: #168 M3-08b → PR #172 (7b2f01a; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-11-09 builder: #158 M3-11b → PR #167 (68a10ce, main merged; CI green, 3 checks; think-and-ship gates verified, 1 shell-syntax record re-run; QA pass)
- 2026-11-09 reviewer: merged #166 (M3-11a) (CI green, 3 checks; QA report + ship report + 3 images verified; no local run, CI covers the occt job); sent back #167 (M3-11b: conflicts with main after #166); no roadmap check-off (M3-11 needs #158); planned #168 M3-08b, #169 M3-10a (ready: #162, #163, #168, #169, all unblocked and independent); merged branch not deleted (delete denied); retro: PR #170 (2 changes, needs-human)
- 2026-11-09 builder: #158 M3-11b → PR #167 (bdb1ae6; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-11-09 builder: #154 M3-11a → PR #166 (fb08e8a; CI green, 3 checks; think-and-ship gates verified, 1 shell-syntax record re-run; QA pass)
- 2026-11-08 documenter: M3-05c + M3-06b + M3-06c + M3-08a → PR #165 merged (CI green, 3 checks; 128 pages; no feedback; no issues opened)
- 2026-11-08 reviewer: merged #160 (M3-06c) (CI green, 3 checks; QA report + ship report + 2 images verified; no local run) and #161 (M3-08a) (CI green, 3 checks; QA report + ship report verified, nothing visual; local occt clippy + 156 swx-kernel tests on main+#161 after #160); checked off M3-06; planned #162 M3-12a, #163 M3-12b (ready: #154, #158, #162, #163, all unblocked and independent); merged branches not deleted (not retried); no retro (not a Monday night)
- 2026-11-08 builder: #144 M3-08a → PR #161 (59eb655; CI green, 3 checks; think-and-ship gates verified; QA pass)
