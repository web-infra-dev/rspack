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
