import * as styles from "./style.modules.css";

it(`should work with URLs in CSS`, done => {
	const links = document.getElementsByTagName("link");
	const css = [];
	const path = __non_webpack_require__("path");

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		css.push(link.sheet.css);
	}

	expect(css).toMatchFileSnapshot(path.join(__SNAPSHOT__, `css.${__STATS_I__}.txt`));
	expect(styles).toMatchFileSnapshot(path.join(__SNAPSHOT__, `styles.${__STATS_I__}.txt`));
	done();
});
