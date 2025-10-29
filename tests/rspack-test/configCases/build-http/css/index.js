import "./style.css";
const path = __non_webpack_require__("path");

it(`should work with URLs in CSS`, async () => {
	const links = Array.from(document.getElementsByTagName("link"));
	const css = [];

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		css.push(getLinkSheet(link));
	}

	expect(css).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'bundle0.css.txt'));
});
