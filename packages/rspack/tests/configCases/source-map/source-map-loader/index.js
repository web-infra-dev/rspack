it("should include a.ts in SourceMap", function() {
	var fs = require("fs");
	var source = fs.readFileSync(__filename + ".map", "utf-8");
	var map = JSON.parse(source);
	expect(map.sources).toContain("webpack:///./a.ts");
});

if (Math.random() < 0) require("./a.js");
