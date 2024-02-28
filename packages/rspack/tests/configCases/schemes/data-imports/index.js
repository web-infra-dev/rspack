import a from 'data:text/javascript,export default "a";';
import b from "data:text/javascript,export default 'b';";
import c from "data:text/javascript,export default `c`;";
import "data:text/css,.red{color: red;}";
import "./index.css";
import inlineSvg from 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg"></svg>';
import fs from "node:fs";
import path from "node:path";

import('data:text/javascript,global.d = "d";');
import("data:text/javascript,global.e = 'e';");
import("data:text/javascript,global.f = `f`;");

it("data imports", () => {
	expect(a).toBe("a");
	expect(b).toBe("b");
	expect(c).toBe("c");

	expect(
		fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8")
	).toMatchSnapshot();
	expect(inlineSvg).toBe(
		'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg"></svg>'
	);
});
