// TEMP DIAG (revert before merge): diagnose the `afterAll hook timed out in 10000ms`
// flake on Config.part2 / RuntimeModeConfig.part2 (afterAll -> tester.resume ->
// compiler.close native callback). The decisive question: when close hangs, is the
// event loop ALIVE (genuine native-close stall) or STARVED (cross-process CPU contention)?
//
// A watchdog timer scheduled at a fixed delay answers it:
//   - fires on time (firedAfter ~= scheduled) while close still pending => loop alive => native stall
//   - fires much later than scheduled                                   => loop was starved
// The process-wide heartbeat additionally tags which pid stalled and for how long.

const PID = process.pid;

function log(msg: string): void {
  // rstest CI config sets disableConsoleIntercept:true, so this reaches the job log.
  process.stdout.write(`[CLOSEDIAG] pid=${PID} t=${Date.now()} ${msg}\n`);
}

log('armed'); // confirms the instrumentation compiled & this worker process loaded it

let lastTick = Date.now();
let maxLag = 0;
const HEARTBEAT_MS = 250;
const timer = setInterval(() => {
  const now = Date.now();
  const lag = now - lastTick - HEARTBEAT_MS;
  lastTick = now;
  if (lag > maxLag) maxLag = lag;
  if (lag > 300) log(`LAG gap=${lag}ms (loop stalled)`);
}, HEARTBEAT_MS);
if (typeof timer.unref === 'function') timer.unref();

function dumpAsync(): string {
  const p = process as unknown as {
    _getActiveRequests?: () => unknown[];
    _getActiveHandles?: () => unknown[];
  };
  const reqs = p._getActiveRequests?.() ?? [];
  const handles = p._getActiveHandles?.() ?? [];
  const tally = (arr: unknown[]) => {
    const m: Record<string, number> = {};
    for (const x of arr) {
      const n =
        (x as { constructor?: { name?: string } })?.constructor?.name ??
        typeof x;
      m[n] = (m[n] ?? 0) + 1;
    }
    return JSON.stringify(m);
  };
  const mem = process.memoryUsage();
  return `reqs=${reqs.length}${tally(reqs)} handles=${handles.length}${tally(handles)} rss=${Math.round(mem.rss / 1048576)}MB heap=${Math.round(mem.heapUsed / 1048576)}MB`;
}

export async function withCloseDiag(
  name: string,
  close: () => Promise<void>,
): Promise<void> {
  const start = Date.now();
  const lagAtStart = maxLag;
  let settled = false;

  const watch = (scheduledMs: number) =>
    setTimeout(() => {
      if (settled) return;
      const firedAfter = Date.now() - start;
      log(
        `name="${name}" scheduled=+${scheduledMs}ms firedAfter=${firedAfter}ms pending=${!settled} maxLagSinceStart=${maxLag - lagAtStart}ms ${dumpAsync()}`,
      );
    }, scheduledMs);

  const t4 = watch(4000);
  const t9 = watch(9000);

  try {
    await close();
  } finally {
    settled = true;
    const dur = Date.now() - start;
    clearTimeout(t4);
    clearTimeout(t9);
    if (dur > 1000) {
      log(
        `name="${name}" RESOLVED dur=${dur}ms maxLagSinceStart=${maxLag - lagAtStart}ms`,
      );
    }
  }
}
