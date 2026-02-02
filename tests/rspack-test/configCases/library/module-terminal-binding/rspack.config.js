"use strict";

module.exports = {
	mode: "production",
	target: "web",
	optimization: {
		minimize: false
	},
	experiments: {
		},
	output: {
		module: true,
		library: {
			type: "module"
		},
		module: true
	}
};
