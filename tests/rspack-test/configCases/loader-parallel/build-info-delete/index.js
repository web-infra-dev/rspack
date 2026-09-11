it("should run the loaders in a worker", () => {
	expect(require("./lib.js")).toBe(true);
	expect(require("./lib2.js")).toBe(true);
});
