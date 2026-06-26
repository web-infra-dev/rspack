import * as styles from "./style.module.css";

it("should allow to disable options", () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const get = (obj, key) => obj[key];

	expect(styles.class).toBe("my-app-style_module_css-class");
	expect(get(styles, "localkeyframes")).toBeUndefined();
	expect(get(styles, "local-color")).toBeUndefined();
	expect(get(styles, "progressAnimationLocal")).toBeUndefined();

	const cssOutputFilename = `bundle6.css`;

	const cssContent = fs.readFileSync(
		path.join(__dirname, cssOutputFilename),
		"utf-8"
	);
	expect(cssContent).toContain("@keyframes localkeyframes");
	expect(cssContent).toContain("--local-color: red");
	expect(cssContent).toContain("@property local(--progress)");
	expect(cssContent).toContain("var(local(--progress))");
});
