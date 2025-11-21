import fs from "fs";
import path from "path";

it("should work with array type of externals", function () {
	var external = require("external");
	expect(external).toBe("test");

	const js = fs.readFileSync(path.resolve(__dirname, "bundle0.js"), "utf-8");
	expect(js.includes('factory(root["a"]["b"])')).toBe(true);
});
