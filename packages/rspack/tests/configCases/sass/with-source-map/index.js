const fs = require("fs");
const path = require("path");

it("basic", () => {
	const css = require("./index.scss");
	expect(css).toEqual({});
	const source = fs.readFileSync(
		path.resolve(__dirname, "main.css.map"),
		"utf-8"
	);
	const map = JSON.parse(source);
	const scss = fs.readFileSync(
		path.resolve(__dirname, "../index.scss"),
		"utf-8"
	);
	expect(map.sources).toEqual(["./index.scss"]);
	expect(map.sourcesContent).toEqual([scss]);
	expect(map.file).toEqual("main.css");
});
