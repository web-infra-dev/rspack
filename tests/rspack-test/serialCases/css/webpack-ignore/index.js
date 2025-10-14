import "./style.css";

it("should compile", () => new Promise(done => {
	const links = document.getElementsByTagName("link");
	const css = links[1].sheet.css;

	expect(css).toMatchFileSnapshot(`${__SNAPSHOT__}/css.txt`);
	done();
}));
