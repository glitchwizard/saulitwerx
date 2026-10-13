# Saulitwerx — agent instructions

You are working on Saulitwerx, an open-source parametric CAD modeler written in Rust.

**Before doing anything, read `docs/factory/FACTORY.md` and follow it.** It defines your role, the issue queue, the git rules, and the budget limits. If `docs/factory/STATE.md` does not say `status: running`, stop.

## Binding rules (summary; FACTORY.md has the detail)

- Never commit or push to `main`. Use branches `feat/…`, `fix/…`, `docs/…` or `chore/…` and open PRs.
- No AI attribution anywhere: no `Co-Authored-By` trailers, no "Generated with …" lines.
- Follow `CLEAN_ROOM.md`: no vendor names or assets, no proprietary formats, no GPL code.
- Follow `docs/ARCHITECTURE.md`: crate dependency direction, allowed licenses, testing strategy.
- Before pushing, this must pass: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
- Keep `unsafe` out of all crates except a kernel FFI module, and document every `unsafe` block there.
- Every non-obvious algorithm gets a doc comment citing its public source (paper, textbook, standard).
