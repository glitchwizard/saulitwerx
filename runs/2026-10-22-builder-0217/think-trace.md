### Step 1/4: PICK TONIGHT'S ITEM

**Context:** Night 2026-10-22, 2:17am builder run. 12:17am run took #25 (count 1 of 2). main CI green (run 76). No priority/changes issues.

**Thought:** Ready issues: #28 depends on #25 (open, in review), so it's skipped. #49 depends on #22 (closed). That leaves one slot, so this run takes one item.

**Outcome:** Take #49 M2-05 DOF analysis only.

**Next Action:** Implement swx-sketch::dof
- *Rationale:* FACTORY §3 order and nightly cap.

**Execution Ref:** `issue:#49`

---

### Step 2/4: APPROACH FOR RANK AND REDUNDANT ROWS

**Context:** Issue asks for rank via rank-revealing QR with relative tol 1e-9×max|Rii|, and redundant = rows dependent on EARLIER rows.

**Thought:** Plain column pivoting on J finds the rank but not which rows. Instead, do Householder QR of Jᵀ and process its columns (the residual rows) in their original order. A column whose remaining norm is ≤ τ is dependent on the earlier accepted ones, so it is marked redundant and skipped. The tolerance is τ = rel × the max column norm of Jᵀ, which equals the first |R11| of the Businger–Golub pivoted QR and bounds every |Rii|. Rank is the number of accepted columns, so m − rank == redundant.len().

**Outcome:** One Householder pass gives both the rank and the ordered redundant set. The forward difference is reused from solver (pub(crate)).

**Next Action:** Write dof.rs with tests (a)–(f) plus the non-finite case
- *Rationale:* Meets both criteria consistently, with no new deps.

**Execution Ref:** `issue:#49`

---

### Step 3/4: QA VERDICT FOR #49 / PR #52

**Context:** Gates verified (fmt, clippy, test: exit 0). Scratch checks: triangle with 3 lengths and a fixed vertex gives dof 1; adding horizontal AB gives FullyConstrained; x=1, y=2, x+y=3 gives OverConstrained with redundant [2]; an infinite param gives NonFinite; a wrong length gives DimensionMismatch.

**Thought:** Every criterion is met and tested. Nothing is visual: swx-qa sketch (QA-02) doesn't exist yet, so that is a known gap. One deviation from the issue: the method is ordered Householder QR, not largest-norm pivoting, so that the redundant rows are the later ones. It is documented in the module doc and the PR.

**Outcome:** Verdict: pass with gaps (no sketch visual tool yet).

**Next Action:** Post the QA report once CI finishes, then the outcome comments
- *Rationale:* FACTORY §6b: a gap in (e) means pass with gaps.

**Execution Ref:** `issue:#49`

---
