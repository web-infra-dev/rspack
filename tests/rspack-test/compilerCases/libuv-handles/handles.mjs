import assert from "node:assert/strict";
import { setImmediate, setTimeout } from "node:timers/promises";

// Initialize stdio before taking a baseline. Reports otherwise observe handles
// created by the diagnostic output itself.
process.stdout;
process.stderr;
process.report.excludeNetwork = true;

const CLEANUP_TIMEOUT_MS = 5000;
const SAMPLE_INTERVAL_MS = 10;
const REQUIRED_STABLE_SAMPLES = 3;

export function captureHandleSnapshot() {
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

/** @param {{ baselineScope?: "all" | "async" }} options */
export function hasHandleCountGrowth(
  current,
  baseline,
  { baselineScope = "all" } = {},
) {
  return Object.entries(current.counts).some(
    ([type, count]) =>
      (baselineScope === "all" || type === "async") &&
      count > (baseline.counts[type] || 0),
  );
}

export function formatHandleDiagnostics(stage, baseline, current) {
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

/**
 * Without a baseline, return a stable snapshot. With a baseline, also require
 * the selected counts to stay within it. Stability always compares all types;
 * baselineScope only selects which types are checked for growth.
 * @param {string} stage
 * @param {{ baseline?: ReturnType<typeof captureHandleSnapshot>, baselineScope?: "all" | "async" }} options
 */
export async function waitForStableHandles(
  stage,
  { baseline, baselineScope = "all" } = {},
) {
  assert.equal(typeof global.gc, "function", "requires --expose-gc");
  const deadline = performance.now() + CLEANUP_TIMEOUT_MS;
  let previousCountsSignature;
  let stableSamples = 0;
  let current;
  do {
    global.gc();
    await setTimeout(SAMPLE_INTERVAL_MS);
    await setImmediate();
    current = captureHandleSnapshot();
    const countsSignature = JSON.stringify(
      Object.entries(current.counts).sort(),
    );
    const withinBaseline =
      !baseline || !hasHandleCountGrowth(current, baseline, { baselineScope });
    if (!withinBaseline) {
      stableSamples = 0;
    } else if (countsSignature === previousCountsSignature) {
      stableSamples++;
    } else {
      stableSamples = 1;
    }
    if (stableSamples >= REQUIRED_STABLE_SAMPLES) return current;
    previousCountsSignature = countsSignature;
  } while (performance.now() < deadline);
  throw new Error(
    formatHandleDiagnostics(
      stage,
      baseline || { counts: {}, handles: [] },
      current,
    ),
  );
}
