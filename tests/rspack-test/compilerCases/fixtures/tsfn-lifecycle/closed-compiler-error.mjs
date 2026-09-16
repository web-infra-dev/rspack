import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import { closeCompiler, runCompiler } from "./helpers.mjs";

const EXPECTED_MESSAGE =
  "Rspack compiler has already been closed by `compiler.close()`. Do not call Rspack compiler APIs after close; create a new compiler instead.";

async function main() {
  const compiler = rspack({
    context: import.meta.dirname,
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
