// Saulitwerx user manual: page design and helpers.
//
// Core Typst only (no packages). Every visual convention used by the
// chapters lives here, so the book looks the same however many nights
// wrote it. See STYLE.md for when to use each helper.

// ---------------------------------------------------------------- tokens

#let ink = rgb("#1d2430")
#let muted = rgb("#5b6574")
#let rule-c = rgb("#d5d9e0")
#let paper = rgb("#ffffff")
#let accent = rgb("#3d5a80")       // steel blue, the colour of rendered parts
#let accent-soft = rgb("#e8edf4")
#let render-bg = rgb(245, 245, 245) // swx-qa render background
#let tip-c = rgb("#2f7d5b")
#let tip-soft = rgb("#e6f2ec")
#let caution-c = rgb("#b5532a")
#let caution-soft = rgb("#f8ebe4")
#let dev-c = rgb("#6b4fa0")
#let dev-soft = rgb("#efeaf6")

#let sans = ("Inter",)
#let serif = ("Libertinus Serif",)
#let mono = ("DejaVu Sans Mono",)

// ---------------------------------------------------------------- index

// Marks `term` for the index at this spot. Invisible.
#let idx(term) = [#metadata(term)<swx-index>]

// A defined term at its first use: italic, and indexed.
#let term(body, key: none) = {
  let k = if key == none { lower(if type(body) == str { body } else { body.text }) } else { key }
  idx(k)
  emph(body)
}

#let print-index() = context {
  let entries = (:)
  for m in query(<swx-index>) {
    let k = m.value
    let p = counter(page).at(m.location()).first()
    let pages = entries.at(k, default: ())
    if p not in pages { pages.push(p) }
    entries.insert(k, pages)
  }
  let keys = entries.keys().sorted(key: k => lower(k))
  set text(size: 9pt)
  set par(justify: false, first-line-indent: 0pt, spacing: 0.45em)
  columns(3, gutter: 14pt, {
    let letter = none
    for k in keys {
      let first = upper(k.first())
      if first != letter {
        letter = first
        v(0.6em, weak: true)
        text(font: sans, weight: "semibold", fill: accent, size: 9.5pt, first)
        v(0.2em)
      }
      block(spacing: 0.35em, [#k #h(1fr) #text(fill: muted, entries.at(k).map(str).join(", "))])
    }
  })
}

// ---------------------------------------------------------------- blocks

#let callout(kind, title, fill, stroke-c, body) = block(
  width: 100%,
  breakable: false,
  fill: fill,
  inset: (left: 12pt, right: 12pt, top: 9pt, bottom: 10pt),
  radius: (right: 3pt),
  stroke: (left: 3pt + stroke-c),
  spacing: 1.1em,
  {
    set par(first-line-indent: 0pt)
    text(font: sans, size: 8pt, weight: "bold", fill: stroke-c, tracking: 0.06em, upper(kind))
    if title != none { text(font: sans, size: 8.5pt, weight: "semibold", fill: ink, [ #h(4pt) #title]) }
    v(3pt)
    body
  },
)

#let note(title: none, body) = callout("Note", title, accent-soft, accent, body)
#let tip(title: none, body) = callout("Tip", title, tip-soft, tip-c, body)
#let caution(title: none, body) = callout("Caution", title, caution-soft, caution-c, body)

// A section written for developers (before the desktop app exists, how-tos
// run real programs). Marked with a purple band so lay readers can skip it.
#let for-developers(body) = callout("For developers", none, dev-soft, dev-c, body)

// A short "what you'll learn / what you need" box at the top of a task.
#let task-intro(goal: none, needs: none, time: none) = block(
  width: 100%,
  inset: (x: 12pt, y: 9pt),
  stroke: 0.6pt + rule-c,
  radius: 3pt,
  spacing: 1.2em,
  breakable: false,
  {
    set par(first-line-indent: 0pt)
    set text(size: 9.5pt)
    grid(
      columns: (auto, 1fr),
      column-gutter: 10pt,
      row-gutter: 6pt,
      ..if goal != none { (text(font: sans, size: 8pt, weight: "bold", fill: muted, "YOU WILL"), goal) },
      ..if needs != none { (text(font: sans, size: 8pt, weight: "bold", fill: muted, "YOU NEED"), needs) },
      ..if time != none { (text(font: sans, size: 8pt, weight: "bold", fill: muted, "TIME"), time) },
    )
  },
)

// Numbered steps with round markers. Each item: [Do this.] — and what you see.
#let steps(..items) = {
  let items = items.pos()
  block(spacing: 1.1em, grid(
    columns: (20pt, 1fr),
    row-gutter: 9pt,
    ..items.enumerate().map(((i, it)) => (
      align(top, box(
        width: 15pt, height: 15pt, radius: 50%, fill: accent,
        align(center + horizon, text(font: sans, size: 8pt, weight: "bold", fill: white, str(i + 1))),
      )),
      { set par(first-line-indent: 0pt); it },
    )).flatten(),
  ))
}

// "You should see" line after a step or a procedure.
#let result(body) = block(spacing: 0.9em, {
  set par(first-line-indent: 0pt)
  text(font: sans, size: 8pt, weight: "bold", fill: tip-c, "RESULT  ")
  body
})

