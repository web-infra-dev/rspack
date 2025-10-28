import "./style.css";

it(`should work with URLs in CSS`, () => new Promise(done => {
	const links = document.getElementsByTagName("link");
	const css = [];

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		css.push(link.sheet.css);
	}

	expect(css).toMatchFileSnapshot(`${__SNAPSHOT__}/css.txt`);
	done();
}));
