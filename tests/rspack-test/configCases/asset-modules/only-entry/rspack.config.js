const path = require("path");
const fs = require("fs");
const { rspack } = require("@rspack/core");

/** @type {(number, any) => import("@rspack/core").Configuration} */
const common = (i, options) => ({
	target: "web",
	output: {
		filename: `${i}/[name].js`,
		chunkFilename: `${i}/[name].js`,
		cssFilename: `${i}/[name].css`,
		cssChunkFilename: `${i}/[name].css`,
		assetModuleFilename: `${i}/[name][ext][query]`
	},
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset"
			},
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("Test", compilation => {
					compilation.hooks.processAssets.tap(
						{
							name: "copy-rspack-plugin",
							stage:
								compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
						},
						() => {
							const data = fs.readFileSync(
								path.resolve(__dirname, "./test.js")
							);

							compilation.emitAsset(
								"test.js",
								new rspack.sources.RawSource(data)
							);
						}
					);
				});
			}
		}
	],
	...options
});

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	common(0, {
		entry: "../_images/file.png"
	}),
	common(1, {
		entry: {
			"asset-entry": {
				import: "../_images/file.png"
			},
			"js-entry": {
				import: "./entry.js"
			}
		}
	}),
	common(2, {
		entry: {
			"asset-entry": {
				import: "../_images/file.png"
			},
			"css-entry": {
				import: "./entry.css"
			}
		}
	}),
	common(3, {
		entry: {
			"asset-entry": {
				import: "../_images/file.png"
			},
			"js-entry": {
				import: "./entry.js"
			},
			"css-entry": {
				import: "./entry.css"
			}
		}
	}),
	common(4, {
		target: "node",
		entry: {
			"asset-entry": {
				import: "../_images/file.png"
			},
			"js-entry": {
				import: "./entry.js"
			},
			"css-entry": {
				import: "./entry.css"
			}
		}
	}),
	common(5, {
		entry: {
			"mixed-entry": {
				import: ["./entry.js", "../_images/file.png"]
			}
		}
	}),
	common(6, {
		entry: {
			"mixed-entry": {
				import: ["../_images/file.png", "./entry.js"]
			}
		}
	})
];
