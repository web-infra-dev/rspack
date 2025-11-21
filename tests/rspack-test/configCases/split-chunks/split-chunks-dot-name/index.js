() => import("./foo");

import fs from "fs";
import path from "path";

export default "index.js";

it("split-chunks-dot-name", () => {
	expect(fs.existsSync(path.resolve(__dirname, "./overall-foo.js"))).toBe(true);
	expect(fs.existsSync(path.resolve(__dirname, "./main.js"))).toBe(true);
});
