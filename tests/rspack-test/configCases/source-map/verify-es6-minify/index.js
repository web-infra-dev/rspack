const checkMap = require("@rspack/test-tools/helper/util/checkSourceMap").default;

try {
	require("./a.js");
} catch (e) {
	// ignore
}

it("verify es6 (esmodule) minify bundle source map", async () => {
	const fs = require("fs");
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	let sourceUrl = source => `webpack:///${source}`;
	let runtimeSource = name => sourceUrl(`webpack/runtime/${name}`);
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
		runtimeSource = name => sourceUrl(`rspack/runtime/${name}`);
	}
	expect(map.sources.sort()).toEqual([
		sourceUrl(`../../../../../packages/rspack-test-tools/dist/helper/util/checkSourceMap.js`),
		sourceUrl("./a.js"),
		sourceUrl("./b-dir/b.js"),
		sourceUrl("./b-dir/c-dir/c.js"),
		sourceUrl("./index.js"),
		runtimeSource("define_property_getters"),
		runtimeSource("has_own_property"),
		runtimeSource("make_namespace_object"),
	].sort());
	expect(map.file).toEqual("bundle0.js");
	const out = fs.readFileSync(__filename, "utf-8");
	expect(
		await checkMap(out, source, {
			// *${id}* as the search key to avoid conflict with `Object.defineProperty(exports, ${id}, ...)`
			// "*a0*", "*a1*" is eliminate by minify
			['"*a2*"']: checkColumn(sourceUrl("a.js")),
			// "*b0*", "*b1*" is eliminate by minify
			['"*b2*"']: checkColumn(sourceUrl("b-dir/b.js")),
			// "*c0*" is eliminate by minify
			// "*c1*" is eliminate by minify
			['"*c2*"']: sourceUrl("b-dir/c-dir/c.js")
		})
	).toBe(true);
});

const checkColumn = (s) => {
	return {
		inSource: s,
		checkColumn: true,
	}
}
