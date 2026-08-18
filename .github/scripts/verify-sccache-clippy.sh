#!/usr/bin/env bash
#
# Guard for sccache caching of `cargo clippy`.
#
# `cargo clippy` runs as `sccache clippy-driver <rustc> <args>`. sccache only
# skips the injected `<rustc>` argument when cargo passes it as the bare name
# `rustc` (mozilla/sccache#2438), and it refuses to cache anything compiled with
# `-C incremental`. So clippy is cacheable only with both `RUSTC=rustc` and
# `CARGO_INCREMENTAL=0`.
#
# Caching a lint pass is only safe if a cache hit still reports the lints, and
# if a `cargo check` entry can never be served to `cargo clippy`. This script
# asserts both against a fixture whose only defect is a clippy-only lint.
set -uo pipefail

SCC="${SCCACHE_PATH:-sccache}"
if ! command -v "$SCC" >/dev/null 2>&1; then
  echo "sccache not available on this runner, skipping"
  exit 0
fi

# Use an isolated local cache: the assertions below need deterministic
# hit/miss counts and must not touch the shared remote cache.
unset SCCACHE_BUCKET SCCACHE_REGION SCCACHE_ENDPOINT SCCACHE_S3_KEY_PREFIX \
      SCCACHE_S3_USE_SSL SCCACHE_S3_ENABLE_VIRTUAL_HOST_STYLE \
      AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY SCCACHE_GHA_ENABLED SCCACHE_DIR

WORK="$(mktemp -d)"
trap '"$SCC" --stop-server >/dev/null 2>&1; rm -rf "$WORK"' EXIT
export SCCACHE_DIR="$WORK/cache"
export RUSTC_WRAPPER="$SCC"

FIX="$WORK/fixture"
mkdir -p "$FIX/src"
cat > "$FIX/Cargo.toml" <<'EOF'
[package]
name = "clippy_cache_fixture"
version = "0.1.0"
edition = "2021"

[profile.dev]
incremental = true
EOF
# `needless_return` is a clippy-only lint: rustc alone accepts this file.
cat > "$FIX/src/lib.rs" <<'EOF'
pub fn answer() -> i32 {
    return 42;
}
EOF
if [ -f "${TOOLCHAIN_FILE:-}" ]; then
  channel="$(sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$TOOLCHAIN_FILE" | head -1)"
  [ -n "$channel" ] && printf '[toolchain]\nchannel = "%s"\n' "$channel" > "$FIX/rust-toolchain.toml"
fi
cd "$FIX" || exit 1

fail=0
ok()  { echo "  PASS  $1"; }
bad() { echo "  FAIL  $1"; fail=1; }

stats() { "$SCC" --show-stats --stats-format=json 2>/dev/null; }
hits()  { stats | python3 -c 'import json,sys;print(json.load(sys.stdin)["stats"]["cache_hits"]["counts"].get("Rust",0))'; }
reason(){ stats | python3 -c "import json,sys;print(json.load(sys.stdin)['stats']['not_cached'].get('$1',0))"; }
restart() {
  "$SCC" --stop-server >/dev/null 2>&1
  rm -rf "$SCCACHE_DIR"
  : > "$WORK/sccache.log"
  SCCACHE_LOG=sccache=debug SCCACHE_ERROR_LOG="$WORK/sccache.log" "$SCC" --start-server >/dev/null 2>&1
}
last_key() { grep -o "Hash key: [0-9a-f]*" "$WORK/sccache.log" | tail -1 | awk '{print $3}'; }
norm() { awk '/^(error|warning|note|help)/'; }

echo "=== 0. rustc alone accepts the fixture (the lint is clippy-only) ==="
restart
if CARGO_INCREMENTAL=0 cargo check -q 2>/dev/null; then ok "cargo check succeeds"; else bad "cargo check should succeed"; fi

echo
echo "=== 1. cacheability matrix: both env vars are required ==="
for incr in 1 0; do
  for rustc_mode in path bare; do
    restart
    cargo clean -q
    if [ "$rustc_mode" = bare ]; then export RUSTC=rustc; else unset RUSTC; fi
    if [ "$incr" = 0 ]; then export CARGO_INCREMENTAL=0; else unset CARGO_INCREMENTAL; fi
    cargo clippy -q >/dev/null 2>&1
    mif="$(reason 'multiple input files')"; inc="$(reason incremental)"
    cargo clean -q
    before="$(hits)"; cargo clippy -q >/dev/null 2>&1; after="$(hits)"
    printf "  CARGO_INCREMENTAL=%s RUSTC=%-4s  multiple-input=%s incremental=%s hits-on-rerun=%s\n" \
      "$incr" "$rustc_mode" "$mif" "$inc" "$((after - before))"
    if [ "$incr" = 0 ] && [ "$rustc_mode" = bare ]; then
      [ "$((after - before))" -gt 0 ] && ok "clippy is cached with both vars set" || bad "clippy still not cached with both vars set"
    else
      [ "$((after - before))" -eq 0 ] && ok "clippy stays uncached otherwise (as expected)" || bad "unexpected cache hit"
    fi
  done
