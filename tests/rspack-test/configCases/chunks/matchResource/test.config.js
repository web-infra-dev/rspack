"use strict";

const fs = require("fs");

module.exports = {
	afterExecute(options) {
		const outputPath = options.output.path;
		const files = fs.readdirSync(outputPath);

		if (files.length > 1) {
			throw new Error('should not generate vendor chunk')
		}
	}
};
