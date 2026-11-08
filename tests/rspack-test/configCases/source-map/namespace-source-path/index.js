it("should include webpack://mynamespace/./test.js in SourceMap", function() {
	var fs = require("fs");
	var source = fs.readFileSync(__filename + ".map", "utf-8");
	var map = JSON.parse(source);
	let scheme = "webpack";
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		scheme = "rspack";
	}
	expect(map.sources).toContain(`${scheme}://mynamespace/./test.js`);
});

if (Math.random() < 0) require("./test.js");
