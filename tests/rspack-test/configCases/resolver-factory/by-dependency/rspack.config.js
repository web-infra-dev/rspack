const path = require("path");

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		{
			const normalResolver = compiler.resolverFactory.get("normal", {
				dependencyType: "commonjs"
			});
			expect(() => normalResolver.resolveSync({}, __dirname, "foo")).toThrow(
				'NotFound("foo")'
			);
		}
		{
			const normalResolver = compiler.resolverFactory.get("normal", {
				dependencyType: "esm"
			});
			const request = normalResolver.resolveSync({}, __dirname, "foo");
			expect(request).toBe(path.join(__dirname, "index.js"));
		}
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	resolve: {
		byDependency: {
			esm: {
				alias: {
					foo: path.resolve(__dirname, "index.js")
				}
			}
		}
	},
	plugins: [new Plugin()]
};
