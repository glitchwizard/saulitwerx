# Factory state

status: running
issues_per_night: 4

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->
<!-- issues_per_night caps how many issues the Builder takes per night, across its runs (FACTORY §3). -->

## Notes for the factory

- Owner priority: every run does real work. Two issues per Builder run, one after the other, so up to `issues_per_night` (4) a night across the 12:17am and 2:17am runs (FACTORY §3). Each Reviewer run merges what the Builder run before it opened.
- Current focus: M3 Part features, in roadmap order.

## Log

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
- 2026-11-02 builder: idle (no qualifying issue; #113 and #114 wait on #105, still in review)
- 2026-11-02 builder: #117 M3-03b → PR #120 (4b58b12; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-02 builder: #105 M3-04b → PR #116 merged main (8723c52; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-01 documenter: M3-02a + M3-03a → PR #119 merged (CI green, 3 checks; 87 pages; no feedback; no issues opened)
- 2026-11-01 reviewer: merged #112 (M3-03a) (CI green, 3 checks; QA report + ship report + 3 images verified; local gates on #112 exit 0, incl. swx-kernel occt); sent back #116 (M3-04b: conflicts with main after #112); no roadmap check-off (M3-03 needs #117); planned #117 M3-03b (ready: #113, #114, #117; #113 depends on #105, #114 on #105 and #113); merged branch not deleted (delete denied); no retro (not a Monday night)
- 2026-11-01 builder: #105 M3-04b → PR #116 (39cc854; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-11-01 builder: #104 M3-03a → PR #112 merged main (5e4aa69; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-11-01 reviewer: merged #111 (M3-02a) (CI green, 3 checks; QA report + ship report + 2 images verified; local gates on #111 exit 0, incl. swx-kernel and swx-qa occt); sent back #112 (M3-03a: conflicts with main after #111); no roadmap check-off (M3-02 needs to face/between faces); planned #113 M3-04c, #114 M3-04d (ready: #105, #113, #114; #113 depends on #105, #114 on #105 and #113); #106 not made ready (owner chose swx-model; new crate needs an ADR first); merged branch not deleted (delete denied); no retro (not a Monday night)
- 2026-11-01 builder: #104 M3-03a → PR #112 (9e47ddd; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-11-01 builder: #97 M3-02a → PR #111 (6a1d0b4; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-10-31 reviewer: no PRs to review (none labeled factory:review; #102 and #103 merged at 1:18am); no new issues (ready: #97, #104, #105); no retro (not a Monday night)
- 2026-10-31 builder: idle (nightly cap reached; 12:17am run took #91 → PR #102 and #98 → PR #103)
- 2026-10-31 reviewer: merged #102 (M3-01b) and #103 (M3-04e) (CI green, 3 checks each; QA reports + ship reports + 2 images verified; local gates on main+both exit 0, incl. swx-kernel and swx-qa occt); no roadmap check-off (M3-01 needs #106, M3-04 needs M3-04b..g); planned #104 M3-03a, #105 M3-04b, #106 M3-01c (needs-human: placement of sketch → kernel loop conversion) (ready: #97, #104, #105; all swx-kernel); merged branches not deleted (delete denied); no retro (not a Monday night)
- 2026-10-31 builder: #98 M3-04e → PR #103 (02dfbb8; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-10-31 builder: #91 M3-01b → PR #102 (6c27aa8; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-10-30 documenter: M3-01a → PR #101 merged (CI green, 3 checks; 76 pages; no feedback; no issues opened)
- 2026-10-30 reviewer: no PRs to review (none labeled factory:review; #95 and #96 merged at 1:20am); no new issues (ready: #91, #97, #98; #97 depends on #91); no retro (not a Monday night)
- 2026-10-30 builder: idle (nightly cap reached; 12:17am run took #90 → PR #95 and #92 → PR #96)
