#import "../template.typ": *

= Installing Saulitwerx <ch-install>

There is no installer yet. Until the first release, you build Saulitwerx from its source code. That sounds harder than it is: three tools to install, then one command builds everything. Plan on about 10 minutes, most of it waiting for downloads.

#task-intro(
  goal: [Install the tools Saulitwerx needs, download its source code, build it, and check that it works.],
  needs: [A computer running Linux (these steps were tested on Ubuntu 24.04), an internet connection, and permission to install software (`sudo`).],
  time: [About 10 minutes.],
)

== What gets installed

#ref-table(
  columns: (auto, 1fr, auto),
  header: ("Tool", "Why Saulitwerx needs it", "Required?"),
  [Git], [Downloads the source code and keeps it up to date.], [Yes],
  [Rust (1.80 or newer)], [The programming language Saulitwerx is written in. Its tool `cargo` builds and runs the program.], [Yes],
  [C++ compiler], [Builds a small bridge between Saulitwerx and the geometry engine.], [For solids and STEP],
  [OpenCascade (OCCT) 7.6 or newer], [The open-source geometry engine that does the solid modeling.], [For solids and STEP],
)

#idx("OpenCascade")#idx("geometry kernel")
Without OpenCascade you can still sketch, try the document model, and draw the built-in sample parts. To make your own solids or read and write STEP files, you need it.

== Install the tools

#steps(
  [Open a terminal. On Ubuntu, press #key("Ctrl") + #key("Alt") + #key("T"). A window with a command prompt appears.],
  [Install Git, a C++ compiler and OpenCascade:
  #term-block("sudo apt-get update\nsudo apt-get install -y git build-essential curl \\\n   libocct-foundation-dev libocct-modeling-data-dev \\\n   libocct-modeling-algorithms-dev libocct-visualization-dev \\\n   libocct-data-exchange-dev")
  Type your password when asked. The terminal prints a list of packages and ends at the prompt again.],
  [Install Rust with its official installer, and accept the default choices when it asks:
  #term-block("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
  It ends with `Rust is installed now. Great!`],
  [Close the terminal and open a new one, so it finds the new tools. Then check them:
  #term-block("cargo --version")
  You should see a version line such as `cargo 1.97.0 (…)`. Any version 1.80 or newer works.],
)

== Get the source and build it

#steps(
  [Download the source code into a folder named `saulitwerx`, and go into it:
  #term-block("git clone https://github.com/glitchwizard/saulitwerx\ncd saulitwerx")],
  [Build Saulitwerx and run its own checks:
  #term-block("cargo test --workspace")
  The first build downloads a few small libraries and takes a minute or two. It ends with several lines that start `test result: ok.` and none that say `FAILED`.],
  [Build the picture tool with OpenCascade support:
  #term-block("cargo build --release -p swx-qa --features occt")
  It ends with a line that starts `Finished`. The tool is now at `target/release/swx-qa`.],
  [Check that the picture tool runs:
  #term-block("target/release/swx-qa --help")
  You should see its help text, which starts `Usage: swx-qa render`.],
)

#result[Saulitwerx is built, its tests pass, and the `swx-qa` picture tool is ready. Go on to @ch-first.]

#tip(title: [Staying up to date])[Saulitwerx changes most nights. To get the newest version, go into the `saulitwerx` folder and run `git pull`, then build again.]

#caution(title: [Commands run from the `saulitwerx` folder])[Every command in this manual assumes your terminal is in the `saulitwerx` folder. If a command says it can't find a file, run `cd saulitwerx` first. @ch-help lists other problems and their fixes.]

== Other systems

The build is regularly checked only on Ubuntu Linux. Saulitwerx aims to run on macOS and Windows too, and the same steps should work there once Rust and OpenCascade are installed, but they are untested. Without OpenCascade, leave out `--features occt` in step 3.
