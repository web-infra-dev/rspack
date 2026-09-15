it("should have map from url comments if inline options is used", function () {
	const fs = require("fs");
	const source = fs.readFileSync(__filename, "utf-8");
	const base64 =
		/sourceMappingURL\s*=\s*data:application\/json;charset=utf-8;base64,(.*)/.exec(
			source
		)[1];
	const map = JSON.parse(Buffer.from(base64, "base64").toString("utf-8"));
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(map.sources).toContain(sourceUrl("./index.js"));
	expect(map.file).toEqual("bundle0.js");
});
