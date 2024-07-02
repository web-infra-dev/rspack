const common = {
	mode: "production",
	optimization: {
		moduleIds: "named",
	},
	module: {
		rules: [
			{
				test: /\.module\.css$/,
				type: "css/module",
				oneOf: [
					{
						resourceQuery: /\?camel-case$/,
						generator: {
							exportsConvention: "camel-case",
							localIdentName: "[path][name][ext][query][fragment]-[local]"
						}
					}
				]
			}
		]
	},
	experiments: {
		css: true
	}
};

/** @type {import("../../../../").Configuration} */
module.exports = [
	{
		...common,
		target: "web"
	},
	{
		...common,
		target: "node"
	}
];
