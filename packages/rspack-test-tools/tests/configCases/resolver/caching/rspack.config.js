class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		const cases = [".mjs", ".js", ".mjs"];
		for (const ext of cases) {
			const normalResolver = compiler.resolverFactory.get("normal", {
				extensions: [ext, ".js"]
			});
			const request = "./foo";
			const resolved = normalResolver.resolveSync(
				{},
				compiler.context,
				request
			);
			expect(resolved && resolved.split("/").at(-1)).toBe(`foo${ext}`);
		}
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	resolve: { extensions: [".mjs", ".js"] },
	plugins: [new Plugin()]
};
