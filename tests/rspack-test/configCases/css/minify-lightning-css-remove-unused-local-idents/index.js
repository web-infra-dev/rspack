import * as styles from "./style.module.css";

it("should remove unused local idents", async () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	expect(styles.a).toBe("./style.module-a");
	expect(styles["local/used"]).toBe("./style.module-local/used");
	expect(styles["composed-local"]).toBe("./style.module-composed-local");
	expect(styles.camelCase).toBe("./style.module-camel-case");

	if (!EXPORTS_ONLY) {
		const css = await fs.promises.readFile(path.resolve(__dirname, "./bundle0.css"), "utf-8");
		expect(css).not.toContain(".module-b")
		expect(css).toContain("local\\/used")
		expect(css).not.toContain("local\\/unused")
		expect(css).toContain("composed-local")
		expect(css).toContain("camel-case")
	}
})
