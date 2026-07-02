"use strict";

const fs = require("fs");
const path = require("path");

require("./test2");

it("should extract source map - 2", () => {
	const fileData = fs
		.readFileSync(path.resolve(__dirname, "bundle1.js.map"))
		.toString("utf-8");
	const { sources } = JSON.parse(fileData);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(sources).toContain(sourceUrl("./external-source-map.txt"));
	expect(sources).toContain(sourceUrl("./extract2.js"));
});
