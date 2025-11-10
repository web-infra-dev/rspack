const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should generate share container bundle with expected content", async () => {
	const {Button} = await import("ui-lib");
	expect(Button).toEqual('Button');

	const bundlePath = path.join(
		__dirname,
		"node_modules_ui-lib_index_js.js"
	);
	const bundleContent = fs.readFileSync(bundlePath, "utf-8");
	expect(bundleContent).toContain("Button");
	expect(bundleContent).not.toContain("Badge");
	expect(bundleContent).not.toContain("List");
});
