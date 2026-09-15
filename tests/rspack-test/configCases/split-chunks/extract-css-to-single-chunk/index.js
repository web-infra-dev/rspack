import "./index.css";

export default "index.js";

() => import("./foo");

it("should extract css to single chunk", () => {
	const fs =  require("fs");
	const path =  require("path");
	expect(fs.existsSync(path.resolve(__dirname, "./foo_js.css"))).toBe(false);
	expect(fs.existsSync(path.resolve(__dirname, "./main.css"))).toBe(false);
	expect(fs.existsSync(path.resolve(__dirname, "./styles.css"))).toBe(true);
});
