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

async function main() {
  let resolveCompiler1Done;
  const compiler1Done = new Promise(resolve => {
    resolveCompiler1Done = resolve;
  });

  const fixtureDir = __dirname;
  const compiler1 = rspack({
    context: fixtureDir,
    mode: "development",
    entry: "./entry.js",
    output: {
      path: "/",
      filename: "compiler-1.js",
    },
  });
  compiler1.outputFileSystem = createFsFromVolume(new Volume());
  compiler1.hooks.done.tap("TsfnLifecycleParallelCompiler1Done", () => {
    resolveCompiler1Done();
  });

  let compiler2FilenameCalls = 0;
  let compiler2ProcessAssetsCalls = 0;
  const compiler2 = rspack({
    context: fixtureDir,
    mode: "development",
    entry: "./entry.js",
    output: {
      path: "/",
      filename: () => {
        compiler2FilenameCalls += 1;
        return "compiler-2.js";
      },
    },
  });
  compiler2.outputFileSystem = createFsFromVolume(new Volume());
  compiler2.hooks.make.tapPromise(
    "TsfnLifecycleParallelWaitForCompiler1Done",
    async () => {
      await compiler1Done;
    },
  );
  compiler2.hooks.compilation.tap(
    "TsfnLifecycleParallelCompilation",
    compilation => {
      compilation.hooks.processAssets.tap(
        "TsfnLifecycleParallelProcessAssets",
        () => {
          compiler2ProcessAssetsCalls += 1;
        },
      );
    },
  );

  try {
    await Promise.all([runCompiler(compiler1), runCompiler(compiler2)]);
  } finally {
    await Promise.all([closeCompiler(compiler1), closeCompiler(compiler2)]);
  }

  if (compiler2FilenameCalls === 0) {
    throw new Error("compiler2 output.filename callback was not invoked");
  }

  if (compiler2ProcessAssetsCalls === 0) {
    throw new Error("compiler2 processAssets hook was not invoked");
  }
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
