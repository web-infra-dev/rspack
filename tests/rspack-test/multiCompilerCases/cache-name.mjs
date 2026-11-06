import path from "node:path";

const cacheContext = path.join(import.meta.dirname, "../fixtures");

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
export default [false, true].map(newCache => ({
	description: newCache
		? "should use compiler namespaces for new cache paths"
		: "should preserve the existing multi-compiler cache paths",
	options() {
		const names = newCache
			? [undefined, undefined, "client", "client"]
			: [undefined, undefined, undefined];
		return names.map(name => ({
			name,
			context: cacheContext,
			entry: "./a.js",
			mode: "development",
			experiments: { newCache },
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
		const cacheNames = newCache
			? [
					"development",
					"development-1",
					"client-development-2",
					"client-development-3"
				]
			: ["development", "development-1", "development-2"];

		expect(compiler.compilers).toHaveLength(cacheNames.length);
		compiler.compilers.forEach((childCompiler, index) => {
			expect(childCompiler.compilerPath).toBe(
				newCache ? cacheNames[index] : ""
			);
			const cache = childCompiler.options.cache;
			expect(cache.name).toBe(cacheNames[index]);
			expect(cache.storage.directory).toBe(cacheDirectory);
			expect(cache.storage.location).toBe(
				path.resolve(
					cacheDirectory,
					newCache ? "new-cache" : "",
					cacheNames[index]
				)
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
