import("./a");
import("./b");

it("conflict", () => {
	const fs = require("fs");
	const source_a = fs.readFileSync(__dirname + "/a_js.bundle0.js.map", "utf-8");
	const source_b = fs.readFileSync(__dirname + "/b_js.bundle0.js.map", "utf-8");
	const map_a = JSON.parse(source_a);
  const map_b = JSON.parse(source_b);
	expect(map_a.sources).toStrictEqual([
    "webpack:///./a.js",
    "webpack:///./common.js",
  ]);
	expect(map_b.sources).toStrictEqual([
    "webpack:///./b.js",
    "webpack:///./common.js",
  ]);
});
