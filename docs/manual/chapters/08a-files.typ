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

*In Saulitwerx today:* `swx-io` reads and writes both forms and checks the manifest, and the `sltwx` command turns a folder into a file and back (@sec-files-sltwx). In a part's `model.json`, every sketch, feature and body, and every piece and rule of a sketch, must carry its own ID, and the file is written in a fixed order (@sec-files-ids). Beyond that, `model.json` and `properties.json` are read and written as plain JSON objects: Saulitwerx can't yet fill `model.json` from a part's history or build a part from it, so a part can't be saved and opened again yet. The stored pictures and previews are planned (*Coming next*).

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
  #term-block(label: "Output", "wrote packed/plate.sltwx (633 bytes)\nsame as written: true\nwritten twice, identical: true\nfolder packed/folder/plate.sltwx -> file packed/from-folder.sltwx,\n    identical to plate.sltwx: true\nnot a zip -> error: malformed ZIP archive: no end-of-central-directory\n    record\ncut short -> error: malformed ZIP archive: no end-of-central-directory\n    record\ndamaged -> error: archive entry `model.json` fails its CRC-32 check\nwrong extension -> error: file extension `.sltwxa` does not match the\n    manifest kind `part` (expected `.sltwx`)")
  Four long lines are wrapped here to fit the page.]],
  [Read the first two lines. The whole document is one file of 633 bytes, and reading it gives back exactly what was written.],
  [Read the third line. The program wrote the same document a second time, to `plate-again.sltwx`, and the two files are identical to the last byte. Saulitwerx stamps every file inside with the same fixed date (1 January 1980, the earliest a ZIP can hold), so saving an unchanged design never changes its file.],
  [Read the fourth line. The program wrote the document as a folder, `packed/folder/plate.sltwx/`, read the folder, and wrote what it read as a file. That file is identical to `plate.sltwx`: the two forms hold the same thing, and you can turn either into the other with nothing lost.],
  [#block(breakable: false)[Look inside the file, if your computer has the `unzip` command (most Linux and macOS systems do):
  #term-block("unzip -l packed/plate.sltwx")
  It lists the same three files the folder holds, in the order Saulitwerx always writes them, each with the fixed date:
  #term-block(label: "Output", "Archive:  packed/plate.sltwx\n  Length      Date    Time    Name\n---------  ---------- -----   ----\n      273  1980-01-01 00:00   manifest.json\n       87  1980-01-01 00:00   model.json\n       56  1980-01-01 00:00   properties.json\n---------                     -------\n      416                     3 files")
  The three files hold 416 bytes of text, squeezed down to 307. The ZIP's own records of names, sizes and checks make up the rest of the 633 bytes. A real part's model is far larger, and squeezes down much more.]],
  [Read the errors. A text file that only has a document's name isn't a ZIP, and neither is a file cut short: both lose the ZIP's closing record, which a reader looks for first. One damaged byte inside the model still unpacks, but to different text, and the file's built-in check (its #term[CRC-32]) catches it. A part can't be written under an assembly's name, and the message says _file_, because this is the one-file form.],
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
A `.sltwx` file is a plain ZIP. Any ZIP tool can list it or take out one file to read, such as `unzip -p plate.sltwx model.json`. To change a file inside, convert the document to a folder with `sltwx unpack`, edit it, and convert it back with `sltwx pack` (@sec-files-sltwx), so Saulitwerx writes it the way it always does.]

// Keeps the task's heading and intro with its first step.
#pagebreak(weak: true)
== Convert between a folder and a file with `sltwx` <sec-files-sltwx>

The folder is the form to edit and keep under version control; the file is the form to send. The #term(key: "sltwx command")[`sltwx` command] turns one into the other, so you can move between them whenever you need to. It needs no program of your own.

#task-intro(
  goal: [Build the `sltwx` command, pack a document folder into one file, and unpack the file into a new folder.],
  needs: [Saulitwerx built from source (@ch-install), and the folder `files/plate.sltwx` from @sec-files. OpenCascade isn't needed.],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Build the command:
  #term-block("cargo build --release -p swx-io")
  It ends with a line that starts `Finished`. The command is now at `target/release/sltwx`.]],
  [#block(breakable: false)[Ask it what it does:
  #term-block("target/release/sltwx --help")
  It prints its three forms:
  #term-block(label: "Output", "usage: sltwx pack <folder> <file>     read the unpacked <folder>,\n    write the ZIP <file>\n       sltwx unpack <file> <folder>   read the ZIP <file>, write the\n    unpacked <folder>\n       sltwx --help | -h              print this usage\n\nThe output path must not exist; it is never overwritten.")
  Two long lines are wrapped here to fit the page.]],
  [#block(breakable: false)[Pack the folder from @sec-files into one file, and look at the file:
  #term-block("target/release/sltwx pack files/plate.sltwx plate.sltwx\nls -l plate.sltwx")
  `pack` prints nothing when it works. The file is 633 bytes, the same size as the file the sample program wrote in @sec-files-packed, because it holds the same document.]],
  [#block(breakable: false)[Unpack the file into a new folder, and list the folder:
  #term-block("target/release/sltwx unpack plate.sltwx plate-copy.sltwx\nls plate-copy.sltwx")
  The new folder holds the same three files as the one you started from:
  #term-block(label: "Output", "manifest.json\nmodel.json\nproperties.json")]],
  [#block(breakable: false)[Pack the folder again, to the same file name:
  #term-block("target/release/sltwx pack files/plate.sltwx plate.sltwx")
  This time it refuses, and leaves the file you made in step 3 alone:
  #term-block(label: "Output", "sltwx: plate.sltwx: already exists")]],
)

#result[The same document as a folder and as one file, converted both ways with one command.]

`sltwx` never overwrites anything: the file or folder it writes must not exist yet, so a slip of the keyboard can't destroy your work. To write over an old copy, delete it first, or pick a new name. `unpack` makes the new folder itself, but the folder it goes in must already exist. Both forms are checked on the way in exactly as a program's `read_unpacked` and `read_sltwx` check them (@sec-files-rules); on a problem, `sltwx` prints the same message after `sltwx: `, and writes nothing. Its exit status, for scripts, is 0 when it works, 1 when reading or writing fails or the output is already there, and 2, with the usage, when the words after `sltwx` aren't one of its three forms. @sec-ref-sltwx has the details.

#tip(title: [Run it from anywhere])[`target/release/sltwx` works only from the `saulitwerx` folder. To type just `sltwx` in any folder, copy the file to a folder on your search path, for example `cp target/release/sltwx ~/.local/bin/` on most Linux systems.]

== IDs in a part's model <sec-files-ids>

#idx("ID")
When you change a design, Saulitwerx must be able to tell which sketch, which line or which feature you changed, even after you rename or reorder things. So in a part's `model.json`, each of these carries an #term[ID]: a short name of its own, such as `"sketch-1"` or `"line-3"`, that never changes for as long as the thing exists. The IDs are what a feature will use to point at the sketch it was made from, and what lets a later version of the file be compared with an earlier one.

The rules cover five lists in `model.json`: `sketches`, `features` and `bodies`, and inside each sketch, its pieces (`entities`) and its rules (`constraints`). Each item in these lists must have an `"id"` that is a piece of text, not empty, and different from every other ID in the same list. A list may be left out altogether.

Order matters only for features: they are the part's history (@ch-history), so they stay in the order they were made. The other four lists are #term(key: "set")[sets]: only what is in them counts, not the order. Saulitwerx writes every set sorted by its IDs, so the same design always gives the same file, however its lists were built. Assemblies and drawings aren't checked yet.

#task-intro(
  goal: [Write a small model with IDs out of order, see Saulitwerx sort them, and see the two ID errors.],
  needs: [The `sltwx` command (@sec-files-sltwx), and the folder `files/plate.sltwx` from @sec-files.],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Make a new document folder, `ids.sltwx`, with the plate's manifest in it:
  #term-block("mkdir ids.sltwx\ncp files/plate.sltwx/manifest.json ids.sltwx/")]],
  [#block(breakable: false)[Write a model into the folder. Type this in the terminal, or put the lines between the first and the last into `ids.sltwx/model.json` with any text editor:
  #term-block("cat > ids.sltwx/model.json <<'EOF'\n{\n  \"bodies\": [{ \"id\": \"body-2\" }, { \"id\": \"body-1\" }],\n  \"features\": [{ \"id\": \"extrude-1\" }, { \"id\": \"chamfer-1\" }],\n  \"sketches\": [\n    { \"id\": \"sketch-1\",\n      \"entities\": [{ \"id\": \"line-2\" }, { \"id\": \"line-1\" }] }\n  ]\n}\nEOF")
  The bodies and the lines are out of order. The extrude comes before the chamfer, as it would in a part's history.]],
  [#block(breakable: false)[Pack the folder, unpack the file into a new folder, and look at its model:
  #term-block("target/release/sltwx pack ids.sltwx ids-file.sltwx\ntarget/release/sltwx unpack ids-file.sltwx ids-sorted.sltwx\ncat ids-sorted.sltwx/model.json")
  The bodies and the lines are now sorted by ID. The features kept their order, though `chamfer-1` comes before `extrude-1` in the alphabet:
  #term-block(label: "Output", "{\n  \"bodies\": [\n    {\n      \"id\": \"body-1\"\n    },\n    {\n      \"id\": \"body-2\"\n    }\n  ],\n  \"features\": [\n    {\n      \"id\": \"extrude-1\"\n    },\n    {\n      \"id\": \"chamfer-1\"\n    }\n  ],\n  \"sketches\": [\n    {\n      \"entities\": [\n        {\n          \"id\": \"line-1\"\n        },\n        {\n          \"id\": \"line-2\"\n        }\n      ],\n      \"id\": \"sketch-1\"\n    }\n  ]\n}")]],
  [#block(breakable: false)[Give both bodies the same ID, and try to pack the folder again:
  #term-block("sed -i 's/\"body-1\"/\"body-2\"/' ids.sltwx/model.json\ntarget/release/sltwx pack ids.sltwx dup.sltwx")
  It refuses, names the ID and the list, and writes no file:
  #term-block(label: "Output", "sltwx: duplicate id `body-2` in `bodies`")]],
  [#block(breakable: false)[Replace the second body's ID with a name, so it has no ID at all, and try once more:
  #term-block("sed -i 's/{ \"id\": \"body-2\" }, { \"id\": \"body-2\" }/{ \"id\": \"body-2\" }, { \"name\": \"base\" }/' \\\n    ids.sltwx/model.json\ntarget/release/sltwx pack ids.sltwx dup.sltwx")
  The message points at the item by its list and its place, counting from 0:
  #term-block(label: "Output", "sltwx: `bodies[1]` needs a non-empty string `id`")]],
)

#result[A part's model written in one fixed order, with every ID checked on the way in and out.]

#note(title: [Reading never reorders])[Saulitwerx checks the IDs when it reads a part, and refuses a file that breaks the rules, but it gives you the lists in the order the file has them. Only writing sorts. Your program's own copy of the model isn't changed either: the sorted lists go only into the file.]

== What is read and written <sec-files-rules>

- *Reading* (`read_unpacked`) needs `manifest.json` and `model.json`; `properties.json` is optional. Anything else in the folder is left alone. The manifest is checked first, then the folder's name against its kind, then the other two files, which must each be a JSON object.
- *A part's IDs* are checked when a part is read, after the model is found to be an object, and again before it is written (@sec-files-ids). Assemblies and drawings aren't checked yet.
- *Writing* (`write_unpacked`) checks the name, the two objects and a part's IDs before it writes anything, so it never writes a document it would refuse to read back. A part's sets are written sorted by ID; its features keep their order. It writes the three files, deletes `properties.json` when the document has no properties, and leaves everything else in the folder as it was.
- *Reading one file* (`read_sltwx`) needs the same two files, inside the ZIP. Each one is unpacked, then checked against its CRC-32 and its recorded size, so damage is reported rather than read. Anything else in the file, such as stored previews, is skipped. After that, the checks are the same as for a folder.
- *Writing one file* (`write_sltwx`) makes the same checks, and the same sorting, before it writes anything, then writes `manifest.json`, `model.json` and `properties.json` (when the document has properties), in that order, each compressed, with the fixed date. It replaces the whole file.
- *One manifest at a time.* `read_manifest` checks the text of a `manifest.json` on its own, and `write_manifest` writes one. `to_canonical_json` writes any JSON value the way every document file is written.

The manifest's checks, and the rest of the rules, are in @sec-ref-files.

#for-developers[
*Check a manifest from your own program.* The rules for `manifest.json` are also published as a #term(key: "JSON Schema")[JSON Schema], a standard description that JSON tools in any language can check a file against: `docs/specs/schema/manifest-0.1.json` in the source code, for format version 0.1. It sets the same required fields and allowed values as the table in @sec-ref-files, and, like Saulitwerx, it lets a manifest carry fields it doesn't know. `read_manifest` stays the final word; the project's tests check that the two agree. Schemas for `model.json` and `properties.json` aren't published yet.
]

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
- *A file named for the wrong kind.* #raw("file extension `.sltwxa` does not match the manifest kind `part` (expected `.sltwx`)"), or #raw("file has no extension; the manifest kind `part` needs `.sltwx`"). The message says _file_ for the one-file form and _folder_ for a folder. Name the file for its kind, as for a folder.
- *An ID used twice.* #raw("duplicate id `body-2` in `bodies`"). Two items in the same list have the same ID. Give one of them a new ID that isn't used anywhere in that list. The list is named by its place in the model, such as #raw("sketches[0].entities") for the pieces of the first sketch.
- *An item without an ID.* #raw("`bodies[1]` needs a non-empty string `id`"). That item has no `"id"`, an empty one, or one that isn't text (`"id": 7` is a number, so it doesn't count). Give it a text ID. A list that isn't a list at all gives #raw("invalid field `bodies`: must be an array").
- *Converting onto an existing name.* `sltwx: plate.sltwx: already exists`. `sltwx` never overwrites. Delete the old copy first, or choose another name.
- *Unpacking into a folder that isn't there.* `sltwx: copy/plate.sltwx: parent folder does not exist`. `unpack` makes the last folder in the path, not the ones above it. Make them first with `mkdir -p copy`.
- *A file from a future version.* `unsupported format_version "1.0.0": only major version 0 is supported`. This Saulitwerx can't read it; use a newer one.
