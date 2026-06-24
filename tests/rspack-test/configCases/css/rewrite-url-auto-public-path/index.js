require("./index.css");
const fs = require("fs");
const path = require("path");

it("should rewrite the css url() with auto publicPath when output.cssFilename is set", function () {
	const css = fs.readFileSync(path.resolve(__dirname, "css/main.css"), "utf-8");
	const a = /a: url\((.*)\);/.exec(css)[1];
	expect(a === "../image/logo.png").toBe(true);
});

it("should rewrite the css url() with auto publicPath and ~@ prefix", function () {
	const css = fs.readFileSync(path.resolve(__dirname, "css/main.css"), "utf-8");
	const b = /b: url\((.*)\);/.exec(css)[1];
	expect(b === "../image/logo.png").toBe(true);
});
