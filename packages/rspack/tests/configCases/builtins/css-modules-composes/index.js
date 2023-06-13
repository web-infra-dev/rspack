it("css modules composes", () => {
	const style = require("./index.css");
	expect(style).toMatchSnapshot();
});
