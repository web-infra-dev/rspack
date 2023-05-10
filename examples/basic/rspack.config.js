/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	module: {
		rules: [
			// {
			// 	test: /\.svg$/,
			// 	type: 'asset/inline'
			// }
		]
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		],
	}
};
module.exports = config;
