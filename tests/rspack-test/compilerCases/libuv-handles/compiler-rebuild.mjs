import { createCompiler, build, lifecycle } from "./compiler.mjs";
import { closeCompiler } from "@rspack/test-tools/helper/lifecycle";
import { settle } from "./handles.mjs";
async function repeatedBuilds() {
  const { compiler, state } = createCompiler(4);
  try {
    // The first build clears hook caches on completion; rebuilds retain them
    // until the next compilation. Warm the rebuild path before comparing it.
    await build(compiler, state, 4, false);
    await build(compiler, state, 4, false);
    const liveBaseline = await settle("compiler-rebuild warmup");
    for (let round = 1; round <= 5; round++) {
      await build(compiler, state, 4, false);
      await settle(`compiler-rebuild ${round}`, liveBaseline, true);
    }
  } finally {
    await closeCompiler(compiler);
  }
}

export default async function run() {
  await lifecycle(4);
  const baseline = await settle("warmup");
  await repeatedBuilds();
  await settle("compiler-rebuild: closed and collected", baseline);
}
