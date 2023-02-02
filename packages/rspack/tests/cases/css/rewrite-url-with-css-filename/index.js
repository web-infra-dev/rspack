require("./index.css");
const fs = require("fs");
const path = require("path");

it("should rewrite the css url() with publicPath when output.cssFilename is set", function () {
	const css = fs.readFileSync(path.resolve(__dirname, "css/main.css"), "utf-8");
	const a = /a: url\("(.*)"\);/.exec(css)[1];
	expect(a.startsWith("./")).toBe(false);
	expect(a.includes("./logo.png")).toBe(false);
	expect(a.endsWith(".png")).toBe(true);
	expect(a.startsWith("/")).toBe(true);
	expect(a === "/image/logo.19f79ab9.png").toBe(true);
});
