const fs = require("fs");
const os = require("os");
const path = require("path");

function createCase(nativeWatcher) {
  const fixturePath = path.join(
    os.tmpdir(),
    `rspack-watch-options-ignored-${nativeWatcher ? "native" : "node"}`
  );
  const entryPath = path.join(fixturePath, "index.js");
  const watchedPath = path.join(fixturePath, "watched.js");
  const ignoredCachePath = path.join(fixturePath, ".cache", "ignored.js");
  const generatedPath = path.join(fixturePath, "generated.js");
  const normalizePath = (item) => {
    if (!item) return item;
    try {
      return fs.realpathSync.native(item);
    } catch {
      return path.resolve(item);
    }
  };
  const includesPath = (paths, expected) =>
    Array.from(paths || []).some(
      (item) => normalizePath(item) === normalizePath(expected)
    );

  return {
    description: `should support mixed watchOptions.ignored with ${nativeWatcher ? "native" : "node"} watcher`,
    options() {
      fs.rmSync(fixturePath, { recursive: true, force: true });
      fs.mkdirSync(path.dirname(ignoredCachePath), { recursive: true });
      fs.writeFileSync(
        entryPath,
        "require('./watched'); require('./.cache/ignored'); require('./generated');",
        "utf-8"
      );
      fs.writeFileSync(watchedPath, "module.exports = 'initial';", "utf-8");
      fs.writeFileSync(ignoredCachePath, "module.exports = 'ignored';", "utf-8");
      fs.writeFileSync(generatedPath, "module.exports = 'generated';", "utf-8");
      return {
        mode: "development",
        context: fixturePath,
        entry: "./index.js",
        experiments: {
          nativeWatcher
        },
        output: {
          path: "/watch-options-ignored",
          filename: "bundle.js"
        }
      };
    },
    async build(context, compiler) {
      await new Promise((resolve, reject) => {
        let initialBuildDone = false;
        let settleTimer;
        let ignoredTimer;
        const cleanup = (cb) => {
          clearTimeout(settleTimer);
          clearTimeout(ignoredTimer);
          watcher.close(() => {
            fs.rmSync(fixturePath, { recursive: true, force: true });
            cb();
          });
        };
        const watcher = compiler.watch(
          {
            aggregateTimeout: 50,
            ignored: ["**/.cache", /generated/g]
          },
          (err, stats) => {
            if (err) return reject(err);
            if (stats?.hasErrors()) return reject(new Error(stats.toString()));
          }
        );
        compiler.hooks.done.tap("WatchOptionsIgnoredTest", () => {
          if (!initialBuildDone) {
            initialBuildDone = true;
            settleTimer = setTimeout(() => {
              fs.writeFileSync(
                ignoredCachePath,
                "module.exports = 'ignored-updated';",
                "utf-8"
              );
              fs.writeFileSync(
                generatedPath,
                "module.exports = 'generated-updated';",
                "utf-8"
              );
              ignoredTimer = setTimeout(() => {
                fs.writeFileSync(
                  watchedPath,
                  "module.exports = 'updated';",
                  "utf-8"
                );
              }, 500);
            }, 1000);
            return;
          }
          if (
            includesPath(compiler.modifiedFiles, ignoredCachePath) ||
            includesPath(compiler.modifiedFiles, generatedPath)
          ) {
            cleanup(() =>
              reject(
                new Error(
                  `ignored files should not trigger rebuild: ${Array.from(
                    compiler.modifiedFiles || []
                  ).join(", ")}`
                )
              )
            );
            return;
          }
          cleanup((err) => (err ? reject(err) : resolve()));
        });
      });
    }
  };
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [createCase(false), createCase(true)];
