import local from "local";
import lib from "./lib";

it("should search the module directories that follow node_modules from a file that is not within any package", () => {
	expect(local).toBe("modules/local");
});

it("should search the module directories that follow node_modules for a package that is not a dependency", () => {
	expect(lib.local).toBe("modules/local");
	expect(lib.dep).toBe("store/dep");
});

it("should report the package map error when no module directory has the module", () => {
	expect(() => require("dep")).toThrow(/Cannot find module 'dep'/);
});
