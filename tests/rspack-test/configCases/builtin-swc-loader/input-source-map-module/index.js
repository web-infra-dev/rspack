const fs = require("fs");
const path = require("path");
const checkMap = require("@rspack/test-tools/helper/util/checkSourceMap").default;

import "./a"

const source = fs.readFileSync(__filename + ".map", "utf-8");
const map = JSON.parse(source);
const output = fs.readFileSync(__filename, "utf-8");
const input = fs.readFileSync(path.resolve(CONTEXT, "a.jsx"), "utf-8");
let sourceUrl = source => `webpack:///${source}`;
let runtimeSource = name => sourceUrl(`webpack/runtime/${name}`);
if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
	sourceUrl = source => `rspack:///${source}`;
	runtimeSource = name => sourceUrl(`rspack/runtime/${name}`);
}

it("should keep the original content with `devtool: \"source-map\"` enabled", () => {
	expect(map.sources.sort()).toEqual([
		sourceUrl("./a.jsx"),
		sourceUrl("./index.js"),
		runtimeSource("define_property_getters"),
		runtimeSource("has_own_property"),
		runtimeSource("make_namespace_object"),
	].sort());
	expect(map.sourcesContent[0]).toEqual(input)
})

it("should keep the mappings to the original content", async () => {
	expect(await checkMap(output, source, {
		"'*a0*'": sourceUrl("a.jsx"),
		"'*a1*'": sourceUrl("a.jsx"),
		"'*a2*'": sourceUrl("a.jsx"),
	})).toBe(true)
})
