"use strict";

module.exports = {
	mode: "production",
	target: "web",
	optimization: {
		minimize: false
	},
	output: {
		filename: '[name].mjs',
		library: {
			type: "module"
		},
	}
};
