it("css modules localIdentName with hash", () => {
	const style = require("./index.css");
	expect(style).toMatchSnapshot();
});
