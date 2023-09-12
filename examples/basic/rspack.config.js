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
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
module.exports = config;
