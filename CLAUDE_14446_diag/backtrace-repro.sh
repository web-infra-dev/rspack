#!/usr/bin/env bash
# Runs the reporter repo against @rspack@beta (= current main, known to
# reproduce #14446) but with the freshly built, INSTRUMENTED binding swapped in,
# so the DIAG14446 backtraces fire from the mixed-separator construction site.
set -u

WS="${GITHUB_WORKSPACE:-$(pwd)}"
WORK="${RUNNER_TEMP:-/tmp}/repro-14446-bt"
PORT="${PORT:-8155}"
BETA='2.1.0-beta.0'

# locate the freshly built instrumented .node
BUILT="$(ls "$WS"/crates/node_binding/rspack.*.node 2>/dev/null | head -1)"
echo "built binding: ${BUILT:-NONE}"
[ -z "$BUILT" ] && { echo "ERROR: built binding .node not found"; exit 1; }

rm -rf "$WORK"
git clone --depth 1 https://github.com/ying-bin/rspack-repo-bug.git "$WORK"
cd "$WORK" || exit 1

corepack enable >/dev/null 2>&1 || true
pnpm install --no-frozen-lockfile 2>&1 | tail -5
pnpm add --no-strict-peer-dependencies "@rspack/core@${BETA}" "@rspack/cli@${BETA}" 2>&1 | tail -5
node -e "console.log('repro @rspack/core', require('@rspack/core/package.json').version)"

# swap every installed platform binding with the instrumented build
BN="$(basename "$BUILT")"
echo "overwriting installed bindings named $BN"
found=0
while IFS= read -r f; do
  cp -f "$BUILT" "$f" && echo "  patched $f" && found=1
done < <(find node_modules -name "$BN" 2>/dev/null)
[ "$found" = 0 ] && { echo "ERROR: no installed binding named $BN found"; find node_modules -name 'rspack.*.node' | head; exit 1; }

export NODE_ENV=development
export NO_COLOR=1 FORCE_COLOR=0 TERM=dumb
export RUST_BACKTRACE=1

./node_modules/.bin/rspack serve --port "$PORT" >dev.log 2>&1 &
SERVE=$!
count() { if [ -f dev.log ]; then grep -cE "compiled.*successfully" dev.log 2>/dev/null || true; else echo 0; fi; }

for i in $(seq 1 120); do [ "$(count)" -ge 1 ] && break; sleep 1; done
C0=$(count)
echo "baseline compiles=$C0"
[ "$C0" -lt 1 ] && { echo "HARNESS ERROR: no first compile"; tail -60 dev.log; kill "$SERVE" 2>/dev/null; exit 1; }

printf '\n// e1 %s\n' "$(date +%s)" >>src/App.tsx
for i in $(seq 1 25); do [ "$(count)" -gt "$C0" ] && break; sleep 1; done
C1=$(count)
printf '\n// e2 %s\n' "$(date +%s)" >>src/App.tsx
for i in $(seq 1 25); do [ "$(count)" -gt "$C1" ] && break; sleep 1; done
C2=$(count)
kill "$SERVE" 2>/dev/null || true
echo "counts: C0=$C0 C1=$C1 C2=$C2"

echo "===================== DIAG14446 hits ====================="
grep -n "DIAG14446" dev.log | head -40
echo "===================== DIAG14446 backtraces (full) ====================="
sed -E 's/\x1b\[[0-9;]*m//g' dev.log | grep -A 60 "DIAG14446 .*MIXED" | head -300