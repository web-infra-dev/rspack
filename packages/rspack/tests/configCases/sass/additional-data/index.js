const fs = require("fs");
const path = require("path");

it("basic", () => {
	require("./index.scss");
	const css = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");
	expect(css.includes("hotpink")).toBe(true);
});
