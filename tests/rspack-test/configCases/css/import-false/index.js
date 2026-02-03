require("./a.css");
const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("import-false should disable @import processing", async () => {
	const css = await fs.promises.readFile(
		path.resolve(__dirname, "bundle0.css"),
		"utf-8"
	);

	// @import should remain in the output CSS (not resolved)
	expect(css).toContain("@import");
	// The imported file content should NOT be in the output
	expect(css).not.toContain(".b {");
	// The main file content should still be present
	expect(css).toContain(".a {");
});
