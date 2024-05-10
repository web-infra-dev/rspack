import fs from "fs";
import path from "path";

it("splitChunks chunks function", async () => {
	const lib1 = __non_webpack_require__("./lib1.js");
	expect(lib1.lib1.value).toBe(43);
	const lib2Asset = await fs.promises.readFile(
		path.resolve(__dirname, "./lib2.js"),
		"utf-8"
	);
	expect(lib2Asset.includes('"./shared.js": ')).toBe(false);
});
