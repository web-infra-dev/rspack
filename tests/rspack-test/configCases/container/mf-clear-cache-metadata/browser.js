it("omits native SSR cache code and reverse graph metadata from web output", () => {
	const load = () => import("./page");
	expect(typeof load).toBe("function");
	expect(__webpack_require__.remotesLoadingData.moduleIdToRemoteDataMapping).toBeTruthy();
	const federation = __webpack_require__.federation;
	expect(federation.remotesLoadingData && federation.remotesLoadingData.cache).toBeUndefined();
});
