require("./index.css");
const fs = require("fs");
const path = require("path");

it("should rewrite the css url()", function () {
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	const a = /a: url\((.*)\);/.exec(css)[1];
	expect(a).toBe("https://test.rspack.dev/cdn/logo.png");
});
