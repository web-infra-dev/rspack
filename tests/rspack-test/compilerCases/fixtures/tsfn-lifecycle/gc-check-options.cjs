const rspack = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");

function runCompiler(compiler) {
  return new Promise((resolve, reject) => {
    compiler.run((err, stats) => {
      if (err) return reject(err);
      resolve(stats);
    });
  });
}

function closeCompiler(compiler) {
  return new Promise((resolve, reject) => {
    compiler.close(err => {
      if (err) return reject(err);
      resolve();
    });
  });
}

function delay() {
  return new Promise(resolve => setTimeout(resolve, 0));
}

async function waitForGC(finalized, label) {
  for (let i = 0; i < 100; i++) {
    global.gc();
    await delay();
    if (finalized.has(label)) {
      return;
    }
  }

  throw new Error(`${label} was not garbage collected`);
}

async function main() {
  if (typeof global.gc !== "function") {
    throw new Error("global.gc is unavailable; run this script with --expose-gc");
  }

  const finalized = new Set();
  const registry = new FinalizationRegistry(label => {
    finalized.add(label);
  });

  const fixtureDir = __dirname;
  let compiler;
  let filenameUsedCompilation = false;
  let bannerUsedCompilation = false;

  compiler = rspack((() => {
    let capturedCompilation;

    return {
      context: fixtureDir,
      mode: "development",
      entry: "./entry.js",
      output: {
        path: "/",
        filename: () => {
          if (capturedCompilation) {
            filenameUsedCompilation = true;
            capturedCompilation.hash;
          }
          return "bundle.js";
        },
      },
      plugins: [
        new rspack.BannerPlugin({
          banner: () => {
            if (capturedCompilation) {
              bannerUsedCompilation = true;
              capturedCompilation.hash;
            }
            return "banner";
          },
        }),
        {
          apply(compiler) {
            compiler.hooks.compilation.tap("PLUGIN", compilation => {
              capturedCompilation = compilation;
            });
          }
        }
      ]
    };
  })());
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  let stats = await runCompiler(compiler);
  if (!filenameUsedCompilation) {
    throw new Error("output.filename callback did not observe compilation");
  }
  if (!bannerUsedCompilation) {
    throw new Error("BannerPlugin callback did not observe compilation");
  }
  registry.register(compiler, "option callback compiler");

  await closeCompiler(compiler);

  stats = null;
  compiler = null;

  await waitForGC(finalized, "option callback compiler");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
