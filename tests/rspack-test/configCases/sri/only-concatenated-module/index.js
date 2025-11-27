
it("should generate SRI hash for chunks with only concatenated modules", async () => {
	const result = await import(/* webpackChunkName: "chunk" */ "./chunk");
	expect(result.default()).toBe("aaabbb");
});
