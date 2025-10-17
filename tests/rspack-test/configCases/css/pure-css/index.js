import "./style.css";

it("should compile", () => {
	const path = __non_webpack_require__("path");
	const links = document.getElementsByTagName("link");
	const css = [];

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		css.push(link.sheet.css);
	}

	expect(css).toMatchFileSnapshot(path.join(__SNAPSHOT__, `css.txt`));
});
