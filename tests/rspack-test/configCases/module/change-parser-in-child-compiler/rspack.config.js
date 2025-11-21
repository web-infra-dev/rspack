/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	plugins: [
		function (compiler) {
			compiler.hooks.make.tapAsync("test", (compilation, callback) => {
				const child = compilation.createChildCompiler(
					"test",
					{
						filename: "__child-[name].js",
						publicPath: ""
					},
					[
						new compiler.webpack.library.EnableLibraryPlugin("commonjs"),
						new compiler.webpack.EntryPlugin(
							compilation.options.context,
							"./child-entry.js",
							{
								name: "main",
								library: {
									type: "commonjs"
								}
							}
						)
					]
				);
				child.options.module.parser.javascript.url = "relative";
				child.runAsChild(callback);
			});
		}
	]
};