// A terminal command or program output. `label` names the panel. Panels
// up to 24 lines stay on one page, so a label is never stranded at a page
// foot; longer ones may break.
#let term-block(label: "Terminal", body) = block(
  width: 100%,
  breakable: type(body) != str or body.split("\n").len() > 24,
  spacing: 1em,
  {
    block(
      width: 100%, spacing: 0pt, fill: rgb("#2b3340"), radius: (top: 3pt),
      inset: (x: 9pt, y: 4pt),
      text(font: sans, size: 7pt, weight: "semibold", fill: rgb("#c3cad6"), tracking: 0.05em, upper(label)),
    )
    block(
      width: 100%, spacing: 0pt, fill: rgb("#f3f4f7"), radius: (bottom: 3pt),
      stroke: (x: 0.6pt + rule-c, bottom: 0.6pt + rule-c),
      inset: (x: 9pt, y: 7pt),
      {
        set text(font: mono, size: 7.8pt)
        set par(justify: false, leading: 0.55em, first-line-indent: 0pt)
        if type(body) == str { body.split("\n").map(l => text(l)).join(linebreak()) } else { body }
      },
    )
  },
)

// A keyboard key or a literal value a user types.
#let key(k) = box(
  inset: (x: 3pt, y: 0pt), outset: (y: 2pt), radius: 2pt,
  stroke: 0.5pt + muted, fill: rgb("#fafafa"),
  text(font: mono, size: 0.85em, k),
)

// A figure from docs/manual/img/. Images are rendered by tools (§6d); the
// command that made each one sits in a comment next to the call.
//
// `marks` rings spots on the image without painting into it: an array of
// (x, y, label) with x and y as ratios of the image's width and height, and a
// short label (or none) set beside the ring.
#let shot(path, caption: none, alt: none, width: 100%, marks: ()) = figure(
  box(
    width: width, fill: render-bg, radius: 3pt, clip: true, stroke: 0.5pt + rule-c,
    {
      image(path, width: 100%, alt: alt)
      for (x, y, lab) in marks {
        place(top + left, dx: x - 10pt, dy: y - 10pt,
          circle(radius: 10pt, stroke: 1.6pt + caution-c))
        if lab != none {
          place(top + left, dx: x + 13pt, dy: y + 6pt,
            text(font: sans, size: 7.5pt, weight: "bold", fill: caution-c, lab))
        }
      }
    },
  ),
  caption: caption,
  kind: image,
)

// A row of images joined by operator signs drawn by the template, used for
// "this plus that gives this" pictures. `parts` is an array of
// (path, label) pairs; `ops` the signs between them. `marks`, if given, has
// one array of (x, y, label) rings per part, as in `shot`.
#let image-equation(parts, ops, caption: none, alt: none, marks: ()) = figure(
  {
    let cells = ()
    for (i, (p, lab)) in parts.enumerate() {
      cells.push(stack(spacing: 5pt,
        box(fill: render-bg, radius: 3pt, clip: true, stroke: 0.5pt + rule-c, {
          image(p, width: 100%, alt: lab)
          for (x, y, mlab) in marks.at(i, default: ()) {
            place(top + left, dx: x - 8pt, dy: y - 8pt,
              circle(radius: 8pt, stroke: 1.4pt + caution-c))
            if mlab != none {
              place(top + left, dx: x + 10pt, dy: y + 4pt,
                text(font: sans, size: 7pt, weight: "bold", fill: caution-c, mlab))
            }
          }
        }),
        align(center, text(font: sans, size: 8pt, fill: muted, lab)),
      ))
      if i < ops.len() {
        cells.push(align(center + horizon, text(font: sans, size: 18pt, weight: "bold", fill: accent, ops.at(i))))
      }
    }
    grid(columns: parts.map(_ => 1fr).intersperse(22pt), column-gutter: 4pt, ..cells)
  },
  caption: caption,
  kind: image,
)

