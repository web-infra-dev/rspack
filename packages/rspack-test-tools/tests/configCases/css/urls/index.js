const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("css urls should works", async () => {
	await import("./urls.css");
	const css = await fs.promises.readFile(
		path.resolve(__dirname, "bundle.css"),
		"utf-8"
	);
	expect(css).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'bundle.css.txt'));
});
