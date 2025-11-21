it("should generate the correct sourceRoot in SourceMap", function () {
	const fs = require("fs");
	const path = require("path");
	const source = JSON.parse(fs.readFileSync(__filename + ".map", "utf-8"));
	expect(source.sourceRoot).toContain(path.resolve(CONTEXT, "./folder") + "/");
});
