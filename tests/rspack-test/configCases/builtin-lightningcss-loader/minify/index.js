import './index.css'

const fs = require("node:fs");
const path = require("node:path");

it("css content minifyed", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);

	expect(css.toString()).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'bundle0.css.txt'))
});
