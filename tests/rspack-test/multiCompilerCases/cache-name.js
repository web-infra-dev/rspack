const path = require("path");

const cacheContext = path.join(__dirname, "../fixtures");

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig} */
module.exports = {
	description: "should preserve the existing multi-compiler cache paths",
	options() {
		return ["a", "b", "a"].map(entry => ({
			context: cacheContext,
			entry: `./${entry}.js`,
			mode: "development",
			cache: {
				type: "persistent"
			}
		}));
	},
	compiler(context, compiler) {
		const cacheDirectory = path.resolve(
			cacheContext,
			"node_modules/.cache/rspack"
		);
		const cacheNames = ["development", "development-1", "development-2"];

		expect(compiler.compilers).toHaveLength(cacheNames.length);
		compiler.compilers.forEach((childCompiler, index) => {
			const cache = childCompiler.options.cache;
			expect(cache.name).toBe(cacheNames[index]);
			expect(cache.storage.directory).toBe(cacheDirectory);
			expect(cache.storage.location).toBe(
				path.resolve(cacheDirectory, cacheNames[index])
			);
		});
	},
	build(context, compiler) {
		return new Promise((resolve, reject) => {
			compiler.close(error => {
				if (error) reject(error);
				else resolve();
			});
		});
	}
};
