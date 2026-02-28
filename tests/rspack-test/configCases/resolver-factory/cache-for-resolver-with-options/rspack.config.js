const path = require("path");

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		{
			const normalResolver1 = compiler.resolverFactory.get("normal");
			const child1 = normalResolver1.withOptions({
				alias: {
					foo: path.resolve(__dirname, "index.js")
				}
			});

			const normalResolver2 = compiler.resolverFactory.get("normal");
			const child2 = normalResolver2.withOptions({
				alias: {
					foo: path.resolve(__dirname, "index.js")
				}
			});

			expect(child1 === child2).toBe(true);
		}
		{
			const normalResolver1 = compiler.resolverFactory.get("normal", {
				alias: {
					foo: path.resolve(__dirname, "index.js")
				}
			});
			const child1 = normalResolver1.withOptions({});

			const normalResolver2 = compiler.resolverFactory.get("normal");
			const child2 = normalResolver2.withOptions({
				alias: {
					foo: path.resolve(__dirname, "index.js")
				}
			});

			expect(child1 === child2).toBe(true);
		}
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	plugins: [new Plugin()]
};
