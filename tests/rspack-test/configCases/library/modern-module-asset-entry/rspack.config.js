const assert = require("assert");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	entry: {
		index: "./img.png"
	},
	output: {
		module: true,
		filename: `[name].js`,
		chunkFilename: `async.js`,
		module: true,
		library: {
			type: "modern-module"
		},
		iife: false,
		chunkFormat: "module",
		chunkLoading: "import"
	},
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource",
				generator: {
					filename: "static/img/[name].png",
					importMode: "preserve"
				}
			}
		]
	},
	optimization: {
		concatenateModules: true,
		avoidEntryIife: true,
		minimize: false
	},
	plugins: [
		new (class {
			apply(compiler) {
				compiler.hooks.compilation.tap("MyPlugin", compilation => {
					compilation.hooks.processAssets.tap("MyPlugin", assets => {
						let list = Object.keys(assets);
						const js = list.find(item => item.endsWith("js"));
						const jsContent = assets[js].source().toString();

						const preseveImport =
							/import\simg_namespaceObject\sfrom ['"]\.\/static\/img\/img\.png['"]/.test(
								jsContent
							);
						assert(preseveImport);
						const hasExports =
							/export\sdefault\simg_namespaceObject/.test(jsContent);
						assert(hasExports);
					});
				});
			}
		})()
	]
};
