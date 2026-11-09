import { createRequire } from "node:module";
import { Worker, isMainThread, workerData } from "node:worker_threads";
import {
  closeCompiler,
  createGCTracker,
  forceGC,
  runCompiler,
} from "./helpers.mjs";

if (isMainThread) {
  // Register an owner env, let it disappear, then compile in a replacement.
  // Both isolates load the same native addon and its process-global registry.
  for (let i = 0; i < 2; i++) {
    await new Promise((resolve, reject) => {
      const worker = new Worker(new URL(import.meta.url), { workerData: i });
      worker.on("error", reject);
      worker.on("exit", code => {
        if (code === 0) resolve();
        else reject(new Error(`compiler env exited with ${code}`));
      });
    });
  }
} else if (workerData === 0) {
  const { registerMainObjectRelease } = createRequire(
    import.meta.resolve("@rspack/core"),
  )("@rspack/binding");
  registerMainObjectRelease(() => {});
} else {
  const keepAlive = setInterval(() => {}, 1000);
  try {
    const { rspack } = await import("@rspack/core");
    const { createFsFromVolume, Volume } = await import("memfs");
    const require = createRequire(import.meta.url);
    const consumerPath = require.resolve("./main-additional-data-loader.cjs");
    const tracker = createGCTracker();
    async function build() {
      const compiler = rspack({
        context: import.meta.dirname,
        mode: "development",
        entry: "./entry.js",
        module: {
          rules: [{
            test: /entry\.js$/,
            use: [
              { loader: consumerPath },
              { loader: "builtin:swc-loader", options: {} },
              { loader: require.resolve("./env-additional-data-loader.cjs") },
            ],
          }],
        },
        output: { path: "/", filename: "bundle.js" },
      });
      tracker.track(compiler, "compiler env owner");
      compiler.outputFileSystem = createFsFromVolume(new Volume());
      try {
        const stats = await runCompiler(compiler);
        if (stats.hasErrors()) throw new Error(stats.toString());
      } finally {
        await closeCompiler(compiler);
      }
    }
    await build();
    await tracker.waitForCollection("compiler env owner");
    const loader = require(consumerPath);
    for (let i = 0; i < 300 && !loader.wasCollected(); i++) {
      await forceGC(1, 5);
    }
    if (!loader.wasCollected()) {
      throw new Error("additionalData was not released in the replacement JS env");
    }
  } finally {
    clearInterval(keepAlive);
  }
}
