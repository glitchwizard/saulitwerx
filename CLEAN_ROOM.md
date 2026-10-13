# Clean-room policy

Saulitwerx is an independent implementation of well-known CAD concepts. These concepts are decades old and described in public textbooks and papers: parametric features, constraint sketching, B-rep solids, and STEP exchange. To keep the project legally clean and safe for everyone who uses it:

## Never

- Copy, decompile, disassemble, or inspect the binaries of any commercial CAD product.
- Use another product's name, logos, icons, splash screens, sounds, or documentation text in the code, UI, or assets. Comparing to other products in prose is fine; branding is not.
- Reproduce another product's UI layout pixel for pixel, or its proprietary terminology where a generic term exists. Use "feature tree", not a vendor's trademarked panel name.
- Read or write proprietary native file formats such as `.SLDPRT`, `.SLDASM`, `.ipt` or `.prt`. Interoperability goes through open standards: STEP (ISO 10303), IGES, STL, 3MF and DXF.
- Paste code from sources whose license is incompatible with MIT/Apache-2.0. GPL code may not be copied into this repo. LGPL libraries may only be used as separate, dynamically linked dependencies.

## Always

- Work from public references: textbooks, papers, open standards, and the documentation of open-source projects.
- Cite the algorithm or paper in a doc comment when implementing something non-obvious.
- Record any new third-party dependency, with its license, in `docs/ARCHITECTURE.md`.

If you are unsure whether something crosses a line, don't do it. Open an issue labeled `needs-human`.
