const fs = require("fs");
const path = require("path");

it("basic", () => {
	require("./index.scss");
	const sourceMap = fs.readFileSync(__dirname + "/bundle0.css.map", "utf-8");
	const scss = fs.readFileSync(path.resolve(CONTEXT, "./index.scss"), "utf-8");
	const map = JSON.parse(sourceMap);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(map.sources).toContain(sourceUrl("./index.scss"));
	expect(map.file).toEqual("bundle0.css");
	expect(map.sourcesContent).toEqual([scss]);
});
