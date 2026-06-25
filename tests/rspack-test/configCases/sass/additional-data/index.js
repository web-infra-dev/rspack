const fs = require("fs");
const path = require("path");

it("should works as string", () => {
	require("./index.scss");
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css.includes("hotpink")).toBe(true);
});
