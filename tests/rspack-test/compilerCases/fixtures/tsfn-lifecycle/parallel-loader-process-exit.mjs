import path from "node:path";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import { closeCompiler, runCompiler } from "./helpers.mjs";

async function main() {
  const compiler = rspack({
    context: import.meta.dirname,
    mode: "development",
    entry: "./entry.js",
    module: {
      rules: [
        {
          test: /entry\.js$/,
          // First registration happens during module resolution, after make.
          use: () => [
            {
              loader: path.join(import.meta.dirname, "parallel-loader.cjs"),
              parallel: { maxWorkers: 1 },
              options: {},
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
    const stats = await runCompiler(compiler);
    if (stats.hasErrors()) throw new Error(stats.toString());
  } finally {
    await closeCompiler(compiler);
  }
  console.log("parallel-loader-complete");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
