it("omits SSR cache metadata from chunks that consume no remotes", () => {
	const data = __webpack_require__.remotesLoadingData;
	expect(data && data.cache).toBeUndefined();
});
