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
			filename: "index.html"
		}),
		new rspack.HtmlRspackPlugin({
			templateContent:
				"<!DOCTYPE html><html><body><div><%= env %></div></body></html>",
			templateParameters: {
				env: "development"
			},
			filename: "index2.html"
		}),
		{
			apply: compiler => {
				compiler.hooks.thisCompilation.tap("HtmlRspackPlugin", compilation => {
					const hooks =
						rspack.HtmlRspackPlugin.getCompilationHooks(compilation);
					let index = 0;
					hooks.beforeEmit.tap("HtmlRspackPlugin", htmlPluginData => {
						if (index === 0) {
							expect(htmlPluginData.plugin.options.filename).toBe("index.html");
						} else {
							expect(htmlPluginData.plugin.options.filename).toBe(
								"index2.html"
							);
						}
						index++;
					});
				});
			}
		}
	]
};
