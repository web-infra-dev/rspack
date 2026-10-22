/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig} */
export default {
  description: "should wait for a restarted compiler before emitting multi done",
  async build(_, compiler) {
    const [a, b] = compiler.compilers;
    const hashes = [];
    compiler.hooks.done.tap("WatchDoneTest", stats => hashes.push(stats.hash));

    // Both compilers finish their first build.
    await a.hooks.watchRun.promise(a);
    await b.hooks.watchRun.promise(b);
    await a.hooks.done.promise({ hash: "a1" });
    await b.hooks.done.promise({ hash: "b1" });
    expect(hashes).toEqual(["a1b1"]);

    // A is invalidated again during its second build, scheduling a restart.
    a.hooks.invalid.call(null, 1);
    b.hooks.invalid.call(null, 1);
    await a.hooks.watchRun.promise(a);
    await b.hooks.watchRun.promise(b);
    a.hooks.invalid.call(null, 2);
    await a.hooks.done.promise({ hash: "a2" });

    // An async plugin holds A's restart before default-stage watchRun taps.
    let resume;
    a.hooks.watchRun.tapAsync({ name: "WatchDoneTest", stage: -1 }, (_, callback) => {
      resume = callback;
    });
    const restarted = a.hooks.watchRun.promise(a); // No new invalid hook.
    try {
      await b.hooks.done.promise({ hash: "b2" });
      expect(hashes).toEqual(["a1b1"]);
    } finally {
      resume();
      await restarted;
    }

    await a.hooks.done.promise({ hash: "a3" });
    expect(hashes).toEqual(["a1b1", "a3b2"]);
  }
};
