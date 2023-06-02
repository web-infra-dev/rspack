import "./index.css";
import fs from "fs";
import path from "path";

export default "index.js";

() => import("./foo");

it("should extract css to single chunk", () => {
	expect(fs.existsSync(path.resolve(__dirname, "./foo_js.css"))).toBe(false);
	expect(fs.existsSync(path.resolve(__dirname, "./main.css"))).toBe(false);
	expect(fs.existsSync(path.resolve(__dirname, "./styles.css"))).toBe(true);
});
