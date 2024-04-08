import fs from "fs";
import * as styles from "./index.module.css";

it("should use correct generator options", async () => {
	expect(styles).toEqual(nsObj({}));
	const files = await fs.promises.readdir(__dirname);
	expect(files.every(file => !file.endsWith(".css"))).toBe(true)
});
