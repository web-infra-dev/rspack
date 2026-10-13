import assert from "node:assert/strict";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import {
  closeCompiler,
  createGCTracker,
  forceGC,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

export default async function run() {
  const tracker = createGCTracker();
  const refs = [];
  let generation = 0;

  class CustomRuntimeModule extends rspack.RuntimeModule {
    constructor(build, stage) {
      super("runtime-module-multiple-builds", stage);
      this.build = build;
    }

    generate() {
      return `// runtime module from build ${this.build}`;
    }
  }

  const compiler = rspack({
    context: import.meta.dirname,
    mode: "development",
    entry: "./entry.js",
    output: { path: "/", filename: "bundle.js" },
    plugins: [
      (compiler) => {
        compiler.hooks.thisCompilation.tap("RuntimeModuleLifetime", (compilation) => {
          const build = ++generation;
          compilation.hooks.additionalTreeRuntimeRequirements.tap(
            "RuntimeModuleLifetime",
            (chunk) => {
              if (build === 1) {
                const rejected = new CustomRuntimeModule(0, "invalid stage");
                tracker.track(rejected, "rejected module");
                // The generator is converted before the invalid stage is read.
                assert.throws(
                  () => compilation.addRuntimeModule(chunk, rejected),
                  /JsAddingRuntimeModule\.stage/,
                );
              }

              const module = new CustomRuntimeModule(build);
              tracker.track(module, `module ${build}`);
              refs.push(new WeakRef(module));
              compilation.addRuntimeModule(chunk, module);
            },
          );
        });
      },
    ],
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  try {
    for (let build = 1; build <= 3; build++) {
      const stats = await runCompiler(compiler);
      assert(!stats.hasErrors(), stats.toString({ all: false, errors: true }));
      assert.equal(generation, build);
      assert.equal(refs.length, build);
      assert(
        compiler.outputFileSystem.readFileSync("/bundle.js", "utf8")
          .includes(`// runtime module from build ${build}`),
        "the current generator must remain callable",
      );
      await forceGC(2);
      assert(refs[build - 1].deref(), "the native module keeps its JS module alive");

      // Incremental artifacts can retain the previous native compilation.
      // Once that owner is gone, the manager must not keep older modules alive.
      if (build === 3) {
        await tracker.waitForCollection("module 1");
        await tracker.waitForCollection("rejected module");
      }
    }
  } finally {
    await closeCompiler(compiler);
  }

  // Keep the compiler alive: close must revoke callbacks in its remaining
  // native modules even though those consumers have not been destroyed yet.
  await tracker.waitForCollection("module 2");
  await tracker.waitForCollection("module 3");
  assert(compiler);
}
