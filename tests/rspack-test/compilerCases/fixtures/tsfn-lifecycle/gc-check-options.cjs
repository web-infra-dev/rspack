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

async function waitForCollection(ref, label) {
  for (let i = 0; i < 100; i++) {
    global.gc();
    await new Promise(resolve => setTimeout(resolve, 0));
    if (ref.deref() === undefined) {
      return;
    }
  }

  throw new Error(`${label} was not garbage collected`);
}

async function main() {
  if (typeof global.gc !== "function") {
    throw new Error("global.gc is unavailable; run this script with --expose-gc");
  }

  const fixtureDir = __dirname;
  let compiler;
  const filename = () => {
    if (compiler && compiler.running) {
      return "bundle.js";
    }
    return "bundle.js";
  };

  compiler = rspack({
    context: fixtureDir,
    mode: "development",
    entry: "./entry.js",
    output: {
      path: "/",
      filename,
    },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  let stats = await runCompiler(compiler);
  const compilerRef = new WeakRef(compiler);

  await closeCompiler(compiler);

  stats = null;
  compiler = null;

  await waitForCollection(compilerRef, "option callback compiler");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
