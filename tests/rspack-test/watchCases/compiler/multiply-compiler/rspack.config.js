/** @type {import('@rspack/core').Configuration} */
module.exports = [
	{
		entry: {
			entry1: "./entry1.js"
		},
		output: {
			filename: "./bundle1.js"
		}
	},
	{
		entry: {
			entry2: "./entry2.js"
		},
		output: {
			filename: "./bundle2.js"
		}
	}
];
