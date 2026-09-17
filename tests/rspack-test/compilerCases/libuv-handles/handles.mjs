import assert from "node:assert/strict";
import { setImmediate, setTimeout } from "node:timers/promises";

// Initialize stdio before taking a baseline. Reports otherwise observe handles
// created by the diagnostic output itself.
process.stdout;
process.stderr;
process.report.excludeNetwork = true;

export function snapshot() {
  const handles = process.report
    .getReport()
    .libuv.filter((h) => h.type !== "loop");
  const counts = {};
  for (const handle of handles) {
    // Unreferenced/inactive async handles still cost libuv traversal work.
    counts[handle.type] = (counts[handle.type] || 0) + 1;
  }
  return { counts, handles };
}

export function exceedsBaseline(current, baseline, asyncOnly = false) {
  return Object.entries(current.counts).some(
    ([type, count]) =>
      (!asyncOnly || type === "async") && count > (baseline.counts[type] || 0),
  );
}

export function diagnostics(stage, baseline, current) {
  const addresses = new Set(baseline.handles.map((h) => h.address));
  return JSON.stringify(
    {
      stage,
      baseline: baseline.counts,
      actual: current.counts,
      addedHandles: current.handles.filter((h) => !addresses.has(h.address)),
    },
    null,
    2,
  );
}

export async function waitForStableHandles(stage, baseline, asyncOnly = false) {
  assert.equal(typeof global.gc, "function", "requires --expose-gc");
  const deadline = performance.now() + 5000;
  let previous;
  let consecutive = 0;
  let current;
  do {
    global.gc();
    await setTimeout(10);
    await setImmediate();
    current = snapshot();
    const counts = JSON.stringify(Object.entries(current.counts).sort());
    const acceptable =
      !baseline || !exceedsBaseline(current, baseline, asyncOnly);
    consecutive = acceptable ? (counts === previous ? consecutive + 1 : 1) : 0;
    if (consecutive >= 3) return current;
    previous = counts;
  } while (performance.now() < deadline);
  throw new Error(
    diagnostics(stage, baseline || { counts: {}, handles: [] }, current),
  );
}
