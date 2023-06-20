it("css modules localIdentName with path", () => {
	const style = require("./index.css");
	expect(style).toMatchSnapshot();
});
