() => import("./foo");

import fs from "fs";
import path from "path";

export default "index.js";

it("disable-reuse-existing-chunk-simple", () => {
	expect(fs.existsSync(path.resolve(__dirname, "./splittedFoo.js"))).toBe(true);
	expect(fs.existsSync(path.resolve(__dirname, "./foo_js.js"))).toBe(false);
});
