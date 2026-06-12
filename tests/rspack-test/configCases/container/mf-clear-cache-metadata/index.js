it("should expose clear cache metadata for remotes", () => {
	const loadConsumer = () => import("./consumer");
	expect(typeof loadConsumer).toBe("function");

	const data = __webpack_require__.remotesLoadingData;
	expect(data).toBeTruthy();

	const remoteModuleId = Object.entries(data.moduleIdToRemoteDataMapping).find(
		([, remoteData]) =>
			remoteData.remoteName === "remoteA" && remoteData.name === "./A"
	)?.[0];
	expect(remoteModuleId).toBeTruthy();

	const remoteData = data.moduleIdToRemoteDataMapping[remoteModuleId];
	expect(data.remoteKeyToRemoteModuleIds.remoteA).toContain(remoteModuleId);
	expect(data.remoteKeyToExternalModuleIds.remoteA).toContain(
		remoteData.externalModuleId
	);

	const consumerModuleIds =
		data.remoteModuleIdToConsumerModuleIds[remoteModuleId];
	expect(consumerModuleIds).toEqual(expect.arrayContaining(["./consumer.js"]));
	expect(data.consumerModuleIdToParentModuleIds["./consumer.js"]).toEqual(
		expect.arrayContaining(["./index.js"])
	);

	const remoteChunkIds = data.remoteKeyToChunkIds.remoteA;
	expect(remoteChunkIds.length).toBeGreaterThan(0);
	for (const chunkId of remoteChunkIds) {
		expect(data.chunkMapping[chunkId]).toContain(remoteModuleId);
	}
});
