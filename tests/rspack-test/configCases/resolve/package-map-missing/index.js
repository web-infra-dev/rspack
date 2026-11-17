import local from "./local";

it("should resolve relative requests without the package map", () => {
	expect(local).toBe("local");
});

it("should not resolve packages when the package map does not exist", () => {
	expect(() => require("dep")).toThrow(/Cannot find module 'dep'/);
});
