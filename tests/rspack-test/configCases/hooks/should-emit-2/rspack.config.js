const assert = require("assert");
const path = require("path");
const fs = require("fs");

const customBundleFile = `
it("should emit this file", () => {
	expect(3).toBe(3);
});
`;

class Plugin {
	apply(compiler) {
		let count = 0;
		compiler.hooks.shouldEmit.tap("should-emit-should-works", compilation => {
			assert(typeof compilation !== "undefined");
			assert(typeof compilation.hooks !== "undefined");

			count += 1;
			const filePath = path.resolve(
				__dirname,
				compiler.options.output.path,
				"./bundle0.js"
			);
			if (!fs.existsSync(path.dirname(filePath))) {
				fs.mkdirSync(path.dirname(filePath), { recursive: true });
			}
			fs.writeFileSync(filePath, customBundleFile);
			return false;
		});

		compiler.hooks.done.tap("check", () => {
			assert(count === 1);
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	plugins: [new Plugin()],
	node: false
};
