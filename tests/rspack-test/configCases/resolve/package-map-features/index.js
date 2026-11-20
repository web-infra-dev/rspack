import depShared from "dep/shared";
import nestedShared from "nested";
import scopedSub from "@scope/pkg/sub";
import singleFile from "single-file";
import browser from "browser-pkg";
import self from "app/self";
import aliased from "aliased";
import missing from "missing";

it("should use the nearest package that contains the importer", () => {
	expect(depShared).toBe("shared@1.0.0");
	expect(nestedShared).toBe("shared@2.0.0");
});

it("should resolve a subpath of a scoped package", () => {
	expect(scopedSub).toBe("@scope/pkg/sub");
});

it("should resolve a package whose location is a file", () => {
	expect(singleFile).toBe("single-file");
});

it("should apply the alias fields of the package", () => {
	expect(browser).toBe("browser-pkg/browser");
});

it("should prefer a package referencing itself to its dependencies", () => {
	expect(self).toBe("app/self");
});

it("should resolve the target of an alias with the package map", () => {
	expect(aliased).toBe("dep");
});

it("should use the fallback for a package that is not a dependency", () => {
	expect(missing).toBe("fallback");
});

it("should resolve a context in a package", () => {
	const context = require.context("dep/dir", false, /\.js$/);
	expect(context.keys().sort()).toEqual(["./a.js", "./b.js"]);
	expect(context("./a.js").default).toBe("dep/dir/a");
});

it("should not resolve a subpath that the package does not export", () => {
	expect(() => require("browser-pkg/private")).toThrow();
});
