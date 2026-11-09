it("should materialize builtin loader options from native state in a worker", () => {
	expect(require("./module")).toBe("worker:ok");
});
