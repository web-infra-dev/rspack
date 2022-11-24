const path = require("path");
const postcssLoader = require("@rspack/plugin-postcss");
const resolve = filename => path.resolve(__dirname, filename);

module.exports = {
	module: {
		rules: [
			{
				test: resolve("modules-true.module.css"),
				uses: [
					{
						loader: postcssLoader,
						options: {
							modules: true
						}
					}
				]
			},
			{
				test: resolve("modules-false.module.css"),
				uses: [
					{
						loader: postcssLoader,
						options: {
							modules: false
						}
					}
				]
			},
			{
				test: resolve("auto-true.module.css"),
				uses: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								auto: true
							}
						}
					}
				]
			},
			{
				test: resolve("auto-false.module.css"),
				uses: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								auto: false
							}
						}
					}
				]
			},
			{
				test: resolve("auto-regex.css"),
				uses: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								auto: /auto-regex.css$/
							}
						}
					}
				]
			},
			{
				test: resolve("auto-function.css"),
				uses: [
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
				]
			},
			{
				test: resolve("generateScopedName.module.css"),
				uses: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								generateScopedName: "[name]__[local]___[hash:base64:5]"
							}
						}
					}
				]
			},
			{
				test: resolve("localsConvention.module.css"),
				uses: [
					{
						loader: postcssLoader,
						options: {
							modules: {
								localsConvention: "camelCase"
							}
						}
					}
				]
			}
		]
	}
};
