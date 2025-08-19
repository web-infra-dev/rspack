import './index.css'

const fs = __non_webpack_require__("node:fs");
const path = __non_webpack_require__("node:path");

it("should transform CSS and add prefixes correctly", () => {
	const css = fs.readFileSync(
		path.resolve(__dirname, "./bundle0.css"),
		"utf-8"
	);

	expect(css.includes('-webkit-user-select: none;')).toBeTruthy();
	expect(css.includes('-ms-user-select: none;')).toBeTruthy();
	expect(css.includes('user-select: none;')).toBeTruthy();
	expect(css.includes('background: -webkit-linear-gradient(top, #fff, #000);')).toBeTruthy();
	expect(css.includes('background: linear-gradient(#fff, #000);')).toBeTruthy();
	expect(css.includes('-webkit-transition: all .5s;')).toBeTruthy();
	expect(css.includes('transition: all .5s;')).toBeTruthy();
});
