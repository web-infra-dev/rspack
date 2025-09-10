/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	output: {
		chunkFilename: "[name].js",
		crossOriginLoading: "anonymous"
	},
	plugins: [
		{
			apply(compiler) {
				const RuntimePlugin = compiler.webpack.RuntimePlugin;
				compiler.hooks.compilation.tap("mock-plugin", compilation => {
					const hooks = RuntimePlugin.getCompilationHooks(compilation);
					hooks.linkPrefetch.tap("mock-plugin", (code, chunk) => {
						return `${code}\nlink.setAttribute("data-prefetch-injected", "true");`;
					});
					hooks.linkPreload.tap("mock-plugin", (code, chunk) => {
						return `${code}\nlink.setAttribute("data-preload-injected", "true");`;
					});
				});
			}
		}
	]
};
