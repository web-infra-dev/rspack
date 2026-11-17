it("should not resolve packages when the package map is not valid JSON", () => {
	expect(() => require("dep")).toThrow(/Cannot find module 'dep'/);
});
