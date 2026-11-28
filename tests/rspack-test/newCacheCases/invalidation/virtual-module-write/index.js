it("should use the virtual module written in thisCompilation on restart", () => {
	expect(require("./virtual.js")).toBe(COMPILER_INDEX === 0 ? "first" : "second");
});
