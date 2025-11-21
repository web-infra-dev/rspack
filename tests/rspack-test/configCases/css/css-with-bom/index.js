import "./index.css";

it("should not contain BOM at the start of the CSS file", async () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const css = await fs.promises.readFile(
		path.resolve(__dirname, "bundle0.css"),
		"utf-8"
	);
	expect(css[0]).not.toBe("\uFEFF");
});
