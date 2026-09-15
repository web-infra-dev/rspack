import * as styles from "./style.module.css";

it("should allow to disable options", () => {
	const fs = require("fs");
	const path = require("path");
	expect(styles).toMatchFileSnapshotSync(
		path.join(__SNAPSHOT__, `options-classes.${__STATS_I__}.txt`)
	);

	const cssOutputFilename = `bundle6.css`;

	const cssContent = fs.readFileSync(
		path.join(__dirname, cssOutputFilename),
		"utf-8"
	);
	expect(cssContent).toMatchFileSnapshotSync(
		path.join(__SNAPSHOT__, `options-css.${__STATS_I__}.txt`)
	);
});
