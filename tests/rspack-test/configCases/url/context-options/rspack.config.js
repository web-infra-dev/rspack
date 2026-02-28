"use strict";

/** @type {import("../../../../").Configuration} */
module.exports = {
	output: {
		assetModuleFilename: "[path][name][ext]"
	},
	module: {
		parser: {
			javascript: {
				// this is always true in rspack
				// dynamicUrl: true
			}
		}
	}
};
