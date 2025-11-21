() => import(/* webpackChunkName: "dyn-foo" */ "./foo");

import fs from "fs";
import path from "path";

export default "index.js";

it("disable-reuse-existing-chunk-simple", () => {
	expect(fs.existsSync(path.resolve(__dirname, "./dyn-foo.js"))).toBe(false);
});
