#!/usr/bin/env bash
# Same reporter repo, but driven by `rspack build --watch` (NO dev-server / HMR).
# Isolates whether the mixed-separator bug needs the dev-server or just the
# repo's loader/build config.
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${RUNNER_TEMP:-/tmp}/repro-14446-build"
rm -rf "$WORK"
git clone --depth 1 https://github.com/ying-bin/rspack-repo-bug.git "$WORK"
cd "$WORK" || exit 1

corepack enable >/dev/null 2>&1 || true
pnpm install --no-frozen-lockfile 2>&1 | tail -5

export NODE_ENV=development
PRELOAD="${ROOT}/preload-wrap.cjs"
if command -v cygpath >/dev/null 2>&1; then PRELOAD="$(cygpath -m "$PRELOAD")"; fi
export NODE_OPTIONS="--require ${PRELOAD}"
export NO_COLOR=1 FORCE_COLOR=0 TERM=dumb

./node_modules/.bin/rspack build --watch >build.log 2>&1 &
BUILD=$!

count() {
  if [ -f build.log ]; then grep -cE "compiled.*successfully" build.log 2>/dev/null || true; else echo 0; fi
}
wait_grow() {
  local base="$1" i
  for i in $(seq 1 25); do
    [ "$(count)" -gt "$base" ] && return 0
    sleep 1
  done
  return 1
}

for i in $(seq 1 90); do
  [ "$(count)" -ge 1 ] && break
  sleep 1
done

C0=$(count)
if [ "$C0" -lt 1 ]; then
  echo "HARNESS ERROR: build --watch never produced a successful compile."
  kill "$BUILD" 2>/dev/null || true
  tail -80 build.log
  exit 1
fi

echo ">> baseline compiles=$C0; EDIT #1"
printf '\n// e1 %s\n' "$(date +%s)" >>src/App.tsx
wait_grow "$C0"; C1=$(count)
echo ">> after EDIT #1 compiles=$C1; EDIT #2"
printf '\n// e2 %s\n' "$(date +%s)" >>src/App.tsx
wait_grow "$C1"; C2=$(count)
echo ">> after EDIT #2 compiles=$C2"

kill "$BUILD" 2>/dev/null || true

echo "compiled-successfully counts: C0=$C0 C1=$C1 C2=$C2"
if [ "$C1" -gt "$C0" ] && [ "$C2" -gt "$C1" ]; then
  VERDICT="WATCH OK — both edits recompiled (#14446 NOT reproduced via build --watch)"
else
  VERDICT="WATCH STOPPED — second edit ignored (#14446 REPRODUCED via build --watch)"
fi
echo "VERDICT: $VERDICT"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  { echo "### build --watch repro — ${RUNNER_OS:-local}"; echo "compiles: C0=$C0 C1=$C1 C2=$C2"; echo ""; echo "$VERDICT"; } >>"$GITHUB_STEP_SUMMARY"
fi
echo "===================== build.log (tail) ====================="
tail -120 build.log
