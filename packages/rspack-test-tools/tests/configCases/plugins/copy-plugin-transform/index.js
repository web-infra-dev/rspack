const fs = require('fs');
const path = require('path');
const assert = require('assert');

function test(msg, filename) {
	it(msg, () => {
		const outputPath = path.join(__dirname, "dist", filename);
		assert(fs.existsSync(outputPath), "Output file should exist");
	});
}

// Export a function that will be called by the test runner
module.exports = function () {
	// List the contents of the dist directory
	if (fs.existsSync(path.join(__dirname, "dist"))) {
		console.log("Dist directory contents:", fs.readdirSync(path.join(__dirname, "dist")));
	} else {
		console.log("Dist directory does not exist");
	}
	test("should transform file when transform is an sync function", "test-sync-fn.txt");
	test("should transform file when transform is an async function", "test-async-fn.txt");
	test("should transform file when transform is an object contains sync `transformer`", "test-sync-obj.txt");
	test("should transform file when transform is an object contains async `transformer`", "test-async-obj.txt");
};
