import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import {
  closeCompiler,
  forceGC,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

export default async function run() {
  const fixtureDir = import.meta.dirname;
  let filenameCalls = 0;
  let bannerCalls = 0;
  let observedCompiler = false;
  let observedCompilation = false;

  const compiler = rspack(
    (() => {
      let compilerRef;
      let latestCompilation;

      return {
        context: fixtureDir,
        mode: "development",
        entry: "./entry.js",
        output: {
          path: "/",
          filename: () => {
            filenameCalls += 1;
            if (compilerRef) {
              observedCompiler = true;
              compilerRef.outputFileSystem;
            }
            if (latestCompilation) {
              observedCompilation = true;
              latestCompilation.hash;
            }
            return "bundle.js";
          },
        },
        plugins: [
          new rspack.BannerPlugin({
            banner: () => {
              bannerCalls += 1;
              if (compilerRef) {
                observedCompiler = true;
                compilerRef.outputFileSystem;
              }
              if (latestCompilation) {
                observedCompilation = true;
                latestCompilation.hash;
              }
              return "banner";
            },
          }),
          {
            apply(compiler) {
              compilerRef = compiler;
              compiler.hooks.compilation.tap(
                "TsfnLifecycleOptionMultipleBuilds",
                (compilation) => {
                  latestCompilation = compilation;
                },
              );
            },
          },
        ],
      };
    })(),
  );
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  try {
    await runCompiler(compiler);
    await forceGC(5);
    await runCompiler(compiler);
    await forceGC(5);
    await runCompiler(compiler);
  } finally {
    await closeCompiler(compiler);
  }

  if (!observedCompiler || !observedCompilation) {
    throw new Error(
      "option callbacks did not observe both compiler and compilation",
    );
  }

  if (filenameCalls < 3 || bannerCalls < 3) {
    throw new Error("option callbacks were not invoked across repeated builds");
  }
}
