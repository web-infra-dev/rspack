/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.module\.css$/,
				type: "css/module",
				parser: {
					exportsOnly: false,
					namedExports: false
				},
				generator: {
					localIdentName: "[path]_[name]_[path]_[local]--/__[hash:42]<[hash:3]"
				}
			}
		]
	}
};
