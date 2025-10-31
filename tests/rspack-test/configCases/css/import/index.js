import "./style.css";
import path from "path";

it("should compile", () => {
	const links = document.getElementsByTagName("link");
	const css = [];

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		css.push(link.sheet.css);
	}

	expect(css).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'bundle0.css.txt'));
});
