it("css modules localsConvention with dashes", () => {
	const style = require("./index.css");
	expect(style).toMatchSnapshot();
});
