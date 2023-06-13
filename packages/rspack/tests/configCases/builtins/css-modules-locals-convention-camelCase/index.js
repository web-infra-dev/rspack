it("css modules localsConvention with camelCase", () => {
	const style = require("./index.css");
	expect(style).toMatchSnapshot();
});
