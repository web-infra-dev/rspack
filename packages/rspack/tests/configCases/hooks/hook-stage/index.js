const fs = require("fs");

it("compiler.hooks.compilation stage should works", () => {
	const mainFile = fs.readFileSync(__filename, "utf-8");
	expect(
		mainFile.startsWith(
			"/* banner3 */\n/* banner2 */\n/* banner4 */\n/* banner1 */"
		)
	).toBeTruthy();
});
