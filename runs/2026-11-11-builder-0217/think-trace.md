### Step 1/10: PICK WORK FOR THE 02:17 BUILDER RUN, NIGHT 2026-11-11

**Context:** Run log shows 2 builder items tonight (cap 4). Main CI green. No priority, wip or changes issues. Ready: #183 M3-12c, #184 M4-01a, #189 M4-01b (depends on #184), #190 MB-03a.

**Thought:** Lowest roadmap item is #183 (M3-12c, swx-kernel). Its deps #162, #163 are closed. Second in order is #184 (M4-01a, new swx-io crate), independent of #183 and in a different crate, which §3 prefers.

**Outcome:** Take #183 first, then #184, one at a time.

**Next Action:** Branch feat/183-tapped-hole from origin/main and implement HoleKind::Tapped.
- *Rationale:* §3 step 3.4 roadmap order; step 3 second item independent and in a different crate.

**Execution Ref:** `issue:#183`

---

### Step 2/10: QA VERDICT FOR #183 / PR #192

**Context:** CI green on d9a49e7 (3 checks). 7 ship gates verified. Scratch program: M6 through 23607.3009 / 7 faces, wall box (17.5,12.5,0)-(22.5,17.5,20); pitch>=nominal InvalidInput; render shows M6 and M10 holes.

**Thought:** Implementation reuses the Simple cylinder plan, so the tapped hole is geometrically identical to Simple at D-P. No deviation from the issue. ship_start/ship_complete returned output-schema errors but state shows the task completed with 7 checks; finalize had no warnings.

**Outcome:** Verdict: pass. Every criterion verified; no gaps.

**Next Action:** Post QA report on PR #192, outcome comments, then start #184.
- *Rationale:* All six acceptance criteria met by tests and gates; QA scenario checks and image confirm.

**Execution Ref:** `issue:#183`

---

### Step 3/10: RESOLVE A LICENSE CONFLICT IN #184'S DEPENDENCY LIST

**Context:** Issue asks for serde (feature derive) + serde_json. derive pulls serde_derive -> syn/proc-macro2 -> unicode-ident, licensed (MIT OR Apache-2.0) AND Unicode-3.0. ARCHITECTURE.md allowed licenses: MIT, Apache-2.0, BSD, Zlib, ISC, MPL-2.0, LGPL (dynamic). Unicode-3.0 is not listed, and the allowed list is protected (only register rows/ADR lines may be appended).

**Thought:** Options: (1) keep derive and flag needs-human; (2) drop the derive feature and build the manifest's JSON value by hand. serde without derive plus serde_json pulls only serde, serde_core, serde_json, itoa, memchr (Unlicense OR MIT), zmij (MIT): all allowed. The public API (Manifest, read/write, to_canonical_json<T: Serialize>) is unchanged. ARCHITECTURE is binding and outranks the issue text.

**Outcome:** Use serde without the derive feature; write Manifest to a serde_json::Value by hand. Check criterion 1 and explain the deviation in the PR and outcome comment; flag the Unicode-3.0 question for the owner for when derive is needed later.

**Next Action:** Remove derive from Cargo.toml and replace derived Serialize with a hand-built Value.
- *Rationale:* Keeps every dependency inside the allowed-license list without an owner decision, at the cost of ~25 lines.

**Execution Ref:** `issue:#184`

---

### Step 4/5: QA VERDICT FOR #184 / PR #193

**Context:** CI green on b722eb1 (3 checks). Gates: fmt, clippy pass; cargo test first record exit 2 because of bash-only PIPESTATUS under sh (all tests passed), re-run plain: pass. Scratch: golden -> change saved_by -> diff shows one changed line; format_version 1.0.0 -> UnsupportedVersion.

**Thought:** Every criterion is met and tested. Criterion 1 is met differently from its text (serde without derive), for the license reason in step 3. That is an explained deviation, not a gap in QA coverage, but the Reviewer and owner should see it. Nothing visual.

**Outcome:** Verdict: pass with gaps. Gap: the derive deviation and the lock-only Unicode-3.0 crate need the owner/Reviewer to accept them.

**Next Action:** Post QA report on PR #193, outcome comments, export think trace.
- *Rationale:* Listing the deviation under known gaps is more honest than a plain pass.

**Execution Ref:** `issue:#184`

---
