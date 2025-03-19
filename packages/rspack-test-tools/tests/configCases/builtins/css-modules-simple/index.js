const path = __non_webpack_require__("path");

it("css modules simple test", () => {
	const style = require("./index.module.css");
	expect(style).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'index.module.css.txt'));
});
