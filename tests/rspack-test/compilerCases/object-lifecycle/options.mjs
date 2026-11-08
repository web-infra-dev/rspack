import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import {
  closeCompiler,
  createGCTracker,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

export default async function run() {
  const gcTracker = createGCTracker();
  const fixtureDir = import.meta.dirname;
  const capturedCompilations = [];
  let filenameCalls = 0;
  let bannerCalls = 0;
  let filenameUsedCompiler = false;
  let filenameUsedCompilation = false;
  let bannerUsedCompiler = false;
  let bannerUsedCompilation = false;

  let compiler = rspack(
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
              filenameUsedCompiler = true;
              compilerRef.outputFileSystem;
            }
            if (latestCompilation) {
              filenameUsedCompilation = true;
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
                bannerUsedCompiler = true;
                compilerRef.outputFileSystem;
              }
              if (latestCompilation) {
                bannerUsedCompilation = true;
                latestCompilation.hash;
              }
              return "banner";
            },
          }),
          {
            apply(compiler) {
              compilerRef = compiler;
              compiler.hooks.compilation.tap(
                "TsfnLifecycleOptionCapture",
                (compilation) => {
                  latestCompilation = compilation;
                  capturedCompilations.push(compilation);
                },
              );
            },
          },
        ],
      };
    })(),
  );
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  let firstStats;
  let firstCompilation;
  let secondStats;
  let secondCompilation;
  try {
    firstStats = await runCompiler(compiler);
    if (
      !filenameUsedCompiler ||
      !filenameUsedCompilation ||
      !bannerUsedCompiler ||
      !bannerUsedCompilation
    ) {
      throw new Error(
        "option callbacks did not observe both compiler and compilation",
      );
    }

    firstCompilation = capturedCompilations[0];
    gcTracker.track(firstCompilation, "first option compilation");
    firstStats = null;
    firstCompilation = null;
    capturedCompilations[0] = null;

    secondStats = await runCompiler(compiler);
    secondCompilation = capturedCompilations[capturedCompilations.length - 1];
    gcTracker.track(secondCompilation, "second option compilation");

    if (filenameCalls < 2 || bannerCalls < 2) {
      throw new Error("option callbacks were not invoked for both builds");
    }

    await gcTracker.waitForCollection("first option compilation");

    gcTracker.track(compiler, "option compiler");
  } finally {
    await closeCompiler(compiler);
  }

  secondStats = null;
  secondCompilation = null;
  capturedCompilations[capturedCompilations.length - 1] = null;
  compiler = null;

  await gcTracker.waitForCollection("second option compilation");
  await gcTracker.waitForCollection("option compiler");
}
