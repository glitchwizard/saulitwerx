// Saulitwerx user manual: entry point.
// Build: docs/manual/build.sh docs/manual/Saulitwerx-User-Manual.pdf
// Outline and conventions: docs/manual/STYLE.md.

#import "template.typ": *

#show: manual.with(
  commit: sys.inputs.at("commit", default: "unknown"),
  built: sys.inputs.at("built", default: "unknown"),
)

// ------------------------------------------------------------ front matter

#include "chapters/00-about.typ"

#page(header: none)[
  #v(40pt)
  #text(font: sans, size: 24pt, weight: "bold", "Contents")
  #v(4pt)
  #line(length: 40pt, stroke: 2.5pt + accent)
  #v(14pt)
  #show outline.entry.where(level: 1): it => {
    v(10pt, weak: true)
    text(font: sans, weight: "semibold", it)
  }
  #set text(size: 9.8pt)
  #outline(title: none, depth: 2, indent: 1.4em)
]

// ------------------------------------------------------------ main matter

#pagebreak(to: "odd", weak: true)
#set page(numbering: "1")
#counter(page).update(1)

#part("Getting started", [What Saulitwerx is, what it can do today, how to install it, and a first session that ends with a picture of a real part.])
#include "chapters/01-welcome.typ"
#include "chapters/02-install.typ"
#include "chapters/03-first-part.typ"

#part("Concepts", [The ideas behind parametric CAD, in plain words. Read this before the task chapters; every procedure later relies on these terms.])
#include "chapters/04-concepts.typ"

#part("Tasks", [How to do things, one task per section. Until the desktop app arrives, every task here runs a real command or a small program, and is marked for developers.])
#include "chapters/05-sketching.typ"
#include "chapters/06-solids.typ"
#include "chapters/07-pictures.typ"
#include "chapters/08-history.typ"

#part("Reference", [Every command, option, operation and setting that exists today, with its default and its allowed values.])
#include "chapters/09-reference.typ"

#part("Help", [What to do when something goes wrong: symptoms, every error message you can see, common questions and known limitations.])
#include "chapters/10-help.typ"

#part("Back matter", [Glossary, what changed, what comes next, a one-page card, credits and licenses, and the index.])
#include "chapters/11-back.typ"
