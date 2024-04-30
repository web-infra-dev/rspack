const fs = require("fs");
const path = require("path");

it("should have normalize.css content in css code", () => {
	require("./index.css");

	const js = fs.readFileSync(path.resolve(__dirname, "main.js"), "utf-8");
	const cssStub = /".\/index.css":.*\n(.*)/.exec(js)[1];
	expect(cssStub.includes("throw new Error")).toBe(false);
	const css = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");
	expect(css.includes("throw new Error")).toBe(false);
	expect(css.includes("html {")).toBe(true);
});
