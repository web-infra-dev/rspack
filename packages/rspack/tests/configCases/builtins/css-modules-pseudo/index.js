it("css modules pseudo syntax", () => {
	const style = require("./index.css");
	expect(style).toEqual({
		bar: "bar-index.css ",
		bav: "bav-index.css ",
		foo: "foo-index.css ",
		four: "four-index.css ",
		one: "one-index.css ",
		three: "three-index.css ",
		two: "two-index.css "
	});
});
