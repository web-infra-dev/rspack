// TODO: Rspack is not support inline loader like `require("./loader-source-root!")`
it("should run", () => {
	require("./loader-source-root!./foo.js");
	// require("./loader-source-root-slash!./foo.js");
	// require("./loader-source-root-source-slash!./foo.js");
	// require("./loader-source-root-2-slash!./foo.js");
	// require("./loader-no-source-root!./foo.js");
	// require("./loader-pre-relative!./foo.js");
});

it("should generate the correct SourceMap", function () {
	var fs = require("fs");
	var source = JSON.parse(fs.readFileSync(__filename + ".map", "utf-8"));
	expect(source.sources).toContain("webpack:///./folder/test1.txt");
	// expect(source.sources).toContain("webpack:///./folder/test2.txt");
	// expect(source.sources).toContain("webpack:///./folder/test3.txt");
	// expect(source.sources).toContain("webpack:///./folder/test4.txt");
	// expect(source.sources).toContain("webpack:///./folder/test5.txt");
	// expect(source.sources).toContain("webpack:///./folder/test6.txt");
});
