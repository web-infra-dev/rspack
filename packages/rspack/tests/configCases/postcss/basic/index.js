it("basic", () => {
	const css = require("./index.css");
	const style = require("./index.module.css");
	expect(css).toEqual({});
	expect(style).toEqual({
		body: "_body_toys1_1"
	});
});
