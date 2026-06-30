it("should include webpack://library-entry-a/./src/entry-a.js in SourceMap", function() {
	var fs = require("fs");
	var source = fs.readFileSync(__filename + ".map", "utf-8");
	var map = JSON.parse(source);
	let scheme = "webpack";
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		scheme = "rspack";
	}
	expect(map.sources).toContain(`sourceURL=${scheme}://library-entry-a/./src/entry-a.js`);
});
