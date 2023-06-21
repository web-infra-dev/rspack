/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	entry: {
		main: "./src/main.tsx"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	},
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset"
			}
		]
	}
};
