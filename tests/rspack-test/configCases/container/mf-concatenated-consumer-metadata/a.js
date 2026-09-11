it("tracks every concatenated remote consumer for a", () => {
  const load = () => import("./page-a");
  expect(typeof load).toBe("function");
  const data = __webpack_require__.remotesLoadingData;
  const remote = data.remoteKeyToRemoteModuleIds.remote[0];
  const consumers = data.remoteModuleIdToConsumerModuleIds[remote];
  // The shared parent is emitted in three concatenated page/loader modules.
  expect(consumers).toHaveLength(3);
  const pending = [...consumers];
  const ancestors = new Set();
  for (let i = 0; i < pending.length; i++) {
    const id = pending[i];
    if (ancestors.has(id)) continue;
    ancestors.add(id);
    pending.push(...(data.consumerModuleIdToParentModuleIds[id] || []));
  }
  expect(ancestors.has(module.id)).toBe(true);
});
