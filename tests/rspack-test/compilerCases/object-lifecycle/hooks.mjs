import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import {
  closeCompiler,
  createGCTracker,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

export default async function run() {
  const gcTracker = createGCTracker();
  const fixtureDir = import.meta.dirname;

  let compiler = rspack({
    context: fixtureDir,
    mode: "development",
    entry: "./entry.js",
    output: {
      path: "/",
      filename: "bundle.js",
    },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  const capturedCompilations = [];
  let processAssetsCalls = 0;
  let processAssetsUsedCompiler = false;
  let processAssetsUsedCompilation = false;
  {
    const compilerRef = compiler;
    compiler.hooks.compilation.tap("TsfnLifecycleHooks", (compilation) => {
      capturedCompilations.push(compilation);
      const compilationRef = compilation;

      compilation.hooks.processAssets.tap("TsfnLifecycleHooks", () => {
        processAssetsCalls += 1;
        processAssetsUsedCompiler = true;
        processAssetsUsedCompilation = true;
        compilerRef.outputFileSystem;
        compilationRef.hash;
      });
    });
  }

  let firstStats;
  let firstCompilation;
  let secondStats;
  let secondCompilation;
  try {
    firstStats = await runCompiler(compiler);
    if (!processAssetsUsedCompiler || !processAssetsUsedCompilation) {
      throw new Error(
        "hook closures did not observe both compiler and compilation",
      );
    }

    firstCompilation = capturedCompilations[0];
    gcTracker.track(firstCompilation, "first hook compilation");
    firstStats = null;
    firstCompilation = null;
    capturedCompilations[0] = null;

    secondStats = await runCompiler(compiler);
    secondCompilation = capturedCompilations[capturedCompilations.length - 1];
    gcTracker.track(secondCompilation, "second hook compilation");

    if (processAssetsCalls < 2) {
      throw new Error("hook closures were not invoked for both builds");
    }

    await gcTracker.waitForCollection("first hook compilation");

    gcTracker.track(compiler, "hook compiler");
  } finally {
    await closeCompiler(compiler);
  }

  secondStats = null;
  secondCompilation = null;
  capturedCompilations[capturedCompilations.length - 1] = null;
  compiler = null;

  await gcTracker.waitForCollection("second hook compilation");
  await gcTracker.waitForCollection("hook compiler");
}
