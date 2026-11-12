import path from "node:path";

const cacheContext = path.join(import.meta.dirname, "../fixtures");

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
export default [false, true].map(readonly => ({
	description: readonly
		? "should use the same new cache paths in read-only mode"
		: "should preserve compiler cache paths with the new cache enabled",
	options() {
		return [undefined, undefined, "client", "client"].map(name => ({
			name,
			context: cacheContext,
			entry: "./a.js",
			mode: "development",
			experiments: { newCache: true },
			cache: {
				type: "persistent",
				readonly
			}
		}));
	},
	compiler(context, compiler) {
		const cacheDirectory = path.resolve(
			cacheContext,
			"node_modules/.cache/rspack"
		);
		const cacheNames = [
			"development",
			"development-1",
			"client-development-2",
			"client-development-3"
		];

		expect(compiler.compilers).toHaveLength(cacheNames.length);
		compiler.compilers.forEach((childCompiler, index) => {
			expect(childCompiler.compilerPath).toBe(
				childCompiler.options.name || ""
			);
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
}));
