const rspack = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const {
  closeCompiler,
  createGCTracker,
  runCompiler,
} = require("./helpers.cjs");

async function main() {
  const gcTracker = createGCTracker();
  const fixtureDir = __dirname;
  let observedChunks = false;

  let compiler = rspack({
    context: fixtureDir,
    mode: "development",
    entry: "./entry.js",
    output: {
      path: "/",
      filename: "bundle.js",
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.compilation.tap(
            "TsfnLifecycleChunks",
            compilation => {
              compilation.hooks.afterProcessAssets.tap(
                "TsfnLifecycleChunks",
                () => {
                  const chunks = Array.from(compilation.chunks);
                  const chunkGroups = Array.from(compilation.chunkGroups);

                  if (chunks.length === 0) {
                    throw new Error("expected at least one chunk");
                  }

                  if (chunkGroups.length === 0) {
                    throw new Error("expected at least one chunk group");
                  }

                  let chunk = chunks[0];
                  let chunkGroup = chunkGroups[0];

                  gcTracker.track(chunk, "chunk");
                  gcTracker.track(chunkGroup, "chunk group");

                  chunk = null;
                  chunkGroup = null;
                  chunks.length = 0;
                  chunkGroups.length = 0;
                  observedChunks = true;
                },
              );
            },
          );
        },
      },
    ],
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  let stats = await runCompiler(compiler);
  if (!observedChunks) {
    throw new Error("chunks were not observed");
  }

  gcTracker.track(compiler, "chunk compiler");

  await closeCompiler(compiler);

  stats = null;
  compiler = null;

  await gcTracker.waitForCollection("chunk compiler");
  await gcTracker.waitForCollection("chunk");
  await gcTracker.waitForCollection("chunk group");
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
