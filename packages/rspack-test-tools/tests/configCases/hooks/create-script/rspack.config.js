/** @type {import('@rspack/core').Configuration} */
module.exports = {
	target: "web",
	output: {
		chunkLoading: "jsonp",
		crossOriginLoading: "anonymous"
	},
	plugins: [
		{
			apply(compiler) {
				const RuntimePlugin = compiler.webpack.RuntimePlugin;
				compiler.hooks.compilation.tap("mock-plugin", compilation => {
					const hooks = RuntimePlugin.getCompilationHooks(compilation);
					hooks.createScript.tap("mock-plugin", (code, chunk) => {
						return `${code}\nscript.setAttribute("data-create-script-injected", "true");`;
					});
				});
			}
		}
	]
};
