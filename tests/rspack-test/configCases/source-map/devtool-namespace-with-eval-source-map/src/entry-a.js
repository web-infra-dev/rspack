it("should include the entry-a source in SourceMap", function() {
	const fs = require("fs");
	const source = fs.readFileSync(__filename, "utf-8");
	const match = source.match(
		/sourceMappingURL=data:application\/json;charset=utf-8;base64,([A-Za-z0-9+/=]+)/
	);
	expect(match).toBeTruthy();
	const map = JSON.parse(Buffer.from(match[1], "base64").toString());
	let scheme = "webpack";
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		scheme = "rspack";
	}
	expect(
		map.sources.some(source =>
			new RegExp(
				`${scheme}:\\/\\/library-entry-a\\/\\.\\/src\\/entry-a\\.js\\?[a-zA-Z0-9]+`
			).test(source)
		)
	).toBe(true);
});
