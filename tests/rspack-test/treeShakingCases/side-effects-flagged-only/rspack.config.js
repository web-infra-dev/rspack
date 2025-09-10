/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	},
	optimization: {
		sideEffects: "flag"
	}
};
