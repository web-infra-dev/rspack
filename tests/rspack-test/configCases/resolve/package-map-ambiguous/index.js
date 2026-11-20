import lib from "lib";

it("should resolve a package whose location is shared by several packages", () => {
	expect(lib.name).toBe("lib");
});

it("should not resolve packages from a location that is shared by several packages", () => {
	expect(() => lib.dep()).toThrow(/Cannot find module 'dep'/);
});
