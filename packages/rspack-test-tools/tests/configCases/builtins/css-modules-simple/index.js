it("css modules simple test", () => {
	const style = require("./index.module.css");
	expect(style).toMatchSnapshot();
});
