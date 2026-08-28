const path = require("node:path");
const rspack = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const { closeCompiler, runCompiler } = require("./helpers.cjs");

async function main() {
  const compiler = rspack({
    context: __dirname,
    mode: "development",
    entry: "./entry.js",
    module: {
      rules: [
        {
          test: /entry\.js$/,
          use: [
            {
              loader: path.join(__dirname, "parallel-loader.cjs"),
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
  console.log("parallel-loader-complete");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
