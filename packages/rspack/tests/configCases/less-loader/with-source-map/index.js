const fs = require("fs");
const path = require("path");

it("basic", () => {
	const css = require("./index.less");
	expect(css).toEqual({});
	const less = fs.readFileSync(
		path.resolve(__dirname, "../index.less"),
		"utf-8"
	);
	const sourceMap = fs.readFileSync(__dirname + "/main.css.map", "utf-8");
	const map = JSON.parse(sourceMap);
	expect(map.sources).toContain("./index.less");
	expect(map.file).toEqual("main.css");
	expect(map.sourcesContent).toEqual([less]);
});
