const assert = require("assert");
const fs = require("fs");

/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource"
			}
		]
	},
	plugins: [
		new (class {
			apply(compiler) {
				compiler.hooks.compilation.tap("MyPlugin", compilation => {
					compilation.hooks.processAssets.tap("MyPlugin", assets => {
						let list = Object.keys(assets);
						const png = list.find(item => item.endsWith("png"));
						const asset = compilation.getAsset(png);
						const buf = asset.source.buffer();
						const expected = fs.readFileSync(__dirname + "/" + "img.png");
						assert.deepEqual(buf, expected);
					});
				});
			}
		})()
	]
};
