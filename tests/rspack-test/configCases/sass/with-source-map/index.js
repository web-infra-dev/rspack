const fs = require("fs");
const path = require("path");

it("basic", () => {
	const css = require("./index.scss");
	expect(css).toEqual(nsObj({}));
	const source = fs.readFileSync(
		path.resolve(__dirname, "bundle0.css.map"),
		"utf-8"
	);
	const map = JSON.parse(source);
	const scss = fs.readFileSync(path.resolve(CONTEXT, "./index.scss"), "utf-8");
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(map.sources).toEqual([sourceUrl("./index.scss")]);
	expect(map.sourcesContent).toEqual([scss]);
	expect(map.file).toEqual("bundle0.css");
});
