
it("should not process link tags that are not modulepreload, preload, or stylesheet", async () => {
	const result = await import(/* webpackChunkName: "chunk" */ "./chunk");
	expect(result.default()).toBe("aaabbb");
});
