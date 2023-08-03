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
		"./b-dir/c-dir/c.css",
		"./b-dir/b.css",
		"./a.css",
		"./entry.css"
	]);
	expect(map.file).toEqual("main.css");
	const out = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");
	expect(
		await checkMap(out, source, {
			'a:nth-child(0):after { content: "a0"; }': "a.css",
			'a:nth-child(1):after { content: "a1"; }': "a.css",
			'a:nth-child(2):after { content: "a2"; }': "a.css",
			'b:nth-child(0):after { content: "b0"; }': "b-dir/b.css",
			'b:nth-child(1):after { content: "b1"; }': "b-dir/b.css",
			'b:nth-child(2):after { content: "b2"; }': "b-dir/b.css",
			'c:nth-child(0):after { content: "c0"; }': "b-dir/c-dir/c.css",
			'c:nth-child(1):after { content: "c1"; }': "b-dir/c-dir/c.css",
			'c:nth-child(2):after { content: "c2"; }': "b-dir/c-dir/c.css"
		})
	).toBe(true);
});
