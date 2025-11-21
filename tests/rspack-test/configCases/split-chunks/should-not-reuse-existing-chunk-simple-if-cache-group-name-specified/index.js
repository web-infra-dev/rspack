() => import(/* webpackChunkName: "dyn-foo" */ "./foo");

import fs from "fs";
import path from "path";

export default "index.js";

it("should-not-reuse-existing-chunk-simple-if-cache-group-name-specified", () => {
	expect(fs.existsSync(path.resolve(__dirname, "./splittedFoo.js"))).toBe(true);
	expect(fs.existsSync(path.resolve(__dirname, "./dyn-foo.js"))).toBe(false);
});
