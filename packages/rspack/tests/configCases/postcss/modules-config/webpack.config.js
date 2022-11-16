const postcssLoader = require("@rspack/plugin-postcss");
module.exports = {
	module: {
		rules: [
			{
				test: "modules-true.module.css$",
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
				test: "modules-false.module.css$",
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
				test: "auto-true.module.css$",
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
				test: "auto-false.module.css$",
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
				test: "auto-regex.css$",
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
				test: "auto-function.css$",
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
				test: "generateScopedName.module.css$",
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
				test: "localsConvention.module.css$",
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
