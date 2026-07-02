it("should include webpack://library-entry-b/./src/entry-b.js in SourceMap", function() {
	const fs = require("fs");
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	let scheme = "webpack";
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		scheme = "rspack";
	}
	expect(map.sources).toContain(`${scheme}://library-entry-b/./src/entry-b.js`);
});
