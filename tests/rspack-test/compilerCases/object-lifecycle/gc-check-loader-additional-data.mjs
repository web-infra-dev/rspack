import path from "node:path";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import {
  closeCompiler,
  forceGC,
  runCompiler,
} from "./helpers.mjs";

async function main() {
  const mainLoaderPath = path.join(
    import.meta.dirname,
    "main-additional-data-loader.cjs",
  );
  const compiler = rspack({
    context: import.meta.dirname,
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
                import.meta.dirname,
                "parallel-additional-data-loader.cjs",
              ),
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
