#import "../template.typ": *
#set heading(numbering: none, outlined: false)

#heading(numbering: none, outlined: false)[How to use this manual]

This manual teaches you to use Saulitwerx, an open-source program for designing mechanical parts on a computer. You don't need to know anything about computer-aided design (CAD) to read it. Where a word has a special meaning, the manual explains it the first time it appears, and again in the glossary at the back.

== Saulitwerx is young

Saulitwerx is being built in the open, one small step each night. Today it is *pre-alpha*: the parts that do the real geometry work exist and are tested, but there is no desktop app with windows and buttons yet. That changes what this edition of the manual can offer:

- The *Concepts* part is for everyone. It explains how parametric CAD works, with pictures, so you will be ready when the app arrives.
- The *Tasks* part shows what you can already do, by running real commands and small programs. These sections carry a purple *For developers* band, because they need a terminal and some comfort with installing software.
- Nothing in this manual describes a feature as working unless it works on the version named on the cover. Plans are kept apart, in *Coming next* at the back.

== How the book is organized

#ref-table(
  columns: (auto, 1fr),
  header: ("Part", "Read it when you want to"),
  [I. Getting started], [Know what Saulitwerx is, install it, and make your first picture of a part.],
  [II. Concepts], [Understand sketches, constraints, dimensions, parameters, features and solids.],
  [III. Tasks], [Do one specific thing. Each section is one task, with numbered steps.],
  [IV. Reference], [Look up an exact option, default or allowed value.],
  [V. Help], [Fix a problem, or understand an error message.],
  [Back matter], [Look up a word, see what changed, or find a topic in the index.],
)

== Conventions

#ref-table(
  columns: (auto, 1fr),
  header: ("You see", "It means"),
  [`swx-qa render`], [Text you type exactly, or text the program prints.],
  [#key("Enter")], [A key on your keyboard.],
  [_sketch_], [A term defined at this spot. It is also in the glossary and the index.],
  [#box(width: 15pt, height: 15pt, radius: 50%, fill: accent, align(center + horizon, text(font: sans, size: 8pt, weight: "bold", fill: white, "1")))], [A numbered step. Steps start with what to do, then say what you should see.],
  [#text(font: sans, size: 8pt, weight: "bold", fill: tip-c, "RESULT")], [What you have when a task is done.],
)

#tip[Callouts like this one hold shortcuts and good habits.]
#caution[Cautions warn about a mistake that is easy to make, and how to avoid it.]
#for-developers[Sections with this band run programs from a terminal. You can skip them and still follow every concept.]

== Which version this describes

The cover names the commit of the `main` branch this edition describes, and the date it was built. The latest edition is always at #link("https://github.com/glitchwizard/saulitwerx/blob/main/docs/manual/Saulitwerx-User-Manual.pdf")[github.com/glitchwizard/saulitwerx], in `docs/manual/`. *What's new*, near the back, lists what each edition added.
