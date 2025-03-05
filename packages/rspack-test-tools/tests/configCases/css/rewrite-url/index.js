require("./index.css");
const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should rewrite the css url()", function () {
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	const a = /a: url\((.*)\);/.exec(css)[1];
	expect(a.startsWith("./")).toBe(false);
	expect(a.includes("./logo.png")).toBe(false);
	expect(a.endsWith(".png")).toBe(true);
	expect(a).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'a.snap'));
	const b = /b: url\((.*)\);/.exec(css)[1];
	expect(b).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'b.snap'));
	const c = /c: url\((.*)\);/.exec(css)[1];
	expect(c).toBe("#ccc");
	const d = /d: url\((.*)\);/.exec(css)[1];
	expect(d).toBe("https://rspack.dev/tests/~img.png");
});
