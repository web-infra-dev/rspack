const rspack = require("@rspack/core");

let index = 0;
let ENV = {
	ENV_A: 0,
	ENV_B: 0,
	ENV_C: 0
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	entry: "./entry.js",
	optimization: {
		minimize: false
	},
	experiments: {
		cache: {
			type: "persistent"
		}
	},
	plugins: [
		{
			apply(_compiler) {
				if (index === 1) {
					ENV.ENV_A++;
				}
				if (index === 2) {
					ENV.ENV_B++;
				}
				if (index === 3) {
					ENV.ENV_C++;
				}
				index++;
			}
		},
		new rspack.DefinePlugin({
			"process.env": ENV
		})
	]
};
