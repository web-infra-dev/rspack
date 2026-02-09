require("./a.css");
const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("import filter function should filter imports based on return value", async () => {
	const css = await fs.promises.readFile(
		path.resolve(__dirname, "bundle0.css"),
		"utf-8"
	);

	// b.css and c.css should be included (filter returned true)
	expect(css).toContain(".b {");
	expect(css).toContain(".c {");

	// d.css should NOT be included (filter returned false)
	expect(css).not.toContain(".d {");

	// Main file content should still be present
	expect(css).toContain(".a {");
});
