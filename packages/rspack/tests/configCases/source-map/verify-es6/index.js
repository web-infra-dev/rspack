const checkMap = require("../checkSourceMap").default;

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
		"../checkSourceMap.js",
		"./a.js",
		"./b-dir/b.js",
		"./b-dir/c-dir/c.js",
		"./index.js"
	]);
	expect(map.file).toEqual("main.js");
	const out = fs.readFileSync(__filename, "utf-8");
	expect(
		await checkMap(out, source, {
			// *${id}* as the search key to aviod conflict with `Object.defineProperty(exports, ${id}, ...)`
			["*a0*"]: "a.js",
			["*a1*"]: "a.js",
			["*a2*"]: "a.js",
			["*b0*"]: "b-dir/b.js",
			["*b1*"]: "b-dir/b.js",
			["*b2*"]: "b-dir/b.js",
			["*c0*"]: "b-dir/c-dir/c.js",
			["*c1*"]: "b-dir/c-dir/c.js",
			["*c2*"]: "b-dir/c-dir/c.js"
		})
	).toBe(true);
});
