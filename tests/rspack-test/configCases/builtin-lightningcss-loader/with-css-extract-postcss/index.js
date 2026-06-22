import './index.css'

const fs = require("node:fs");
const path = require("node:path");

it("should transform CSS and add prefixes correctly", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);

	expect(css.includes('-ms-user-select: none;')).toBeTruthy();
	expect(css.includes('user-select: none;')).toBeTruthy();
});

it("should perform px to rem transformation", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);

	expect(css.includes('px')).toBeFalsy();
	expect(css.includes('rem')).toBeTruthy();
});
