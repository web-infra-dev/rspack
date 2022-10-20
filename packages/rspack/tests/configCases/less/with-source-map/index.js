const fs = require("fs");

it("basic", () => {
	const css = require("./index.less");
	expect(css).toEqual({});
	const sourceMap = fs.readFileSync(__dirname + "/main.css.map", "utf-8");
	const map = JSON.parse(sourceMap);
	expect(map.sources).toContain("index.less");
	expect(map.file).toEqual("main.css");
});
