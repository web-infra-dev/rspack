#!/usr/bin/env bash
# Faithful reproduction of web-infra-dev/rspack#14446 using the reporter's own
# repo, driven exactly like the issue: `rspack serve`, append to a source file,
# count how many times it recompiles.
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${RUNNER_TEMP:-/tmp}/repro-14446"
PORT="${PORT:-8147}"
rm -rf "$WORK"
git clone --depth 1 https://github.com/ying-bin/rspack-repo-bug.git "$WORK"
cd "$WORK" || exit 1

corepack enable >/dev/null 2>&1 || true
pnpm install --no-frozen-lockfile 2>&1 | tail -5

export NODE_ENV=development
export NODE_OPTIONS="--require ${ROOT}/preload-wrap.cjs"
# Disable ANSI color so the success line is plain text; the matcher below is
# also tolerant of color codes between the two words just in case.
export NO_COLOR=1 FORCE_COLOR=0 TERM=dumb

# Invoke the rspack binary directly (the `pnpm serve` wrapper re-runs a deps
# check that aborts on the ignored build script). Use a free port to avoid any
# local collision; on a fresh CI runner this is moot.
./node_modules/.bin/rspack serve --port "$PORT" >dev.log 2>&1 &
SERVE=$!

count() {
  if [ -f dev.log ]; then grep -cE "compiled.*successfully" dev.log 2>/dev/null || true; else echo 0; fi
}
wait_grow() {
  local base="$1" i
  for i in $(seq 1 25); do
    [ "$(count)" -gt "$base" ] && return 0
    sleep 1
  done
  return 1
}

# wait for first successful compile
for i in $(seq 1 90); do
  [ "$(count)" -ge 1 ] && break
  sleep 1
done

C0=$(count)
if [ "$C0" -lt 1 ]; then
  echo "HARNESS ERROR: dev server never produced a successful compile."
  kill "$SERVE" 2>/dev/null || true
  echo "===================== dev.log (tail) ====================="
  tail -80 dev.log
  if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    echo "### Faithful serve repro — ${RUNNER_OS:-local}: HARNESS ERROR (serve did not start)" >>"$GITHUB_STEP_SUMMARY"
  fi
  exit 1
fi

echo ">> baseline compiles=$C0; EDIT #1"
printf '\n// e1 %s\n' "$(date +%s)" >>src/App.tsx
wait_grow "$C0"; C1=$(count)
echo ">> after EDIT #1 compiles=$C1; EDIT #2"
printf '\n// e2 %s\n' "$(date +%s)" >>src/App.tsx
wait_grow "$C1"; C2=$(count)
echo ">> after EDIT #2 compiles=$C2"

kill "$SERVE" 2>/dev/null || true

echo "compiled-successfully counts: C0=$C0 C1=$C1 C2=$C2"
if [ "$C1" -gt "$C0" ] && [ "$C2" -gt "$C1" ]; then
  VERDICT="WATCH OK — both edits recompiled (#14446 NOT reproduced)"
else
  VERDICT="WATCH STOPPED — second edit ignored (#14446 REPRODUCED)"
fi
echo "VERDICT: $VERDICT"

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    echo "### Faithful serve repro — ${RUNNER_OS:-local}"
    echo "compiles: C0=$C0 C1=$C1 C2=$C2"
    echo ""
    echo "$VERDICT"
  } >>"$GITHUB_STEP_SUMMARY"
fi

echo "===================== dev.log (tail) ====================="
tail -120 dev.log
