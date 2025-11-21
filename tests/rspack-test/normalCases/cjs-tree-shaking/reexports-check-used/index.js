it("should allow to reexport a imported property (exports)", () => {
	expect(require("./exports-property-assign-require-chain").abc).toBe("abc");
	expect(require("./exports-property-assign-require-chain").usedExports).toEqual(["abc", "usedExports"]);
});
