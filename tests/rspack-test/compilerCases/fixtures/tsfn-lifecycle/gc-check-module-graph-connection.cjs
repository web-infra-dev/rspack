const rspack = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const {
  closeCompiler,
  createGCTracker,
  runCompiler,
} = require("./helpers.cjs");

async function main() {
  const gcTracker = createGCTracker();
  let build = 0;

  const compiler = rspack({
    context: __dirname,
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
            "TsfnLifecycleModuleGraphConnection",
            compilation => {
              compilation.hooks.afterProcessAssets.tap(
                "TsfnLifecycleModuleGraphConnection",
                () => {
                  const entry = compilation.entries.values().next().value;
                  const connection = compilation.moduleGraph.getConnection(
                    entry.dependencies[0],
                  );

                  if (!connection?.module) {
                    throw new Error(
                      "expected an entry module graph connection",
                    );
                  }

                  build += 1;
                  gcTracker.track(
                    connection,
                    `module graph connection ${build}`,
                  );
                },
              );
            },
          );
        },
      },
    ],
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  try {
    let firstStats = await runCompiler(compiler);
    firstStats = null;
    await runCompiler(compiler);

    if (build !== 2) {
      throw new Error(`expected two builds, received ${build}`);
    }

    await gcTracker.waitForCollection("module graph connection 1");
  } finally {
    await closeCompiler(compiler);
  }
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
