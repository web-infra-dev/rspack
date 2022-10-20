const fs = require("fs");

it("basic", () => {
	const css = require("./index.css");
	const style = require("./index.module.css");
	expect(css).toEqual({});
	expect(style).toEqual({
		body: "_body_toys1_1"
	});
	const sourceMap = fs.readFileSync(__dirname + "/main.css.map", "utf-8");
	const map = JSON.parse(sourceMap);
	expect(map.sources).toContain("index.css");
	expect(map.sources).toContain("index.module.css");
	expect(map.file).toEqual("main.css");
});
