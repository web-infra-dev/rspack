it("should have hmr flag in loader context", function() {
	expect(require("./loader.mjs!")).toBe(!!module.hot);
});
