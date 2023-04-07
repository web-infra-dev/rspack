/**@type {import('@rspack/core').Configuration} */
module.exports = {
	entry: {
		polyfill: "./polyfill.js",
		main: "./index.js"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
