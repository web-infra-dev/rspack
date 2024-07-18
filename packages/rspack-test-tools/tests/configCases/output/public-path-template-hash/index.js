it("should support [hash] and [fullhash] in `output.publicPath`", () => {
	expect(__webpack_public_path__).toMatch(/^\/static\/[a-f0-9]{11}\/$/);
});
