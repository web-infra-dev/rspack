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

async function waitForCollection(finalized, label) {
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
  compiler.hooks.compilation.tap("TsfnLifecyclePlugin", compilation => {
    capturedCompilations.push(compilation);
    compilation.hooks.processAssets.tap("TsfnLifecyclePlugin", () => { });
  });

  let firstStats = await runCompiler(compiler);
  let firstCompilation = capturedCompilations[0];
  registry.register(firstCompilation, "first compilation");
  firstStats = null;
  firstCompilation = null;
  capturedCompilations[0] = null;

  let secondStats = await runCompiler(compiler);
  let secondCompilation = capturedCompilations[capturedCompilations.length - 1];
  registry.register(secondCompilation, "second compilation");

  await waitForCollection(finalized, "first compilation");

  registry.register(compiler, "compiler");

  await closeCompiler(compiler);

  secondStats = null;
  secondCompilation = null;
  capturedCompilations[capturedCompilations.length - 1] = null;
  compiler = null;

  await waitForCollection(finalized, "second compilation");
  await waitForCollection(finalized, "compiler");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
