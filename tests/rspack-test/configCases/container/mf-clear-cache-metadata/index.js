it("should expose clear cache metadata for remotes", () => {
  const loadConsumer = () =>
    Promise.all([import("./page"), import("./sibling")]);
  expect(typeof loadConsumer).toBe("function");

  const data = __webpack_require__.federation.remotesLoadingData;
  const cache = data.cache;
  expect(data).toBeTruthy();

  const remoteModuleId = Object.entries(data.moduleIdToRemoteDataMapping).find(
    ([, remoteData]) =>
      remoteData.remoteName === "remoteA" && remoteData.name === "./A",
  )?.[0];
  expect(remoteModuleId).toBeTruthy();

  expect(cache.remoteKeyToRemoteModuleIds.remoteA).toContain(remoteModuleId);
  expect(data.moduleIdToRemoteDataMapping[remoteModuleId].externalModuleId).toBeDefined();
  expect(cache.modules).toBe(__webpack_require__.c);
  expect(cache.factories).toBe(__webpack_require__.m);

  const consumerModuleIds =
    cache.remoteModuleIdToConsumerModuleIds[remoteModuleId];
  expect(consumerModuleIds).toEqual(expect.arrayContaining(["./consumer.js"]));
  expect(cache.consumerModuleIdToParentModuleIds["./consumer.js"]).toEqual(
    expect.arrayContaining(["./middle.js"]),
  );

  expect(cache.consumerModuleIdToParentModuleIds["./middle.js"]).toContain(
    "./page.js",
  );
  expect(cache.consumerModuleIdToParentModuleIds["./page.js"]).toContain(
    "./index.js",
  );
  expect(cache.consumerModuleIdToParentModuleIds["./consumer.js"]).toContain(
    "./sibling.js",
  );
  expect(cache.consumerModuleIdToParentModuleIds["./sibling.js"]).toContain(
    "./index.js",
  );
  expect(cache.consumerModuleIdToParentModuleIds["./page.js"]).toContain(
    "./middle.js",
  );
  expect(Object.values(data.chunkMapping).some(ids => ids.includes(remoteModuleId))).toBe(true);
});
