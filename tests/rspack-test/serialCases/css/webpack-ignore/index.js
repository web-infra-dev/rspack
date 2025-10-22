import "./style.css";

it("should compile", () => {
	const links = document.getElementsByTagName("link");
	const css = links[1].sheet.css;

	expect(css).toMatchFileSnapshot(`${__SNAPSHOT__}/css.txt`);
});
