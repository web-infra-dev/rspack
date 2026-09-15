const rspack = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const { closeCompiler, runCompiler } = require("./helpers.cjs");

const EXPECTED_MESSAGE =
  "Rspack compiler has already been closed by `compiler.close()`. Do not call Rspack compiler APIs after close; create a new compiler instead.";

async function main() {
  const compiler = rspack({
    context: __dirname,
    mode: "development",
    entry: "./entry.js",
    output: {
      path: "/",
      filename: () => "bundle.js",
    },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  await runCompiler(compiler);
  await closeCompiler(compiler);

  try {
    await runCompiler(compiler);
  } catch (error) {
    if (!error.message.includes(EXPECTED_MESSAGE)) {
      throw new Error(
        `Expected closed compiler error to include:\n${EXPECTED_MESSAGE}\n\nReceived:\n${error.message}`,
      );
    }
    return;
  }

  throw new Error("Expected compiler.run() after compiler.close() to fail");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
