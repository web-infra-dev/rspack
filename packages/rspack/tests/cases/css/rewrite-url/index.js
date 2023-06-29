require("./index.css");
const fs = require("fs");
const path = require("path");

it("should rewrite the css url()", function () {
	const css = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");
	const a = /a: url\((.*)\);/.exec(css)[1];
	expect(a.startsWith("./")).toBe(false);
	expect(a.includes("./logo.png")).toBe(false);
	expect(a.endsWith(".png")).toBe(true);
	expect(a).toMatchSnapshot();
	const b = /b: url\((.*)\);/.exec(css)[1];
	expect(b).toMatchSnapshot();
	const c = /c: url\((.*)\);/.exec(css)[1];
	expect(c).toBe("#ccc");
	const d = /d: url\((.*)\);/.exec(css)[1];
	expect(d).toBe("https://rspack.dev/tests/~img.png");
});
