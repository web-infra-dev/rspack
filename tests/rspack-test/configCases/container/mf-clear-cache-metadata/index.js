it("should expose clear cache metadata for remotes", () => {
  const loadConsumer = () =>
    Promise.all([import("./page"), import("./sibling")]);
  expect(typeof loadConsumer).toBe("function");

  const data = __webpack_require__.remotesLoadingData;
  expect(data).toBeTruthy();

  const remoteModuleId = Object.entries(data.moduleIdToRemoteDataMapping).find(
    ([, remoteData]) =>
      remoteData.remoteName === "remoteA" && remoteData.name === "./A",
  )?.[0];
  expect(remoteModuleId).toBeTruthy();

  const remoteData = data.moduleIdToRemoteDataMapping[remoteModuleId];
  expect(data.remoteKeyToRemoteModuleIds.remoteA).toContain(remoteModuleId);
  expect(data.remoteKeyToExternalModuleIds.remoteA).toContain(
    remoteData.externalModuleId,
  );

  const consumerModuleIds =
    data.remoteModuleIdToConsumerModuleIds[remoteModuleId];
  expect(consumerModuleIds).toEqual(expect.arrayContaining(["./consumer.js"]));
  expect(data.consumerModuleIdToParentModuleIds["./consumer.js"]).toEqual(
    expect.arrayContaining(["./middle.js"]),
  );

  expect(data.consumerModuleIdToParentModuleIds["./middle.js"]).toContain(
    "./page.js",
  );
  expect(data.consumerModuleIdToParentModuleIds["./page.js"]).toContain(
    "./index.js",
  );
  expect(data.consumerModuleIdToParentModuleIds["./consumer.js"]).toContain(
    "./sibling.js",
  );
  expect(data.consumerModuleIdToParentModuleIds["./sibling.js"]).toContain(
    "./index.js",
  );
  expect(data.consumerModuleIdToParentModuleIds["./page.js"]).toContain(
    "./middle.js",
  );
  const remoteChunkIds = data.remoteKeyToChunkIds.remoteA;
  expect(remoteChunkIds.length).toBeGreaterThan(0);
  for (const chunkId of remoteChunkIds) {
    expect(data.chunkMapping[chunkId]).toContain(remoteModuleId);
  }
});

function deferred() {
  let resolve;
  const promise = new Promise(done => { resolve = done; });
  return { promise, resolve };
}

async function withNativeCache(run) {
  const runtime = {
    remotesLoadingData: __webpack_require__.remotesLoadingData,
    federation: __webpack_require__.federation,
    remoteCache: __webpack_require__.remoteCache,
    c: __webpack_require__.c,
    m: __webpack_require__.m,
    get chunkCacheControls() { return __webpack_require__.chunkCacheControls; },
    set chunkCacheControls(value) { __webpack_require__.chunkCacheControls = value; }
  };
  const data = runtime.remotesLoadingData;
  const id = data.remoteKeyToRemoteModuleIds.remoteA[0];
  const record = data.moduleIdToRemoteDataMapping[id];
  const external = record.externalModuleId;
  const consumers = ["./consumer.js", "./middle.js", "./page.js", "./sibling.js"];
  const previousCache = { ...runtime.c };
  const previousFactories = { ...runtime.m };
  const previousControls = runtime.chunkCacheControls;
  const previousRecord = Object.getOwnPropertyDescriptor(record, "p");
  let generation = 3;
  let chunk = "loaded";
  runtime.chunkCacheControls = {
    test: {
      clear() { chunk = undefined; },
      snapshot: () => chunk,
      restore: saved => { chunk = saved; },
      invalidate() { generation++; },
      getGeneration: () => generation,
      restoreGenerations: saved => { generation = Object.values(saved)[0]; }
    }
  };
  runtime.m[id] = () => {};
  for (const key of [id, external, ...consumers]) runtime.c[key] = { exports: key };
  runtime.c.unrelated = { exports: "unrelated" };
  try {
    await run({
      runtime, id, record, external, consumers,
      chunk: () => chunk, generation: () => generation
    });
  } finally {
    for (const key of Object.keys(runtime.c)) delete runtime.c[key];
    for (const key of Object.keys(runtime.m)) delete runtime.m[key];
    Object.assign(runtime.c, previousCache);
    Object.assign(runtime.m, previousFactories);
    runtime.chunkCacheControls = previousControls;
    if (previousRecord) Object.defineProperty(record, "p", previousRecord);
    else delete record.p;
  }
}

