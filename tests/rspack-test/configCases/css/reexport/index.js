import * as styles from "./styles.module.css";

it("should work with asset modules", async () => {
	const fs = require("fs");
	const path = require("path");
	const css = fs.readFileSync(
		path.join(__dirname, `bundle${__STATS_I__}.css`),
		"utf-8"
	);

	expect(css).toMatchFileSnapshotSync(
		path.join(__SNAPSHOT__, `bundle${__STATS_I__}.css.txt`)
	);
	expect(styles).toMatchFileSnapshotSync(
		path.join(__SNAPSHOT__, `exports.${__STATS_I__}.txt`)
	);
});
