const fs = require("fs");

const getChanges = (compiler) => {
  const modifiedFiles = compiler.modifiedFiles;
  const removedFiles = compiler.removedFiles;
  return {
    removed: removedFiles && [...removedFiles],
    modified: modifiedFiles && [...modifiedFiles]
  };
};

let changes = null;
/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
  description: "should not track modified/removed files during initial watchRun",
  options(context) {
    changes = null;
    return {
      entry: context.getDist("temp-file.js"),
      output: {
        path: context.getDist("dist"),
        filename: "bundle.js"
      },
    };
  },
  async compiler(context, compiler) {
    compiler.hooks.watchRun.tap("ChangesAndRemovalsTest", (compiler) => {
      changes = getChanges(compiler);
    });
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      const watcher = compiler.watch({ aggregateTimeout: 200 }, (err) => {
        if (err) reject(err);
        watcher.close(resolve);
      });
    });
  },
  async check() {
    expect(changes).toEqual({
      removed: [],
      modified: []
    });
  }
}, {
  description: "should track modified files when they've been modified",
  options(context) {
    changes = null;
    return {
      entry: context.getDist("temp-file.js"),
      output: {
        path: context.getDist("dist"),
        filename: "bundle.js"
      },
    };
  },
  async build(context, compiler) {
    let firstWatchRun = true;
    let firstDone = true;
    return new Promise((resolve, reject) => {
      let watcher = null;
      compiler.hooks.watchRun.tap("ChangesAndRemovalsTest", (compiler) => {
        if (firstWatchRun) {
          firstWatchRun = false;
          return;
        }
        changes = getChanges(compiler);
        watcher.close(resolve);
      });
      compiler.hooks.done.tap("ChangesAndRemovalsTest", () => {
        if (!firstDone) return;
        firstDone = false;
        setTimeout(() => {
          fs.appendFileSync(context.getDist("temp-file.js"), "\nlet x = 'file modified';");
        }, 300);
      });
      watcher = compiler.watch({ aggregateTimeout: 200 }, (err) => {
        if (err) reject(err);
      });
    });
  },
  async check({ context }) {
    expect(changes).toEqual({
      removed: [],
      modified: [context.getDist("temp-file.js")]
    });
  }
}, {
  description: "should track removed file when removing file",
  options(context) {
    changes = null;
    return {
      entry: context.getDist("temp-file.js"),
      output: {
        path: context.getDist("dist"),
        filename: "bundle.js"
      },
    };
  },
  async build(context, compiler) {
    let firstWatchRun = true;
    let firstDone = true;
    return new Promise((resolve, reject) => {
      let watcher = null;
      compiler.hooks.watchRun.tap("ChangesAndRemovalsTest", (compiler) => {
        if (firstWatchRun) {
          firstWatchRun = false;
          return;
        }
        changes = getChanges(compiler);
        watcher.close(resolve);
      });
      compiler.hooks.done.tap("ChangesAndRemovalsTest", () => {
        if (!firstDone) return;
        firstDone = false;
        setTimeout(() => {
          fs.unlinkSync(context.getDist("temp-file.js"));
        }, 300);
      });
      watcher = compiler.watch({ aggregateTimeout: 200 }, (err) => {
        if (err) reject(err);
      });
    });
  },
  async check({ context }) {
    expect(changes).toEqual({
      removed: [context.getDist("temp-file.js")],
      modified: []
    });
  }
}];
