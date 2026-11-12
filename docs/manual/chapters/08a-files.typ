#import "../template.typ": *

= Saulitwerx document files <ch-files>

#for-developers[This chapter calls `swx-io`, the crate that reads and writes Saulitwerx's own files, from Rust. It needs no OpenCascade.]

#idx("document file")
A STEP file (@ch-solids) keeps only the finished shape. Open it again and you get a solid you can't edit: the sketches, dimensions, parameters and history that made it are gone. To come back to a design and change it, you need a file that keeps all of that. Saulitwerx has its own file for this, the #term[document file], and its layout is published in full, so any program may read or write it.

There are three kinds of document, told apart by the end of their name:

#ref-table(
  columns: (auto, auto, 1fr),
  header: ("Kind", "Name ends in", "Holds"),
  [Part], [`.sltwx`], [One part: its sketches, features, parameters and history.],
  [Assembly], [`.sltwxa`], [Parts put together (later milestones).],
  [Drawing], [`.sltwxd`], [2D drawings of parts and assemblies (later milestones).],
)

A document is a set of small text files in #term[JSON], a plain-text way of writing down named values that people and programs can both read:

- *`manifest.json`* (#term[manifest]) says what the document is: its kind, the format version that wrote it, its id, its units, and which program made and last saved it.
- *`model.json`* holds the design itself, and is what Saulitwerx rebuilds the part from.
- *`properties.json`*, if present, holds facts about the document such as a part number or a material.

The files can live in one folder, the #term[unpacked form], named like the document (`plate.sltwx/`). Each file is written the same way every time, with its names in alphabetical order and one value per line, so when you keep a design under version control a change shows up as a change to just the lines it touched.

*In Saulitwerx today:* `swx-io` reads and writes the unpacked form and checks the manifest. `model.json` and `properties.json` are read and written as plain JSON objects: Saulitwerx can't yet fill `model.json` from a part's history or build a part from it, so a part can't be saved and opened again yet. The single-file form (one zipped file), the stored pictures and previews, and the `sltwx` command that converts between the forms are planned (*Coming next*).

// Keeps the task's heading and intro with its first step.
#pagebreak(weak: true)
== Write and read a document folder <sec-files>

#task-intro(
  goal: [Write a part document as a folder, read it back, and see the errors.],
  needs: [Saulitwerx built from source (@ch-install). OpenCascade isn't needed.],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to write into a folder called `files`:
  #term-block("cargo run -p swx-io --example unpacked_document -- files")
  It prints what it wrote and read, the manifest, then four errors on purpose:
  #term-block(label: "Output", "wrote files/plate.sltwx\nfiles: manifest.json, model.json, properties.json\nread back: part 0.1.0 0b6d3a52-4c1e-4f7a-9e21-5d8c7f3b2a10, units mm / deg\n    / kg\nsame as written: true\nmanifest.json:\n{\n  \"created_by\": \"saulitwerx 0.0.1\",\n  \"document_id\": \"0b6d3a52-4c1e-4f7a-9e21-5d8c7f3b2a10\",\n  \"format\": \"sltwx\",\n  \"format_version\": \"0.1.0\",\n  \"kind\": \"part\",\n  \"saved_by\": \"saulitwerx 0.0.1\",\n  \"units\": {\n    \"angle\": \"deg\",\n    \"length\": \"mm\",\n    \"mass\": \"kg\"\n  }\n}\nwrong extension -> error: folder extension `.sltwxa` does not match the\n    manifest kind `part` (expected `.sltwx`)\nmissing folder -> error: files/missing.sltwx: No such file or directory\n    (os error 2)\nversion 1.0.0 -> error: unsupported format_version \"1.0.0\": only major\n    version 0 is supported\nbad model -> error: invalid JSON: model.json: expected value at line 1\n    column 17")
  Five long lines are wrapped here to fit the page.]],
  [Read the first lines. The program wrote a folder `files/plate.sltwx` holding the three files, read it back, and got exactly what it wrote.],
  [Read the manifest. Its names are in alphabetical order, nested values are indented by two spaces, and it ends with one empty line. It has no `model_hash`, because the program didn't give one.],
  [#block(breakable: false)[Look at the model:
  #term-block("cat files/plate.sltwx/model.json")
  The program built it with `name` before `expr`; the file has them the other way round, in alphabetical order:
  #term-block(label: "Output", "{\n  \"parameters\": [\n    {\n      \"expr\": \"10 mm\",\n      \"name\": \"thickness\"\n    }\n  ]\n}")]],
  [Read the errors. A part's folder must end in `.sltwx`, and nothing was written under the wrong name. A folder that isn't there can't be read. A manifest from a future major version is refused rather than misread. A model that isn't valid JSON is refused, with the file and the place named.],
)

#result[A part document on disk, read back unchanged, in files a person can read.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-io/examples/unpacked_document.rs`. It does three things.

#steps(
  [*Fill in the manifest.* Every field is required except `model_hash`; `extra` holds fields this version doesn't know (see the tip below):
  ```rust
  let manifest = Manifest {
      format: "sltwx".into(),
      format_version: "0.1.0".into(),
      kind: DocumentKind::Part,
      document_id: "0b6d3a52-4c1e-4f7a-9e21-5d8c7f3b2a10".into(),
      units: Units { length: "mm".into(), angle: "deg".into(),
                     mass: "kg".into(), extra: BTreeMap::new() },
      created_by: "saulitwerx 0.0.1".into(),
      saved_by: "saulitwerx 0.0.1".into(),
      model_hash: None,
      extra: BTreeMap::new(),
  };
  ```],
  [*Make the document.* A `Document` is the manifest, the model and the optional properties, the last two as JSON objects:
  ```rust
  let doc = Document {
      manifest,
      model: json!({ "parameters": [{ "name": "thickness",
                                      "expr": "10 mm" }] }),
      properties: Some(json!({ "part_number": "PL-001" })),
  };
  ```],
  [*Write it, and read it back.* The folder is made if it's missing:
  ```rust
  write_unpacked(&folder, &doc)?;
  let back = read_unpacked(&folder)?;
  ```],
)
]

