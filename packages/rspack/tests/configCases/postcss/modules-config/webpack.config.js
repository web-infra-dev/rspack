const path = require("path");
const postcssLoader = require("@rspack/postcss-loader");
const resolve = filename => path.resolve(__dirname, filename);

module.exports = {
	module: {
		defaultRules: [],
		rules: [
			{
				test: resolve("modules-true.css"),
				use: [
					{
						loader: postcssLoader,
						options: {
							modules: true
						}
					}
				],
				type: "css"
			},
			{
				test: resolve("modules-false.module.css"),
				use: [
					{
						loader: postcssLoader,
						options: {
							modules: false
						}
					}
				],
				type: "css"
			},
			{
				test: resolve("auto-true.module.css"),
				use: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								auto: true
							}
						}
					}
				],
				type: "css"
			},
			{
				test: resolve("auto-false.module.css"),
				use: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								auto: false
							}
						}
					}
				],
				type: "css"
			},
			{
				test: resolve("auto-regex.css"),
				use: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								auto: /auto-regex.css$/
							}
						}
					}
				],
				type: "css"
			},
			{
				test: resolve("auto-function.css"),
				use: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								auto(p) {
									return p.endsWith("auto-function.css");
								}
							}
						}
					}
				],
				type: "css"
			},
			{
				test: resolve("generateScopedName.module.css"),
				use: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								generateScopedName: "[name]__[local]___[hash:base64:5]"
							}
						}
					}
				],
				type: "css"
			},
			{
				test: resolve("localsConvention.module.css"),
				use: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								localsConvention: "camelCase"
							}
						}
					}
				],
				type: "css"
			}
		]
	}
};
