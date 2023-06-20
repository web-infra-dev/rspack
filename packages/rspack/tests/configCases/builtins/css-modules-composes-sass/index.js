it("css modules in scss", () => {
	const style = require("./index.scss");
	expect(style).toMatchSnapshot();
});
