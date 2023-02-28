const fs_cjs = require("fs");
import fs_esm from "fs";

it("node external should works", () => {
	expect(fs_cjs.existsSync(__filename)).toBeTruthy();
	expect(fs_esm.existsSync(__filename)).toBeTruthy();
});
