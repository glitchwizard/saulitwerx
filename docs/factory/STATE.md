# Factory state

status: running
issues_per_night: 4

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->
<!-- issues_per_night caps how many issues the Builder takes per night, across its runs (FACTORY §3). -->

## Notes for the factory

- Owner priority: every run does real work. Two issues per Builder run, one after the other, so up to `issues_per_night` (4) a night across the 12:17am and 2:17am runs (FACTORY §3). Each Reviewer run merges what the Builder run before it opened.
- Current focus: M3 Part features, in roadmap order.

## Log

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
- 2026-11-08 builder: #143 M3-06c → PR #160 (69d08a1; CI green, 3 checks; think-and-ship gates verified, 1 shell-syntax record re-run; QA pass with gaps)
- 2026-11-08 reviewer: merged #150 (M3-05c) (CI green, 3 checks; QA report r3 + ship report r3 verified, nothing visual; no local run) and #157 (M3-06b) (CI green, 3 checks; QA report + ship report + 2 images verified; local occt clippy + 139 tests on main+#157 after #150); no roadmap check-off (M3-05 needs referenced edges, M3-06 needs #143); planned #158 M3-11b (ready: #143, #144, #154, #158, all unblocked and independent); merged branches not deleted (not retried); no retro (not a Monday night)
- 2026-11-08 builder: #132 M3-06b → PR #157 (c3fd548; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-08 builder: #147 M3-05c → PR #150 (a4b3b83, main merged; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-07 documenter: M3-06a + M3-09a → PR #156 merged (CI green, 3 checks; 113 pages; no feedback; no issues opened)
- 2026-11-07 reviewer: merged #153 (M3-09a) (CI green, 3 checks; QA report + ship report verified, nothing visual; no local run); checked off M3-09; sent back #150 (M3-05c: intermittent failure of fillet_variable_too_large_is_an_error_and_keeps_the_input on the input-still-usable call, criterion 4 unchecked); planned #154 M3-11a (ready: #132, #143, #144, #154, all unblocked and independent); merged branch not deleted (not retried); no retro (not a Monday night)
- 2026-11-07 builder: #151 M3-09a → PR #153 (2b44946; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-07 builder: #147 M3-05c → PR #150 (8415f7d, main merged; CI green, 3 checks; think-and-ship gates verified, 1 flaky occt test record re-run; QA pass with gaps)
- 2026-11-07 reviewer: merged #146 (M3-06a) (CI green, 3 checks; QA report r2 + ship report r2 + 2 images verified; no local run, CI covers the occt job); sent back #150 (M3-05c: conflicts with main after #146); no roadmap check-off (M3-06 needs #132, #143); planned #151 M3-09a (ready: #132, #143, #144, #151, all unblocked and independent); merged branch not deleted (not retried); no retro (not a Monday night)
- 2026-11-07 builder: #147 M3-05c → PR #150 (0125bf8; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-07 builder: #131 M3-06a → PR #146 (b0f45d0, main merged; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-06 documenter: M3-04d + M3-05b → PR #149 merged (CI green, 3 checks; 106 pages; no feedback; no issues opened)
- 2026-11-06 reviewer: merged #142 (M3-05b) (CI green, 3 checks; QA report r2 + ship report r2 + 3 images verified; no local run, CI covers the occt job); sent back #146 (M3-06a: conflicts with main after #142); no roadmap check-off (M3-05 needs more slices); planned #147 M3-05c (ready: #132, #143, #144, #147, all unblocked and independent); think-and-ship reasoning tools not used this run (§6c fallback); merged branch not deleted (delete denied, not retried); no retro (not a Monday night)
- 2026-11-06 builder: #131 M3-06a → PR #146 (0d2c03c; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-06 builder: #138 M3-05b → PR #142 (1f145e3, main merged; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-06 reviewer: merged #137 (M3-04d) (CI green, 3 checks; QA report r2 + ship report r2 + 2 images verified; no local run, CI covers the occt job); sent back #142 (M3-05b: conflicts with main after #137); no roadmap check-off (M3-04 needs M3-04f/g); planned #143 M3-06c, #144 M3-08a (ready: #131, #132, #143, #144, all unblocked and independent); M3-04f still held (swx-model ADR); merged branch not deleted (delete denied); no retro (not a Monday night)
- 2026-11-06 builder: #138 M3-05b → PR #142 (3dadb57; CI green, 3 checks after 1 infra re-run; think-and-ship gates verified; QA pass with gaps)