// A row of labeled images with no signs between them, for "one start, several
// outcomes" pictures. `parts` is an array of (path, label, alt) triples.
#let image-row(parts, caption: none) = figure(
  grid(
    columns: parts.map(_ => 1fr),
    column-gutter: 8pt,
    ..parts.map(((p, lab, alt)) => stack(spacing: 5pt,
      box(fill: render-bg, radius: 3pt, clip: true, stroke: 0.5pt + rule-c, image(p, width: 100%, alt: alt)),
      align(center, text(font: sans, size: 8pt, fill: muted, lab)),
    )),
  ),
  caption: caption,
  kind: image,
)

// A plain reference table with a tinted header. Tables of up to eight rows
// stay on one page, so no row is left alone on the next.
#let ref-table(columns: auto, header: (), ..rows) = block(
  spacing: 1.1em,
  breakable: rows.pos().len() / calc.max(1, if type(columns) == array { columns.len() } else { 1 }) > 8,
{
  set par(justify: false, first-line-indent: 0pt)
  set text(size: 8.8pt)
  table(
    columns: columns,
    stroke: (x, y) => (bottom: if y == 0 { 0.8pt + accent } else { 0.4pt + rule-c }),
    fill: (x, y) => if y == 0 { accent-soft },
    inset: (x: 6pt, y: 5pt),
    align: left + top,
    table.header(..header.map(h => text(font: sans, weight: "semibold", size: 8pt, h))),
    ..rows,
  )
})

// Status pill for "available now / coming".
#let pill(body, c: accent) = box(
  inset: (x: 5pt, y: 1.5pt), outset: (y: 1pt), radius: 6pt, fill: c,
  text(font: sans, size: 7pt, weight: "bold", fill: white, upper(body)),
)
#let now = pill("Now", c: tip-c)
#let later = pill("Later", c: muted)

// ---------------------------------------------------------------- parts

#let part-counter = counter("swx-part")

// A part opener: a full page with the part number, title and a short
// summary of what it holds.
#let part(title, summary) = {
  pagebreak(weak: true)
  part-counter.step()
  page(header: none, footer: none, fill: accent, margin: (x: 72pt, y: 90pt), {
    set text(fill: white)
    v(1fr)
    context text(font: sans, size: 11pt, weight: "semibold", tracking: 0.2em, fill: rgb("#c9d6e8"),
      "PART " + numbering("I", part-counter.get().first()))
    v(6pt)
    text(font: sans, size: 34pt, weight: "bold", title)
    [#metadata(title)<swx-part>]
    v(14pt)
    block(width: 75%, text(size: 12pt, fill: rgb("#e3eaf3"), summary))
    v(2fr)
  })
}

// ---------------------------------------------------------------- document

