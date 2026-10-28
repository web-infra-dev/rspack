import {
  createCompiler,
  buildAndCheckLoaders,
  lifecycle,
} from "./compiler.mjs";
import { closeCompiler } from "@rspack/test-tools/helper/lifecycle";
import { waitForStableHandles } from "./handles.mjs";
async function measureAsyncHandleDelta(count, processBaseline) {
  const { compiler, loaderState } = createCompiler(count);
  try {
    await buildAndCheckLoaders(compiler, loaderState, count, false);
    // Measure while the compiler is still alive: per-module handles can hurt
    // the event loop during use even if they are all released after close.
    const current = await waitForStableHandles(
      `live compiler with ${count} modules`,
    );
    return current.counts.async - processBaseline.counts.async;
  } finally {
    await closeCompiler(compiler);
  }
}

export default async function run(count) {
  // Separate processes compare relative counts, not platform-specific totals.
  await lifecycle(4);
  const processBaseline = await waitForStableHandles("warmup");
  const asyncDelta = await measureAsyncHandleDelta(count, processBaseline);
  await waitForStableHandles("module-count: closed and collected", {
    baseline: processBaseline,
  });
  return { asyncDelta };
}
