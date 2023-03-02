import fs from "fs";

it("should have a variable 'Lib'", async function () {
	const code = (await fs.promises.readFile(__filename, "utf-8")).trim();
	expect(code.startsWith("var Lib;\n")).toBe(true);
	expect(code.endsWith("Lib = __webpack_exports__;\n})();")).toBe(true);
});
