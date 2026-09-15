it("should include the entry-a sourceURL in SourceMap", function() {
	const fs = require("fs");
	const source = fs.readFileSync(__filename, "utf-8");
	let scheme = "webpack";
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		scheme = "rspack";
	}
	expect(source).toMatch(
		new RegExp(
			`sourceURL=${scheme}:\\/\\/library-entry-a\\/\\.\\/src\\/entry-a\\.js\\?[a-zA-Z0-9]+`
		)
	);
});
