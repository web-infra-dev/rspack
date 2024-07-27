import './index.css'

const fs = require("node:fs");
const path = require("node:path");

it("should transform CSS and add prefixes correctly", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);

	expect(css.includes('-webkit-user-select: none;')).toBeFalsy();
	expect(css.includes('-ms-user-select: none;')).toBeFalsy();
	expect(css.includes('user-select: none;')).toBeFalsy();
	expect(css.includes('background: -webkit-linear-gradient(#fff, #000);')).toBeFalsy();
	expect(css.includes('background: linear-gradient(#fff, #000);')).toBeFalsy();
	expect(css.includes('-webkit-transition: all .5s;')).toBeFalsy();
	expect(css.includes('transition: all .5s;')).toBeFalsy();
});
