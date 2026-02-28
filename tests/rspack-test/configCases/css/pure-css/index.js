import "./style.css";

it("should compile", () => {
	const path = __non_webpack_require__("path");
	const links = Array.from(document.getElementsByTagName("link"));
	const css = [];

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		css.push(getLinkSheet(link));
	}

	expect(css).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, `css.txt`));
});
