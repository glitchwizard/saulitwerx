# Factory state

status: running

<!-- Set status to "paused" to stop all factory runs immediately. Any other value also stops them. -->

## Notes for the factory

- Owner priority: slow and steady, budget first. One issue per night.
- Next focus: M1 kernel spike (M1-01). Don't start M2 until M1-01 is merged.

## Log

- 2026-10-13 reviewer: merged nothing (no ci.yml on main, #3); PRs #4 #5 #6 await owner (#5 and #6 touch protected files); planned #7 M1-03, #8 M1-04, #9 M1-05
- 2026-10-13 builder: idle (no ready issues; #1-#3 needs-human, CI pending #5)
- 2026-10-12 bootstrap: M0 complete (workspace, swx-core, factory docs); seeded M1 issues
