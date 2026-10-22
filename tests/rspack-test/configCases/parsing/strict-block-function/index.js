import { viaImport, viaRequire } from "./module";

const blockFunctions = require("./block-functions");

it("should scope a strict or async block function to its block", () => {
	expect(blockFunctions).toEqual({
		strictOuter: "lib",
		strictInner: "local ./lib",
		sloppyAsync: "lib",
		sloppyGenerator: "lib",
		sloppyAnnexB: "annex b",
		sloppyLexicalBlocker: "lib"
	});
});

it("should scope a block function to its block in a module", () => {
	expect(viaImport).toBe("lib");
	expect(viaRequire).toBe("lib");
});

it("should rename a block function that shadows the runtime require", () => {
	expect(require("./runtime-require")).toEqual(["local", "lib"]);
});
