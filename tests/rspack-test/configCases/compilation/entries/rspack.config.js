const PLUGIN_NAME = "plugin";

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			compilation.hooks.seal.tap(PLUGIN_NAME, () => {
				expect(Array.from(compilation.entries.keys())).toEqual(["main", "foo"]);

				const entry = compilation.entries.get("foo");
				expect(entry.dependencies.length).toEqual(1);
				expect(entry.options.asyncChunks).toEqual(true);

				compilation.entries.delete("foo");
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	entry: {
		main: {
			import: "./index.js"
		},
		foo: {
			import: "./foo.js",
			asyncChunks: true
		}
	},
	plugins: [new Plugin()]
};
