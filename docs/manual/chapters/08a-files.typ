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

#idx("ZIP")
The same files can be kept in two ways, and both carry the same name:

- *One file,* the #term[packed form] (`plate.sltwx`). The files are squeezed together into a single #term[ZIP] archive, the standard compressed bundle that every computer can open. This is the form to send, attach or keep beside your other files.
- *One folder,* the #term[unpacked form] (`plate.sltwx/`), holding the files side by side. This is the form to keep under version control, or to read and edit with any text editor.

Each file is written the same way every time, with its names in alphabetical order and one value per line, in either form. So the same document always gives the same bytes, and when you keep a design under version control a change shows up as a change to just the lines it touched.

*In Saulitwerx today:* `swx-io` reads and writes both forms and checks the manifest. `model.json` and `properties.json` are read and written as plain JSON objects: Saulitwerx can't yet fill `model.json` from a part's history or build a part from it, so a part can't be saved and opened again yet. The stored pictures and previews, and the `sltwx` command that converts between the forms, are planned (*Coming next*); for now a small program converts them (@sec-files-packed).

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

== Write and read a document as one file <sec-files-packed>

The packed form is the one to hand to someone else: one file instead of a folder. This task writes the plate from @sec-files as a single `plate.sltwx`, checks that it reads back unchanged, turns a document folder into a file, and shows what happens when a file is damaged.

#task-intro(
  goal: [Write a part document as one file, read it back, convert a folder to a file, and see the errors.],
  needs: [Saulitwerx built from source (@ch-install). OpenCascade isn't needed.],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to write into a folder called `packed`:
  #term-block("cargo run -p swx-io --example packed_document -- packed")
  It writes the file, reads it back, makes a few more files, then shows four errors on purpose:
  #term-block(label: "Output", "wrote packed/plate.sltwx (633 bytes)\nsame as written: true\nwritten twice, identical: true\nfolder packed/folder/plate.sltwx -> file packed/from-folder.sltwx,\n    identical to plate.sltwx: true\nnot a zip -> error: malformed ZIP archive: no end-of-central-directory\n    record\ncut short -> error: malformed ZIP archive: no end-of-central-directory\n    record\ndamaged -> error: archive entry `model.json` fails its CRC-32 check\nwrong extension -> error: folder extension `.sltwxa` does not match the\n    manifest kind `part` (expected `.sltwx`)")
  Four long lines are wrapped here to fit the page.]],
  [Read the first two lines. The whole document is one file of 633 bytes, and reading it gives back exactly what was written.],
  [Read the third line. The program wrote the same document a second time, to `plate-again.sltwx`, and the two files are identical to the last byte. Saulitwerx stamps every file inside with the same fixed date (1 January 1980, the earliest a ZIP can hold), so saving an unchanged design never changes its file.],
  [Read the fourth line. The program wrote the document as a folder, `packed/folder/plate.sltwx/`, read the folder, and wrote what it read as a file. That file is identical to `plate.sltwx`: the two forms hold the same thing, and you can turn either into the other with nothing lost.],
  [#block(breakable: false)[Look inside the file, if your computer has the `unzip` command (most Linux and macOS systems do):
  #term-block("unzip -l packed/plate.sltwx")
  It lists the same three files the folder holds, in the order Saulitwerx always writes them, each with the fixed date:
  #term-block(label: "Output", "Archive:  packed/plate.sltwx\n  Length      Date    Time    Name\n---------  ---------- -----   ----\n      273  1980-01-01 00:00   manifest.json\n       87  1980-01-01 00:00   model.json\n       56  1980-01-01 00:00   properties.json\n---------                     -------\n      416                     3 files")
  The three files hold 416 bytes of text, squeezed down to 307. The ZIP's own records of names, sizes and checks make up the rest of the 633 bytes. A real part's model is far larger, and squeezes down much more.]],
  [Read the errors. A text file that only has a document's name isn't a ZIP, and neither is a file cut short: both lose the ZIP's closing record, which a reader looks for first. One damaged byte inside the model still unpacks, but to different text, and the file's built-in check (its #term[CRC-32]) catches it. A part can't be written under an assembly's name.],
)

#result[A part document as one file, read back unchanged, the same bytes every time it is written, and damage reported instead of misread.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-io/examples/packed_document.rs`. It makes the same `Document` as the folder sample (@sec-files), then:

#steps(
  [*Write it as one file, and read it back.* The file is replaced if it's there:
  ```rust
  write_sltwx(&file, &doc)?;
  let back = read_sltwx(&file)?;
  ```],
  [*Turn a folder into a file.* Read the unpacked form, and write what you read in the packed form. The other way round is `read_sltwx`, then `write_unpacked`:
  ```rust
  write_sltwx(&packed, &read_unpacked(&folder)?)?;
  ```],
)
]

