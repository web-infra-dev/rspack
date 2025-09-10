it("should support `[hash]`, `[fullhash]` and function type for `output.publicPath`", () => {
	expect(__webpack_public_path__).toMatch(/^\/static\/[a-f0-9]{11}\/$/);
});
