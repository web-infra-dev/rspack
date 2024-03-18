const fs = require("fs");

it("compiler.hooks.compilation stage should works", () => {
	const mainFile = fs.readFileSync(__filename, "utf-8");
	expect(
		mainFile.startsWith(
`
/* sync banner5 */
/* async banner5 */
/* promise banner5 */
/* sync banner3 */
/* async banner3 */
/* promise banner3 */
/* sync banner2 */
/* async banner2 */
/* promise banner2 */
/* sync banner4 */
/* async banner4 */
/* promise banner4 */
/* sync banner1 */
/* async banner1 */
/* promise banner1 */
/* sync banner6 */
/* async banner6 */
/* promise banner6 */
`.trim()
		)
	).toBeTruthy();
});
