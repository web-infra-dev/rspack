require("./index.css");
const fs = require("fs");
const path = require("path");

it("should rewrite the css url()", function () {
	const css = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");
	const url = /url\((.*)\)/.exec(css)[1];
	expect(url.startsWith("./")).toBe(false);
	expect(url.includes("./logo.png")).toBe(false);
});
