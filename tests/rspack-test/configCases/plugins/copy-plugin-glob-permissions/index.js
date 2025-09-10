const fs = require('fs');
const path = require('path');
const assert = require('assert');

// Export a function that will be called by the test runner
module.exports = function () {
	it("should copy files with glob pattern and preserve permissions", () => {
		const sourceDir = path.join(__dirname, "src");
		const outputDir = path.join(__dirname, "dist");

		// List the contents of the dist directory
		if (fs.existsSync(outputDir)) {
			const distFiles = fs.readdirSync(outputDir);

			// Verify that both test files were copied
			assert(distFiles.includes("test1.txt"), "test1.txt should be copied");
			assert(distFiles.includes("test2.txt"), "test2.txt should be copied");

			// Check permissions for each copied file
			const files = ["test1.txt", "test2.txt"];
			files.forEach(file => {
				const sourcePath = path.join(sourceDir, file);
				const outputPath = path.join(outputDir, file);

				// Verify the file exists
				assert(fs.existsSync(outputPath), `Output file ${file} should exist`);

				// Get and compare permissions
				const sourceStats = fs.statSync(sourcePath);
				const outputStats = fs.statSync(outputPath);

				assert.strictEqual(
					sourceStats.mode,
					outputStats.mode,
					`File permissions for ${file} should match`
				);
			});
		} else {
			assert.fail("Dist directory should exist");
		}
	});
};
