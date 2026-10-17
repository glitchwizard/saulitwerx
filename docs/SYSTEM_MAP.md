# System map

How Saulitwerx fits together. GitHub renders the diagrams. Milestone status lives in `docs/ROADMAP.md`; decisions live in `docs/ARCHITECTURE.md`.

Update this file when a crate is added or the data flow changes. The weekly report flags it when the crate list here differs from `crates/`.

## Crates and dependency direction

An arrow means "may depend on", per the layering in `ARCHITECTURE.md`: app, then render and io, then kernel and sketch, then core. `swx-core` and `swx-sketch` never depend on the kernel. Solid boxes exist on `main`; dashed boxes are planned. Today no workspace crate depends on another.

```mermaid
flowchart TD
  app["swx-app<br/>desktop app · M5"]
  render["swx-render<br/>wgpu viewport, picking · M5"]
  io["swx-io<br/>.sltwx, STEP, STL · M4"]
  qa["swx-qa<br/>QA renders · QA-01"]
  kernel["swx-kernel<br/>Kernel trait + OCCT"]
  sketch["swx-sketch<br/>2D geometry, solver"]
  core["swx-core<br/>parameters, feature history"]

  app --> render
  app --> io
  render --> kernel
  render --> sketch
  io --> kernel
  io --> sketch
  qa --> kernel
  kernel --> core
  sketch --> core

  classDef planned stroke-dasharray: 5 5
  class app,render,io,qa planned
```

## Data flow

```mermaid
flowchart LR
  params["Parameters<br/>swx-core"] --> sk["Sketch + constraints<br/>swx-sketch"]
  sk -->|solved profiles| feat["Feature history<br/>swx-core"]
  params --> feat
  feat -->|feature operations| brep["B-rep solid<br/>swx-kernel, OCCT"]
  brep -->|tessellate| mesh["Mesh"]
  mesh --> view["Viewport<br/>swx-render"]
  mesh --> png["QA PNG<br/>swx-qa"]
  feat -->|model.json| file[".sltwx<br/>swx-io"]
  brep -->|cache, STEP, STL| file
```
