import * as styles from "./index.module.css";

styles["switch"];
styles["default"];

it("should works", async () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const js = await fs.promises.readFile(
		path.resolve(__dirname, "./bundle0.js"),
		"utf-8"
	);
	expect(js).toContain("_switch = ");
	expect(js).toContain("_default = ");
});
