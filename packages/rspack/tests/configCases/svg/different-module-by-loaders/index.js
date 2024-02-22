require("./index.css");
require("./star.svg");

const fs = require("fs");
const path = require("path");

it("should both work for `type: 'asset/inline'` svg and `type: 'javascript/auto'` svg", () => {
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	const a = /a: url\((.*)\);/.exec(css)[1];
	expect(a.startsWith("data:image/svg+xml;base64,")).toBe(true);
	expect(fs.existsSync(path.resolve(__dirname, "star.svg"))).toBe(true);
	const js = fs.readFileSync(__filename, "utf-8");
	expect(
		js.includes('var _default = __webpack_require__.p + "star.svg";')
	).toBe(true);
});
