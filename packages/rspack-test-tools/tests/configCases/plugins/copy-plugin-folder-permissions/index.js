const fs = require('fs');
const path = require('path');
const assert = require('assert');

// Export a function that will be called by the test runner
module.exports = function () {
	it("should copy folder contents and preserve permissions", () => {
		const sourceDir = path.join(__dirname, "src");
		const outputDir = path.join(__dirname, "dist");

		// List the contents of the dist directory
		if (fs.existsSync(outputDir)) {
			const distFiles = fs.readdirSync(outputDir);

			// Verify that the root file was copied
			assert(distFiles.includes("test.txt"), "test.txt should be copied");

			// Verify that the subfolder was copied
			assert(distFiles.includes("subfolder"), "subfolder should be copied");

			// Verify that the nested file was copied
			const subfolderPath = path.join(outputDir, "subfolder");
			if (fs.existsSync(subfolderPath) && fs.statSync(subfolderPath).isDirectory()) {
				const subfolderFiles = fs.readdirSync(subfolderPath);
				assert(subfolderFiles.includes("nested.txt"), "nested.txt should be copied");
			} else {
				assert.fail("Subfolder should exist and be a directory");
			}

			// Check permissions for root file
			const sourceRootPath = path.join(sourceDir, "test.txt");
			const outputRootPath = path.join(outputDir, "test.txt");

			// Verify the root file exists
			assert(fs.existsSync(outputRootPath), "Output root file should exist");

			// Get and compare permissions for root file
			const sourceRootStats = fs.statSync(sourceRootPath);
			const outputRootStats = fs.statSync(outputRootPath);

			assert.strictEqual(
				sourceRootStats.mode,
				outputRootStats.mode,
				"Root file permissions should match"
			);

			// Check permissions for nested file
			const sourceNestedPath = path.join(sourceDir, "subfolder", "nested.txt");
			const outputNestedPath = path.join(outputDir, "subfolder", "nested.txt");

			// Verify the nested file exists
			assert(fs.existsSync(outputNestedPath), "Output nested file should exist");

			// Get and compare permissions for nested file
			const sourceNestedStats = fs.statSync(sourceNestedPath);
			const outputNestedStats = fs.statSync(outputNestedPath);

			assert.strictEqual(
				sourceNestedStats.mode,
				outputNestedStats.mode,
				"Nested file permissions should match"
			);
		} else {
			assert.fail("Dist directory should exist");
		}
	});
};
