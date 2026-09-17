import { lifecycle } from "./compiler.mjs";
import { settle } from "./handles.mjs";

// Warm the same path, then require every closed compiler to restore its baseline.
export default async function run() {
  await lifecycle(4, false, true);
  const baseline = await settle("warmup");
  for (let round = 1; round <= 5; round++) {
    await lifecycle(4, false, true);
    await settle(`compiler-close ${round}: closed and collected`, baseline);
  }
}
