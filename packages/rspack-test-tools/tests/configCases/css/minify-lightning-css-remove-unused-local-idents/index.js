import * as styles from "./style.module.css";

it("should remove unused local idents", async () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	expect(styles.a).toBe("./style.module-a");

	const css = await fs.promises.readFile(path.resolve(__dirname, "./bundle0.css"));
	expect(css).not.toContain("./style.module-b")
})
