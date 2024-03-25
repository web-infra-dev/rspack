const fs = require("fs");
const path = require("path");

it("basic", () => {
	const css = require("./index.scss");
	expect(css).toEqual({});
	const source = fs.readFileSync(
		path.resolve(__dirname, "bundle0.css.map"),
		"utf-8"
	);
	const map = JSON.parse(source);
	const scss = fs.readFileSync(path.resolve(CONTEXT, "./index.scss"), "utf-8");
	expect(map.sources).toEqual(["webpack:///./index.scss"]);
	expect(map.sourcesContent).toEqual([scss]);
	expect(map.file).toEqual("bundle0.css");
});
