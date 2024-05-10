const isWindows = process.platform === "win32";

const entry = `
it("should generate valid code", async () => {
  ${
		isWindows
			? `expect("skip windows").toBe("skip windows");`
			: `const { staticA, dynamicA } = await import("./entry.mjs"); expect(staticA.a).toBe(1); expect(dynamicA.a).toBe(1);`
	}
});
`;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: `data:text/javascript,${entry}`,
	plugins: [
		function skipWindows(compiler) {
			// windows' path can't include *
			if (!isWindows) {
				const fs = require("fs");
				const path = require("path");
				const dir = path.resolve(__dirname, "star*");
				fs.mkdirSync(dir);
				fs.writeFileSync(path.resolve(dir, "a.js"), "export const a = 1;");
				// cleanup
				compiler.hooks.done.tap("skipWindows", () => {
					fs.rmSync(dir, { recursive: true, force: true });
				});
			}
		}
	]
};
