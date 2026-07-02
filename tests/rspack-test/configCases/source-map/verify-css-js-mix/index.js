const checkMap = require("@rspack/test-tools/helper/util/checkSourceMap").default;
const fs = require("fs");
const path = require("path");

try {
	require("./a.js");
} catch (e) {
	// ignore
}

it("verify importing css js source map", async () => {
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	let sourceUrl = source => `webpack:///${source}`;
	let runtimeSource = name => sourceUrl(`webpack/runtime/${name}`);
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
		runtimeSource = name => sourceUrl(`rspack/runtime/${name}`);
	}
	expect(map.sources.sort()).toEqual([
		sourceUrl("./a.js"),
		sourceUrl("./index.js"),
		runtimeSource("make_namespace_object"),
	].sort());
	expect(map.file).toEqual("bundle0.js");
	const out = fs.readFileSync(__filename, "utf-8");
	expect(
		await checkMap(out, source, {
			// *${id}* as the search key to avoid conflict with `Object.defineProperty(exports, ${id}, ...)`
			['"*a0*"']: sourceUrl("a.js"),
			['"*a1*"']: sourceUrl("a.js")
		}, false)
	).toBe(true);
});

it("verify css source map", async () => {
	const cssSource = fs.readFileSync(
		path.resolve(__dirname, "bundle0.css.map"),
		"utf-8"
	);
	const cssMap = JSON.parse(cssSource);
	let cssSourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		cssSourceUrl = source => `rspack:///${source}`;
	}
	expect(cssMap.sources).toEqual([cssSourceUrl("./a.css")]);
	expect(cssMap.file).toEqual("bundle0.css");
	const cssOut = fs.readFileSync(
		path.resolve(__dirname, "bundle0.css"),
		"utf-8"
	);
	expect(
		await checkMap(cssOut, cssSource, {
			[`a:nth-child(0):after { content: "a0"; }`]: cssSourceUrl("a.css"),
			[`a:nth-child(1):after { content: "a1"; }`]: cssSourceUrl("a.css"),
			[`a:nth-child(2):after { content: "a2"; }`]: cssSourceUrl("a.css")
		})
	).toBe(true);
});
