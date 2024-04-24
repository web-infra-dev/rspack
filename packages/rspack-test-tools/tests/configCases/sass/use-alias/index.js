require("./index.scss");
const fs = require("fs");
const path = require("path");

it(`should work when "@use" with an alias`, () => {
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css).toContain(".alias");
	expect(css).toContain(".directory-6-file");
	expect(css).toContain(".dir-with-underscore-index");
});
