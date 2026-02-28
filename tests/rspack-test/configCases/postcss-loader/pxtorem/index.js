const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should transform px to rem with postcss-loader", () => {
	require("./index.css");
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);
	expect(css.includes("rem")).toBe(true);
	expect(css.includes("px")).toBe(false);
});
