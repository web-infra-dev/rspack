const fs = require("fs");
const path = require("path");

it("basic", () => {
	require("./index.css");
	const sourceMap = fs.readFileSync(__dirname + "/bundle0.css.map", "utf-8");
	const css = fs.readFileSync(path.resolve(CONTEXT, "./index.css"), "utf-8");
	const map = JSON.parse(sourceMap);
	expect(map.sources).toContain("webpack:///./index.css");
	expect(map.file).toEqual("bundle0.css");
	expect(map.sourcesContent).toEqual([css]);
});
