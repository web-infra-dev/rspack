const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("css/auto can handle css module correctly", () => {
	const style = require("./index.module.css");
	expect(style).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'index.module.css.txt'));
});

it("css/auto can handle css correctly", () => {
	require("./index.css");
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css.includes("aliceblue")).toBe(true);
});
