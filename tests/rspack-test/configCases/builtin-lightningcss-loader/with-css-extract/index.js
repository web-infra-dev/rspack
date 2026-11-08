import './index.css'

const fs = require("node:fs");
const path = require("node:path");

it("should normalize builtin loader options for CSS extraction", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);

	expect(css.includes('-ms-user-select: none;')).toBeTruthy();
	expect(css.includes('user-select: none;')).toBeTruthy();
	expect(css).toContain('margin-inline-end: 100px;');
	expect(css).toContain('.foo .bar');
	expect(css).not.toContain('& .bar');
});
