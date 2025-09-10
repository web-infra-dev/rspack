const path = require("path");

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		{
			const normalResolver = compiler.resolverFactory.get("normal");
			const request = normalResolver.resolveSync({}, __dirname, "foo");
			expect(request).toBe(path.join(__dirname, "index.js"));
		}
		{
			const normalResolver = compiler.resolverFactory.get("normal", {
				alias: {
					bar: path.resolve(__dirname, "index.js")
				}
			});
			const request = normalResolver.resolveSync({}, __dirname, "bar");
			expect(request).toBe(path.join(__dirname, "index.js"));
		}
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	resolve: {
		alias: {
			foo: path.resolve(__dirname, "index.js")
		}
	},
	plugins: [new Plugin()]
};
