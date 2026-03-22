"use strict";

module.exports = {
	target: [`async-node${process.versions.node.split(".").map(Number)[0]}`],
	output: {
		pathinfo: "verbose"
	},
	optimization: {
		sideEffects: true,
		usedExports: true,
		providedExports: true,
		concatenateModules: false
	},
	experiments: {
		deferImport: true
	}
};
