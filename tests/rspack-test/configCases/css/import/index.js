import "./style.css";

it("should compile", () => {
	const links = Array.from(document.getElementsByTagName("link"));
	const path = __non_webpack_require__("path");
	const css = [];

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		css.push(getLinkSheet(link));
	}

	expect(css).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'bundle0.css.txt'));
});
