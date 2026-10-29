# Factory state

status: running
issues_per_night: 2

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->
<!-- issues_per_night caps how many issues the Builder takes per night, across its runs (FACTORY §3). -->

## Notes for the factory

- Owner priority: steady progress on a tight budget. Up to `issues_per_night` issues per night, at most two per Builder run (FACTORY §3).
- Current focus: M2 Sketcher in roadmap order. QA-01 (#32) has priority and is independent of the M2 chain (M2-02 → M2-03 → M2-04).

## Log

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
- 2026-10-25 builder: #62 M2-07a → PR #69 (CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-10-25 builder: #68 M2-05a → PR #70 (CI green, 3 checks; think-and-ship gates verified; QA pass)
- 2026-10-24 documenter: M2-05 + QA-02 → PR #67 merged (CI green, 3 checks; 50 pages; no feedback; opened #68)
- 2026-10-24 builder: #55 M2-06 → PR #66 (CI green, 3 checks; QA report posted; think-and-ship gates verified)
- 2026-10-24 builder: #28 M2-04 → PR #65 (CI green, 3 checks; QA report posted; think-and-ship gates verified)
- 2026-10-23 reviewer: merged #52 (M2-05) and #61 (QA-02) (CI green, 3 checks each; QA reports + ship reports + 2 images verified; local gates on main+both exit 0); docs PR #64 left open, no run-log comment
- 2026-10-23 builder: #49 → PR #52 QA report posted (6753e68), back to factory:review; #54 → PR #61 QA report posted (e989880), back to factory:review
- 2026-10-23 builder: #49 partial → PR #52 (6753e68); #54 partial → PR #61 (e989880); QA reports pending
- 2026-10-23 reviewer: sent back #52 (M2-05: QA report is for c08d404, not head 6753e68) and #61 (QA-02: no QA report); planned #62 M2-07a (ready: #28, #55, #62)
- 2026-10-23 builder: idle (nightly cap reached; 12:17am run took #49 → PR #52 and #54 → PR #61 but posted no QA reports or outcome comments)
- 2026-10-22 documenter: first manual edition → PR #60 merged (CI green, 3 checks; 45 pages; no feedback; no issues opened)
- 2026-10-22 reviewer: idle (no PR labeled factory:review; #52 awaits builder fix for conflicts; ready: #28, #54, #55)
- 2026-10-22 reviewer: merged #51 (M2-03; CI green, 3 checks; QA report + ship report verified; local gates on merge result exit 0); checked off M2-03; sent back #52 (M2-05: conflicts with main after #51); planned #54 QA-02 and #55 M2-06 (ready: #28, #54, #55)
- 2026-10-22 builder: #49 M2-05 → PR #52 (CI green, 3 checks; think-and-ship gates verified)
- 2026-10-22 builder: #25 M2-03 → PR #51 (CI green, 3 checks; think-and-ship gates verified)
- 2026-10-21 reviewer: merged #48 (M2-02; CI green, 3 checks; QA report + ship report verified; local gates on merge result exit 0); checked off M2-02; no new issues needed (ready: #25, #28, #49)
- 2026-10-21 builder: idle (no qualifying item; #25/#28/#49 wait on #22 in review)
