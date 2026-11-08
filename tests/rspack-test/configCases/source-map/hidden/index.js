it("should not have map from url comments if hidden options is used", function () {
	const fs = require("fs");
	const source = fs.readFileSync(__filename, "utf-8");
	expect(/sourceMappingURL\s*=\s*(.*)/.test(source)).toBe(false);
	const mapSource = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(mapSource);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(map.sources).toContain(sourceUrl("./index.js"));
	expect(map.file).toEqual("bundle0.js");
});
