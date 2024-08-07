require("./index.css");

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("module resolve preferRelative should work", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);
	const a = /a: url\((.*)\);/.exec(css)[1];
	expect(a.startsWith("images")).toBe(false);
	expect(a.endsWith(".png")).toBe(true);
});
