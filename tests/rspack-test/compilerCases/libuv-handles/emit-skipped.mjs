import { lifecycle } from "./compiler.mjs";
import { waitForStableHandles } from "./handles.mjs";

// Warm the same path, then require every closed compiler to restore its baseline.
export default async function run() {
  await lifecycle(4, false, false);
  const baseline = await waitForStableHandles("warmup");
  for (let round = 1; round <= 5; round++) {
    await lifecycle(4, false, false);
    await waitForStableHandles(
      `emit-skipped ${round}: closed and collected`,
      baseline,
    );
  }
}
