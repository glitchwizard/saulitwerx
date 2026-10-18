# Plan: think-and-ship in the factory

Status: **phases 1 and 2 adopted** (FACTORY §6c and the §4 retro). Written 2026-10-16, updated 2026-10-17. Both need the owner's setup-script step below; until then runs use the §6c fallback.

## What it is

[think-and-ship](https://github.com/AlrikOlson/think-and-ship) 0.6.0 (MIT) is a single-binary MCP server with four tool families and a set of agent skills:

| Family | Records |
|---|---|
| `ship_*` | Objectives, tasks, and quality gates. Given a command, `ship_check` runs it itself and records the real exit code. |
| `think_*` | Reasoning traces: steps, branches, revisions. |
| `roadmap_*` | The roadmap as native server state, exported back to `ROADMAP.md`. |
| `signal_*` | A stakeholder-feedback inbox that feeds the roadmap. |

The skills `switch-work` and `advance-work` drive the families with their own protocol. The runs call it as a tool; Saulitwerx never links it, so it is not a product dependency and does not belong in the `ARCHITECTURE.md` register.

## Measured in a cloud container

| Check | Result |
|---|---|
| `npm install think-and-ship` | Hangs. Its postinstall downloads the binary with Node's `https` module, which ignores the container's proxy. |
| `curl` of the release tarball and `SHA256SUMS` from GitHub | 1 s. Checksum matches. |
| `ship_check` with a command | The server runs it. Exit 0 is recorded `passed: true, verified: true`; exit 3 is `passed: false, verified: true`. |
| Tool schemas, all families | 53 tools, 165 KB |
| Tool schemas, `ship` only | 13 tools, 28 KB |
| Tool schemas, `ship` + `think` | 24 tools, 71 KB |
| `roadmap import --dry-run` of `docs/ROADMAP.md` | 12 chunks, one per milestone, not one per item. M0 and M1 come out `pending` although every item is checked. |

## Where it conflicts with how the factory works

1. **State dies with the container.** Each routine run starts fresh, and the data directory lives under `$HOME`. The only built-in ways to keep it are mirroring into the repo or the author's hosted cloud backend.
2. **Mirroring into the repo breaks two rules.** With `THINK_AND_SHIP_SYNC_TARGET=repo-git` the server makes its own commits (`chore(trace): session …`) on whatever branch is checked out. Its code-action records carry `contributor.type: "ai"` and a model id (the Agent Trace format). That is AI attribution in the repo, and commits on PR branches that the Builder didn't make.
3. **The cloud backend** sends reasoning and execution records to a third-party service, and needs a long-lived token in the routine environment.
4. **`roadmap_*` would be a second source of truth** beside `ROADMAP.md` and the issue queue, and its importer doesn't read our format (see the table above).
5. **The bundled skills are a second protocol.** `advance-work` selects work from native roadmap chunks and a per-lane focus. `FACTORY.md` already defines how a run picks and finishes work; two protocols in one run is how runs go wrong. Its discipline (one unit, real gates, a receipt) already matches §3 and §6b.
6. **`ship_*` state is one file per project, and its tools take no session id.** Two parallel Builder subagents sharing one server would overwrite each other's objective.
7. **Context cost.** With every family on, 165 KB of tool schemas join each run.
8. **`think-and-ship init --full` writes `CLAUDE.md`**, an owner file. Never run it. `init` without `--full` is also unnecessary: the config below is written by hand.

## Recommendation: phase 1 adopts `ship_*` only

Use `ship_check` as the evidence behind QA report section (b), keep all state inside the run, and publish the ship report where QA images already go (`qa-artifacts`).

**Install (owner: cloud environment, Setup script).** Pinned version and checksum, no npm. A failed install doesn't stop the run; the run falls back (below).

```sh
TAS_VERSION=0.6.0
TAS_SHA256=79204189f7c08850cd635a30a9bdd69fbea6037a3c2436b63e9018fce3ec290e
TAS_TGZ="think-and-ship-v${TAS_VERSION}-x86_64-unknown-linux-gnu.tar.gz"
( cd /tmp \
  && curl -fsSLO "https://github.com/AlrikOlson/think-and-ship/releases/download/v${TAS_VERSION}/${TAS_TGZ}" \
  && echo "${TAS_SHA256}  ${TAS_TGZ}" | sha256sum -c - \
  && tar xzf "${TAS_TGZ}" think-and-ship \
  && install -m 0755 think-and-ship /usr/local/bin/think-and-ship \
) || echo "think-and-ship install failed; runs fall back per FACTORY §6c"
```

**Config (committed, in the same PR as the FACTORY change):**

- `.mcp.json`:

  ```json
  {
    "mcpServers": {
      "think-and-ship": {
        "command": "think-and-ship",
        "args": ["serve"],
        "env": {
          "THINK_AND_SHIP_FAMILIES": "ship",
          "THINK_AND_SHIP_PERSIST": "true",
          "THINK_AND_SHIP_AUTO_IMPORT": "false",
          "THINK_AND_SHIP_CALL_COUNTS": "off"
        }
      }
    }
  }
  ```

  `THINK_AND_SHIP_SYNC_TARGET` stays at its default, `local`. No `THINK_AND_SHIP_MODEL_ID`, no cloud variables. Telemetry is off by default and stays off.
- `.claude/settings.json` with `"enabledMcpjsonServers": ["think-and-ship"]`, so an unattended run doesn't stop at an MCP approval prompt.
- `.think-and-ship/project.json` from `think-and-ship project mark --name saulitwerx`, so every worktree resolves the same project id.

**Builder (a FACTORY §3 and §6b change):**

1. Before the gates, warm the build in Bash (`cargo test --workspace --no-run`), so no gate compiles from cold. The server kills a check after 900 s.
2. Per issue: `ship_set_objective` with the issue's acceptance criteria copied verbatim, one task, then `ship_check` with the exact command for each gate: fmt, clippy and test, plus the `occt` commands when the change touches `swx-kernel`.
3. `ship_finalize`, then `ship_export` to `qa-artifacts/pr-<N>/ship-report.md`.
4. QA report (b) quotes each check's exit code and `verified` flag and links the ship report.
5. Two issues in one run: the subagents implement in their own worktrees. The parent run then does steps 2–4 for each issue in turn, with each command prefixed by `cd <worktree> && CARGO_TARGET_DIR=<its dir>`.

**Reviewer:** section (b) must match the ship report. A required check with `verified: false` or a non-zero exit fails the PR.

**Fallback:** if the server isn't available, (b) says so, the gap goes in (e), and the run reports gates as it does today. The verdict can then be at most `pass with gaps`.

**Cost:** 28 KB of tool schemas per run (Claude Code defers large tool sets behind tool search), about 8 small calls per issue, and no extra test time: the gates run anyway, now through the server.

## Phase 2: the factory iterates on itself

The owner asked for the factory to improve itself with think-and-ship, the way think-and-ship's own repo is developed with its own tools.

- **Reasoning traces.** Builder and Reviewer runs record their decisions with `think_record_step` and export the trace to `qa-artifacts/runs/<night>-<role>-<HHMM>/think-trace.md`. The export is plain markdown with no model attribution (checked with 0.6.0).
- **Weekly retro.** Each Monday night the Reviewer reads the week's traces, ship reports, outcome comments and send-backs. It opens one `needs-human` PR with at most three FACTORY.md changes, each backed by evidence and an expected effect. Next week's retro checks whether each change worked.
- **The owner stays the gate.** FACTORY.md stays protected; the retro PR is the one exception, and only the owner merges it.
- **Cost.** `.mcp.json` now enables `ship,think`: 71 KB of tool schemas instead of 28 KB, plus about 6 short steps per item. Persisting native state across runs was rejected: the server starts before a run could restore it, and the exports carry what the retro needs.

## Not now

- **`roadmap_*`, `signal_*`, tracker sync, cloud `connect`, and the bundled skills:** no, for the reasons above.

## Owner decisions

1. Approve phase 1 as scoped: `ship_*` only, state local to each run.
2. Add the install lines to the Setup script of the cloud environment the Builder and Reviewer routines use.
3. Merge the FACTORY + config PR when it comes; `FACTORY.md` is an owner file.

## Sequencing

After queue item 2 (two tickets per night) merges. Phase 1 changes the same Builder steps and has to handle parallel subagents (conflict 6).
