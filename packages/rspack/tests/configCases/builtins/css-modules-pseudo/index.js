it("css modules pseudo syntax", () => {
	const style = require("./index.css");
	expect(style).toEqual({
		bar: "-index-css__bar ",
		bav: "-index-css__bav ",
		foo: "-index-css__foo ",
		four: "-index-css__four ",
		one: "-index-css__one ",
		three: "-index-css__three ",
		two: "-index-css__two "
	});
});
