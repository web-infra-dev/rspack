const fs = require("fs");

it("compiler.hooks.compilation stage should works", () => {
	const mainFile = fs.readFileSync(__filename, "utf-8");
	expect(
		mainFile.startsWith(
`
/* banner5 */
/* banner3 */
/* banner2 */
/* banner4 */
/* banner1 */
/* banner6 */
`.trim()
		)
	).toBeTruthy();
});
