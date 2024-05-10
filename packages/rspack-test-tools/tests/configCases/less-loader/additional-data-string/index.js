const fs = require("fs");
const path = require("path");

it("should work as string", () => {
	require("./index.less");
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css.includes("coral")).toBe(true);
});
