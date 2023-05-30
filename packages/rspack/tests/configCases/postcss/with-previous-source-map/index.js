const fs = require("fs");
const path = require("path");

it("basic", () => {
	require("./index.scss");
	const sourceMap = fs.readFileSync(__dirname + "/main.css.map", "utf-8");
	const scss = fs.readFileSync(
		path.resolve(__dirname, "../index.scss"),
		"utf-8"
	);
	const map = JSON.parse(sourceMap);
	expect(map.sources).toContain("./index.scss");
	expect(map.file).toEqual("main.css");
	expect(map.sourcesContent).toEqual([scss]);
});
