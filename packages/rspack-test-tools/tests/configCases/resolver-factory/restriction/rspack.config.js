const path = require("path");

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		const normalResolver = compiler.resolverFactory.get("normal");
		const newResolver = normalResolver.withOptions({
			extensions: [".css"],
			mainFields: ["main"],
			restrictions: [/\.css$/]
		});
		const request = newResolver.resolveSync({}, __dirname, "style");
		expect(request).toBe(path.join(__dirname, "/node_modules/style/index.css"));
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	plugins: [new Plugin()]
};
