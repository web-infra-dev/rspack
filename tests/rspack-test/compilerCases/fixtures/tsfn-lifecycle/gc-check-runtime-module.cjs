const rspack = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const {
  closeCompiler,
  createGCTracker,
  runCompiler,
} = require("./helpers.cjs");

class CustomRuntimeModule extends rspack.RuntimeModule {
  constructor() {
    super("tsfn-lifecycle");
  }

  generate() {
    return "";
  }
}

async function main() {
  const gcTracker = createGCTracker();
  const fixtureDir = __dirname;
  let compilation;
  let runtimeModule;

  let compiler = rspack({
    context: fixtureDir,
    mode: "development",
    entry: "./entry.js",
    output: {
      path: "/",
      filename: "bundle.js",
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.thisCompilation.tap(
            "TsfnLifecycleRuntimeModule",
            currentCompilation => {
              compilation = currentCompilation;
              currentCompilation.hooks.additionalTreeRuntimeRequirements.tap(
                "TsfnLifecycleRuntimeModule",
                chunk => {
                  runtimeModule = new CustomRuntimeModule();
                  currentCompilation.addRuntimeModule(chunk, runtimeModule);
                },
              );
            },
          );
        },
      },
    ],
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  let stats = await runCompiler(compiler);
  if (!compilation || !runtimeModule) {
    throw new Error("custom runtime module was not added to the compilation");
  }

  gcTracker.track(compilation, "custom runtime module compilation");
  gcTracker.track(runtimeModule, "custom runtime module");
  gcTracker.track(compiler, "custom runtime module compiler");

  await closeCompiler(compiler);
  stats = null;
  compilation = null;
  runtimeModule = null;
  compiler = null;

  await gcTracker.waitForCollection("custom runtime module");
  await gcTracker.waitForCollection("custom runtime module compilation");
  await gcTracker.waitForCollection("custom runtime module compiler");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
