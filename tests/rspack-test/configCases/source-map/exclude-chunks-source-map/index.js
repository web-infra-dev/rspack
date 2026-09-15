it("should include test.js in SourceMap for bundle0 chunk", function() {
	var fs = require("fs");
	var source = fs.readFileSync(__filename + ".map", "utf-8");
	var map = JSON.parse(source);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(map.sources).toContain(sourceUrl("./test.js"));
});

it("should not produce a SourceMap for vendors chunk", function() {
	var fs = require("fs"),
		path = require("path"),
		assert = require("assert");
	expect(fs.existsSync(path.join(__dirname, "vendors.js.map"))).toBe(false);
});

if (Math.random() < 0) require("./test.js");
