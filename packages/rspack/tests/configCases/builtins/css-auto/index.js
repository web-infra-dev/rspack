const fs = require("fs");
const path = require("path");

it("css/auto can handle css module correctly", () => {
	const style = require("./index.module.css");
	expect(style).toMatchSnapshot();
});

it("css/auto can handle css correctly", () => {
	require("./index.css");
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css.includes("aliceblue")).toBe(true);
});
