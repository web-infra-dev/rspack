const fs = require("fs");
const path = require("path");

it("basic", () => {
	require("./index.css");
	const sourceMap = fs.readFileSync(__dirname + "/bundle0.css.map", "utf-8");
	const css = fs.readFileSync(path.resolve(CONTEXT, "./index.css"), "utf-8");
	const map = JSON.parse(sourceMap);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(map.sources).toContain(sourceUrl("./index.css"));
	expect(map.file).toEqual("bundle0.css");
	expect(map.sourcesContent).toEqual([css]);
});
