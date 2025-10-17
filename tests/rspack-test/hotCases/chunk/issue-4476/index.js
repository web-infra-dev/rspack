it("should work with custom chunkLoadingGlobal value", async () => {
	// same as `require('./webpack.config').output.chunkLoadingGlobal`
	const chunkLoadingGlobal = "__LOADED_CHUNKS__";
	await import("./file");
	expect(Array.isArray(self[chunkLoadingGlobal])).toBeTruthy();
});
