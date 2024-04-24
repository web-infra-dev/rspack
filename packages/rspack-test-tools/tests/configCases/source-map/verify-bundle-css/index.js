const fs = require("fs");
const path = require("path");
const checkMap = require("../../../../dist/helper/util/checkSourceMap").default;

require("./entry.css");

it("verify css bundle source map", async () => {
	const source = fs.readFileSync(
		path.resolve(__dirname, "bundle0.css.map"),
		"utf-8"
	);
	const map = JSON.parse(source);
	expect(map.sources).toEqual([
		"webpack:///./b-dir/c-dir/c.css",
		"webpack:///./b-dir/b.css",
		"webpack:///./a.css",
		"webpack:///./entry.css"
	]);
	expect(map.file).toEqual("bundle0.css");
	const out = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(
		await checkMap(out, source, {
			'a:nth-child(0):after { content: "a0"; }': "webpack:///a.css",
			'a:nth-child(1):after { content: "a1"; }': "webpack:///a.css",
			'a:nth-child(2):after { content: "a2"; }': "webpack:///a.css",
			'b:nth-child(0):after { content: "b0"; }': "webpack:///b-dir/b.css",
			'b:nth-child(1):after { content: "b1"; }': "webpack:///b-dir/b.css",
			'b:nth-child(2):after { content: "b2"; }': "webpack:///b-dir/b.css",
			'c:nth-child(0):after { content: "c0"; }': "webpack:///b-dir/c-dir/c.css",
			'c:nth-child(1):after { content: "c1"; }': "webpack:///b-dir/c-dir/c.css",
			'c:nth-child(2):after { content: "c2"; }': "webpack:///b-dir/c-dir/c.css"
		})
	).toBe(true);
});
