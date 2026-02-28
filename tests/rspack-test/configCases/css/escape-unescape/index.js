import * as styles from "./style.modules.css";

it(`should work with URLs in CSS`, () => {
	const links = Array.from(document.getElementsByTagName("link"));
	const css = [];
	const path = __non_webpack_require__("path");

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		css.push(getLinkSheet(link));
	}

	expect(css).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, `css.${__STATS_I__}.txt`));
	expect(styles).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, `styles.${__STATS_I__}.txt`));
});
