"use strict";

module.exports = {
	mode: "production",
	target: "web",
	optimization: {
		minimize: false
	},
	output: {
		module: true,
		library: {
			type: "module"
		},
		module: true
	}
};
