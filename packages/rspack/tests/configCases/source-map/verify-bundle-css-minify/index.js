const fs = require("fs");
const path = require("path");
const checkMap = require("../checkSourceMap").default;

require("./entry.css");

it("verify css bundle source map", async () => {
	const source = fs.readFileSync(
		path.resolve(__dirname, "main.css.map"),
		"utf-8"
	);
	const map = JSON.parse(source);
	expect(map.sources).toEqual([
		"webpack:///./b-dir/c-dir/c.css",
		"webpack:///./b-dir/b.css",
		"webpack:///./a.css"
	]);
	expect(map.file).toEqual("main.css");
	const out = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");
	expect(
		await checkMap(out, source, {
			'"a0"': "webpack:///a.css",
			'"a1"': "webpack:///a.css",
			'"a2"': "webpack:///a.css",
			'"b0"': "webpack:///b-dir/b.css",
			'"b1"': "webpack:///b-dir/b.css",
			'"b2"': "webpack:///b-dir/b.css",
			'"c0"': "webpack:///b-dir/c-dir/c.css",
			'"c1"': "webpack:///b-dir/c-dir/c.css",
			'"c2"': "webpack:///b-dir/c-dir/c.css"
		})
	).toBe(true);
});
