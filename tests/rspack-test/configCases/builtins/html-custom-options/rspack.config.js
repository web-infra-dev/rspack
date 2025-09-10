const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.HtmlRspackPlugin({
			templateContent:
				"<!DOCTYPE html><html><body><div><%= env %></div></body></html>",
			templateParameters: {
				env: "production"
			},
			filename: "index.html",
			customOptions: {
				property: "value"
			}
		}),
		{
			apply: compiler => {
				compiler.hooks.thisCompilation.tap("HtmlRspackPlugin", compilation => {
					const hooks =
						rspack.HtmlRspackPlugin.getCompilationHooks(compilation);
					hooks.beforeEmit.tap("HtmlRspackPlugin", htmlPluginData => {
						expect(htmlPluginData.plugin.options.customOptions.property).toBe(
							"value"
						);
					});
				});
			}
		}
	]
};
