/**
 * @type {import('@rspack/core').Configuration}
 */
const config = {
	module: {
		rules: [
			{
				test: /index\.js$/,
				loader: require.resolve("./loader.js")
			}
		]
	}
};
module.exports = config;
