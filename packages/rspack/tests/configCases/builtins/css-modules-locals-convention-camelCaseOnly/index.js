it("css modules localsConvention with camelCaseOnly", () => {
	const style = require("./index.css");
	expect(style).toMatchSnapshot();
});
