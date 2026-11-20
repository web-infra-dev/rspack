import lib from "./lib";

it("should resolve relative requests of a file that is not within any package", () => {
	expect(lib).toBe("store/dep");
});

it("should not resolve packages from a file that is not within any package", () => {
	expect(() => require("dep")).toThrow(/Cannot find module 'dep'/);
});
