const { createFsFromVolume, Volume } = require("memfs");

defineCompileCase(Utils.basename(__filename), [(() => {
  const runCb = jest.fn();
  const doneHookCb = jest.fn();
  return {
    description: "should call afterDone hook after other callbacks (run)",
    options(context) {
      return {
        entry: "./c",
      };
    },
    async compiler(context, compiler) {
      compiler.outputFileSystem = createFsFromVolume(new Volume());
      compiler.hooks.done.tap("afterDoneRunTest", doneHookCb);
    },
    async build(context, compiler) {
      return new Promise((resolve, reject) => {
        compiler.hooks.afterDone.tap("afterDoneRunTest", () => {
          expect(runCb).toHaveBeenCalled();
          expect(doneHookCb).toHaveBeenCalled();
          resolve();
        });
        compiler.run((err, stats) => {
          if (err) return reject(err);
          runCb();
        });
      });
    },
  }
})(), (() => {
  const invalidHookCb = jest.fn();
  const doneHookCb = jest.fn();
  const watchCb = jest.fn();
  const invalidateCb = jest.fn();

  return {
    description: "should call afterDone hook after other callbacks (watch)",
    options(context) {
      return {
        entry: "./c",
      };
    },
    async compiler(context, compiler) {
      compiler.outputFileSystem = createFsFromVolume(new Volume());
      compiler.hooks.invalid.tap("afterDoneWatchTest", invalidHookCb);
      compiler.hooks.done.tap("afterDoneWatchTest", doneHookCb);
    },
    async build(context, compiler) {
      return new Promise((resolve, reject) => {
        compiler.hooks.afterDone.tap("afterDoneWatchTest", () => {
          expect(invalidHookCb).toHaveBeenCalled();
          expect(doneHookCb).toHaveBeenCalled();
          expect(watchCb).toHaveBeenCalled();
          expect(invalidateCb).toHaveBeenCalled();
          watching.close(resolve);
        });
        const watching = compiler.watch({}, (err, stats) => {
          if (err) return reject(err);
          watchCb();
        });
        process.nextTick(() => {
          watching.invalidate(invalidateCb);
        });
      });
    },
  };
})(), (() => {
  const invalidHookCb = jest.fn();
  const watchCloseCb = jest.fn();
  const watchCloseHookCb = jest.fn();
  const invalidateCb = jest.fn();
  return {
    description: "should call afterDone hook after other callbacks (watch close)",
    options(context) {
      return {
        entry: "./c",
      };
    },
    async compiler(context, compiler) {
      compiler.outputFileSystem = createFsFromVolume(new Volume());
      compiler.hooks.invalid.tap("afterDoneWatchTest", invalidHookCb);
      compiler.hooks.watchClose.tap("afterDoneWatchTest", watchCloseHookCb);
    },
    async build(context, compiler) {
      return new Promise((resolve, reject) => {
        compiler.hooks.afterDone.tap("afterDoneWatchTest", () => {
          expect(invalidHookCb).toHaveBeenCalled();
          expect(watchCloseCb).toHaveBeenCalled();
          expect(watchCloseHookCb).toHaveBeenCalled();
          expect(invalidateCb).toHaveBeenCalled();
          resolve();
        });
        const watch = compiler.watch({}, (err, stats) => {
          if (err) return reject(err);
          watch.close(watchCloseCb);
        });
        process.nextTick(() => {
          watch.invalidate(invalidateCb);
        });
      });
    },
  }
})(), (() => {
  const instanceCb = jest.fn();
  const doneHookCb = jest.fn();
  let rejection = null;
  return {
    description: "should call afterDone hook after other callbacks (instance cb)",
    options(context) {
      return {
        entry: "./c",
      };
    },
    compilerCallback(err, stats) {
      if (err) {
        rejection(err);
      };
      instanceCb();
    },
    async compiler(context, compiler) {
      compiler.outputFileSystem = createFsFromVolume(new Volume());
      compiler.hooks.done.tap("afterDoneRunTest", doneHookCb);
    },
    async build(context, compiler) {
      return new Promise((resolve, reject) => {
        rejection = reject;
        compiler.hooks.afterDone.tap("afterDoneRunTest", () => {
          expect(instanceCb).toHaveBeenCalled();
          expect(doneHookCb).toHaveBeenCalled();
          resolve();
        });
      });
    },
  };
})()]);
