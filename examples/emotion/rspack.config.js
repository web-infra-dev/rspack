const { createSwcLoaderExperiments } = require("@rspack/cli");

const rspackExperiments = createSwcLoaderExperiments()
	.useEmotion(true, process.env.NODE_ENV === "production")
	.useReact();

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
		]
	},
	module: {
		rules: [
			{
				test: /\.jsx$/,
				loader: "builtin:swc-loader",
				options: {
					jsc: {
						parser: {
							syntax: "ecmascript",
							jsx: true
						}
					},
					rspackExperiments
				},
				type: "javascript/auto"
			}
		]
	}
};
module.exports = config;
