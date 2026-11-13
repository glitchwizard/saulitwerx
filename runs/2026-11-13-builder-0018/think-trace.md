### Step 1: Pick this run's item(s)

Builder run, night 2026-11-13 (started 00:18 PT, resumed 15:26 PT). main CI green at 5ec8b09. #202 (M4-01d) was factory:wip from an interrupted run, with no branch or PR; its dependency #199 is closed. Roadmap order puts M4-01 before MB-03, so #202 goes first. #194 (MB-03b) is independent and would be second, decided after #202 is complete. (issue:#202)

### Step 2: Approach and QA verdict for #202

Wrote a minimal ZIP reader and writer in swx-io per the issue (APPNOTE, stored and deflate, CRC-32). One deviation: clippy flagged `is_none_or` as newer than the 1.80 MSRV, so `is_some_and` is used. The shared ExtensionMismatch message says "folder" for files. Verdict: pass with gaps (extension message wording; ship_start and ship_complete returned output-schema errors but took effect; nothing visual). (issue:#202)
