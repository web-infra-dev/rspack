const checkMap = require("@rspack/test-tools/helper/util/checkSourceMap").default;

try {
	require("./a.js");
} catch (e) {
	// ignore
}

it("verify es6 (esmodule) bundle source map", async () => {
	const fs = require("fs");
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	const out = fs.readFileSync(__filename, "utf-8");
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(map.sources.filter(source =>
			!source.startsWith("webpack:///webpack/runtime/") &&
			!source.startsWith("rspack:///rspack/runtime/")
		)).toEqual([
			sourceUrl(`../../../../../packages/rspack-test-tools/dist/helper/util/checkSourceMap.js`),
			sourceUrl("./b-dir/c-dir/c.js"),
			sourceUrl("./b-dir/b.js"),
			sourceUrl("./a.js"),
			sourceUrl("./index.js"),
		]);
	} else {
		expect(map.sources).toEqual([
			sourceUrl(`../../../../../packages/rspack-test-tools/dist/helper/util/checkSourceMap.js`),
			sourceUrl("./b-dir/c-dir/c.js"),
			sourceUrl("./b-dir/b.js"),
			sourceUrl("./a.js"),
			sourceUrl("./index.js"),
		]);
	}
	expect(map.file).toEqual("bundle0.js");
	expect(
		await checkMap(out, source, {
			// *${id}* as the search key to avoid conflict with `Object.defineProperty(exports, ${id}, ...)`
			['"*a0*"']: sourceUrl("a.js"),
			['"*a1*"']: sourceUrl("a.js"),
			// The result is generated upon `OriginalSource`
			// and webpack generates sourcemap of`("xx")` as a block.
			['("*a2*")']: checkColumn(sourceUrl("a.js")),
			['"*b0*"']: sourceUrl("b-dir/b.js"),
			['"*b1*"']: sourceUrl("b-dir/b.js"),
			// The result is generated upon `OriginalSource`
			// and webpack generates sourcemap of`("xx")` as a block.
			['("*b2*")']: checkColumn(sourceUrl("b-dir/b.js")),
			['"*c0*"']: sourceUrl("b-dir/c-dir/c.js"),
			['"*c1*"']: sourceUrl("b-dir/c-dir/c.js"),
			['"*c2*"']: sourceUrl("b-dir/c-dir/c.js")
		}, false)
	).toBe(true);
});

const checkColumn = (s) => {
	return {
		inSource: s,
		checkColumn: true,
	}
}
