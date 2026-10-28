it("should work with custom chunkLoadingGlobal value", async () => {
	// same as `output.chunkLoadingGlobal in rspack.config.mjs`
	const chunkLoadingGlobal = "__LOADED_CHUNKS__";
	await import("./file");
	expect(Array.isArray(self[chunkLoadingGlobal])).toBeTruthy();
});
