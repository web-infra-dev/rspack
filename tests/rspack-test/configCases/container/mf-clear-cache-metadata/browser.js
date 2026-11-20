it("omits native SSR cache code and reverse graph metadata from web output", () => {
	const load = () => import("./page");
	expect(typeof load).toBe("function");
	const data = __webpack_require__.remotesLoadingData;
	expect(data.moduleIdToRemoteDataMapping).toBeTruthy();
	expect(data.cache).toBeUndefined();
});
