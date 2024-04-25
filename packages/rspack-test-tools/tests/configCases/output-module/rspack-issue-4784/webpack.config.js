const path = require("path");
const fs = require("fs");
const assert = require("assert");

module.exports = {
	// USE `development` as `production` will be failed.
	// See: https://github.com/web-infra-dev/rspack/issues/5738
	mode: "development",
	entry: {
		main: "./index.js",
		m: "./m.js"
	},
	output: {
		filename: "[name].mjs",
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
					const dest = path.resolve(
						__dirname,
						"../../.././js/ConfigTestCases/output-module/rspack-issue-4784/m.mjs"
					);
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
