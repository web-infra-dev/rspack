const { rspack } = require("@rspack/core");

class AddLinksPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("AddLinksPlugin", compilation => {
			rspack.HtmlRspackPlugin.getCompilationHooks(
				compilation
			).alterAssetTagGroups.tapPromise("AddLinksPlugin", async data => {
				data.headTags.push(
					{
						tagName: "link",
						attributes: {
							href: "https://example.com",
							rel: "dns-prefetch"
						},
						voidTag: true
					},
					{
						tagName: "link",
						attributes: {
							href: "https://example.com",
							rel: "preconnect"
						},
						voidTag: true
					},
					{
						tagName: "link",
						attributes: {
							rel: "prefetch",
							href: "https://example.com"
						},
						voidTag: true
					}
				);
			});
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	entry: {
		main: "./index.js"
	},
	externals: {
		path: "require('path')",
		fs: "require('fs')"
	},
	node: {
		__dirname: false
	},
	output: {
		crossOriginLoading: "anonymous"
	},
	plugins: [
		new rspack.HtmlRspackPlugin(),
		new rspack.experiments.SubresourceIntegrityPlugin(),
		new AddLinksPlugin()
	]
};
