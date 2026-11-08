"use strict";

const fs = require("fs");

require("./test1");
require("./no-source-map");

it("should extract source map - 1", () => {
	const fileData = fs.readFileSync(__filename + ".map").toString("utf-8");
	const { sources } = JSON.parse(fileData);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(sources).toContain(sourceUrl("./extract1.js"));
	expect(sources).toContain(sourceUrl("./charset-inline-source-map.txt"));
	expect(sources).toContain(sourceUrl("./no-source-map.js"));
});
