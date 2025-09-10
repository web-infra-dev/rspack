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
	expect(map.sources.sort()).toEqual([
		`webpack:///../../../../../packages/rspack-test-tools/dist/helper/util/checkSourceMap.js`,
		"webpack:///./a.js",
		"webpack:///./b-dir/b.js",
		"webpack:///./b-dir/c-dir/c.js",
		"webpack:///./index.js",
		"webpack:///webpack/runtime/define_property_getters",
		"webpack:///webpack/runtime/has_own_property",
		"webpack:///webpack/runtime/make_namespace_object",
	]);
	expect(map.file).toEqual("bundle0.js");
	const out = fs.readFileSync(__filename, "utf-8");
	expect(
		await checkMap(out, source, {
			// *${id}* as the search key to avoid conflict with `Object.defineProperty(exports, ${id}, ...)`
			// "*a0*", "*a1*" is eliminate by minify
			['"*a2*"']: checkColumn("webpack:///a.js"),
			// "*b0*", "*b1*" is eliminate by minify
			['"*b2*"']: checkColumn("webpack:///b-dir/b.js"),
			// "*c0*" is eliminate by minify
			// "*c1*" is eliminate by minify
			['"*c2*"']: "webpack:///b-dir/c-dir/c.js"
		})
	).toBe(true);
});

const checkColumn = (s) => {
	return {
		inSource: s,
		checkColumn: true,
	}
}
