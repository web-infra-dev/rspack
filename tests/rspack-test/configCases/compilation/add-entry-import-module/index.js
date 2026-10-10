it("should have built the dynamically added entry", () => {
	expect(__STATS__.compilation.namedChunks).toHaveProperty("dynamic");
});
