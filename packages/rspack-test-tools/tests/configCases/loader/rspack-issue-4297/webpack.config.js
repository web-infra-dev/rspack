module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /lib\.js$/,
				resourceQuery: /source/,
			},
			{
				test: /lib\.js$/,
				resourceQuery: { not: [/source/] },
				loader: "./queryloader.js"
			},
			{
				test: /lib\.js$/,
				resourceFragment: /source/,
			},
			{
				test: /lib\.js$/,
				resourceFragment: { not: [/source/] },
				loader: "./fragmentloader.js"
			},
		]
	}
};
