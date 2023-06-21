it("css modules dedupe", () => {
	const style = require("./source.css");
	expect(style).toMatchSnapshot();
});
