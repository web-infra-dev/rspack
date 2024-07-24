import * as styles from "./style.module.css";
import fs from "fs";
import path from "path";

it("should remove unused local idents", async () => {
	expect(styles.a).toBe("./style.module-a");

	const css = await fs.promises.readFile(path.resolve(__dirname, "./bundle0.css"));
	expect(css).not.toContain("./style.module-b")
})