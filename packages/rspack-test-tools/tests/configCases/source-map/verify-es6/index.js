const checkMap = require("../../../../dist/helper/util/checkSourceMap").default;

try {
	require("./a.js");
} catch (e) {
	// ignore
}

it("verify es6 (esmodule) bundle source map", async () => {
	const fs = require("fs");
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	expect(map.sources).toEqual([
		"webpack:///../../../../dist/helper/util/checkSourceMap.js",
		"webpack:///./a.js",
		"webpack:///./b-dir/b.js",
		"webpack:///./b-dir/c-dir/c.js",
		"webpack:///./index.js",
	]);
	expect(map.file).toEqual("bundle0.js");
	const out = fs.readFileSync(__filename, "utf-8");
	expect(
		await checkMap(out, source, {
			// *${id}* as the search key to aviod conflict with `Object.defineProperty(exports, ${id}, ...)`
			['"*a0*"']: "webpack:///a.js",
			['"*a1*"']: "webpack:///a.js",
			// The result is generated upon `OriginalSource`
			// and webpack generates sourcemap of`("xx")` as a block.
			['("*a2*")']: checkColumn("webpack:///a.js"),
			['"*b0*"']: "webpack:///b-dir/b.js",
			['"*b1*"']: "webpack:///b-dir/b.js",
			// The result is generated upon `OriginalSource`
			// and webpack generates sourcemap of`("xx")` as a block.
			['("*b2*")']: checkColumn("webpack:///b-dir/b.js"),
			['"*c0*"']: "webpack:///b-dir/c-dir/c.js",
			['"*c1*"']: "webpack:///b-dir/c-dir/c.js",
			['"*c2*"']: "webpack:///b-dir/c-dir/c.js"
		}, false)
	).toBe(true);
});

const checkColumn = (s) => {
	return [s, true]
}