#tip(title: [Open a document file with any ZIP tool])[#set par(justify: false)
A `.sltwx` file is a plain ZIP. Any ZIP tool can list it or take out one file to read, such as `unzip -p plate.sltwx model.json`. To change a file inside, convert the document to a folder, edit it, and convert it back, so Saulitwerx writes it the way it always does.]

== What is read and written <sec-files-rules>

- *Reading* (`read_unpacked`) needs `manifest.json` and `model.json`; `properties.json` is optional. Anything else in the folder is left alone. The manifest is checked first, then the folder's name against its kind, then the other two files, which must each be a JSON object.
- *Writing* (`write_unpacked`) checks the name and the two objects before it writes anything, so it never writes a document it would refuse to read back. It writes the three files, deletes `properties.json` when the document has no properties, and leaves everything else in the folder as it was.
- *Reading one file* (`read_sltwx`) needs the same two files, inside the ZIP. Each one is unpacked, then checked against its CRC-32 and its recorded size, so damage is reported rather than read. Anything else in the file, such as stored previews, is skipped. After that, the checks are the same as for a folder.
- *Writing one file* (`write_sltwx`) makes the same checks before it writes anything, then writes `manifest.json`, `model.json` and `properties.json` (when the document has properties), in that order, each compressed, with the fixed date. It replaces the whole file.
- *One manifest at a time.* `read_manifest` checks the text of a `manifest.json` on its own, and `write_manifest` writes one. `to_canonical_json` writes any JSON value the way every document file is written.

The manifest's checks, and the rest of the rules, are in @sec-ref-files.

== Common mistakes <sec-files-mistakes>

#set par(justify: false)

- *A folder name that doesn't match the kind.* #raw("folder extension `.sltwxa` does not match the manifest kind `part` (expected `.sltwx`)"), or #raw("folder has no extension; the manifest kind `part` needs `.sltwx`"). The manifest decides what a document is; name the folder to match.
- *A missing file.* #raw("missing required file `model.json`") (or `manifest.json`). Both are required, even for an empty model: write `{}`.
- *A model that isn't an object.* #raw("invalid field `model.json`: must be a JSON object"). The model and the properties must each be a JSON object, in braces, not a list or a single value.
- *A broken file.* `invalid JSON: model.json: expected value at line 1 column 17`. The message names the file and the place where it stopped making sense.
- *A malformed id or hash.* #raw("invalid field `document_id`: must be a UUID in 8-4-4-4-12 hex form, got \"plate-1\""). A `model_hash` must be `sha256:` and 64 lowercase hexadecimal digits, or left out.
- *A file that isn't a document file.* `malformed ZIP archive: no end-of-central-directory record`. The file isn't a ZIP at all, or it was cut short, for example by a copy or download that didn't finish. Copy it again from where it came from. Other `malformed ZIP archive` messages name what is wrong, such as an entry that is truncated.
- *A damaged file.* #raw("archive entry `model.json` fails its CRC-32 check"). A byte inside the file changed after it was written. Use an earlier copy.
- *A ZIP made by another tool.* #raw("archive entry `model.json` uses unsupported compression method 12 (only 0 and 8 are supported)"). Saulitwerx reads only stored and standard compressed (deflate) entries, which is what nearly every ZIP tool writes by default. Re-zip the files with default settings, or unpack them to a folder.
- *A file named for the wrong kind.* The message says #raw("folder extension"), even for a file: #raw("folder extension `.sltwxa` does not match the manifest kind `part` (expected `.sltwx`)"). Name the file for its kind, as for a folder.
- *A file from a future version.* `unsupported format_version "1.0.0": only major version 0 is supported`. This Saulitwerx can't read it; use a newer one.
