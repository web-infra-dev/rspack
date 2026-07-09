const rspack = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const { closeCompiler, runCompiler } = require("./helpers.cjs");

async function main() {
  const compiler = rspack({
    context: __dirname,
    mode: "development",
    entry: "./entry.js",
    output: {
      path: "/",
      filename: "bundle.js",
    },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  const stats = await runCompiler(compiler);
  if (!compiler._lastCompilation) {
    throw new Error(
      "Expected compiler to hold the last compilation before close",
    );
  }

  await closeCompiler(compiler);

  if (compiler._lastCompilation !== undefined) {
    throw new Error("Expected compiler._lastCompilation to be cleared after close");
  }

  try {
    stats.toJson();
  } catch (error) {
    if (
      error.message.includes(
        "Unable to access `Stats` after the compiler was shutdown",
      )
    ) {
      return;
    }
    throw error;
  }

  throw new Error("Expected stats.toJson() after close to fail");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
