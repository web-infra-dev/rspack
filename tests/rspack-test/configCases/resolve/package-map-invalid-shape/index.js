it("should not resolve packages when a package of the package map has no url", () => {
	expect(() => require("dep")).toThrow(/Cannot find module 'dep'/);
});
