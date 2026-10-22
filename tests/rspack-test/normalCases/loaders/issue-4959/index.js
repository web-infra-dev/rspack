it("should resolve module dependencies recursively", function() {
	expect(require("!./loaders/index.mjs!a")).toBe("c");
});
