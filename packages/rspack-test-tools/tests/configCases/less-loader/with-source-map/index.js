const fs = require("fs");
const path = require("path");

it("basic", () => {
	const css = require("./index.less");
	expect(css).toEqual(nsObj({}));
	const sourceMap = fs.readFileSync(__dirname + "/bundle0.css.map", "utf-8");
	const map = JSON.parse(sourceMap);
	expect(map.sources).toContain("webpack:///./index.less");
	expect(map.file).toEqual("bundle0.css");
	expect(map.sourcesContent).toEqual([
		fs.readFileSync(
			__dirname + "/" + require("!!./index.less?resource"),
			"utf-8"
		)
	]);
});
