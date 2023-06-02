() => import(/* webpackChunkName: "dyn-foo" */ "./foo");

import fs from "fs";
import path from "path";

export default "index.js";

it("reuse-existing-chunk-simple", () => {
	expect(fs.existsSync(path.resolve(__dirname, "./dyn-foo.js"))).toBe(true);
});
