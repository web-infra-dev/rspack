require("./index.scss");
const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it(`should work when "@import" with an alias`, () => {
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css).toContain(".alias");
	expect(css).toContain(".directory-6-file");
	expect(css.match(/\.dir-with-underscore-index/g).length).toBe(3);
});