done

export CARGO_INCREMENTAL=0 RUSTC=rustc

echo
echo "=== 2. NEGATIVE: a warm 'cargo check' cache must NOT satisfy clippy ==="
restart
cargo clean -q
if cargo check -q >/dev/null 2>&1; then ok "cargo check populated the cache (exit 0)"; else bad "cargo check failed"; fi
cargo clean -q
out="$(cargo clippy -q -- -D warnings 2>&1)"; rc=$?
if [ $rc -ne 0 ]; then ok "clippy still FAILS after a warm check cache (exit $rc)"; else bad "clippy passed - a check artifact was reused"; fi
if grep -q "needless_return" <<<"$out"; then ok "clippy diagnostic present"; else bad "clippy lint text missing"; fi

echo
echo "=== 3. a clippy cache HIT must replay the clippy diagnostics ==="
restart
cargo clean -q
b="$(hits)"; out1="$(cargo clippy 2>&1)"; a="$(hits)"
[ "$a" = "$b" ] && ok "run 1 is a miss (compiled for real)" || bad "run 1 unexpectedly hit"
grep -q "needless_return" <<<"$out1" && ok "run 1 emits the clippy warning" || bad "run 1 lost the warning"
cargo clean -q
b="$(hits)"; out2="$(cargo clippy 2>&1)"; a="$(hits)"
[ "$a" -gt "$b" ] && ok "run 2 is a cache HIT (Rust hits $b -> $a)" || bad "run 2 did not hit the cache"
grep -q "needless_return" <<<"$out2" && ok "run 2 replays the clippy warning from cache" || bad "cached run swallowed the lint"

echo
echo "=== 4. lint level is part of the key ==="
cargo clean -q
cargo clippy -q -- -D warnings >/dev/null 2>&1; rc=$?
[ $rc -ne 0 ] && ok "-D warnings still fails against a warm -W entry (exit $rc)" || bad "-D warnings wrongly satisfied by cached -W run"

echo
echo "=== 5. RUSTC=rustc resolves to the same compiler as before ==="
restart
cargo clean -q; RUSTC=rustc cargo check -q >/dev/null 2>&1; b="$(hits)"
cargo clean -q; unset RUSTC; cargo check -q >/dev/null 2>&1; a="$(hits)"
export RUSTC=rustc
[ "$a" -gt "$b" ] && ok "check entries are shared with/without RUSTC=rustc" || bad "RUSTC=rustc changed the check cache key"

echo
echo "=== 6. EQUIVALENCE: RUSTC=rustc must not degrade clippy into plain rustc ==="
restart
cargo clean -q; env -u RUSTC -u RUSTC_WRAPPER cargo clippy 2>&1 | norm > "$WORK/out.default"
cargo clean -q; env -u RUSTC_WRAPPER RUSTC=rustc cargo clippy 2>&1 | norm > "$WORK/out.bare"
cargo clean -q; cargo clippy 2>&1 | norm > "$WORK/out.sccache.miss"
cargo clean -q; cargo clippy 2>&1 | norm > "$WORK/out.sccache.hit"
[ -s "$WORK/out.default" ] && ok "baseline output is non-empty (lint really fires)" || bad "no diagnostics captured - comparison is vacuous"
diff -q "$WORK/out.default" "$WORK/out.bare"         >/dev/null && ok "RUSTC=rustc: identical diagnostics"     || { bad "RUSTC=rustc changed clippy output"; diff "$WORK/out.default" "$WORK/out.bare"; }
diff -q "$WORK/out.default" "$WORK/out.sccache.miss" >/dev/null && ok "sccache miss: identical diagnostics"    || { bad "sccache miss changed clippy output"; diff "$WORK/out.default" "$WORK/out.sccache.miss"; }
diff -q "$WORK/out.default" "$WORK/out.sccache.hit"  >/dev/null && ok "sccache HIT: identical diagnostics"     || { bad "sccache hit changed clippy output"; diff "$WORK/out.default" "$WORK/out.sccache.hit"; }

echo
echo "=== 7. check and clippy live in different cache keys ==="
restart
cargo clean -q; cargo check -q >/dev/null 2>&1;  key_check="$(last_key)"
cargo clean -q; cargo clippy -q >/dev/null 2>&1; key_clippy="$(last_key)"
echo "  check  key: ${key_check:-<none>}"
echo "  clippy key: ${key_clippy:-<none>}"
if [ -n "$key_check" ] && [ -n "$key_clippy" ] && [ "$key_check" != "$key_clippy" ]; then
  ok "clippy-driver produces a distinct cache key from rustc"
else
  bad "cache keys collide (or were not captured)"
fi

echo
[ $fail -eq 0 ] && echo "ALL ASSERTIONS PASSED" || echo "SOME ASSERTIONS FAILED"
exit $fail
