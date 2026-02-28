import "./index.css";
const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should build success", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);
	const invalidRaw =
		"data:application/x-font-ttf;charset=utf-8;base64,AAEAAAAQAQAABAAARkZUTZA8qYoAACe8AAAAHEdERUY";
	expect(css.includes(invalidRaw)).toBeTruthy();
});
