it("basic", () => {
	const style = require("./index.module.css");
	expect(style).toEqual({
		style: "style-index.module.css "
	});
});
