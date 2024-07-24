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
		"webpack:///./a.css"
	]);
	expect(map.file).toEqual("bundle0.css");
	const out = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(
		await checkMap(out, source, {
			'a:nth-child(0):after { content: "a0"; }': {
				inSource: "webpack:///a.css",
				outId: 'a:nth-child(0):after{content:"a0"}'
			},
			'a:nth-child(1):after { content: "a1"; }': {
				inSource: "webpack:///a.css",
				outId: 'a:first-child:after{content:"a1"}'
			},
			'a:nth-child(2):after { content: "a2"; }': {
				inSource: "webpack:///a.css",
				outId: 'a:nth-child(2):after{content:"a2"}'
			},
			'b:nth-child(0):after { content: "b0"; }': {
				inSource: "webpack:///b-dir/b.css",
				outId: 'b:nth-child(0):after{content:"b0"}'
			},
			'b:nth-child(1):after { content: "b1"; }': {
				inSource: "webpack:///b-dir/b.css",
				outId: 'b:first-child:after{content:"b1"}'
			},
			'b:nth-child(2):after { content: "b2"; }': {
				inSource: "webpack:///b-dir/b.css",
				outId: 'b:nth-child(2):after{content:"b2"}'
			},
			'c:nth-child(0):after { content: "c0"; }': {
				inSource: "webpack:///b-dir/c-dir/c.css",
				outId: 'c:nth-child(0):after{content:"c0"}'
			},
			'c:nth-child(1):after { content: "c1"; }': {
				inSource: "webpack:///b-dir/c-dir/c.css",
				outId: 'c:first-child:after{content:"c1"}'
			},
			'c:nth-child(2):after { content: "c2"; }': {
				inSource: "webpack:///b-dir/c-dir/c.css",
				outId: 'c:nth-child(2):after{content:"c2"}'
			},
		})
	).toBe(true);
});
