/** @type {import('@rspack/core').Configuration} */
module.exports = [
	{
		entry: {
			case1: "./case1/index.js"
		},
		output: {
			filename: "case1.js"
		}
	},
	{
		entry: {
			case2: "./case2/index.js"
		},
		output: {
			filename: "case2.js"
		}
	},
	{
		entry: {
			case3: "./case3/index.js"
		},
		output: {
			filename: "case3.js"
		}
	}
];
