it("css modules composes", () => {
	const style = require("./index.css");
	expect(style).toEqual({
		"simple-bar": "simple-bar-index.css imported-simple-imported-simple.css ",
		"simple-foo": "simple-foo-index.css imported-simple-imported-simple.css "
	});
});
