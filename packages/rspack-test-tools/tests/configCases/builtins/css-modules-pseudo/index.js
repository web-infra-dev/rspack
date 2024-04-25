it("css modules pseudo syntax", () => {
	const style = require("./index.css");
	expect(style).toMatchSnapshot();
});
