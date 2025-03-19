const fs = require('fs');
const path = require('path');
const assert = require('assert');

// Export a function that will be called by the test runner
module.exports = function () {
	it("should transform file when transform is an object contains `transformer`", () => {
		const sourcePath = path.join(__dirname, "src", "test.txt");
		const outputPath = path.join(__dirname, "dist", "test.txt");

		// List the contents of the dist directory
		if (fs.existsSync(path.join(__dirname, "dist"))) {
			console.log("Dist directory contents:", fs.readdirSync(path.join(__dirname, "dist")));
		} else {
			console.log("Dist directory does not exist");
		}

		assert(fs.existsSync(outputPath), "Output file should exist");
	});
};
