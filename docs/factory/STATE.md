# Factory state

status: running
issues_per_night: 2

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->
<!-- issues_per_night caps how many issues the Builder takes per night, across its runs (FACTORY §3). -->

## Notes for the factory

- Owner priority: steady progress on a tight budget. Up to `issues_per_night` issues per night, at most two per Builder run (FACTORY §3).
- Current focus: M2 Sketcher in roadmap order. QA-01 (#32) has priority and is independent of the M2 chain (M2-02 → M2-03 → M2-04).

## Log

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
- 2026-10-21 builder: #22 M2-02 → PR #48 QA report posted (send-back fix; CI green, 3 checks; think-and-ship gates verified)
- 2026-10-20 builder: idle (no qualifying item; #25/#28 wait on #22; #47/#48 still await review)
- 2026-10-20 builder: idle (no qualifying item; #25/#28 wait on #22; #47/#48 await review)
- 2026-10-19 builder: idle (nightly cap reached; #47 and #48 opened by the 12:17am run lack QA reports and outcome comments)
- 2026-10-18 reviewer: merged #38 (QA-01; CI green, 3 checks; QA report and 3 images checked); checked off QA-01; no new issues needed (ready: #22, #25, #28, #39); branch feat/32-swx-qa-render not deleted (API write blocked)
- 2026-10-18 builder: #32 QA-01 → PR #38 (CI green; 3 checks; no think-and-ship)
- 2026-10-17 reviewer: merged #27 (M2-01; CI green, 3 checks); checked off M2-01; planned #28 M2-04 (ready: #22, #25, #28); branch feat/18-sketch-geometry not deleted (API write blocked)
- 2026-10-17 builder: #18 M2-01 → PR #27 (CI pending at log time)
- 2026-10-16 reviewer: no open PRs; owner merged #24 (M1-05); checked off M1-05; planned #25 M2-03 (ready: #18, #22, #25)
- 2026-10-16 builder: #9 M1-05 → PR #24 (CI green; 3 checks; owner-merge: adds libocct-data-exchange-dev to CI)
- 2026-10-15 reviewer: merged #20 (M1-04; CI green, 3 checks); checked off M1-04; planned #22 M2-02 (ready: #9, #18, #22); branch feat/8-tessellation not deleted (API write blocked)
