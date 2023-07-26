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
			a0: "a.css",
			a1: "a.css",
			a2: "a.css",
			b0: "b-dir/b.css",
			b1: "b-dir/b.css",
			b2: "b-dir/b.css",
			c0: "b-dir/c-dir/c.css",
			c1: "b-dir/c-dir/c.css",
			c2: "b-dir/c-dir/c.css"
		})
	).toBe(true);
});
