import './index.css'

const fs = __non_webpack_require__("node:fs");
const path = __non_webpack_require__("node:path");

it("css content minifyed", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);

	expect(css.toString()).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'bundle0.css.txt'))
});
