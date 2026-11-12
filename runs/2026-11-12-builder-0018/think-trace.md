### Step 1/10: PICK TONIGHT'S BUILDER ITEMS

**Context:** Night 2026-11-12, run 00:18 PT. Tonight's count 0 of 4, so up to 2 items. main CI green (ffe6caf). No priority, factory:changes or factory:wip issues. Ready: #197 M3-12d, #189 M4-01b (dep #184 closed), #199 M4-01c, #190 MB-03a, #194 MB-03b.

**Thought:** Roadmap order puts M3-12d first. Next is M4-01b (#189), which is in swx-io while #197 is in swx-kernel; neither depends on the other.

**Outcome:** Take #197 first, then #189, one at a time.

**Next Action:** Claim #197 and branch fix/197-csk-error-noise.
- *Rationale:* FACTORY §3 step 3.4 order, and the second item is independent and in a different crate.

**Execution Ref:** `issue:#197`

---

### Step 2/10: QA VERDICT FOR #197 / PR #200

**Context:** Approach: private helper rounds computed values to 6 decimals for the message only; angle error adds degrees. CI green on e189fa8 (3 checks). All 7 gate commands verified; one occt test record failed on sh shell syntax (PIPESTATUS) and was re-run with the plain command, passing. Scenarios: hole_screw example shows 'countersink depth 2.8' and '(180°)'; scratch example shows 5.196152 for a 60° countersink and 229.183118° for 4 rad.

**Thought:** Every criterion is met and verified by tests and hands-on runs. Nothing visual: the change is error text only. No gaps to list.

**Outcome:** QA verdict: pass.

**Next Action:** Post the QA report on PR #200, then outcome comments, then start #189.
- *Rationale:* FACTORY §6b: pass requires all criteria verified and no gaps.

**Execution Ref:** `issue:#197`

---

### Step 3/10: APPROACH FOR #189 M4-01B

**Context:** swx-io has Manifest/read_manifest/write_manifest/to_canonical_json; FormatError derives Clone+Eq, so std::io::Error can't be stored directly.

**Thought:** New module unpacked.rs with Document, read_unpacked, write_unpacked. FormatError gains Io { path: PathBuf, message: String }, MissingFile(String) and ExtensionMismatch { extension: Option<String>, kind: DocumentKind }. A path that is a file is the Io variant ('not a directory'). Non-object model/properties is InvalidField naming the file. write_unpacked also checks the extension against the kind and that model is an object before creating anything, so it never writes a folder read_unpacked would reject. Tests use a Drop guard so temp folders are removed even on failure. Fixture model.json generated in canonical form and confirmed byte for byte by the round-trip test.

**Outcome:** Implement in one module plus FormatError variants; no new dependencies.

**Next Action:** Write the fixture folder, then unpacked.rs and tests.
- *Rationale:* Smallest change meeting the criteria; the write-side checks are a small addition that keeps written folders readable, noted as a deviation in the outcome comment.

**Execution Ref:** `issue:#189`

---

### Step 4/5: QA VERDICT FOR #189 / PR #201

**Context:** CI green on 051e01f (3 checks). fmt/clippy/test verified through ship_check. 9 new swx-io tests, all passing on the first run. Scratch: fixture read, saved_by changed, written; diff -r shows one changed line in manifest.json. Assembly manifest in x.sltwx gives ExtensionMismatch.

**Thought:** All criteria met and verified by tests plus hands-on runs. Deviation: write_unpacked also refuses an extension/kind mismatch and non-object model/properties before writing; this adds checks but changes no stated behavior. Nothing visual (file-format data). No gaps.

**Outcome:** QA verdict: pass.

**Next Action:** Post QA report on PR #201, outcome comments, export think trace.
- *Rationale:* FACTORY §6b pass criteria are met.

**Execution Ref:** `issue:#189`

---
