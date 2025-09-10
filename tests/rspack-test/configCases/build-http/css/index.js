import "./style.css";
const path = __non_webpack_require__("path");

it(`should work with URLs in CSS`, done => {
	const links = document.getElementsByTagName("link");
	const css = [];

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		css.push(link.sheet.css);
	}

	expect(css).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'bundle0.css.txt'));
	done();
});
