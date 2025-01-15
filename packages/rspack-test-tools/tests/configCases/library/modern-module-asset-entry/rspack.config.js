/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	entry: {
		index: './img.png'
	},
	output: {
		filename: `[name].js`,
    chunkFilename: `async.js`,
		module: true,
		library: {
			type: "modern-module",
		},
		iife: false,
		chunkFormat: "module",
		chunkLoading: 'import',
	},
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource",
				generator: {
					filename: 'static/img/[name].png',
					experimentalLibPreserveImport: true
				}
			}
		]
	},
	experiments: {
		outputModule: true
	},
	optimization: {
		concatenateModules: true,
		avoidEntryIife: true,
		minimize: false
	},
	plugins:[
		new (class {
			apply(compiler) {
				compiler.hooks.compilation.tap("MyPlugin", compilation => {
					compilation.hooks.processAssets.tap("MyPlugin", assets => {
						let list = Object.keys(assets);
						const js = list.find(item => item.endsWith("js"));
						const jsContent = assets[js].source().toString();

						/import img_namespaceObject from ['"]\.\/static\/img\/img\.png['"]/.test(jsContent);
						/export\s{\simg_namespaceObject\sas\sdefault\s}/.test(jsContent);
					})
				});
			}
		})()
	]
};
