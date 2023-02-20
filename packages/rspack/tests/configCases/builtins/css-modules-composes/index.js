it("css modules composes", () => {
	const style = require("./index.css");
	expect(style).toEqual({
		"simple-bar":
			"-index-css__simple-bar -imported-simple-css__imported-simple ",
		"simple-foo":
			"-index-css__simple-foo -imported-simple-css__imported-simple "
	});
});
