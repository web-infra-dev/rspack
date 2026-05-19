const fs = require("fs");
const path = require("path");

/*! LICENSE - uppercase should be extracted */

/*! license - lowercase should NOT be extracted */

it("should respect case sensitivity in regex", () => {
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
		
		// Should extract uppercase LICENSE
		expect(content).toContain("LICENSE - uppercase should be extracted");
		
		// Should NOT extract lowercase license
		expect(content).not.toContain("license - lowercase should NOT be extracted");
	}
});
