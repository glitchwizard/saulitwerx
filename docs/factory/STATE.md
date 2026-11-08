# Factory state

status: running
issues_per_night: 4

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->
<!-- issues_per_night caps how many issues the Builder takes per night, across its runs (FACTORY §3). -->

## Notes for the factory

- Owner priority: every run does real work. Two issues per Builder run, one after the other, so up to `issues_per_night` (4) a night across the 12:17am and 2:17am runs (FACTORY §3). Each Reviewer run merges what the Builder run before it opened.
- Current focus: M3 Part features, in roadmap order.

## Log

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
- 2026-11-06 builder: #114 M3-04d → PR #137 (f77d68b, main merged; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-05 documenter: M3-02b + M3-05a → PR #140 merged (CI green, 3 checks; 101 pages; no feedback; no issues opened)
- 2026-11-05 reviewer: merged #128 (M3-05a) (CI green, 3 checks; QA report r2 + ship report r2 + 2 images verified; radius-15 deviation accepted; no local run, CI covers the occt job); sent back #137 (M3-04d: conflicts with main after #128); no roadmap check-off (M3-05 needs more slices); planned #138 M3-05b (ready: #131, #132, #138, all unblocked); merged branch not deleted (delete denied); no retro (not a Monday night)
- 2026-11-05 builder: #114 M3-04d → PR #137 (32cabfd; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-05 builder: #122 M3-05a → PR #128 (78b7aaf, main merged; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-05 reviewer: merged #130 (M3-02b) (CI green, 3 checks; QA report r2 + ship report r2 + 1 image verified; Extent::FromTo deviation accepted; no local run, CI covers the occt job); checked off M3-02; no new issues (ready: #114, #131, #132, all unblocked); #128 stays factory:changes; merged branch not deleted (delete denied); no retro (not a Monday night)
- 2026-11-05 builder: #121 M3-02b → PR #130 (479a346, main merged; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-04 reviewer: 1:18am run resumed at 14:00 on owner request; sent back #130 (M3-02b: conflicts with main after #127); #127 already merged and #128 already sent back by the 3:17am run; no new issues (ready: #114, #131, #132); no roadmap check-off; no retro (not a Monday night)
- 2026-11-04 documenter: M3-04c → PR #134 merged (CI green, 3 checks; 94 pages; no feedback; no issues opened)
- 2026-11-04 reviewer: merged #127 (M3-04c) (CI green, 3 checks; QA report + ship report + 1 image verified; no local run, CI covers the occt job); sent back #128 (M3-05a: conflicts with main after #127); #130 not reviewed (2-PR cap); no roadmap check-off (M3-04 needs M3-04d..g); planned #131 M3-06a, #132 M3-06b (ready: #114, #131, #132, all independent); M3-04f held (needs the swx-model ADR and #106; depends on #114); merged branch not deleted (delete denied); no retro (not a Monday night)
- 2026-11-04 builder: idle (no qualifying issue; #114 waits on #113, still in review)
- 2026-11-04 reviewer: stalled (run interrupted while reviewing #127; nothing merged or created)
- 2026-11-04 builder: #121 M3-02b → PR #130 (45854e1; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-03 documenter: M3-03b → PR #129 merged (CI green, 3 checks; 91 pages; no feedback; no issues opened)
