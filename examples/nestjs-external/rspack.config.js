const nodeExternals = require("webpack-node-externals");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	target: "node",
	entry: {
		main: "./src/main.ts"
	},
	externalsType: "commonjs",
	externals: [nodeExternals()]
};
module.exports = config;
