it("omits SSR cache metadata from chunks that consume no remotes", () => {
	const federation = __webpack_require__.federation;
	expect(federation.remotesLoadingData && federation.remotesLoadingData.cache).toBeUndefined();
});
