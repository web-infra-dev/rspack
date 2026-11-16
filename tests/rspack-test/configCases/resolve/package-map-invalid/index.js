it("should not resolve packages with an invalid package map", () => {
	expect(() => require("dep")).toThrow(/Cannot find module 'dep'/);
});
