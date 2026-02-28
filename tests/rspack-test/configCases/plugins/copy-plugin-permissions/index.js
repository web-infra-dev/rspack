const fs = require('fs');
const path = require('path');
const assert = require('assert');

// Export a function that will be called by the test runner
module.exports = function () {
	it("should copy file permissions correctly", () => {
		const sourcePath = path.join(__dirname, "src", "test.txt");
		const outputPath = path.join(__dirname, "dist", "test.txt");

		// Print the current directory and check if the files exist
		console.log("Current directory:", __dirname);
		console.log("Source path:", sourcePath);
		console.log("Output path:", outputPath);
		console.log("Source exists:", fs.existsSync(sourcePath));
		console.log("Output exists:", fs.existsSync(outputPath));

		// List the contents of the dist directory
		if (fs.existsSync(path.join(__dirname, "dist"))) {
			console.log("Dist directory contents:", fs.readdirSync(path.join(__dirname, "dist")));
		} else {
			console.log("Dist directory does not exist");
		}

		// Verify the file exists
		assert(fs.existsSync(outputPath), "Output file should exist");

		// Get and compare permissions
		const sourceStats = fs.statSync(sourcePath);
		const outputStats = fs.statSync(outputPath);

		assert.strictEqual(
			sourceStats.mode,
			outputStats.mode,
			"File permissions should match"
		);
	});
};
