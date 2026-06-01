const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", async () => {
  const { default: x } = await import("./use-style.js");
	expect(x).toEqual({
		cssModuleWithCustomFileExtension: prod
			? "my-app-666-pQ"
			: "style_module_my-css-myCssClass",
		notAValidCssModuleExtension: true,
		UsedClassName: prod
			? "my-app-744-GW"
			: "identifiers_module_css-UsedClassName",
		exportLocalVarsShouldCleanup: "false false"
	});

	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const cssOutputFilename = "use-style_js.bundle0.css";

	const cssContent = fs.readFileSync(
		path.join(__dirname, cssOutputFilename),
		"utf-8"
	);
	expect(cssContent).not.toContain(".my-app--");
	expect(cssContent).toContain(".style_module_my-css-myCssClass");
	expect(cssContent).toContain(".identifiers_module_css-UsedClassName");
});
