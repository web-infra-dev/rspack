/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	},
	optimization: {
		usedExports: true
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
module.exports = config;
