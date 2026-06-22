import * as styles from "./entry.modules.css";

it("orders composes-from-file imports topologically across rules", () => {
	const fs = require("fs");
	const path = require("path");
	const css = fs.readFileSync(path.join(__dirname, "bundle0.css"), "utf-8");

	expect(css).toMatchFileSnapshotSync(
		path.join(__SNAPSHOT__, "bundle0.css.txt")
	);
	expect(styles).toMatchFileSnapshotSync(
		path.join(__SNAPSHOT__, "exports.txt")
	);
});
