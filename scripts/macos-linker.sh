#!/usr/bin/env bash
set -euo pipefail

# Keep clang for the -Wl arguments emitted by NAPI build scripts.
lld_libdir=$(rustc --print target-libdir)
exec "$(xcrun --find clang)" "-fuse-ld=${lld_libdir%/lib}/bin/gcc-ld/ld64.lld" "$@"
