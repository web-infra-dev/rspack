const fs = require("fs");
const path = require("path");

it("basic", () => {
	require("./index.css");
	const sourceMap = fs.readFileSync(__dirname + "/main.css.map", "utf-8");
	const css = fs.readFileSync(path.resolve(__dirname, "../index.css"), "utf-8");
	const map = JSON.parse(sourceMap);
	expect(map.sources).toContain("./index.css");
	expect(map.file).toEqual("main.css");
	expect(map.sourcesContent).toEqual([css]);
});
