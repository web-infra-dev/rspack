import a from 'data:text/javascript,export default "a";';
import b from "data:text/javascript,export default 'b';";
import c from "data:text/javascript,export default `c`;";
import "data:text/css,.red{color: red;}";
import "./index.css";
import inlineSvg from 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg"></svg>';

const p = Promise.all([
	import('data:text/javascript,globalThis.d = "d";'),
	import("data:text/javascript,globalThis.e = 'e';"),
	import("data:text/javascript,globalThis.f = `f`;"),
])

it("data imports", async () => {
	const fs = __non_webpack_require__("node:fs");
	const path = __non_webpack_require__("node:path");
	await p;
	expect(a).toBe("a");
	expect(b).toBe("b");
	expect(c).toBe("c");

	expect(
		fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8")
	).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'bundle0.css.txt'));
	expect(inlineSvg).toBe(
		'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg"></svg>'
	);
});
