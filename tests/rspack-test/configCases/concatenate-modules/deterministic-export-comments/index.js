const { usedA, usedB } = require("./root");
const { expectSourceToContain } = require("@rspack/test-tools/helper/legacy/expectSource");

it("should render concatenated export comments in deterministic order", () => {
	expect(usedA()).toBe("marker-a");
	expect(usedB()).toBe("marker-b");

	const fs = require("fs");
	const source = fs.readFileSync(__filename, "utf-8");

	/*********** DO NOT MATCH BELOW THIS LINE ***********/

	expectSourceToContain(source, "usedA: () => (/* binding */ usedA)");
	expectSourceToContain(source, "usedB: () => (/* binding */ usedB)");
	expectSourceToContain(source, "// UNUSED EXPORTS: unusedA, unusedB");
});
