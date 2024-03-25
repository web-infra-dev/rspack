require("./index.css");
const fs = require("fs");
const path = require("path");

it("should rewrite the css url() in css variables", function () {
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	const a = /--a: url\((.*)\);/.exec(css)[1];
	expect(a.startsWith("./")).toBe(false);
	expect(a.includes("./logo.png")).toBe(false);
	expect(a.endsWith(".png")).toBe(true);
	expect(a).toMatchSnapshot();
	const b = /--b: url\((.*)\);/.exec(css)[1];
	expect(b.startsWith("./")).toBe(false);
	expect(b.includes("./logo.png")).toBe(false);
	expect(b.endsWith(".png")).toBe(true);
	expect(b).toMatchSnapshot();
	const c = /--c: (.*?);/.exec(css)[1];
	expect(c).toBe(JSON.stringify(""));
});
