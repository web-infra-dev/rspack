import "./index.css";
import fs from "fs";
import path from "path";

export default "index.js";

() => import("./foo");

it("css-simple", () => {
	expect(fs.existsSync(path.resolve(__dirname, "./foo_js.css"))).toBe(true);
	expect(fs.existsSync(path.resolve(__dirname, "./main.css"))).toBe(true);
});
