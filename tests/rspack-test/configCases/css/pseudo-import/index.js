import './style.modules.css';

it("should compile", () => {
	const path = __non_webpack_require__("path");
	const links = Array.from(document.getElementsByTagName("link"));
	const css = [];

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		css.push(getLinkSheet(link));
	}

	expect(css).toMatchFileSnapshot(path.join(__SNAPSHOT__, `css.txt`));
});

it("should re-export", async () => {
	const module = await import("./reexport.modules.css");
	expect(module).toEqual(nsObj({
		"className": "_reexport_modules_css-className",
		"primary-color": "constructor",
		"secondary-color": "toString",
	}));
});
