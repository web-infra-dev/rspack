import local from "local";
import lib from "./lib";

it("should search the module directories that follow node_modules for a package that is not a dependency", () => {
	expect(local).toBe("modules/local");
	expect(lib.local).toBe("modules/local");
});

it("should use the package map before the module directories that follow node_modules", () => {
	expect(lib.dep).toBe("store/dep");
});

it("should not resolve a package that is neither a dependency nor in a module directory", () => {
	expect(() => require("dep")).toThrow(/Cannot find module 'dep'/);
});
