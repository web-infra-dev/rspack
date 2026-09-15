const foo = Math.random() > 0.5 ? "a" : "b";
require(`./foo/${foo}.js`);

it("context module should use relative path in source map file", () => {
	var fs = require("fs");
	var source = fs.readFileSync(__filename + ".map", "utf-8");
	var map = JSON.parse(source);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}

	expect(map.sources).toContain(sourceUrl("./foo|sync|/^\\.\\/.*\\.js$/"));
});
