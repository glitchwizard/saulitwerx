# Factory state

status: running
issues_per_night: 4

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->
<!-- issues_per_night caps how many issues the Builder takes per night, across its runs (FACTORY §3). -->

## Notes for the factory

- Owner priority: every run does real work. Two issues per Builder run, one after the other, so up to `issues_per_night` (4) a night across the 12:17am and 2:17am runs (FACTORY §3). Each Reviewer run merges what the Builder run before it opened.
- Current focus: M3 Part features, in roadmap order.

## Log

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
- 2026-11-03 builder: #122 M3-05a → PR #128 (729d475; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-03 builder: #113 M3-04c → PR #127 (de18914; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-03 reviewer: merged #120 (M3-03b) (CI green, 3 checks; QA report + ship report r3 + 1 image verified; no local run, CI covers the occt job); checked off M3-03; no new issues (ready: #113, #114, #122; #113 and #122 unblocked; #114 depends on #113; #121 needs-human after the Builder stall; routine prompt says 3 ready, FACTORY §4 says 4); merged branch not deleted (delete denied); no retro (not a Monday night)
- 2026-11-03 builder: #121 M3-02b stalled (run interrupted mid-implementation; nothing pushed; needs-human)
- 2026-11-03 builder: #117 M3-03b → PR #120 merged main (34d3598; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-02 documenter: M3-04b → PR #125 merged (CI green, 3 checks; 90 pages; no feedback; no issues opened)
- 2026-11-02 reviewer: merged #116 (M3-04b) (CI green, 3 checks; QA report + ship report r2 verified, nothing visual); sent back #120 (M3-03b: conflicts with main after #116); no roadmap check-off (M3-03 needs #117, M3-04 needs M3-04c..g); planned #121 M3-02b, #122 M3-05a (ready: #113, #114, #121, #122; #114 depends on #113); retro PR #123 (needs-human; 1 change: Planner counts only unblocked ready issues; #80 still unmerged); 1:17am reviewer run left no trace; merged branch not deleted (delete denied)
