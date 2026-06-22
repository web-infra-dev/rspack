import "./index.css";

export default "index.js";

() => import("./foo");

it("css-simple", () => {
	const fs = require("fs");
	const path = require("path");
	expect(fs.existsSync(path.resolve(__dirname, "./foo_js.css"))).toBe(true);
	expect(fs.existsSync(path.resolve(__dirname, "./main.css"))).toBe(true);
});
