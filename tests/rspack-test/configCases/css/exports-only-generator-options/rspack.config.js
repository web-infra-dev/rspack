/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		target: "web",
		mode: "development",
		module: {
			generator: {
				css: {
					exportsOnly: true
				},
				"css/module": {
					exportsOnly: false
				}
			},
			rules: [
				{
					resourceQuery: /\?module/,
					type: "css/module"
				},
				// {
				// 	resourceQuery: /\?exportsOnly/,
				// 	generator: {
				// 		exportsOnly: true
				// 	},
				// 	type: "css/global"
				// },
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		},

		node: {
			__dirname: false
		}
	}
];
