module.exports = {
	plugins: [
		function skipWindows(compiler) {
			// windows' path can't include *
			if (process.platform !== "win32") {
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
