it("should work with custom chunkLoadingGlobal value", function () {
	// same as `require('./webpack.config').output.chunkLoadingGlobal`
	const chunkLoadingGlobal = "__LOADED_CHUNKS__";
	import("./file").then(() => {
		expect(Array.isArray(window[chunkLoadingGlobal])).toBeTruthy();
	});
});
