#!/usr/bin/env bash
# Builds the Saulitwerx user manual PDF.
#
#   docs/manual/build.sh <out.pdf> [extra typst args]
#
# Uses a pinned Typst release (Apache-2.0). When it is missing, the script
# downloads it from Typst's GitHub releases page into docs/manual/.tools/
# and checks its SHA-256 before use (FACTORY §6d). Core Typst only: the
# package registry is not used.
set -euo pipefail

TYPST_VERSION="0.15.1"
TYPST_ASSET="typst-x86_64-unknown-linux-musl.tar.xz"
TYPST_SHA256="a6d077d0a95eed5a2eba715b2dae06be954f624ccbf85758a03f389ded33118c"

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="${1:?usage: build.sh <out.pdf> [typst args]}"
shift

tools="$here/.tools"
typst="$tools/typst-$TYPST_VERSION/typst"
if [ ! -x "$typst" ]; then
  mkdir -p "$tools"
  archive="$tools/$TYPST_ASSET"
  curl -fsSL -o "$archive" \
    "https://github.com/typst/typst/releases/download/v$TYPST_VERSION/$TYPST_ASSET"
  echo "$TYPST_SHA256  $archive" | sha256sum -c --quiet -
  mkdir -p "$tools/typst-$TYPST_VERSION"
  tar -xJf "$archive" -C "$tools/typst-$TYPST_VERSION" --strip-components=1
  rm -f "$archive"
fi

commit="$(git -C "$root" rev-parse --short=7 HEAD 2>/dev/null || echo unknown)"
built="$(TZ=America/Los_Angeles date +%Y-%m-%d)"

exec "$typst" compile \
  --root "$root" \
  --font-path "$here/fonts" \
  --ignore-system-fonts \
  --input "commit=${SWX_MANUAL_COMMIT:-$commit}" \
  --input "built=${SWX_MANUAL_DATE:-$built}" \
  "$@" \
  "$here/manual.typ" "$out"
