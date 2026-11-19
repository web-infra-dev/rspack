it("omits SSR cache metadata from Node builds without @module-federation/node", () => {
	const load = () => import("./page");
	expect(typeof load).toBe("function");
	expect(__webpack_require__.remotesLoadingData.moduleIdToRemoteDataMapping).toBeTruthy();
	const federation = __webpack_require__.federation;
	expect(federation.remotesLoadingData && federation.remotesLoadingData.cache).toBeUndefined();
});
