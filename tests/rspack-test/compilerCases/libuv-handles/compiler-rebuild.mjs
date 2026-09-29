import {
  createCompiler,
  buildAndCheckLoaders,
  lifecycle,
} from "./compiler.mjs";
import { closeCompiler } from "@rspack/test-tools/helper/lifecycle";
import { waitForStableHandles } from "./handles.mjs";
async function repeatedBuilds() {
  const { compiler, loaderState } = createCompiler(4);
  try {
    // The first build clears hook caches on completion; rebuilds retain them
    // until the next compilation. Warm the rebuild path before comparing it.
    await buildAndCheckLoaders(compiler, loaderState, 4, false);
    await buildAndCheckLoaders(compiler, loaderState, 4, false);
    const rebuildBaseline = await waitForStableHandles(
      "compiler-rebuild warmup",
    );
    for (let round = 1; round <= 5; round++) {
      await buildAndCheckLoaders(compiler, loaderState, 4, false);
      await waitForStableHandles(`compiler-rebuild ${round}`, {
        baseline: rebuildBaseline,
        baselineScope: "async",
      });
    }
  } finally {
    await closeCompiler(compiler);
  }
}

export default async function run() {
  await lifecycle(4);
  const processBaseline = await waitForStableHandles("warmup");
  await repeatedBuilds();
  await waitForStableHandles("compiler-rebuild: closed and collected", {
    baseline: processBaseline,
  });
}
