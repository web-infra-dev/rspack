/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: 'web',
node: {
  __dirname: false,
  __filename: false
},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		}
	},
	experiments: {
		css: true
	}
};
