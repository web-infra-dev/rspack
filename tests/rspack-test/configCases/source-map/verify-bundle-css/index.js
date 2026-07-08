const fs = require("fs");
const path = require("path");
const checkMap = require("@rspack/test-tools/helper/util/checkSourceMap").default;

require("./entry.css");

it("verify css bundle source map", async () => {
	const source = fs.readFileSync(
		path.resolve(__dirname, "bundle0.css.map"),
		"utf-8"
	);
	const map = JSON.parse(source);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	expect(map.sources).toEqual([
		sourceUrl("./b-dir/c-dir/c.css"),
		sourceUrl("./b-dir/b.css"),
		sourceUrl("./a.css"),
		sourceUrl("./entry.css")
	]);
	expect(map.file).toEqual("bundle0.css");
	const out = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(
		await checkMap(out, source, {
			'a:nth-child(0):after { content: "a0"; }': sourceUrl("a.css"),
			'a:nth-child(1):after { content: "a1"; }': sourceUrl("a.css"),
			'a:nth-child(2):after { content: "a2"; }': sourceUrl("a.css"),
			'b:nth-child(0):after { content: "b0"; }': sourceUrl("b-dir/b.css"),
			'b:nth-child(1):after { content: "b1"; }': sourceUrl("b-dir/b.css"),
			'b:nth-child(2):after { content: "b2"; }': sourceUrl("b-dir/b.css"),
			'c:nth-child(0):after { content: "c0"; }': sourceUrl("b-dir/c-dir/c.css"),
			'c:nth-child(1):after { content: "c1"; }': sourceUrl("b-dir/c-dir/c.css"),
			'c:nth-child(2):after { content: "c2"; }': sourceUrl("b-dir/c-dir/c.css")
		})
	).toBe(true);
});
