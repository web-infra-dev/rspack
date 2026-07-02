it("should include the entry-b sourceURL in SourceMap", function() {
	const fs = require("fs");
	const source = fs.readFileSync(__filename, "utf-8");
	let scheme = "webpack";
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		scheme = "rspack";
	}
	expect(source).toMatch(
		new RegExp(
			`sourceURL=${scheme}:\\/\\/library-entry-b\\/\\.\\/src\\/entry-b\\.js\\?[a-zA-Z0-9]+`
		)
	);
});
