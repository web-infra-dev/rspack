const fs = require("fs");
const path = require("path");

it("css modules exportsOnly", () => {
	const style = require("./index.css");
	expect(fs.existsSync(path.resolve(__dirname, "./main.css"))).toBe(false);
	expect(style).toMatchSnapshot();
});
