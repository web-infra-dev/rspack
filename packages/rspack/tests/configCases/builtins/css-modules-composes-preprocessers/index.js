it("css modules with css preprocessers", () => {
	const style = require("./index.css");
	expect(style).toMatchSnapshot();
});
