it("omits SSR cache metadata from Node builds without @module-federation/node", () => {
	const load = () => import("./page");
	expect(typeof load).toBe("function");
	const data = __webpack_require__.remotesLoadingData;
	expect(data.moduleIdToRemoteDataMapping).toBeTruthy();
	expect(data.cache).toBeUndefined();
});
