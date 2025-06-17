const path = require("path");

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		const cases = [".mjs", ".js", ".mjs"];
		for (const ext of cases) {
			const resolver = compiler.resolverFactory.get("normal", {
				extensions: [ext, ".js"]
			});
			const request = "./foo";
			const resolved = resolver.resolveSync({}, compiler.context, request);
			expect(resolved && path.basename(resolved)).toBe(`foo${ext}`);
		}
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	resolve: { extensions: [".mjs", ".js"] },
	plugins: [new Plugin()]
};
