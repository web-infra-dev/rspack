// TODO: Rspack is not support inline loader like `require("./loader-source-root!")`
it("should run", () => {
	require("./loader-source-root!");
	// require("./loader-source-root-slash!");
	// require("./loader-source-root-source-slash!");
	// require("./loader-source-root-2-slash!");
	// require("./loader-no-source-root!");
	// require("./loader-pre-relative!");
});

it("should generate the correct SourceMap", function () {
	var fs = require("fs");
	var source = JSON.parse(fs.readFileSync(__filename + ".map", "utf-8"));
	let sourceUrl = sourceName => `webpack:///${sourceName}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = sourceName => `rspack:///${sourceName}`;
	}
	expect(source.sources).toContain(sourceUrl("./folder/test1.txt"));
	// expect(source.sources).toContain("webpack:///./folder/test2.txt");
	// expect(source.sources).toContain("webpack:///./folder/test3.txt");
	// expect(source.sources).toContain("webpack:///./folder/test4.txt");
	// expect(source.sources).toContain("webpack:///./folder/test5.txt");
	// expect(source.sources).toContain("webpack:///./folder/test6.txt");
});
