#!/usr/bin/env bash
set -euo pipefail

# The project's compiler bundles LLD 22, which cannot read Xcode 27 SDK stubs.
# Use LLD 23.1.4 independently of the compiler until its toolchain is upgraded.
lld_toolchain=nightly-2026-10-29

if [[ "${1:-}" == "--setup" ]]; then
  rustup toolchain install "$lld_toolchain" --profile minimal --no-self-update
  exit 0
fi

lld_libdir=$(rustup run "$lld_toolchain" rustc --print target-libdir)
exec "$(xcrun --find clang)" "-fuse-ld=${lld_libdir%/lib}/bin/gcc-ld/ld64.lld" "$@"
