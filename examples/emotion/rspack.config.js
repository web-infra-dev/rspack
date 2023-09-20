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
						},
						transform: {
							react: {
								importSource: "@emotion/react",
								runtime: "automatic"
							}
						}
					},
					rspackExperiments: {
						emotion: true
					}
				},
				type: "javascript/auto"
			}
		]
	}
};
module.exports = config;
