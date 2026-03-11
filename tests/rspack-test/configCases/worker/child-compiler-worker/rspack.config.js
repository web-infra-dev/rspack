const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	output: {
		filename: "[name].js"
	},
	plugins: [
		compiler => {
			compiler.hooks.make.tapAsync("ChildCompilerWorkerTest", (compilation, callback) => {
				const childCompiler = compilation.createChildCompiler(
					"child-compiler-worker-test",
					{ filename: "__child-[name].js", publicPath: "" },
					[
						new compiler.webpack.library.EnableLibraryPlugin("commonjs"),
						new compiler.webpack.EntryPlugin(
							compiler.context,
							path.join(__dirname, "child-entry.js"),
							{
								name: "child-entry",
								library: { type: "commonjs" }
							}
						)
					]
				);
				childCompiler.runAsChild(err => callback(err));
			});
		}
	]
};
