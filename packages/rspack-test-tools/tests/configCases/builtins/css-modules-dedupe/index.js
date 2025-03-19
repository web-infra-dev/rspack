const path = __non_webpack_require__("path");

it("css modules dedupe", () => {
	const style = require("./source.css");
	expect(style).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'source.css.txt'));
});
