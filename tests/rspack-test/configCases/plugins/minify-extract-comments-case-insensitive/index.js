const fs = require("fs");
const path = require("path");

/*! LICENSE - uppercase should be extracted */

/*! license - lowercase should also be extracted */

it("should be case-insensitive with 'i' flag", () => {
	// Check that the license file was created
	const licenseExists = fs.existsSync(
		path.resolve(__dirname, "bundle0.js.LICENSE.txt")
	);
	expect(licenseExists).toBe(true);
	
	if (licenseExists) {
		const content = fs.readFileSync(
			path.resolve(__dirname, "bundle0.js.LICENSE.txt"),
			"utf-8"
		);
		
		// Should extract both uppercase and lowercase
		expect(content).toContain("LICENSE - uppercase should be extracted");
		expect(content).toContain("license - lowercase should also be extracted");
	}
});
