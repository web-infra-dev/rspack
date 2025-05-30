require("./index.css");
const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should rewrite the css url()", function () {
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	const a = /a: url\((.*)\);/.exec(css)[1];
	expect(a).toBe("https://test.rspack.rs/cdn/logo.png");
});
