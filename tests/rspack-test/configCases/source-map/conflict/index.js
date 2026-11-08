import("./a");
import("./b");

it("conflict", () => {
	const fs = require("fs");
	const source_a = fs.readFileSync(__dirname + "/a_js.bundle0.js.map", "utf-8");
	const source_b = fs.readFileSync(__dirname + "/b_js.bundle0.js.map", "utf-8");
	const map_a = JSON.parse(source_a);
	const map_b = JSON.parse(source_b);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(map_a.sources).toStrictEqual([
		sourceUrl("./a.js"),
		sourceUrl("./common.js"),
	]);
	expect(map_b.sources).toStrictEqual([
		sourceUrl("./b.js"),
		sourceUrl("./common.js"),
	]);
});
