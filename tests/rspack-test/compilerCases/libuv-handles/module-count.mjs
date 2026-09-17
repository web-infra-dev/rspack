import { createCompiler, build, lifecycle } from "./compiler.mjs";
import { closeCompiler } from "@rspack/test-tools/helper/lifecycle";
import { settle } from "./handles.mjs";
async function scale(count, baseline) {
  const { compiler, state } = createCompiler(count);
  try {
    await build(compiler, state, count, false);
    // Measure while the compiler is still alive: per-module handles can hurt
    // the event loop during use even if they are all released after close.
    const current = await settle(`live compiler with ${count} modules`);
    return current.counts.async - baseline.counts.async;
  } finally {
    await closeCompiler(compiler);
  }
}

export default async function run(count) {
  // Separate processes compare relative counts, not platform-specific totals.
  await lifecycle(4);
  const baseline = await settle("warmup");
  const asyncDelta = await scale(count, baseline);
  await settle("module-count: closed and collected", baseline);
  return { asyncDelta };
}