it("invalidates native factories, cyclic consumers and chunks without touching MF state", async () => {
  await withNativeCache(async ({ runtime, id, external, consumers, chunk, generation }) => {
    const instance = runtime.federation.instance;
    const moduleCache = new Map(instance.moduleCache);
    await runtime.remoteCache.clear(["remoteA"]);
    for (const key of [id, external, ...consumers]) expect(runtime.c[key]).toBeUndefined();
    expect(runtime.m[id]).toBeUndefined();
    expect(runtime.m[external]).toBeDefined();
    expect(runtime.c.unrelated.exports).toBe("unrelated");
    expect(chunk()).toBeUndefined();
    expect(generation()).toBe(4);
    expect(instance.moduleCache).toEqual(moduleCache);
    expect(runtime.remoteCache.hasPending()).toBe(false);
  });
});

it("evicts before pending loads settle, blocks new loads and removes late factories", async () => {
  await withNativeCache(async ({ runtime, id, record }) => {
    const pending = deferred();
    record.p = pending.promise;
    const clear = runtime.remoteCache.clear(["remoteA"]);
    expect(runtime.c[id]).toBeUndefined();
    expect(runtime.m[id]).toBeUndefined();
    expect(runtime.remoteCache.hasPending()).toBe(true);
    const wait = runtime.remoteCache.wait(["remoteA"]);
    const chunkWait = runtime.remoteCache.waitForChunk(runtime.remotesLoadingData.remoteKeyToChunkIds.remoteA[0]);
    expect(wait).toBeInstanceOf(Promise);
    expect(chunkWait).toBeInstanceOf(Promise);
    expect(runtime.remoteCache.wait(["unrelated"])).toBeUndefined();
    let finished = false;
    clear.then(() => { finished = true; });
    await Promise.resolve();
    expect(finished).toBe(false);
    pending.promise.then(() => { runtime.m[id] = () => {}; });
    pending.resolve();
    await Promise.all([clear, wait, chunkWait]);
    expect(runtime.m[id]).toBeUndefined();
    expect(record.p).toBeUndefined();
    expect(runtime.remoteCache.hasPending()).toBe(false);
  });
});

for (const pass of [1, 2]) {
  it(`restores native caches, loader state and generation when clear pass ${pass} fails`, async () => {
    await withNativeCache(async ({ runtime, record, chunk, generation }) => {
      const modules = { ...runtime.c };
      const factories = { ...runtime.m };
      const pending = Promise.resolve();
      record.p = pending;
      const originalClear = runtime.chunkCacheControls.test.clear;
      let calls = 0;
      runtime.chunkCacheControls.test.clear = () => {
        originalClear();
        if (++calls === pass) throw Error("native chunk cleanup failed");
      };
      await expect(runtime.remoteCache.clear(["remoteA"])).rejects.toThrow("native chunk cleanup failed");
      expect(runtime.c).toEqual(modules);
      expect(runtime.m).toEqual(factories);
      expect(record.p).toBe(pending);
      expect(chunk()).toBe("loaded");
      expect(generation()).toBe(3);
      expect(runtime.remoteCache.hasPending()).toBe(false);
    });
  });
}

it("removes stale writes after a timed-out load eventually settles", async () => {
  const previous = globalThis.__rspack_clear_cache_wait_timeout__;
  globalThis.__rspack_clear_cache_wait_timeout__ = 0;
  try {
    await withNativeCache(async ({ runtime, record, id }) => {
      const pending = deferred();
      record.p = pending.promise;
      await runtime.remoteCache.clear(["remoteA"]);
      expect(runtime.remoteCache.hasPending()).toBe(true);
      pending.promise.then(() => { runtime.m[id] = () => {}; });
      pending.resolve();
      await new Promise(resolve => setTimeout(resolve, 0));
      expect(runtime.m[id]).toBeUndefined();
      expect(runtime.remoteCache.hasPending()).toBe(false);
    });
  } finally {
    globalThis.__rspack_clear_cache_wait_timeout__ = previous;
  }
});
