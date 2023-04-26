/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	entry: {
		main: "./src/index.jsx"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		],
		emotion: true,
		react: {
			importSource: "@emotion/react",
			runtime: "automatic"
		}
	}
};
