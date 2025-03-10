const path = __non_webpack_require__("path");

it("css modules with css preprocessers", () => {
	const style = require("./index.css");
	expect(style).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'index.css.txt'));
});
