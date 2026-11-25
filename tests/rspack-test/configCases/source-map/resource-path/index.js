it("should use POSIX separators and omit layer and type in absoluteResourcePath", function () {
	var fs = require("fs");
	var path = require("path");
	var source = fs.readFileSync(__filename + ".map", "utf-8");
	var map = JSON.parse(source);
	var expectedPath = path
		.resolve(__TEST_PATH__, "configCases/source-map/resource-path/test.js")
		.replaceAll(path.sep, "/");
	expect(map.sources).toContain(expectedPath);
});

if (Math.random() < 0) require("./test.js");
