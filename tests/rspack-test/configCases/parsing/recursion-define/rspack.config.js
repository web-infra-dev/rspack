const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.DefinePlugin({
			"process.env.a": "process.env.a",
			a: "b",
			b: "a",
			"typeof process.env.b": "typeof process.env.b",
			"typeof a": "typeof b",
			"typeof b": "typeof a"
		})
	]
};
