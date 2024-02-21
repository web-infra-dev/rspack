const path = require("path");
const fs = require("fs");
const assert = require("assert");

module.exports = {
	entry: {
		main: "./index.js",
		m: "./m.js"
	},
	output: {
		chunkFormat: "module",
		chunkLoading: "import",
		library: {
			type: "module"
		}
	},
	experiments: {
		outputModule: true,
		rspackFuture: {
			newTreeshaking: true
		}
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.afterEmit.tap("test", () => {
					const dest = path.resolve(__dirname, "./dist/m.mjs");
					assert(fs.existsSync(dest));
					const testRaw = `
import { a, b } from './main.mjs';
it('should get correctly exports', () => {
	expect(a).toBe('a')
	expect(b).toBe('b')
})`;
					fs.writeFileSync(dest, testRaw);
				});
			}
		}
	]
};
