const path = require("node:path");
const rspack = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const {
  closeCompiler,
  forceGC,
  runCompiler,
} = require("./helpers.cjs");

async function main() {
  const mainLoaderPath = path.join(
    __dirname,
    "main-additional-data-loader.cjs",
  );
  const compiler = rspack({
    context: __dirname,
    mode: "development",
    entry: "./entry.js",
    module: {
      rules: [
        {
          test: /entry\.js$/,
          use: [
            { loader: mainLoaderPath },
            {
              loader: path.join(
                __dirname,
                "parallel-additional-data-loader.cjs",
              ),
              parallel: { maxWorkers: 1 },
            },
          ],
        },
      ],
    },
    output: {
      path: "/",
      filename: "bundle.js",
    },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  try {
    await runCompiler(compiler);
  } finally {
    await closeCompiler(compiler);
  }

  const mainLoader = require(mainLoaderPath);
  for (let i = 0; i < 300; i++) {
    await forceGC(1, 5);
    if (mainLoader.wasCollected()) return;
  }
  throw new Error("loader additional data registry entry was not released");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
