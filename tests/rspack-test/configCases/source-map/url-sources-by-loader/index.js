it("should run", () => {
	require("./loader.mjs!");
});

it("should make url sources from loaders context relative", function () {
	var fs = require("fs");
	var source = JSON.parse(fs.readFileSync(__filename + ".map", "utf-8"));
	let sourceUrl = sourceName => `webpack:///${sourceName}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = sourceName => `rspack:///${sourceName}`;
	}
	expect(source.sources).toContain(sourceUrl("./folder/file-url.txt"));
	expect(source.sources).toContain(sourceUrl("./my folder/encoded.txt"));
	expect(source.sources).toContain(sourceUrl("../sibling/outside.txt"));
	expect(source.sources).toContain(sourceUrl("file://server/share/remote.txt"));
});
