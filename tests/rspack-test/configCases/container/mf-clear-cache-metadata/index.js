const fs = require("fs");
const path = require("path");

it("should expose compact clear cache metadata only for Node SSR chunks with remotes", () => {
	const loadConsumer = () =>
		Promise.all([import("./page"), import("./sibling")]);
	expect(typeof loadConsumer).toBe("function");

	const data = __webpack_require__.remotesLoadingData;
	const federation = __webpack_require__.federation;
	expect(federation.remotesLoadingData).toBeUndefined();
	const cache = data.cache;
	expect(data).toBeTruthy();
	expect(cache).toBeTruthy();

	const remoteModuleId = Object.entries(data.moduleIdToRemoteDataMapping).find(
		([, remoteData]) =>
			remoteData.remoteName === "remoteA" && remoteData.name === "./A"
	)?.[0];
	expect(remoteModuleId).toBeTruthy();

	expect(cache.remoteKeyToRemoteModuleIds).toBeUndefined();
	expect(cache.remoteModuleIdToConsumerModuleIds).toBeUndefined();
	expect(
		data.moduleIdToRemoteDataMapping[remoteModuleId].externalModuleId
	).toBeDefined();
	expect(cache.modules).toBe(__webpack_require__.c);
	expect(cache.factories).toBe(__webpack_require__.m);

	const parentMap = cache.moduleIdToParentModuleIds;
	expect(parentMap[remoteModuleId]).toEqual(
		expect.arrayContaining(["./consumer.js"])
	);
	expect(parentMap["./consumer.js"]).toEqual(
		expect.arrayContaining(["./middle.js", "./sibling.js"])
	);
	expect(parentMap["./middle.js"]).toContain("./page.js");
	expect(parentMap["./page.js"]).toContain("./index.js");
	expect(parentMap["./sibling.js"]).toContain("./index.js");
	expect(parentMap["./page.js"]).toContain("./middle.js");
	expect(parentMap["./index.js"]).toBeUndefined();
	expect(
		Object.values(data.chunkMapping).some(ids => ids.includes(remoteModuleId))
	).toBe(true);

	const browserOutput = fs.readFileSync(
		path.join(__dirname, "browser.js"),
		"utf8"
	);
	expect(browserOutput).not.toContain("moduleIdToParentModuleIds");
});
