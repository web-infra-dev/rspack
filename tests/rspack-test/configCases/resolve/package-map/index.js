import dep from "dep";
import depSub from "dep/sub";
import depShared from "dep/shared";
import scoped from "@scope/pkg";
import legacy from "legacy";
import legacyFile from "legacy/lib/file";
import shared from "shared";
import internal from "#internal";

it("should locate packages with the package map instead of node_modules", () => {
	expect(dep).toBe("dep@1.0.0");
	expect(scoped).toBe("@scope/pkg@1.0.0");
	expect(legacy).toBe("legacy@1.0.0");
});

it("should finish the resolution in the package location", () => {
	expect(depSub).toBe("dep@1.0.0/sub");
	expect(legacyFile).toBe("legacy@1.0.0/lib/file");
});

it("should resolve the dependencies of each package separately", () => {
	expect(shared).toBe("shared@1.0.0");
	expect(depShared).toBe("shared@2.0.0");
});

it("should resolve package targets of the imports field with the package map", () => {
	expect(internal).toBe("dep@1.0.0/sub");
});

it("should not resolve a package that is not a dependency of the importer", () => {
	expect(() => require("hidden")).toThrow(/Cannot find module 'hidden'/);
});

it("should not resolve a dependency whose package is not defined", () => {
	expect(() => require("undefined-package")).toThrow(
		/Cannot find module 'undefined-package'/
	);
});
