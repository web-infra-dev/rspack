const path = __non_webpack_require__("path");

it("css modules in scss", () => {
	const style = require("./index.scss");
	expect(style).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'index.scss.txt'));
});
