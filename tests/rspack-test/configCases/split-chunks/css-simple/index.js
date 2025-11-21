import "./index.css";

export default "index.js";

() => import("./foo");

it("css-simple", () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	expect(fs.existsSync(path.resolve(__dirname, "./foo_js.css"))).toBe(true);
	expect(fs.existsSync(path.resolve(__dirname, "./main.css"))).toBe(true);
});
