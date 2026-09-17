import assert from "node:assert/strict";
import { MessageChannel } from "node:worker_threads";
import {
  diagnostics,
  exceedsBaseline,
  waitForStableHandles,
  snapshot,
} from "./handles.mjs";

export default async function run() {
  const baseline = await waitForStableHandles("unref-detection baseline");
  // Negative control: deliberately retain unref'ed async handles. Verify the
  // detector rejects them, then verify closing them restores the baseline.
  const { port1, port2 } = new MessageChannel();
  try {
    port1.unref();
    port2.unref();
    const current = snapshot();
    assert(
      exceedsBaseline(current, baseline, true),
      diagnostics("unref ports", baseline, current),
    );
    const addresses = new Set(baseline.handles.map((h) => h.address));
    const added = current.handles.filter(
      (h) => h.type === "async" && !addresses.has(h.address),
    );
    assert(added.length >= 2 && added.every((h) => !h.is_referenced));
    await assert.rejects(
      waitForStableHandles("intentional unref handle leak", baseline),
      /intentional unref handle leak/,
    );
  } finally {
    port1.close();
    port2.close();
  }
  await waitForStableHandles("closed ports", baseline);
}
