# Compressed `@rspack/binding` — RFC / WIP

Ship the native addon (`rspack.<platform>.node`) with the real binary stored
zstd-compressed inside itself, and unwrap it on first load. This shrinks what
rspack ships and installs (a measured run took the ~40 MB `@rspack/binding`
down to ~18 MB) at the same steady-state load speed, and it does **not** depend
on `@napi-rs/cli` — the loader is rspack's own Rust, reusing the `decmpfs` crate.

## Why do this in rspack, now

`@napi-rs/cli` has no `--compress` yet; it is only a proposal. But rspack already
cross-compiles Rust for every platform it ships, so rspack can build a
self-loading addon in Rust today and get the size win first. Tools that lean on
the napi CLI would have to wait for the CLI to add it.

## The hybrid format

The addon is stored zstd-compressed in a `PRESSED_DATA` section (`__PRESSED_DATA`
in segment `SMOL` on Mach-O, `.PRESSED_DATA` on ELF, `.PRESSED` on PE), with a
SHA-512 hash over the compressed bytes. This is the same "pressed-data" layout
the `decmpfs` crate already reads, so its unwrap code works unchanged. The exact
byte layout is in `scripts/pack-compressed-binding.mjs`, a small reference packer
using built-in `node:zlib` (zstd) and `node:crypto` (SHA-512).

## Runtime (all Rust, reuses `decmpfs`)

On first load the addon:

1. reads its own `PRESSED_DATA` section,
2. verifies SHA-512, then zstd-decodes to the raw addon (`decmpfs::addon::unwrap_if_hybrid`),
3. on a compressing filesystem (APFS / NTFS / btrfs) rewrites itself compressed
   in place (`decmpfs::compress_bytes`) so later loads are native; otherwise it
   decompresses once to a per-version cache,
4. hands off to the real addon's napi registration.

No JS glue, no legacy-Node fallback — modern targets only.

## Status

- Format + pack/unpack round-trip: **verified** (the `decmpfs` crate's own tests,
  plus the JS reference packer added here — it packs a real `.node` and unpacks
  to byte-identical bytes that `dlopen` cleanly).
- **WIP, for rspack CI on the cross-compile matrix:** injecting the
  `PRESSED_DATA` section at build time, and the self-loading hand-off.

## Follow-up

Once `@napi-rs/cli` ships `--compress`, this collapses to a one-line build flag;
until then this keeps the win in-tree.
