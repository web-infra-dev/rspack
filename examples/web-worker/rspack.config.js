/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
module.exports = config;
