const path = require("path");

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		/** @param {import('@rspack/core').Resolve} options */
		const resolveFoo = (options = {}) => {
			const resolver = compiler.resolverFactory.get("normal", options);
			const resolved = resolver.resolveSync({}, compiler.context, "./foo");
			return resolved ? path.basename(resolved) : null;
		};

		// expect(resolveFoo()).toBe("foo.js");

		const cases = [".mjs", ".js", ".mjs"];
		for (const ext of cases) {
			// expect(resolveFoo({ extensions: [ext, ".js"] })).toBe(`foo${ext}`);
		}

		compiler.hooks.afterResolvers.tap("Plugin", () => {
			// expect(resolveFoo()).toBe("foo.mjs");
		});

		compiler.hooks.done.tap("Plugin", () => {
			expect(resolveFoo()).toBe("foo.mjs");
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	resolve: { extensions: [".mjs", ".js"] },
	plugins: [new Plugin()]
};
