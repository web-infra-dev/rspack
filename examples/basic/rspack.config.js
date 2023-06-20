const rspack = require("@rspack/core");
console.log(rspack)
process.env.test = "test"

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
	plugins: [new rspack.EnvironmentPlugin("test")]
};
module.exports = config;