#let manual(commit: "unknown", built: "unknown", body) = {
  set document(
    title: "Saulitwerx User Manual",
    author: "The Saulitwerx project",
    keywords: ("CAD", "parametric", "manual"),
  )
  set text(font: serif, size: 10.5pt, fill: ink, lang: "en", region: "us", hyphenate: true)
  set par(justify: true, leading: 0.62em, spacing: 0.95em, first-line-indent: 0pt)

  set page(
    paper: "us-letter",
    margin: (inside: 0.95in, outside: 0.85in, top: 0.95in, bottom: 0.9in),
    header: context {
      let p = here().page()
      let all = query(heading.where(level: 1))
      if all.any(h => h.location().page() == p) { return }
      let chapters = all.filter(h => h.location().page() < p)
      if chapters.len() == 0 { return }
      let ch = chapters.last()
      set text(font: sans, size: 7.5pt, fill: muted)
      let label = if ch.numbering != none {
        [#numbering("1", ..counter(heading).at(ch.location()).slice(0, 1)) #h(6pt) #ch.body]
      } else { ch.body }
      if calc.even(p) [Saulitwerx User Manual #h(1fr) #label] else [#label #h(1fr) Saulitwerx User Manual]
      v(-4pt)
      line(length: 100%, stroke: 0.4pt + rule-c)
    },
    footer: context {
      let p = counter(page).display()
      set text(font: sans, size: 8pt, fill: muted)
      if calc.even(here().page()) [#p #h(1fr)] else [#h(1fr) #p]
    },
  )

  // Headings: chapters (level 1) start on a new page with a large number.
  set heading(numbering: (..n) => {
    let n = n.pos()
    if n.len() == 1 { str(n.first()) } else if n.len() == 2 { numbering("1.1", ..n) } else { none }
  })
  show heading: set text(font: sans, fill: ink)
  show heading.where(level: 1): it => {
    pagebreak(weak: true)
    v(40pt)
    if it.numbering != none {
      text(font: sans, size: 9pt, weight: "semibold", fill: accent, tracking: 0.2em,
        "CHAPTER " + counter(heading).display("1"))
      v(4pt)
    }
    block(text(size: 24pt, weight: "bold", it.body))
    v(4pt)
    line(length: 40pt, stroke: 2.5pt + accent)
    v(18pt)
  }
  show heading.where(level: 2): it => block(above: 1.8em, below: 0.8em, sticky: true, {
    set text(size: 13.5pt, weight: "semibold")
    if it.numbering != none { text(fill: accent, counter(heading).display("1.1")); h(8pt) }
    it.body
  })
  show heading.where(level: 3): it => block(above: 1.4em, below: 0.6em, sticky: true,
    text(size: 10.5pt, weight: "semibold", it.body))

  // Figures and captions.
  set figure(gap: 8pt)
  show figure: set block(breakable: false, above: 1.3em, below: 1.3em)
  show figure.caption: it => {
    set text(size: 8.5pt, fill: muted)
    set par(justify: false)
    text(font: sans, weight: "semibold", fill: ink, [Figure #context it.counter.display(it.numbering)])
    [ #h(3pt) ]
    it.body
  }
  set figure(numbering: n => {
    let ch = counter(heading).get().first()
    str(ch) + "." + str(n)
  })
  show heading.where(level: 1): it => { counter(figure.where(kind: image)).update(0); it }

  // Inline code and raw blocks.
  show raw.where(block: false): it => box(
    fill: rgb("#f0f2f5"), inset: (x: 2.5pt), outset: (y: 2.5pt), radius: 2pt,
    text(font: mono, size: 0.84em, it),
  )
  show raw.where(block: true): set text(font: mono, size: 7.8pt)
  show raw.where(block: true): it => block(
    width: 100%, fill: rgb("#f3f4f7"), inset: 9pt, radius: 3pt, stroke: 0.6pt + rule-c, it,
  )

  // Lists and tables.
  set list(indent: 4pt, body-indent: 7pt, marker: text(fill: accent, sym.bullet))
  set enum(indent: 4pt, body-indent: 7pt)
  show link: it => text(fill: accent, it)
  show ref: it => {
    let el = it.element
    if el != none and el.func() == heading and el.numbering != none {
      let n = numbering(el.numbering, ..counter(heading).at(el.location()))
      link(el.location(), text(fill: accent, [#(if el.level == 1 { "Chapter" } else { "Section" })~#n]))
    } else { text(fill: accent, it) }
  }

  // ------------------------------------------------------------ cover
  page(margin: 0pt, header: none, footer: none, {
    place(top + left, rect(width: 100%, height: 62%, fill: render-bg))
    // swx-qa render --fixture l-bracket --size 1800x1400 -o docs/manual/img/cover-l-bracket.png
    place(top + center, dy: 70pt, image("img/cover-l-bracket.png", width: 72%,
      alt: "A shaded L-shaped bracket, drawn by the Saulitwerx renderer"))
    place(top + left, dy: 62%, rect(width: 100%, height: 6pt, fill: accent))
    place(bottom + left, dx: 72pt, dy: -72pt, block(width: 80%, {
      text(font: sans, size: 10pt, weight: "semibold", fill: accent, tracking: 0.2em, "USER MANUAL")
      v(6pt)
      text(font: sans, size: 40pt, weight: "bold", fill: ink, "Saulitwerx")
      v(2pt)
      text(font: sans, size: 14pt, fill: muted, "Open-source parametric 3D CAD")
      v(28pt)
      set text(font: sans, size: 8.5pt, fill: muted)
      grid(columns: (auto, auto), column-gutter: 14pt, row-gutter: 5pt,
        [Describes], [`main` at commit #raw(commit)],
        [Built], [#built],
        [Status], [Pre-alpha: no desktop app yet],
      )
    }))
  })

  counter(page).update(1)
  set page(numbering: "i")
  body
}
