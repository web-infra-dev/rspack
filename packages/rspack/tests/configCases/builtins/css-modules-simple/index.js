it("css modules simple test", () => {
	const style = require("./index.module.css");
	expect(style).toEqual({
		style: "-index-module-css__style "
	});
});
