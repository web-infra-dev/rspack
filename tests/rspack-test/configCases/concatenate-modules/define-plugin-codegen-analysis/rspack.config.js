const { DefinePlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	optimization: {
		concatenateModules: true,
		minimize: false
	},
	plugins: [
		new DefinePlugin({
			IDENTIFIER_VALUE: "sourceValue",
			EXPRESSION_VALUE: "expressionValue + 2",
			LITERAL_VALUE: JSON.stringify("literal"),
			CONSTANT_VALUE: "1 + 2",
			ARRAY_VALUE: [JSON.stringify("literal"), false, 1],
			OBJECT_VALUE: {
				MODE: JSON.stringify("production"),
				DEV: false,
				PROD: true,
				SSR: false,
				BASE_URL: JSON.stringify("/"),
				ASSET_PREFIX: JSON.stringify(""),
				NESTED: {
					FLAGS: [true, false]
				}
			}
		})
	]
};
