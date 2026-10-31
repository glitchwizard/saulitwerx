# Factory state

status: running
issues_per_night: 4

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->
<!-- issues_per_night caps how many issues the Builder takes per night, across its runs (FACTORY §3). -->

## Notes for the factory

- Owner priority: every run does real work. Two issues per Builder run, one after the other, so up to `issues_per_night` (4) a night across the 12:17am and 2:17am runs (FACTORY §3). Each Reviewer run merges what the Builder run before it opened.
- Current focus: M3 Part features, in roadmap order.

## Log

- 2026-10-31 reviewer: no PRs to review (none labeled factory:review; #102 and #103 merged at 1:18am); no new issues (ready: #97, #104, #105); no retro (not a Monday night)
- 2026-10-31 builder: idle (nightly cap reached; 12:17am run took #91 → PR #102 and #98 → PR #103)
- 2026-10-31 reviewer: merged #102 (M3-01b) and #103 (M3-04e) (CI green, 3 checks each; QA reports + ship reports + 2 images verified; local gates on main+both exit 0, incl. swx-kernel and swx-qa occt); no roadmap check-off (M3-01 needs #106, M3-04 needs M3-04b..g); planned #104 M3-03a, #105 M3-04b, #106 M3-01c (needs-human: placement of sketch → kernel loop conversion) (ready: #97, #104, #105; all swx-kernel); merged branches not deleted (delete denied); no retro (not a Monday night)
- 2026-10-31 builder: #98 M3-04e → PR #103 (02dfbb8; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-10-31 builder: #91 M3-01b → PR #102 (6c27aa8; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-10-30 documenter: M3-01a → PR #101 merged (CI green, 3 checks; 76 pages; no feedback; no issues opened)
- 2026-10-30 reviewer: no PRs to review (none labeled factory:review; #95 and #96 merged at 1:20am); no new issues (ready: #91, #97, #98; #97 depends on #91); no retro (not a Monday night)
- 2026-10-30 builder: idle (nightly cap reached; 12:17am run took #90 → PR #95 and #92 → PR #96)
- 2026-10-30 reviewer: merged #95 (M3-01a) and #96 (M3-04a spec) (CI green, 3 checks each; QA reports + ship reports + 2 images verified; local gates on main+both exit 0, incl. swx-kernel occt); no roadmap check-off (M3-01 needs #91, M3-04 needs M3-04b..g); planned #97 M3-02a, #98 M3-04e (ready: #91, #97, #98; #97 depends on #91); merged branches not deleted (delete denied); no retro (not a Monday night)
- 2026-10-30 builder: #92 M3-04a → PR #96 (fc7aaf0; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-10-30 builder: #90 M3-01a → PR #95 (80f5c3a; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-10-29 documenter: M2-08b + M2-09 → PR #94 merged (CI green, 3 checks; 72 pages; no feedback; no issues opened)
- 2026-10-29 reviewer: merged #84 (M2-09) and #89 (M2-08b) (CI green, 3 checks each; QA reports + ship reports + 3 images verified; local gates on main+both exit 0); checked off M2-08 and M2-09 (M2 complete); planned #90 M3-01a, #91 M3-01b, #92 M3-04a spec (ready: #90, #92; #91 depends on #90); merged branches not deleted (delete denied); no retro (not a Monday night)
- 2026-10-29 builder: idle (nightly cap reached; 12:17am run took #73 → PR #84 and #79 → PR #89)
- 2026-10-29 builder: #79 M2-08b → PR #89 (d6e56e7; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-10-29 builder: #73 M2-09 → PR #84 merged main + arc ends in rule (a) + QA report (e1974ea; CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-10-28 documenter: M2-04a + M2-08a + M2-07b → PR #88 merged (CI green, 3 checks; 65 pages; no feedback; no issues opened)
- 2026-10-28 reviewer: merged #83 (M2-07b) (CI green, 3 checks; QA report + ship report + 2 images verified; local gates on main+#83 exit 0); checked off M2-07; sent back #84 (M2-09: no QA report, conflicts with main after #83); no new issues (ready: #79; every open M2 item has an issue; ROADMAP bars M3); merged branch not deleted (delete denied); no retro (not a Monday night)
- 2026-10-28 builder: idle (nightly cap reached; 12:17am run took #72 → PR #77 and #76 → PR #78)
- 2026-10-28 reviewer: merged #77 (M2-08a) and #78 (M2-04a) (CI green, 3 checks each; QA reports + ship reports + 4 images verified; local gates on main+both exit 0); no roadmap check-off (M2-08 open via #79, M2-04 already checked); no new issues (ready: #79; every M2 item has an issue; ROADMAP bars M3); #83 and #84 wait for the next run; merged branches not deleted (delete denied); no retro (not a Monday night)
- 2026-10-28 builder: #76 M2-04a → PR #78 merged main + QA report (d18d8d6; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-10-28 builder: #72 M2-08a → PR #77 merged main + QA report (938642e; CI green, 3 checks; think-and-ship gates verified; QA pass with gaps)
- 2026-10-27 reviewer: sent back #77 (M2-08a: conflicts with main) and #78 (M2-04a: no QA report); no new issues (all remaining M2 items have issues; ROADMAP bars M3 until M2 is checked off; ready: #79, which depends on #72); #83 and #84 wait for the next run; no retro (not a Monday night)
- 2026-10-27 builder: #71 M2-07b → PR #83 (CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-10-26 documenter: M2-07a + M2-05a → PR #82 merged (CI green, 3 checks; 60 pages; no feedback; no issues opened)
- 2026-10-26 reviewer: merged #69 (M2-07a) and #70 (M2-05a) (CI green, 3 checks each; QA reports + ship reports verified; local gates on main+both exit 0); no roadmap check-off (M2-07 still open, M2-05 already checked); planned #79 M2-08b (ready: #71, #73, #79); #77 and #78 wait for the next run; retro PR #80 (needs-human, 2 changes)
- 2026-10-26 builder: idle (nightly cap reached; 12:17am run took #72 → PR #77 and #76 → PR #78 but posted no outcome comments; #78 has no QA report)
- 2026-10-25 documenter: M2-04 + M2-06 → PR #75 merged (CI green, 3 checks; 58 pages; no feedback; opened #76)
- 2026-10-25 reviewer: merged #65 (M2-04) and #66 (M2-06) (CI green, 3 checks each; QA reports + ship reports + 4 images verified; local gates on merge results exit 0); checked off QA-02, M2-04, M2-05, M2-06 (superseding stale docs PR #64); planned #71 M2-07b, #72 M2-08a, #73 M2-09 (ready: #71, #72, #73); #69 and #70 wait for the next run
- 2026-10-25 builder: idle (nightly cap reached; 12:17am run took #68 → PR #70 and #62 → PR #69)
