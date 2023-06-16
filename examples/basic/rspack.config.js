/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	devtool: false,
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		],
		teeShaking: true
	},
	optimization: {
		moduleIds: 'named',
		sideEffects:true,
		minimize: false
	},
	stats: "all"
};
module.exports = config;
