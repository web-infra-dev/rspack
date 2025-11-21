const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("functional use works", () => {
	require("./index.less");
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css.includes("background-color: coral")).toBe(true);
});

it("resourceQuery should match correctly", () => {
	require("./index.less?test");
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css.includes("background-color: red")).toBe(true);
});
