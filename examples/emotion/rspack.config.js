/** @type {import('@rspack/cli').Configuration} */
const config = {
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
module.exports = config;
