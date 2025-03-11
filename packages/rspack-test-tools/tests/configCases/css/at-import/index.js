require("./a.css");
const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("at-import-in-the-top", async () => {
	const css = await fs.promises.readFile(
		path.resolve(__dirname, "bundle0.css"),
		"utf-8"
	);

	expect(css).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'bundle0.css.txt'));
});