#tip(title: [Newer files keep their extra fields])[A file written by a newer Saulitwerx may hold fields this one doesn't know. Reading keeps them in the manifest's `extra` (and in `units.extra`), and writing puts them back unchanged, so opening and saving a newer file loses nothing. Only a new major version (1.0.0 and up) is refused.]

#note(title: [The document id])[`document_id` is a #term[UUID]: 32 hexadecimal digits in groups of 8, 4, 4, 4 and 12, like the sample's. Give each new document a new one and never change it: assemblies will use it to find their parts again after a file is moved or renamed.]

== What is read and written <sec-files-rules>

- *Reading* (`read_unpacked`) needs `manifest.json` and `model.json`; `properties.json` is optional. Anything else in the folder is left alone. The manifest is checked first, then the folder's name against its kind, then the other two files, which must each be a JSON object.
- *Writing* (`write_unpacked`) checks the name and the two objects before it writes anything, so it never writes a document it would refuse to read back. It writes the three files, deletes `properties.json` when the document has no properties, and leaves everything else in the folder as it was.
- *One file at a time.* `read_manifest` checks the text of a `manifest.json` on its own, and `write_manifest` writes one. `to_canonical_json` writes any JSON value the way every document file is written.

The manifest's checks, and the rest of the rules, are in @sec-ref-files.

== Common mistakes <sec-files-mistakes>

#set par(justify: false)

- *A folder name that doesn't match the kind.* #raw("folder extension `.sltwxa` does not match the manifest kind `part` (expected `.sltwx`)"), or #raw("folder has no extension; the manifest kind `part` needs `.sltwx`"). The manifest decides what a document is; name the folder to match.
- *A missing file.* #raw("missing required file `model.json`") (or `manifest.json`). Both are required, even for an empty model: write `{}`.
- *A model that isn't an object.* #raw("invalid field `model.json`: must be a JSON object"). The model and the properties must each be a JSON object, in braces, not a list or a single value.
- *A broken file.* `invalid JSON: model.json: expected value at line 1 column 17`. The message names the file and the place where it stopped making sense.
- *A malformed id or hash.* #raw("invalid field `document_id`: must be a UUID in 8-4-4-4-12 hex form, got \"plate-1\""). A `model_hash` must be `sha256:` and 64 lowercase hexadecimal digits, or left out.
- *A file from a future version.* `unsupported format_version "1.0.0": only major version 0 is supported`. This Saulitwerx can't read it; use a newer one.
