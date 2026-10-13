import assert from "node:assert/strict";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import {
  closeCompiler,
  createGCTracker,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

export default async function run() {
  const tracker = createGCTracker();
  const labels = [];
  const compiler = rspack({
    context: import.meta.dirname,
    mode: "development",
    entry: "./entry.js",
    output: {
      path: "/",
      filename: "bundle.js",
      assetModuleFilename: "[name][ext]",
    },
    module: {
      rules: [
        { test: /entry\.js$/, use: ["./import-module-loader.mjs"] },
        { test: /\.txt$/, type: "asset/resource" },
      ],
    },
    plugins: [
      (compiler) => {
        compiler.hooks.compilation.tap("ImportModuleLifetime", (compilation) => {
          rspack.NormalModule.getCompilationHooks(compilation).loader.tap(
            "ImportModuleLifetime",
            (context) => {
              context.trackImportCallback = (callback, captured) => {
                const label = `import ${labels.length}`;
                tracker.track(callback, `${label} callback`);
                tracker.track(captured, `${label} capture`);
                labels.push(label);
              };
            },
          );
        });
      },
    ],
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  try {
    for (let build = 0; build < 2; build++) {
      const start = labels.length;
      const stats = await runCompiler(compiler);
      assert(!stats.hasErrors(), stats.toString({ all: false, errors: true }));
      assert.equal(labels.length - start, 3, "the loader must run on every build");
      // Keep both compiler and the current compilation alive during GC.
      for (const label of labels.slice(start)) {
        await tracker.waitForCollection(`${label} callback`);
        await tracker.waitForCollection(`${label} capture`);
      }
    }
  } finally {
    await closeCompiler(compiler);
  }
}
