import "./index.css";

const fs = require("fs");
const path = require("path");

it("should derive targets from rspack target when a child compiler reuses the rules", () => {
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css.includes("-webkit-")).toBe(false);
	expect(css.includes("-moz-")).toBe(false);
	expect(fs.existsSync(path.resolve(__dirname, "__child-main.js"))).toBe(true);
});
