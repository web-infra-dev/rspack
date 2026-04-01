const rspack = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const { closeCompiler, forceGC, runCompiler } = require("./helpers.cjs");

async function main() {
  const fixtureDir = __dirname;
  let filenameCalls = 0;
  let bannerCalls = 0;
  let observedCompiler = false;
  let observedCompilation = false;

  const compiler = rspack((() => {
    let compilerRef;
    let latestCompilation;

    return {
      context: fixtureDir,
      mode: "development",
      entry: "./entry.js",
      output: {
        path: "/",
        filename: () => {
          filenameCalls += 1;
          if (compilerRef) {
            observedCompiler = true;
            compilerRef.outputFileSystem;
          }
          if (latestCompilation) {
            observedCompilation = true;
            latestCompilation.hash;
          }
          return "bundle.js";
        },
      },
      plugins: [
        new rspack.BannerPlugin({
          banner: () => {
            bannerCalls += 1;
            if (compilerRef) {
              observedCompiler = true;
              compilerRef.outputFileSystem;
            }
            if (latestCompilation) {
              observedCompilation = true;
              latestCompilation.hash;
            }
            return "banner";
          },
        }),
        {
          apply(compiler) {
            compilerRef = compiler;
            compiler.hooks.compilation.tap(
              "TsfnLifecycleOptionMultipleBuilds",
              compilation => {
                latestCompilation = compilation;
              },
            );
          },
        },
      ],
    };
  })());
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  try {
    await runCompiler(compiler);
    await forceGC(5);
    await runCompiler(compiler);
    await forceGC(5);
    await runCompiler(compiler);
  } finally {
    await closeCompiler(compiler);
  }

  if (!observedCompiler || !observedCompilation) {
    throw new Error("option callbacks did not observe both compiler and compilation");
  }

  if (filenameCalls < 3 || bannerCalls < 3) {
    throw new Error("option callbacks were not invoked across repeated builds");
  }
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
