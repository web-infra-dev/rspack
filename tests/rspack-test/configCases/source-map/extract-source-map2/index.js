const fs = require("fs");
const path = require("path");

require("./a");

it("should extract source map", () => {
	const fileData = fs
		.readFileSync(path.resolve(__dirname, "bundle0.js.map"))
		.toString("utf-8");
	const { sources, sourcesContent } = JSON.parse(fileData);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(sources.includes(sourceUrl("./external-source-map.txt"))).toBe(true);
	expect(sourcesContent.map(s => s.trim())).toContain("source");
});
